use algorithm::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LOREIPSUM: &[u8] = include_bytes!("../../loreipsum.txt");

pub fn bench_compress_bytes(c: &mut Criterion) {
    c.bench_function("compress", |b| {
        b.iter(|| {
            let tree = generate_huffman_tree(LOREIPSUM).unwrap();
            let table = tree.generate_code_table();
            let compressed = table.encode_bytes(LOREIPSUM);
            black_box(compressed)
        })
    });
}

pub fn bench_decompress_bytes(c: &mut Criterion) {
    let tree = generate_huffman_tree(LOREIPSUM).unwrap();
    let table = tree.generate_code_table();
    let compressed = table.encode_bytes(LOREIPSUM).unwrap();
    c.bench_function("decompress", |b| {
        b.iter(|| {
            let uncompress = tree.decode_bytes(&compressed);
            black_box(uncompress)
        })
    });
}

pub fn bench_compress_file(c: &mut Criterion) {
    c.bench_function("compress_file", |b| {
        b.iter(|| compress_file("../../loreipsum.txt", "compressed"))
    });
}

pub fn bench_decompress_file(c: &mut Criterion) {
    c.bench_function("decompress_file", |b| {
        b.iter(|| decompress_file("compressed", "decompressed"))
    });
}

criterion_group!(
    benches,
    bench_compress_bytes,
    bench_decompress_bytes,
    bench_compress_file,
    bench_decompress_file
);
criterion_main!(benches);
