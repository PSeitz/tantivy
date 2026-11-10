use std::net::Ipv6Addr;

use proptest::prelude::*;

use super::*;
use crate::DateTime;
use crate::schema::{Facet, STORED, TEXT};
use crate::tokenizer::PreTokenizedString;

#[derive(Debug)]
struct TestDocument(Vec<(Field, OwnedValue)>);

impl Document for TestDocument {
    type Value<'a> = &'a OwnedValue;
    type FieldsValuesIter<'a> = std::vec::IntoIter<(Field, &'a OwnedValue)>;

    fn iter_fields_and_values(&self) -> Self::FieldsValuesIter<'_> {
        self.0
            .iter()
            .map(|(field, value)| (*field, value))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

fn text_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        8 => proptest::collection::vec(any::<char>(), 0..32)
            .prop_map(|chars| chars.into_iter().collect()),
        4 => proptest::sample::select(vec![
            String::new(), "Other".into(), "log.message".into(), "shared".into(),
            "customer acme successfully processed 12345 requests! 東京\n".into(),
        ]),
        1 => (u16::MAX as usize..=u16::MAX as usize + 2).prop_map(|len| "x".repeat(len)),
        1 => Just("customer acme processed 123 requests! 東京\n".repeat(128)),
    ]
}

fn value_strategy() -> impl Strategy<Value = OwnedValue> {
    let leaf = prop_oneof![
        Just(OwnedValue::Null),
        text_strategy().prop_map(OwnedValue::Str),
        any::<u64>().prop_map(OwnedValue::U64),
        any::<i64>().prop_map(OwnedValue::I64),
        prop_oneof![
            any::<u64>(),
            proptest::sample::select(vec![
                0,
                1 << 63,
                0x7ff0_0000_0000_0000,
                0xfff0_0000_0000_0000,
                0x7ff8_0000_0000_0123,
                0xfff8_0000_0000_0042
            ]),
        ]
        .prop_map(|bits| OwnedValue::F64(f64::from_bits(bits))),
        any::<bool>().prop_map(OwnedValue::Bool),
        any::<i64>().prop_map(|nanos| OwnedValue::Date(DateTime::from_timestamp_nanos(nanos))),
        text_strategy().prop_map(|text| OwnedValue::Facet(Facet::from_path([text]))),
        proptest::collection::vec(any::<u8>(), 0..32).prop_map(OwnedValue::Bytes),
        proptest::collection::vec(any::<u8>(), 0..32).prop_map(OwnedValue::Custom),
        any::<u128>().prop_map(|ip| OwnedValue::IpAddr(Ipv6Addr::from(ip))),
    ];
    leaf.prop_recursive(3, 32, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..4).prop_map(OwnedValue::Array),
            // A Vec preserves duplicate keys and their order.
            proptest::collection::vec((text_strategy(), inner), 0..4).prop_map(OwnedValue::Object),
        ]
    })
}

