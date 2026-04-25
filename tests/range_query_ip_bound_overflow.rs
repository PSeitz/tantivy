// Regression test: a `RangeQuery` over an IP fast field must not arithmetic-overflow when
// the user supplies `Bound::Excluded(::0)` as an upper bound (or `Bound::Excluded(ipv6_max)`
// as a lower bound). `bound_range_inclusive_ip` in `range_query_fastfield.rs` adjusts an
// excluded bound by `-1` / `+1` on the underlying `u128` representation without checking
// for overflow.
//
// In a debug build (the default for `cargo test`) this panics with
// `attempt to subtract with overflow`. In a release build the value wraps around — for
// `Bound::Excluded(::0)` upper, the end of the inclusive range becomes `u128::MAX`, so the
// query incorrectly matches **every** indexed document instead of returning zero results.

use std::net::IpAddr;
use std::ops::Bound;
use std::str::FromStr;

use tantivy::collector::Count;
use tantivy::query::RangeQuery;
use tantivy::schema::{IntoIpv6Addr, Schema, FAST};
use tantivy::{doc, Index, IndexWriter, Term};

fn ip(addr: &str) -> std::net::Ipv6Addr {
    IpAddr::from_str(addr).unwrap().into_ipv6_addr()
}

fn build_index() -> tantivy::Result<(Index, tantivy::schema::Field)> {
    let mut schema_builder = Schema::builder();
    let ip_field = schema_builder.add_ip_addr_field("ip", FAST);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 50_000_000)?;
    writer.add_document(doc!(ip_field => ip("::1")))?;
    writer.add_document(doc!(ip_field => ip("::2")))?;
    writer.add_document(doc!(ip_field => ip("127.0.0.1")))?;
    writer.commit()?;
    Ok((index, ip_field))
}

#[test]
fn ip_range_excluded_zero_upper_bound_must_be_empty() -> tantivy::Result<()> {
    let (index, ip_field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Unbounded,
        Bound::Excluded(Term::from_field_ip_addr(ip_field, ip("::"))),
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(
        count, 0,
        "no document has an IP strictly less than ::0; underflow in \
         bound_range_inclusive_ip is letting the query match everything"
    );
    Ok(())
}

#[test]
fn ip_range_excluded_max_lower_bound_must_be_empty() -> tantivy::Result<()> {
    let (index, ip_field) = build_index()?;
    let searcher = index.reader()?.searcher();
    let q = RangeQuery::new(
        Bound::Excluded(Term::from_field_ip_addr(
            ip_field,
            ip("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"),
        )),
        Bound::Unbounded,
    );
    let count = searcher.search(&q, &Count)?;
    assert_eq!(
        count, 0,
        "no document has an IP strictly greater than the maximum IPv6 address; \
         overflow in bound_range_inclusive_ip is letting the query match everything"
    );
    Ok(())
}

#[test]
fn ip_range_existing_behaviour_sanity_check() -> tantivy::Result<()> {
    let (index, ip_field) = build_index()?;
    let searcher = index.reader()?.searcher();

    // Sanity: `>= ::1` returns all three documents.
    let q = RangeQuery::new(
        Bound::Included(Term::from_field_ip_addr(ip_field, ip("::1"))),
        Bound::Unbounded,
    );
    assert_eq!(searcher.search(&q, &Count)?, 3);

    // Sanity: `< ::2` returns just `::1`.
    let q = RangeQuery::new(
        Bound::Unbounded,
        Bound::Excluded(Term::from_field_ip_addr(ip_field, ip("::2"))),
    );
    assert_eq!(searcher.search(&q, &Count)?, 1);
    Ok(())
}
