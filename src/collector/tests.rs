use columnar::{BytesColumn, Column};

use super::*;
use crate::query::{AllQuery, EnableScoring, Query, QueryParser, TermQuery};
use crate::schema::{IndexRecordOption, Schema, FAST, STRING, TEXT};
use crate::time::format_description::well_known::Rfc3339;
use crate::time::OffsetDateTime;
use crate::{DateTime, DocAddress, Index, Searcher, TantivyDocument, Term};

pub const TEST_COLLECTOR_WITH_SCORE: TestCollector = TestCollector {
    compute_score: true,
};

pub const TEST_COLLECTOR_WITHOUT_SCORE: TestCollector = TestCollector {
    compute_score: true,
};

#[test]
pub fn test_filter_collector() -> crate::Result<()> {
    let mut schema_builder = Schema::builder();
    let title = schema_builder.add_text_field("title", TEXT);
    let price = schema_builder.add_u64_field("price", FAST);
    let date = schema_builder.add_date_field("date", FAST);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);

    let mut index_writer = index.writer_with_num_threads(1, 20_000_000)?;
    index_writer.add_document(doc!(title => "The Name of the Wind", price => 30_200u64, date => DateTime::from_utc(OffsetDateTime::parse("1898-04-09T00:00:00+00:00", &Rfc3339).unwrap())))?;
    index_writer.add_document(doc!(title => "The Diary of Muadib", price => 29_240u64, date => DateTime::from_utc(OffsetDateTime::parse("2020-04-09T00:00:00+00:00", &Rfc3339).unwrap())))?;
    index_writer.add_document(doc!(title => "The Diary of Anne Frank", price => 18_240u64, date => DateTime::from_utc(OffsetDateTime::parse("2019-04-20T00:00:00+00:00", &Rfc3339).unwrap())))?;
    index_writer.add_document(doc!(title => "A Dairy Cow", price => 21_240u64, date => DateTime::from_utc(OffsetDateTime::parse("2019-04-09T00:00:00+00:00", &Rfc3339).unwrap())))?;
    index_writer.add_document(doc!(title => "The Diary of a Young Girl", price => 20_120u64, date => DateTime::from_utc(OffsetDateTime::parse("2018-04-09T00:00:00+00:00", &Rfc3339).unwrap())))?;
    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title]);
    let query = query_parser.parse_query("diary")?;
    let filter_some_collector = FilterCollector::new(
        "price".to_string(),
        &|value: u64| value > 20_120u64,
        TopDocs::with_limit(2).order_by_score(),
    );
    let top_docs = searcher.search(&query, &filter_some_collector)?;

    assert_eq!(top_docs.len(), 1);
    assert_eq!(top_docs[0].1, DocAddress::new(0, 1));

    let filter_all_collector: FilterCollector<_, _, u64> = FilterCollector::new(
        "price".to_string(),
        &|value| value < 5u64,
        TopDocs::with_limit(2).order_by_score(),
    );
    let filtered_top_docs = searcher.search(&query, &filter_all_collector).unwrap();

    assert_eq!(filtered_top_docs.len(), 0);

    fn date_filter(value: DateTime) -> bool {
        (value.into_utc() - OffsetDateTime::parse("2019-04-09T00:00:00+00:00", &Rfc3339).unwrap())
            .whole_weeks()
            > 0
    }

    let filter_dates_collector = FilterCollector::new(
        "date".to_string(),
        &date_filter,
        TopDocs::with_limit(5).order_by_score(),
    );
    let filtered_date_docs = searcher.search(&query, &filter_dates_collector)?;

    assert_eq!(filtered_date_docs.len(), 2);
    Ok(())
}

/// Stores all of the doc ids.
/// This collector is only used for tests.
/// It is unusable in practise, as it does
/// not store the segment ordinals
pub struct TestCollector {
    pub compute_score: bool,
}

pub struct TestSegmentCollector {
    segment_id: SegmentOrdinal,
    fruit: TestFruit,
}

#[derive(Default)]
pub struct TestFruit {
    docs: Vec<DocAddress>,
    scores: Vec<Score>,
}