// PartialEq treats NaNs as unequal and +0/-0 as equal. Staging must preserve bits.
fn same_value(left: &OwnedValue, right: &OwnedValue) -> bool {
    match (left, right) {
        (OwnedValue::F64(left), OwnedValue::F64(right)) => left.to_bits() == right.to_bits(),
        (OwnedValue::Array(left), OwnedValue::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(l, r)| same_value(l, r))
        }
        (OwnedValue::Object(left), OwnedValue::Object(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|((lk, lv), (rk, rv))| lk == rk && same_value(lv, rv))
        }
        _ => left == right,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn staged_batches_roundtrip(
        batch in proptest::collection::vec(
            proptest::collection::vec((any::<bool>(), value_strategy()), 0..6), 0..5),
        max_fields_per_path in 1usize..5,
        load_priorities in any::<[u64; 6]>(),
        reverse_load_order in any::<bool>(),
    ) {
        let mut schema = Schema::builder();
        let json = schema.add_json_field("json", STORED);
        let message = schema.add_text_field("message", TEXT);
        let schema = schema.build();
        // Exercise a count that grows beyond the encoder's one-byte reservation.
        let mut documents = vec![TestDocument(Vec::new()), TestDocument(vec![
            (json, OwnedValue::Array(vec![OwnedValue::Null; 128])),
        ])];
        documents.extend(batch.into_iter().map(|fields| TestDocument(fields.into_iter()
            .map(|(is_message, value)| (if is_message { message } else { json }, value)).collect())));

        // Cover both narrow and wide layout IDs.
        for wide_layout_ids in [false, true] {
            let mut compressor = DocumentCompressor::new(CompressionConfig {
                max_fields_per_path,
                message_fields: vec!["message".into(), r"json.log\.message".into()],
            }, &schema).unwrap();
            if wide_layout_ids {
                // Layout IDs beyond 63 also need a wider header.
                for id in 0usize..64 {
                    compressor.layouts.get_or_insert_id(&id.to_le_bytes(), &mut compressor.arena);
                }
            }
            let mut load_order = Vec::new();
            for _ in 0..2 {
                for (source_index, document) in documents.iter().enumerate() {
                    load_order.push((compressor.compress(document).unwrap(), source_index));
                    prop_assert!(compressor.key_tree.iter().all(|node| {
                        node.children.len() < max_fields_per_path
                    }), "admission must cap children at every path from the first document");
                }
            }
            prop_assert!(compressor.key_tree.len() <= MAX_PATH_NODES);
            prop_assert!(compressor.path_name_bytes <= MAX_PATH_NAME_BYTES);
            load_order.sort_by_key(|(_, source_index)| load_priorities[*source_index]);
            if reverse_load_order {
                load_order.reverse();
            }
            let mut staged = compressor.into_staged_document();
            for (document, source_index) in load_order {
                staged.load(document).unwrap();
                let actual: Vec<_> = staged.iter_fields_and_values()
                    .map(|(field, value)| (field, OwnedValue::from(value.as_value()))).collect();
                let expected = &documents[source_index].0;
                prop_assert_eq!(actual.len(), expected.len());
                for ((field, value), (expected_field, expected_value)) in actual.iter().zip(expected) {
                    prop_assert_eq!(field, expected_field);
                    prop_assert!(same_value(value, expected_value),
                        "document {}, field {:?}: {:?} != {:?}", source_index, field, value, expected_value);
                }
            }
        }
    }
}

#[test]
fn rejects_pre_tokenized_strings() {
    let mut schema = Schema::builder();
    let field = schema.add_text_field("text", TEXT);
    let mut compressor = DocumentCompressor::new(Default::default(), &schema.build()).unwrap();
    let pre_tokenized = OwnedValue::PreTokStr(PreTokenizedString {
        text: "hello".into(),
        tokens: Vec::new(),
    });
    for value in [
        pre_tokenized.clone(),
        OwnedValue::Object(vec![(
            "nested".into(),
            OwnedValue::Array(vec![pre_tokenized]),
        )]),
    ] {
        let error = compressor
            .compress(&TestDocument(vec![(field, value)]))
            .unwrap_err();
        assert!(matches!(error, TantivyError::InvalidArgument(_)));
        assert!(
            error
                .to_string()
                .contains("does not support pre-tokenized strings")
        );
    }
    let document = TestDocument(vec![(field, OwnedValue::Str("hello".into()))]);
    let encoded = compressor.compress(&document).unwrap();
    let mut staged = compressor.into_staged_document();
    staged.load(encoded).unwrap();
    let (_, value) = staged.iter_fields_and_values().next().unwrap();
    assert_eq!(OwnedValue::from(value.as_value()), document.0[0].1);
}

