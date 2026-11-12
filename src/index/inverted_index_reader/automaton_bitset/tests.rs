use std::future::Future;
use std::io;
use std::ops::Range;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use common::HasLen;
use tantivy_fst::Regex;

use super::InvertedIndexReader;
use crate::directory::{FileHandle, FileSlice, OwnedBytes};
use crate::query::BitSetDocSet;
use crate::schema::{IndexRecordOption, Schema, TextFieldIndexing, TextOptions};
use crate::termdict::{TermDictionary, TermDictionaryBuilder};
use crate::{DocSet, Executor, Index, IndexWriter, TERMINATED};

#[derive(Debug)]
struct AsyncFile {
    bytes: OwnedBytes,
    reject_sync: AtomicBool,
    fail_async: AtomicBool,
    sync_reads: AtomicUsize,
    batches: Mutex<Vec<Vec<Range<usize>>>>,
    buffers: Mutex<Vec<Weak<[u8]>>>,
}

impl AsyncFile {
    fn new(bytes: OwnedBytes) -> Arc<Self> {
        Arc::new(Self {
            bytes,
            reject_sync: AtomicBool::new(false),
            fail_async: AtomicBool::new(false),
            sync_reads: AtomicUsize::new(0),
            batches: Mutex::default(),
            buffers: Mutex::default(),
        })
    }

    fn read_batch(&self, ranges: Vec<Range<usize>>) -> io::Result<Vec<OwnedBytes>> {
        self.batches.lock().unwrap().push(ranges.clone());
        if self.fail_async.load(Ordering::SeqCst) {
            return Err(io::Error::other("injected read failure"));
        }
        Ok(ranges
            .into_iter()
            .map(|range| {
                // Each read owns a fresh buffer. Retaining a decoded result must not retain it.
                let bytes: Arc<[u8]> = Arc::from(&self.bytes[range]);
                self.buffers.lock().unwrap().push(Arc::downgrade(&bytes));
                OwnedBytes::new(bytes)
            })
            .collect())
    }

    fn reject_sync(&self) {
        self.sync_reads.store(0, Ordering::SeqCst);
        self.reject_sync.store(true, Ordering::SeqCst);
    }

    fn assert_released(&self) {
        assert!(self
            .buffers
            .lock()
            .unwrap()
            .iter()
            .all(|buffer| buffer.strong_count() == 0));
        assert_eq!(self.sync_reads.load(Ordering::SeqCst), 0);
    }
}

impl HasLen for AsyncFile {
    fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl FileHandle for AsyncFile {
    fn read_bytes(&self, range: Range<usize>) -> io::Result<OwnedBytes> {
        self.sync_reads.fetch_add(1, Ordering::SeqCst);
        if self.reject_sync.load(Ordering::SeqCst) {
            return Err(io::Error::other("synchronous body reads forbidden"));
        }
        Ok(self.bytes.slice(range))
    }

    // Spell out async_trait's signature to avoid adding a test-only dependency.
    fn read_bytes_async<'a, 'f>(
        &'a self,
        range: Range<usize>,
    ) -> Pin<Box<dyn Future<Output = io::Result<OwnedBytes>> + Send + 'f>>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move { Ok(self.read_batch(vec![range])?.remove(0)) })
    }

    fn read_bytes_batch_async<'a, 'f>(
        &'a self,
        ranges: Vec<Range<usize>>,
    ) -> Pin<Box<dyn Future<Output = io::Result<Vec<OwnedBytes>>> + Send + 'f>>
    where
        'a: 'f,
        Self: 'f,
    {
        Box::pin(async move { self.read_batch(ranges) })
    }
}

fn fixture(
    option: IndexRecordOption,
) -> crate::Result<(InvertedIndexReader, Arc<AsyncFile>, Arc<AsyncFile>)> {
    let mut schema = Schema::builder();
    let options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("raw")
            .set_index_option(option),
    );
    let field = schema.add_text_field("field", options);
    let index = Index::create_in_ram(schema.build());
    let mut writer: IndexWriter = index.writer_for_tests()?;
    writer.add_document(doc!(field => "apple", field => "banana"))?;
    writer.add_document(doc!(field => "apricot"))?;
    writer.add_document(doc!(field => "banana", field => "berry"))?;
    writer.add_document(doc!(field => "carrot"))?;
    for _ in 0..260 {
        writer.add_document(doc!(field => "apple", field => "apricot"))?;
    }
    writer.commit()?;
    let searcher = index.reader()?.searcher();
    let original = searcher.segment_reader(0).inverted_index(field)?;
    let mut dictionary = TermDictionaryBuilder::create(Vec::new())?;
    let mut stream = original.terms().stream()?;
    while let Some((key, info)) = stream.next() {
        dictionary.insert(key, info)?;
    }
    let dictionary_file = AsyncFile::new(OwnedBytes::new(dictionary.finish()?));
    // Opening metadata is allowed. All reads during materialization must be asynchronous.
    let termdict = TermDictionary::open(FileSlice::new(dictionary_file.clone()))?;
    dictionary_file.reject_sync();
    let postings_file = AsyncFile::new(original.postings_file_slice.read_bytes()?);
    postings_file.reject_sync();
    let reader = InvertedIndexReader {
        termdict,
        postings_file_slice: FileSlice::new(postings_file.clone()),
        positions_file_slice: FileSlice::empty(),
        record_option: option,
        total_num_tokens: original.total_num_tokens,
    };
    Ok((reader, dictionary_file, postings_file))
}

