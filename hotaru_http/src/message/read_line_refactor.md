# read_until / read_line: Replace `bool` Truncation Signal with `ReadLimitError`

## Summary

Replaced the `bool` truncation signal in `HotaruBufRead::read_until` / `read_line` with `Err(Self::Error)` carrying partial data via the `ReadLimitError` sub-trait.

## Key Changes

### 1. `ReadLimitError` sub-trait (`error.rs`)

A refinement of `core::error::Error` (the associated type on read.rs:9) — only error types that can carry truncation data implement it:

```rust
pub trait ReadLimitError: core::error::Error + Send + Sync + 'static {
    fn rate_limit_error(data: Vec<u8>) -> Self;
    fn get_read(&self) -> &[u8] { &[] }
}
```

- `rate_limit_error` takes ownership of `Vec<u8>` (no clone)
- `get_read` retrieves the data back from the error; defaults to `&[]`
- `std::io::Error` impl: stores data via internal `RateLimitData(Vec<u8>)` payload, retrieved with `downcast_ref`
- `HotaruIOError` impl: `SizeExceeded(Vec<u8>)` variant (simplified, no separate `limit` field)
- `EmbeddedIoError` impl: same pattern

### 2. `read_until` / `read_line` signature change (`read.rs`)

| | Old | New |
|---|---|---|
| Return | `Result<(usize, bool), E>` | `Result<usize, E>` |
| Truncation | `Ok((n, true))` | `Err(ReadLimitError::rate_limit_error(mem::take(buf)))` |
| Bound | — | `Self::Error: ReadLimitError` |

**No data loss on truncation**: `read_until` uses `core::mem::take(buf)` to move the Vec into the error; `read_line` writes partial data back to the caller's buffer via `get_read` before returning `Err`:

```rust
Err(e) => {
    buf.push_str(&String::from_utf8_lossy(
        ReadLimitError::get_read(&e),
    ));
    Err(e)
}
```

### 3. Call-site simplification

**meta.rs** — removed `map_err` + `if truncated`, bare `?` suffices:

```rust
// Old (10 lines)
let (bytes_read, truncated) = buf_reader.read_line(...).await
    .map_err(|_| InternalServerError(...))?;
if truncated { return Err(PayloadTooLarge); }

// New (1 line)
let bytes_read = buf_reader.read_line(...).await?;
```

**body.rs** — same:

```rust
// Old (10 lines)
let (_, truncated) = buf_reader.read_line(...).await?;
if truncated { return Err(io::Error::new(InvalidData, ...)); }

// New (1 line)
let _ = buf_reader.read_line(...).await?;
```

## Zero-change Files

- `buf_reader.rs` — `Error = R::Error`, `ReadLimitError` auto-forwards
- `hotaru_io_tokio` — uses `std::io::Error`, single impl in hotaru_core covers it
- `hotaru_io_futures` — same
- `hotaru_tls` — same
- `test_support.rs` — `TestWire` (`Error = Infallible`) does not implement `ReadLimitError`

## Design Notes

1. **Truncation is not "failed to read"**: data was successfully read into the buffer; `Err` only signals the delimiter wasn't found
2. **Sub-trait pattern**: `ReadLimitError` refines `core::error::Error` — not every error type must implement it
3. **Ownership transfer**: `rate_limit_error` takes `Vec<u8>` (not `&[u8]`), moved out via `mem::take` with zero copy
4. **Reversible**: `get_read` retrieves data from the error so callers can recover partial results
5. **Zero-cost at call sites**: `?` propagates naturally; error info is preserved for upstream status-code mapping
