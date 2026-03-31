# Plan: Rust Unit Tests

## Context

The extension has E2E tests (Pest/PHP) but no Rust-level unit tests. The goal is to add `#[cfg(test)]` modules that test pure logic — header extraction, message state management, consumer channel logic, and command dispatch — without needing a live RabbitMQ instance.

## Key Challenge

The crate uses `ext-php-rs` macros (`#[php_class]`, `#[php_impl]`) which generate PHP C API bindings. For `cargo test` to work:
1. Add `"rlib"` to `crate-type` in Cargo.toml (cdylib-only crates can't produce test binaries)
2. PHP dev headers must be installed (ext-php-rs build script links against libphp)

If this doesn't compile, fallback: extract pure logic into a `core` module without ext-php-rs annotations.

## Changes

### 1. `Cargo.toml` — add rlib crate type

```toml
crate-type = ["cdylib", "rlib"]
```

### 2. `src/worker.rs` — refactor `extract_headers` + add tests

**Refactor**: Change `extract_headers` from taking `&Delivery` to `Option<&FieldTable>` so it can be tested without constructing a Delivery (which has private fields). Update the call site in `next_delivery` to pass `delivery.properties.headers().as_ref()`.

**Tests** (`#[cfg(test)] mod tests`):
- Empty/None headers → empty HashMap
- LongString value → UTF-8 string
- ShortString value → string
- Boolean → "true"/"false"
- Integer types (ShortInt, LongInt, LongLongInt, ShortUInt, LongUInt) → decimal string
- Float/Double → decimal string
- Timestamp → decimal string
- Unknown type → Debug format
- Multiple headers in one table

### 3. `src/message.rs` — add tests

**Tests**:
- `get_body()` with valid UTF-8 bytes
- `get_body()` with invalid UTF-8 (lossy replacement)
- `get_body()` with empty body
- `get_routing_key()`, `get_exchange()`, `get_delivery_tag()` return correct values
- `get_headers()` returns clone of headers
- `ensure_not_acked()` succeeds first call, fails second call
- `ack()` sends `Command::Ack` with correct delivery_tag
- `nack()` defaults requeue to true, sends correct command
- `nack(false)` sends requeue=false
- `reject()` defaults requeue to true
- `ack()` on closed channel returns error
- Double `ack()` returns error before reaching channel

Helper: create `Message` via `Message::new()` with a `tokio::sync::mpsc::unbounded_channel()` — receive from `rx` to verify commands.

### 4. `src/consumer.rs` — add tests

**Tests**:
- `recv_event` with timeout > 0: returns event when available
- `recv_event` with timeout > 0: returns None on timeout
- `recv_event` with timeout > 0: returns error on disconnect
- `recv_event` with timeout <= 0: blocks and returns event
- `event_to_message` with `Delivery` variant → Some(Message)
- `event_to_message` with `ConsumerCancelled` → None
- `event_to_message` with `Error` → Err
- `next()` returns None when cancelled
- `next()` sets cancelled on ConsumerCancelled
- `next()` with timeout returns None on empty channel
- `cancel()` sends Unsubscribe with correct consumer_tag
- `cancel()` on already-cancelled consumer is no-op

Helper: create `Consumer` via `Consumer::new()` with crossbeam and tokio channels.

### 5. `.github/workflows/test.yml` — add `cargo test` step

Add a step between "Build extension" and "Install Composer dependencies":
```yaml
- name: Run Rust tests
  run: cargo test
```

## Files to modify

| File | Action |
|------|--------|
| `Cargo.toml` | Add `"rlib"` to crate-type |
| `src/worker.rs` | Refactor `extract_headers` signature, add `#[cfg(test)] mod tests` |
| `src/message.rs` | Add `#[cfg(test)] mod tests` |
| `src/consumer.rs` | Add `#[cfg(test)] mod tests` |
| `.github/workflows/test.yml` | Add `cargo test` step |

## Verification

1. `cargo test` — all unit tests pass locally without RabbitMQ
2. `cargo build --release` — extension still builds as cdylib
3. Pest E2E tests still pass against RabbitMQ