#[test]
fn owned_bitset_async_reads() -> crate::Result<()> {
    let executor = Executor::multi_thread(1, "owned-bitset-test-")?;
    for option in [
        IndexRecordOption::Basic,
        IndexRecordOption::WithFreqs,
        IndexRecordOption::WithFreqsAndPositions,
    ] {
        for (pattern, expected) in [
            ("carrot", vec![3]), // singleton
            ("b.*", vec![0, 2]), // overlapping short lists
            ("z.*", vec![]),
            ("a.*", [vec![0, 1], (4..264).collect()].concat()), // full blocks and overlap
            ("apple|berry", [vec![0, 2], (4..264).collect()].concat()), // disjoint ranges
            (".*", (0..264).collect()),
        ] {
            let (reader, dictionary_file, postings_file) = fixture(option)?;
            // Determine the exact expected ranges using a separate in-memory dictionary.
            let dictionary =
                TermDictionary::open(FileSlice::new(Arc::new(dictionary_file.bytes.clone())))?;
            let mut stream = dictionary
                .search(Regex::new(pattern).unwrap())
                .into_stream()?;
            let mut ranges = Vec::new();
            while let Some((_, info)) = stream.next() {
                ranges.push(info.postings_range.clone());
            }
            let calls = AtomicUsize::new(0);
            let bitset = futures::executor::block_on(reader.read_postings_automaton_bitset(
                Regex::new(pattern).unwrap(),
                264,
                |task| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    let handle = executor.spawn_blocking(task);
                    async move {
                        handle
                            .await
                            .map_err(|_| io::Error::other("CPU task panicked"))?
                    }
                },
            ))?;
            assert_eq!(calls.load(Ordering::SeqCst), 2);
            assert_eq!(dictionary_file.batches.lock().unwrap().len(), 1);
            assert_eq!(*postings_file.batches.lock().unwrap(), vec![ranges]);
            dictionary_file.assert_released();
            postings_file.assert_released();
            drop(reader);
            assert_eq!(bitset.len(), expected.len());
            let mut docset = BitSetDocSet::from(bitset);
            let mut actual = Vec::new();
            while docset.doc() != TERMINATED {
                actual.push(docset.doc());
                docset.advance();
            }
            assert_eq!(actual, expected, "{pattern} {option:?}");
        }
    }
    Ok(())
}

#[test]
fn owned_bitset_empty() -> io::Result<()> {
    let reader = InvertedIndexReader::empty(IndexRecordOption::Basic);
    let bitset = futures::executor::block_on(reader.read_postings_automaton_bitset(
        Regex::new(".*").unwrap(),
        0,
        |task| std::future::ready(task()),
    ))?;
    assert_eq!(bitset.len(), 0);
    Ok(())
}

#[test]
fn owned_bitset_cancellation_releases_buffers() -> crate::Result<()> {
    for pause_at in [1, 2] {
        let (reader, dictionary, postings) = fixture(IndexRecordOption::Basic)?;
        let mut calls = 0;
        let mut future = Box::pin(reader.read_postings_automaton_bitset(
            Regex::new(".*").unwrap(),
            264,
            |task| {
                calls += 1;
                let pause = calls == pause_at;
                async move {
                    if pause {
                        std::future::pending::<()>().await;
                    }
                    task()
                }
            },
        ));
        futures::executor::block_on(async {
            assert!(futures::poll!(&mut future).is_pending());
        });
        let pending_file = if pause_at == 1 {
            &dictionary
        } else {
            &postings
        };
        assert!(pending_file
            .buffers
            .lock()
            .unwrap()
            .iter()
            .any(|buffer| buffer.strong_count() > 0));
        drop(future);
        dictionary.assert_released();
        postings.assert_released();
    }
    Ok(())
}

#[test]
fn owned_bitset_errors() -> crate::Result<()> {
    for failure in 0..6 {
        let (reader, dictionary, postings) = fixture(IndexRecordOption::Basic)?;
        dictionary.fail_async.store(failure == 0, Ordering::SeqCst);
        postings.fail_async.store(failure == 1, Ordering::SeqCst);
        let mut calls = 0;
        let result = futures::executor::block_on(reader.read_postings_automaton_bitset(
            Regex::new(".*").unwrap(),
            264,
            |task| {
                calls += 1;
                std::future::ready(match (failure, calls) {
                    (2, 1) | (4, 2) => Err(io::Error::other("executor cancelled")),
                    (3, 1) | (5, 2) => {
                        drop(task);
                        Ok(())
                    }
                    _ => task(),
                })
            },
        ));
        assert!(result.is_err());
        dictionary.assert_released();
        postings.assert_released();
    }
    let (reader, dictionary, postings) = fixture(IndexRecordOption::Basic)?;
    let result = futures::executor::block_on(reader.read_postings_automaton_bitset(
        Regex::new(".*").unwrap(),
        1,
        |task| std::future::ready(task()),
    ));
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    dictionary.assert_released();
    postings.assert_released();
    Ok(())
}