#[test]
fn key_admission_preserves_existing_keys_and_ids() {
    let mut schema = Schema::builder();
    let json = schema.add_json_field("json", STORED);
    let mut compressor = DocumentCompressor::new(
        CompressionConfig {
            max_fields_per_path: 2,
            ..Default::default()
        },
        &schema.build(),
    )
    .unwrap();
    let first = TestDocument(vec![(
        json,
        OwnedValue::Object(vec![(
            "first".into(),
            OwnedValue::Str("first value".into()),
        )]),
    )]);
    let later = TestDocument(vec![(
        json,
        OwnedValue::Object(vec![(
            "later".into(),
            OwnedValue::Object(vec![(
                "nested".into(),
                OwnedValue::Str("later value".into()),
            )]),
        )]),
    )]);
    let first_encoded = compressor.compress(&first).unwrap();
    let root = compressor.field_roots[&json];
    let first_node = compressor.key_tree[root].children["first"];
    let first_text_id = compressor.texts.get_id(b"first value", &compressor.arena);
    let mut later_encoded = compressor.compress(&later).unwrap();
    let nodes = compressor.key_tree.len();
    let terms = (
        compressor.texts.ids_by_bytes.len(),
        compressor.layouts.ids_by_bytes.len(),
    );
    for _ in 0..10_001 {
        later_encoded = compressor.compress(&later).unwrap();
    }
    assert_eq!(compressor.key_tree.len(), nodes);
    assert_eq!(
        (
            compressor.texts.ids_by_bytes.len(),
            compressor.layouts.ids_by_bytes.len()
        ),
        terms
    );
    assert_eq!(compressor.key_tree[root].children.len(), 1);
    assert_eq!(compressor.key_tree[root].children["first"], first_node);
    assert!(!compressor.key_tree[root].children.contains_key("later"));
    assert_eq!(
        compressor.texts.get_id(b"first value", &compressor.arena),
        first_text_id
    );
    let repeated = compressor.compress(&first).unwrap();
    assert_eq!(
        repeated.bytes(&compressor.arena),
        first_encoded.bytes(&compressor.arena)
    );

    let mut staged = compressor.into_staged_document();
    for (encoded, expected) in [(first_encoded, first), (later_encoded, later)] {
        staged.load(encoded).unwrap();
        let actual: Vec<_> = staged
            .iter_fields_and_values()
            .map(|(field, value)| (field, OwnedValue::from(value.as_value())))
            .collect();
        assert_eq!(actual, expected.0);
    }
}

#[test]
fn global_key_budgets_preserve_existing_paths() {
    let mut schema = Schema::builder();
    let json = schema.add_json_field("json", STORED);
    let other = schema.add_json_field("other", STORED);
    let schema = schema.build();
    let mut compressor = DocumentCompressor::new(
        CompressionConfig {
            max_fields_per_path: MAX_PATH_NODES + 1,
            ..Default::default()
        },
        &schema,
    )
    .unwrap();
    let root = compressor.get_or_add_root(json).unwrap();
    for i in 1..MAX_PATH_NODES {
        assert_eq!(
            compressor.get_or_add_child(Some(root), &i.to_string()),
            Some(i)
        );
    }
    assert_eq!(compressor.get_or_add_child(Some(root), "overflow"), None);
    assert_eq!(compressor.get_or_add_root(other), None);
    assert_eq!(compressor.get_or_add_root(json), Some(root));
    assert_eq!(compressor.get_or_add_child(Some(root), "1"), Some(1));
    assert_eq!(compressor.key_tree.len(), MAX_PATH_NODES);

    let mut compressor = DocumentCompressor::new(CompressionConfig::default(), &schema).unwrap();
    let root = compressor.get_or_add_root(json).unwrap();
    let name = "x".repeat(MAX_PATH_NAME_BYTES);
    let child = compressor.get_or_add_child(Some(root), &name).unwrap();
    assert_eq!(compressor.path_name_bytes, MAX_PATH_NAME_BYTES);
    assert_eq!(compressor.get_or_add_child(Some(root), "overflow"), None);
    assert_eq!(compressor.get_or_add_child(Some(child), "nested"), None);
    assert_eq!(compressor.get_or_add_child(Some(root), &name), Some(child));
    assert_eq!(compressor.key_tree.len(), 2);
}

