//! #25 解压炸弹验证测试
//!
//! 验证四个 `decompress_*` 函数在 `max_size` 限制下能正确拦截解压炸弹。

use hotaru_lib::compression::*;

/// 1 MB 全零数据 gzip 压缩后只有几百字节，但解压回到 1 MB。
fn make_bomb_gzip() -> Vec<u8> {
    let zeros = vec![0u8; 1024 * 1024];
    compress_gzip(&zeros).expect("compress")
}

// ── gzip ──────────────────────────────────────────────────────────

#[test]
fn gzip_normal_decompress_works() {
    let data = compress_gzip(b"hello world").unwrap();
    let result = decompress_gzip(&data, 1024).unwrap();
    assert_eq!(result, b"hello world");
}

#[test]
fn gzip_bomb_rejected() {
    let bomb = make_bomb_gzip();
    let result = decompress_gzip(&bomb, 1024);
    assert!(result.is_err(), "decompression bomb must be rejected");
}

#[test]
fn gzip_exact_boundary_passes() {
    let data = compress_gzip(b"hello world").unwrap();
    let result = decompress_gzip(&data, 11);
    assert!(result.is_ok(), "exact boundary should pass");
}

#[test]
fn gzip_one_byte_over_fails() {
    let data = compress_gzip(b"hello world").unwrap();
    let result = decompress_gzip(&data, 10);
    assert!(result.is_err(), "one byte over must fail");
}

// ── deflate ───────────────────────────────────────────────────────

#[test]
fn deflate_normal_decompress_works() {
    let data = compress_deflate(b"hello world").unwrap();
    let result = decompress_deflate(&data, 1024).unwrap();
    assert_eq!(result, b"hello world");
}

#[test]
fn deflate_bomb_rejected() {
    let zeros = vec![0u8; 1024 * 1024];
    let bomb = compress_deflate(&zeros).unwrap();
    let result = decompress_deflate(&bomb, 1024);
    assert!(result.is_err(), "deflate bomb must be rejected");
}

// ── brotli ────────────────────────────────────────────────────────

#[test]
fn brotli_normal_decompress_works() {
    let data = compress_brotli(b"hello world").unwrap();
    let result = decompress_brotli(&data, 1024).unwrap();
    assert_eq!(result, b"hello world");
}

#[test]
fn brotli_bomb_rejected() {
    let zeros = vec![0u8; 1024 * 1024];
    let bomb = compress_brotli(&zeros).unwrap();
    let result = decompress_brotli(&bomb, 1024);
    assert!(result.is_err(), "brotli bomb must be rejected");
}

// ── zstd ──────────────────────────────────────────────────────────

#[test]
fn zstd_normal_decompress_works() {
    let data = compress_zstd(b"hello world", 1).unwrap();
    let result = decompress_zstd(&data, 1024).unwrap();
    assert_eq!(result, b"hello world");
}

#[test]
fn zstd_bomb_rejected() {
    let zeros = vec![0u8; 1024 * 1024];
    let bomb = compress_zstd(&zeros, 1).unwrap();
    let result = decompress_zstd(&bomb, 1024);
    assert!(result.is_err(), "zstd bomb must be rejected");
}
