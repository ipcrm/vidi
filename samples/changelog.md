# Changelog

All releases are entirely imaginary. Do not rely on any of them.

## 0.7.0 — unreleased

### Added

- `ingest.max_body` config key for capping request body size.
- Optional OTLP exporter for spans.
- A second, slightly-better joke in the error catalog.

### Changed

- Default worker count bumped from 4 to 8.
- `E999` errors are no longer retried[^cosmic].

[^cosmic]: Cosmic errors are interesting — they occasionally resolve on
  their own when the operator makes a sandwich and returns calmer. We've
  replaced the retry with a sternly-worded log line.

### Removed

- ~~XML ingest support.~~ Removed in 0.6; removed again here for emphasis.

## 0.6.0

Highlights:

- Prometheus metrics endpoint.
- Pooled database writes; benchmarks show roughly **3×** throughput under
  concurrent load.
- Replaced the ingest queue's `VecDeque` with a bounded mpsc channel — the
  queue now applies backpressure cleanly rather than growing until OOM.

### Breaking

- The `--quiet` CLI flag has been replaced with `--log-level=warn`.
- `listen` config moved from top-level into the `[daemon]` section.

## 0.5.0

- First release with a real schema validator.
- Added ~~XML~~ CBOR support.
- Logged a plaintext database password in `error` level once, during a
  debugging session; redacted in this release. Anyone running 0.4 should
  rotate their credentials out of an abundance of caution :grimacing:.

## 0.4.0

Feature complete for "accept bytes, write rows." Nothing else.

---

## Older

Releases prior to 0.4 existed only as internal builds. Release notes for
those builds were written on a napkin that has since gone missing.