#[test]
fn dictionary_deduplication_and_memory_usage() {
    let mut arena = MemoryArena::default();
    let mut dictionary = Dictionary::default();
    assert_eq!(dictionary.get_or_insert_id(b"", &mut arena), None);
    assert_eq!(dictionary.get_or_insert_id(b"first", &mut arena), Some(0));
    assert_eq!(dictionary.get_or_insert_id(b"second", &mut arena), Some(1));
    assert_eq!(dictionary.get_or_insert_id(b"first", &mut arena), Some(0));
    assert_eq!(dictionary.get_id(b"first", &arena), Some(0));
    assert_eq!(dictionary.get_id(b"missing", &arena), None);
    assert_eq!(dictionary.ids_by_bytes.len(), 2);
    assert_eq!(dictionary.mem_usage(), dictionary.ids_by_bytes.mem_usage());
    let memory = (dictionary.mem_usage(), arena.len());
    assert_eq!(dictionary.get_or_insert_id(b"second", &mut arena), Some(1));
    assert_eq!((dictionary.mem_usage(), arena.len()), memory);

    let mut key = vec![b'x'; u16::MAX as usize];
    assert_eq!(dictionary.get_or_insert_id(&key, &mut arena), Some(2));
    let mut expected = vec![b"first".to_vec(), b"second".to_vec(), key.clone()];
    let memory = (dictionary.mem_usage(), arena.len());
    for suffix in [b'a', b'b'] {
        key.truncate(u16::MAX as usize);
        key.push(suffix);
        assert_eq!(dictionary.get_or_insert_id(&key, &mut arena), None);
        assert_eq!(dictionary.get_id(&key, &arena), None);
    }
    assert_eq!((dictionary.mem_usage(), arena.len()), memory);

    // IDs and byte addresses remain valid through hash-table and arena growth.
    for id in 3u32..4096 {
        let bytes = (id as usize).to_le_bytes().repeat(64);
        assert_eq!(dictionary.get_or_insert_id(&bytes, &mut arena), Some(id));
        expected.push(bytes);
    }
    for (id, bytes) in expected.iter().enumerate() {
        assert_eq!(
            dictionary.get_or_insert_id(bytes, &mut arena),
            Some(id as u32)
        );
        assert_eq!(dictionary.get_id(bytes, &arena), Some(id as u32));
    }
    assert!(arena.mem_usage() > 2 * 1024 * 1024);
    let ranges = dictionary.ranges(&arena);
    assert_eq!(
        ranges
            .iter()
            .map(|range| range.bytes(&arena))
            .collect::<Vec<_>>(),
        expected
    );
    for (bytes, address) in dictionary.ids_by_bytes.iter(&arena) {
        let id = arena.read::<u32>(address);
        assert_eq!(ranges[id as usize].bytes(&arena).as_ptr(), bytes.as_ptr());
    }
    assert!(Dictionary::default().ranges(&arena).is_empty());

    let mut compressor =
        DocumentCompressor::new(CompressionConfig::default(), &Schema::builder().build()).unwrap();
    compressor.arena = arena;
    compressor.texts = dictionary;
    let mut text = vec![type_codes::TEXT_CODE];
    compressor.encode_text(&mut text, "already shared message");
    let entries = compressor.texts.ids_by_bytes.len();
    let mut message = Vec::new();
    compressor.encode_message(&mut message, "already shared message");
    assert_eq!(message, text);
    assert_eq!(compressor.texts.ids_by_bytes.len(), entries);
}

#[test]
fn expanded_size_counts_every_document_independently_of_compression() {
    let mut schema = Schema::builder();
    let json = schema.add_json_field("json", TEXT | STORED);
    let schema = schema.build();
    let mut values: Vec<_> = (0..100)
        .map(|i| (format!("long_repeated_key_{i}"), OwnedValue::U64(i)))
        .collect();
    values.push((
        "message".into(),
        OwnedValue::Str("customer acme processed 123 requests! ".repeat(100)),
    ));
    let document = TestDocument(vec![(json, OwnedValue::Object(values))]);
    let expected = size_of::<Field>() + estimate_owned_value_bytes(&document.0[0].1).unwrap();
    for max_fields_per_path in [1, 64, 256] {
        for message_fields in [vec![], vec!["json.message".into()]] {
            let mut compressor = DocumentCompressor::new(
                CompressionConfig {
                    max_fields_per_path,
                    message_fields,
                },
                &schema,
            )
            .unwrap();
            assert_eq!(compressor.expanded_document_bytes(), 0);
            compressor.compress(&document).unwrap();
            let entries = (
                compressor.texts.ids_by_bytes.len(),
                compressor.layouts.ids_by_bytes.len(),
            );
            let headroom = compressor.indexing_headroom_bytes();
            let reconstructed = compressor.reconstructed_text_bytes;
            for count in 2..=10 {
                compressor.compress(&document).unwrap();
                assert_eq!(compressor.expanded_document_bytes(), count * expected);
                assert_eq!(compressor.reconstructed_text_bytes, count * reconstructed);
                assert_eq!(
                    compressor.indexing_headroom_bytes(),
                    headroom + 2 * (count - 1) * reconstructed
                );
                assert_eq!(
                    entries,
                    (
                        compressor.texts.ids_by_bytes.len(),
                        compressor.layouts.ids_by_bytes.len()
                    )
                );
            }
            compressor.expanded_document_bytes = usize::MAX;
            compressor.compress(&document).unwrap();
            assert_eq!(compressor.expanded_document_bytes(), usize::MAX);
        }
    }
}

