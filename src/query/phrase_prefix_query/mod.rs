mod phrase_prefix_query;
mod phrase_prefix_scorer;
mod phrase_prefix_weight;

pub use phrase_prefix_query::PhrasePrefixQuery;
pub use phrase_prefix_scorer::PhrasePrefixScorer;
pub use phrase_prefix_weight::PhrasePrefixWeight;

pub(crate) fn prefix_end(prefix_start: &[u8]) -> Option<Vec<u8>> {
    let end = prefix_start.iter().rposition(|&byte| byte != u8::MAX)?;
    let mut res = prefix_start[..=end].to_vec();
    res[end] += 1;
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_end() {
        assert_eq!(prefix_end(b""), None);
        assert_eq!(prefix_end(b"\xfe\xff"), Some(b"\xff".to_vec()));
        assert_eq!(prefix_end(b"aaa"), Some(b"aab".to_vec()));
        assert_eq!(prefix_end(b"aa\xff"), Some(b"ab".to_vec()));
        assert_eq!(prefix_end(b"a\xff\xff"), Some(b"b".to_vec()));
        assert_eq!(prefix_end(b"\xff\xff\xff"), None);
    }
}