impl TestFruit {
    /// Return the list of matching documents exhaustively.
    pub fn docs(&self) -> &[DocAddress] {
        &self.docs[..]
    }
    pub fn scores(&self) -> &[Score] {
        &self.scores[..]
    }
}

impl Collector for TestCollector {
    type Fruit = TestFruit;
    type Child = TestSegmentCollector;

    fn for_segment(
        &self,
        segment_id: SegmentOrdinal,
        _reader: &SegmentReader,
    ) -> crate::Result<TestSegmentCollector> {
        Ok(TestSegmentCollector {
            segment_id,
            fruit: TestFruit::default(),
        })
    }

    fn requires_scoring(&self) -> bool {
        self.compute_score
    }

    fn merge_fruits(&self, mut children: Vec<TestFruit>) -> crate::Result<TestFruit> {
        children.sort_by_key(|fruit| {
            if fruit.docs().is_empty() {
                0
            } else {
                fruit.docs()[0].segment_ord
            }
        });
        let mut docs = vec![];
        let mut scores = vec![];
        for child in children {
            docs.extend(child.docs());
            scores.extend(child.scores);
        }
        Ok(TestFruit { docs, scores })
    }
}

impl SegmentCollector for TestSegmentCollector {
    type Fruit = TestFruit;

    fn collect(&mut self, doc: DocId, score: Score) {
        self.fruit.docs.push(DocAddress::new(self.segment_id, doc));
        self.fruit.scores.push(score);
    }

    fn harvest(self) -> <Self as SegmentCollector>::Fruit {
        self.fruit
    }
}

/// Collects in order all of the fast fields for all of the
/// doc in the `DocSet`
///
/// This collector is mainly useful for tests.
pub struct FastFieldTestCollector {
    field: String,
}

pub struct FastFieldSegmentCollector {
    vals: Vec<u64>,
    reader: Column,
}

impl FastFieldTestCollector {
    pub fn for_field(field: impl ToString) -> FastFieldTestCollector {
        FastFieldTestCollector {
            field: field.to_string(),
        }
    }
}

impl Collector for FastFieldTestCollector {
    type Fruit = Vec<u64>;
    type Child = FastFieldSegmentCollector;

    fn for_segment(
        &self,
        _: SegmentOrdinal,
        segment_reader: &SegmentReader,
    ) -> crate::Result<FastFieldSegmentCollector> {
        let reader = segment_reader
            .fast_fields()
            .u64(&self.field)
            .expect("Requested field is not a fast field.");
        Ok(FastFieldSegmentCollector {
            vals: Vec::new(),
            reader,
        })
    }

    fn requires_scoring(&self) -> bool {
        false
    }

    fn merge_fruits(&self, children: Vec<Vec<u64>>) -> crate::Result<Vec<u64>> {
        Ok(children.into_iter().flat_map(|v| v.into_iter()).collect())
    }
}

impl SegmentCollector for FastFieldSegmentCollector {
    type Fruit = Vec<u64>;

    fn collect(&mut self, doc: DocId, _score: Score) {
        self.vals.extend(self.reader.values_for_doc(doc));
    }

    fn harvest(self) -> Vec<u64> {
        self.vals
    }
}

/// Collects in order all of the fast field bytes for all of the
/// docs in the `DocSet`
///
/// This collector is mainly useful for tests.
/// It is very slow.
pub struct BytesFastFieldTestCollector {
    field: String,
}

pub struct BytesFastFieldSegmentCollector {
    vals: Vec<u8>,
    column_opt: Option<BytesColumn>,
    buffer: Vec<u8>,
}

impl BytesFastFieldTestCollector {
    pub fn for_field(field: impl ToString) -> BytesFastFieldTestCollector {
        BytesFastFieldTestCollector {
            field: field.to_string(),
        }
    }
}

impl Collector for BytesFastFieldTestCollector {
    type Fruit = Vec<u8>;
    type Child = BytesFastFieldSegmentCollector;