#[test]
fn arena_ranges_preserve_bytes_at_page_boundaries() {
    let mut arena = MemoryArena::default();
    let bytes = vec![b'x'; 1024 * 1024];
    let first = ByteRange::store(&mut arena, &bytes).unwrap();
    let address = first.bytes(&arena).as_ptr();
    let mut remaining = first;
    assert_eq!(remaining.take_prefix(bytes.len()), first);
    // An empty range can point just past the end of a page.
    assert!(remaining.bytes(&arena).is_empty());
    assert!(remaining.take_prefix(0).bytes(&arena).is_empty());
    let second = ByteRange::store(&mut arena, &bytes).unwrap();
    assert_eq!(arena.mem_usage(), 2 * 1024 * 1024);
    assert_ne!(first, second);
    assert_eq!(first.bytes(&arena), bytes);
    assert_eq!(first.bytes(&arena).as_ptr(), address);
    let allocated = arena.len();
    assert!(
        ByteRange::store(&mut arena, &[])
            .unwrap()
            .bytes(&arena)
            .is_empty()
    );
    assert!(ByteRange::store(&mut arena, &vec![0; 1024 * 1024 + 1]).is_err());
    assert_eq!(arena.len(), allocated);
}

#[test]
fn staging_reports_the_one_mib_allocation_limit() {
    let mut schema = Schema::builder();
    let payload = schema.add_bytes_field("payload", STORED);
    let message = schema.add_text_field("message", STORED);
    let mut compressor = DocumentCompressor::new(
        CompressionConfig {
            message_fields: vec!["message".into()],
            ..Default::default()
        },
        &schema.build(),
    )
    .unwrap();
    let bytes = doc!(payload => vec![42u8; 1024 * 1024 - 32]);
    let encoded_bytes = compressor.compress(&bytes).unwrap();
    // Token compression must enforce the limit on reconstructed text too.
    let text = "word ".repeat((1024 * 1024) / 5);
    let encoded_message = compressor.compress(&doc!(message => text.clone())).unwrap();
    assert_eq!(compressor.reconstructed_text_bytes, text.len());
    for oversized in [
        doc!(payload => vec![42u8; 1024 * 1024]),
        doc!(message => format!("{text}word ")),
    ] {
        let error = compressor.compress(&oversized).unwrap_err();
        assert!(error.to_string().contains("1 MiB per allocation"));
    }
    let mut staged = compressor.into_staged_document();
    staged.load(encoded_bytes).unwrap();
    assert_eq!(staged.to_tantivy_document(), bytes);
    staged.load(encoded_message).unwrap();
    assert_eq!(staged.to_tantivy_document(), doc!(message => text));
}

#[test]
fn config_defaults_and_validation() {
    assert_eq!(CompressionConfig::default().max_fields_per_path, 256);
    assert_eq!(
        serde_json::from_str::<CompressionConfig>("{}").unwrap(),
        CompressionConfig::default()
    );
    assert!(serde_json::from_str::<CompressionConfig>(r#"{"typo":1}"#).is_err());
    let mut schema = Schema::builder();
    let message = schema.add_text_field("message", TEXT);
    let json = schema.add_json_field("json", STORED);
    schema.add_u64_field("number", STORED);
    let schema = schema.build();
    for (path, max_fields_per_path) in [
        ("missing", 1),
        ("message.child", 1),
        ("number", 1),
        ("message", 0),
    ] {
        assert!(
            DocumentCompressor::new(
                CompressionConfig {
                    message_fields: vec![path.into()],
                    max_fields_per_path,
                },
                &schema
            )
            .is_err()
        );
    }
    let config = CompressionConfig {
        message_fields: vec!["message".into(), r"json.log\.message".into()],
        max_fields_per_path: 1,
    };
    assert_eq!(
        serde_json::from_str::<CompressionConfig>(&serde_json::to_string(&config).unwrap())
            .unwrap(),
        config
    );
    let compressor = DocumentCompressor::new(config, &schema).unwrap();
    assert!(compressor.uses_message_tokens(message, &[]));
    assert!(compressor.uses_message_tokens(json, &["log.message"]));
    assert!(!compressor.uses_message_tokens(json, &["log", "message"]));
    assert!(!compressor.uses_message_tokens(json, &[]));
}
