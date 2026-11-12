use std::future::Future;
use std::io;

use common::BitSet;
use tantivy_fst::Automaton;

use super::InvertedIndexReader;
use crate::postings::{BlockSegmentPostings, TermInfo};
use crate::schema::IndexRecordOption;

async fn execute<T, E, F>(
    executor: &mut E,
    task: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T>
where
    T: Send + 'static,
    E: FnMut(Box<dyn FnOnce() -> io::Result<()> + Send>) -> F,
    F: Future<Output = io::Result<()>>,
{
    let (sender, receiver) = futures_channel::oneshot::channel();
    executor(Box::new(move || {
        let result = task()?;
        sender
            .send(result)
            .map_err(|_| io::Error::other("automaton bitset caller dropped"))
    }))
    .await?;
    receiver
        .await
        .map_err(|_| io::Error::other("automaton bitset task dropped without a result"))
}

impl InvertedIndexReader {
    /// Reads postings matched by an automaton and unions their document IDs into a bitset.
    ///
    /// Reads the selected dictionary blocks asynchronously once, then requests the exact matching
    /// posting ranges in one asynchronous batch. No synchronous body reads or directory cache are
    /// required. The returned bitset retains none of the dictionary or posting buffers.
    ///
    /// `max_doc` must be the segment's maximum document ID plus one, including deleted documents.
    /// Deleted documents are included in the result.
    ///
    /// The executor runs both CPU tasks (term enumeration and postings decoding), allowing callers
    /// to choose their priority. Its future must resolve after the task finishes and propagate task
    /// errors, panics, and cancellation as errors. Use a CPU thread pool to keep this work off the
    /// calling thread. The executor controls cancellation of submitted CPU tasks. Their buffers
    /// are released when the task completes or is dropped.
    pub async fn read_postings_automaton_bitset<A, E, F>(
        &self,
        automaton: A,
        max_doc: u32,
        mut executor: E,
    ) -> io::Result<BitSet>
    where
        A: Automaton + Send + 'static,
        A::State: Clone + Send + 'static,
        E: FnMut(Box<dyn FnOnce() -> io::Result<()> + Send>) -> F,
        F: Future<Output = io::Result<()>>,
    {
        let mut stream = self
            .termdict
            .search(automaton)
            .without_keys()
            .into_stream_async()
            .await?
            .into_owned();
        let term_infos: Vec<TermInfo> = execute(&mut executor, move || {
            Ok(std::iter::from_fn(|| stream.next_without_key().cloned()).collect())
        })
        .await?;
        let ranges = term_infos
            .iter()
            .map(|info| info.postings_range.clone())
            .collect();
        let posting_bytes = self
            .postings_file_slice
            .read_bytes_batch_async(ranges)
            .await?;
        if posting_bytes.len() != term_infos.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "posting batch returned the wrong number of buffers",
            ));
        }
        let record_option = self.record_option;
        execute(&mut executor, move || {
            let mut bitset = BitSet::with_max_value(max_doc);
            for (info, bytes) in term_infos.into_iter().zip(posting_bytes) {
                if bytes.len() != info.postings_range.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "posting batch returned the wrong buffer length",
                    ));
                }
                let mut postings = BlockSegmentPostings::open_bytes(
                    info.doc_freq,
                    bytes,
                    record_option,
                    IndexRecordOption::Basic,
                )?;
                while !postings.docs().is_empty() {
                    for &doc in postings.docs() {
                        if doc >= max_doc {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "posting document exceeds max_doc",
                            ));
                        }
                        bitset.insert(doc);
                    }
                    postings.advance();
                }
            }
            Ok(bitset)
        })
        .await
    }
}

#[cfg(test)]
mod tests;