    fn for_segment(
        &self,
        _segment_local_id: u32,
        segment_reader: &SegmentReader,
    ) -> crate::Result<BytesFastFieldSegmentCollector> {
        let column_opt = segment_reader.fast_fields().bytes(&self.field)?;
        Ok(BytesFastFieldSegmentCollector {
            vals: Vec::new(),
            column_opt,
            buffer: Vec::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        false
    }

    fn merge_fruits(&self, children: Vec<Vec<u8>>) -> crate::Result<Vec<u8>> {
        Ok(children.into_iter().flat_map(|c| c.into_iter()).collect())
    }
}

impl SegmentCollector for BytesFastFieldSegmentCollector {
    type Fruit = Vec<u8>;

    fn collect(&mut self, doc: DocId, _score: Score) {
        if let Some(column) = self.column_opt.as_ref() {
            for term_ord in column.term_ords(doc) {
                let (vals, buffer) = (&mut self.vals, &mut self.buffer);
                if column.ord_to_bytes(term_ord, buffer).unwrap() {
                    vals.extend(&buffer[..]);
                }
            }
        }
    }

    fn harvest(self) -> <Self as SegmentCollector>::Fruit {
        self.vals
    }
}

fn make_test_searcher() -> crate::Result<Searcher> {
    let schema = Schema::builder().build();
    let index = Index::create_in_ram(schema);
    let mut index_writer = index.writer_for_tests()?;
    index_writer.add_document(TantivyDocument::default())?;
    index_writer.add_document(TantivyDocument::default())?;
    index_writer.commit()?;
    Ok(index.reader()?.searcher())
}

#[test]
fn test_option_collector_some() -> crate::Result<()> {
    let searcher = make_test_searcher()?;
    let counts = searcher.search(&AllQuery, &Some(Count))?;
    assert_eq!(counts, Some(2));
    Ok(())
}

#[test]
fn test_option_collector_none() -> crate::Result<()> {
    let searcher = make_test_searcher()?;
    let none_collector: Option<Count> = None;
    let counts = searcher.search(&AllQuery, &none_collector)?;
    assert_eq!(counts, None);
    Ok(())
}

#[derive(Clone, Copy)]
struct ScoredBlockCollector;

struct ScoredBlockSegmentCollector {
    block_lengths: Vec<usize>,
}

impl Collector for ScoredBlockCollector {
    type Fruit = Vec<usize>;
    type Child = ScoredBlockSegmentCollector;

    fn for_segment(
        &self,
        _segment_id: SegmentOrdinal,
        _reader: &SegmentReader,
    ) -> crate::Result<Self::Child> {
        Ok(ScoredBlockSegmentCollector {
            block_lengths: Vec::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        true
    }

    fn merge_fruits(&self, children: Vec<Self::Fruit>) -> crate::Result<Self::Fruit> {
        Ok(children.into_iter().flatten().collect())
    }
}

impl SegmentCollector for ScoredBlockSegmentCollector {
    type Fruit = Vec<usize>;

    fn collect(&mut self, _doc: DocId, _score: Score) {
        panic!("scored block forwarding fell back to scalar collection")
    }

    fn collect_scored_block(&mut self, docs: &[DocId], scores: &[Score]) {
        assert_eq!(docs.len(), scores.len());
        self.block_lengths.push(docs.len());
    }

    fn harvest(self) -> Self::Fruit {
        self.block_lengths
    }
}

fn make_all_query_searcher(num_docs: usize) -> crate::Result<Searcher> {
    let schema = Schema::builder().build();
    let index = Index::create_in_ram(schema);
    let mut writer = index.writer_for_tests()?;
    for _ in 0..num_docs {
        writer.add_document(TantivyDocument::default())?;
    }
    writer.commit()?;
    Ok(index.reader()?.searcher())
}

#[test]
fn test_scored_block_forwarding_for_option_tuples_and_multi_collector() -> crate::Result<()> {
    let searcher = make_all_query_searcher(crate::query::SCORE_BLOCK_LEN * 2 + 2)?;
    let expected = vec![
        crate::query::SCORE_BLOCK_LEN,
        crate::query::SCORE_BLOCK_LEN,
        2,
    ];

    assert_eq!(
        searcher.search(&AllQuery, &Some(ScoredBlockCollector))?,
        Some(expected.clone())
    );

    let pair = searcher.search(&AllQuery, &(ScoredBlockCollector, ScoredBlockCollector))?;
    assert_eq!(pair, (expected.clone(), expected.clone()));

    let triple = searcher.search(
        &AllQuery,
        &(
            ScoredBlockCollector,
            ScoredBlockCollector,
            ScoredBlockCollector,
        ),
    )?;
    assert_eq!(
        triple,
        (expected.clone(), expected.clone(), expected.clone())
    );

    let quadruple = searcher.search(
        &AllQuery,
        &(
            ScoredBlockCollector,
            ScoredBlockCollector,
            ScoredBlockCollector,
            ScoredBlockCollector,
        ),
    )?;
    assert_eq!(
        quadruple,
        (
            expected.clone(),
            expected.clone(),
            expected.clone(),
            expected.clone()
        )
    );

    let mut collectors = MultiCollector::new();
    let first = collectors.add_collector(ScoredBlockCollector);
    let second = collectors.add_collector(ScoredBlockCollector);
    let mut fruits = searcher.search(&AllQuery, &collectors)?;
    assert_eq!(first.extract(&mut fruits), expected.clone());
    assert_eq!(second.extract(&mut fruits), expected);
    Ok(())
}

fn assert_weight_scalar_block_parity(query: &dyn Query, searcher: &Searcher) -> crate::Result<()> {
    let weight = query.weight(EnableScoring::enabled_from_searcher(searcher))?;
    for reader in searcher.segment_readers() {
        let mut scalar = Vec::new();
        weight.for_each(reader, &mut |doc, score| scalar.push((doc, score)))?;

        let mut blocked = Vec::new();
        weight.for_each_scored_block(reader, &mut |docs, scores| {
            assert_eq!(docs.len(), scores.len());
            blocked.extend(docs.iter().copied().zip(scores.iter().copied()));
        })?;
        assert_eq!(blocked, scalar);
    }
    Ok(())
}

#[test]
fn test_scored_blocks_tuple_and_multi_parity_with_segments_and_deletes() -> crate::Result<()> {
    let mut schema_builder = Schema::builder();
    let text = schema_builder.add_text_field("text", TEXT);
    let id = schema_builder.add_text_field("id", STRING);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    let mut writer = index.writer_for_tests()?;

    for doc_id in 0..150 {
        let body = if doc_id % 3 == 0 {
            "alpha beta beta"
        } else if doc_id % 2 == 0 {
            "alpha"
        } else {
            "beta"
        };
        writer.add_document(doc!(text => body, id => format!("doc-{doc_id}")))?;
        if doc_id == 79 {
            writer.commit()?;
        }
    }
    writer.commit()?;
    for deleted_doc in [0, 31, 63, 79, 80, 100, 149] {
        writer.delete_term(Term::from_field_text(id, &format!("doc-{deleted_doc}")));
    }
    writer.commit()?;

    let searcher = index.reader()?.searcher();
    assert_eq!(searcher.segment_readers().len(), 2);
    let parser = QueryParser::for_index(&index, vec![text]);
    let union_query = parser.parse_query("alpha OR beta")?;
    let term_query = TermQuery::new(
        Term::from_field_text(text, "alpha"),
        IndexRecordOption::WithFreqs,
    );
    assert_weight_scalar_block_parity(&term_query, &searcher)?;
    assert_weight_scalar_block_parity(union_query.as_ref(), &searcher)?;

    let standalone_count = searcher.search(union_query.as_ref(), &Count)?;
    let standalone_top = searcher.search(
        union_query.as_ref(),
        &TopDocs::with_limit(50).order_by_score(),
    )?;
    let (tuple_count, tuple_top) = searcher.search(
        union_query.as_ref(),
        &(Count, TopDocs::with_limit(50).order_by_score()),
    )?;
    assert_eq!(tuple_count, standalone_count);
    assert_eq!(tuple_count, 143);
    assert_eq!(tuple_top, standalone_top);

    let mut collectors = MultiCollector::new();
    let count_handle = collectors.add_collector(Count);
    let top_handle = collectors.add_collector(TopDocs::with_limit(50).order_by_score());
    let mut fruits = searcher.search(union_query.as_ref(), &collectors)?;
    assert_eq!(count_handle.extract(&mut fruits), standalone_count);
    assert_eq!(top_handle.extract(&mut fruits), standalone_top);
    Ok(())
}
