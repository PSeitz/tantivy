use std::sync::Arc;

use binggan::{InputGroup, black_box};
use common::{BinarySerializable, DateTime, OwnedBytes, VInt};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tantivy_columnar::column_values::{
    CodecType, ColumnStats, load_u64_based_column_values,
    serialize_and_load_u64_based_column_values, serialize_u64_based_column_values,
};
use tantivy_columnar::{ColumnValues, MonotonicallyMappableToU64};

fn get_data() -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(2u64);
    let mut data: Vec<_> = (100..55_000_u64)
        .map(|num| num + rng.random::<u8>() as u64)
        .collect();
    data.push(99_000);
    data.insert(1000, 2000);
    data.insert(2000, 100);
    data.insert(3000, 4100);
    data.insert(4000, 100);
    data.insert(5000, 800);
    data
}

#[inline(never)]
fn value_iter() -> impl Iterator<Item = u64> {
    0..20_000
}

type Col = Arc<dyn ColumnValues<u64>>;

// Re-encode only the metadata into the read-only V1 format. Keep identical stats and
// residuals, and exercise both versions through public codec dispatch without a V1 writer API.
fn to_v1(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes[0], CodecType::BlockwiseLinearV2 as u8);
    let stats = ColumnStats::deserialize(&mut &bytes[1..]).unwrap();
    let widths = &bytes[bytes.len() - 3..];
    let record_size = widths.iter().map(|&width| width as usize).sum::<usize>() + 1;
    let metadata_start = bytes.len() - 3 - stats.num_rows.div_ceil(512) as usize * record_size;
    let mut output = bytes[..metadata_start].to_vec();
    output[0] = CodecType::BlockwiseLinear as u8;
    for record in bytes[metadata_start..bytes.len() - 3].chunks_exact(record_size) {
        let mut pos = 0;
        for &width in &widths[..2] {
            let mut word = [0; 8];
            word[..width as usize].copy_from_slice(&record[pos..pos + width as usize]);
            VInt(u64::from_le_bytes(word))
                .serialize(&mut output)
                .unwrap();
            pos += width as usize;
        }
        output.push(record[record_size - 1]);
    }
    let footer_len = (output.len() - metadata_start) as u32;
    footer_len.serialize(&mut output).unwrap();
    output
}

fn blockwise_columns<T: MonotonicallyMappableToU64>(
    values: &[T],
) -> Vec<(String, Arc<dyn ColumnValues<T>>)> {
    let mut bytes = Vec::new();
    serialize_u64_based_column_values(&values, &[CodecType::BlockwiseLinearV2], &mut bytes)
        .unwrap();
    [to_v1(&bytes), bytes]
        .into_iter()
        .enumerate()
        .map(|(version, bytes)| {
            let column = load_u64_based_column_values::<T>(OwnedBytes::new(bytes)).unwrap();
            assert_eq!(column.iter().collect::<Vec<_>>(), values);
            let mut output = values.to_vec();
            column.get_range(0, &mut output);
            assert_eq!(output, values);
            (format!("blockwise_linear_v{}", version + 1), column)
        })
        .collect()
}

fn bench_ranges<T: MonotonicallyMappableToU64>(name: &str, values: &[T]) {
    let mut inputs = blockwise_columns(values);
    for codec in [CodecType::Bitpacked, CodecType::Linear] {
        let mut bytes = Vec::new();
        serialize_u64_based_column_values(&values, &[codec], &mut bytes).unwrap();
        inputs.push((
            format!("{codec:?}"),
            load_u64_based_column_values::<T>(OwnedBytes::new(bytes)).unwrap(),
        ));
    }
    let mut group = InputGroup::new_with_inputs(inputs);
    group.register(
        format!("get_range/{name}/64"),
        |column: &Arc<dyn ColumnValues<T>>| {
            let mut output = [column.get_val(0); 64];
            for start in (0..20_000).step_by(64) {
                column.get_range(start, &mut output);
                black_box(&output);
            }
        },
    );
    group.run();
}

fn main() {
    let data = get_data();
    let mut inputs: Vec<(String, Col)> = vec![
        (
            "bitpacked".to_string(),
            serialize_and_load_u64_based_column_values(&data.as_slice(), &[CodecType::Bitpacked]),
        ),
        (
            "linear".to_string(),
            serialize_and_load_u64_based_column_values(&data.as_slice(), &[CodecType::Linear]),
        ),
    ];
    inputs.extend(blockwise_columns(&data));

    let mut group: InputGroup<Col> = InputGroup::new_with_inputs(inputs);

    group.register("fastfield_get", |col: &Col| {
        let mut sum = 0u64;
        for pos in value_iter() {
            sum = sum.wrapping_add(col.get_val(pos as u32));
        }
        black_box(sum);
    });

    group.register("fastfield_get_random", |col: &Col| {
        let mut sum = 0u64;
        for pos in value_iter() {
            sum = sum.wrapping_add(col.get_val((pos * 7919 % col.num_vals() as u64) as u32));
        }
        black_box(sum);
    });
    group.run();

    bench_ranges("u64", &data);
    let signed: Vec<i64> = data.iter().map(|&value| value as i64 - 30_000).collect();
    bench_ranges("i64", &signed);
    let floats: Vec<f64> = signed.iter().map(|&value| value as f64 / 10.0).collect();
    bench_ranges("f64", &floats);
    let dates: Vec<DateTime> = signed
        .iter()
        .map(|&value| DateTime::from_timestamp_nanos(1_700_000_000_000_000_000 + value * 1000))
        .collect();
    bench_ranges("datetime", &dates);

    let mut inputs = Vec::new();
    for len in [512, 65_536, 1_048_576] {
        let vals: Vec<u64> = (0..len).map(|i| i + (i % 17)).collect();
        let mut bytes = Vec::new();
        serialize_u64_based_column_values(
            &vals.as_slice(),
            &[CodecType::BlockwiseLinearV2],
            &mut bytes,
        )
        .unwrap();
        inputs.push((format!("BlockwiseLinearV2/{len}"), OwnedBytes::new(bytes)));
    }
    let mut group: InputGroup<OwnedBytes> = InputGroup::new_with_inputs(inputs);
    group.register("open", |bytes: &OwnedBytes| {
        black_box(load_u64_based_column_values::<u64>(bytes.clone()).unwrap());
    });
    group.run();
}
