//! Cold-start benchmark for the `embedded-font` cargo feature.
//!
//! Measures the cost of populating a fresh `fontdb::Database`, which
//! is the first-render dominating cost the `embedded-font` feature
//! eliminates. This bench talks to `fontdb` directly rather than
//! through `render()` because Criterion's per-sample amortisation
//! makes "first-call" latency invisible when the library carries a
//! process-wide lazy singleton.
//!
//! Run with:
//!   cargo bench --bench font_startup                     # fontdb filesystem scan
//!   cargo bench --bench font_startup --features embedded-font  # bundled bytes
//!
//! Expected ordering (measured on macOS arm64, Apple M-series,
//! 2026-04-23): the embedded path costs ~1.1 us per fresh
//! `Database` (zero-copy via `Source::Binary(Arc<&'static [u8]>)`,
//! so the 822 KB of font bytes stay in rodata), vs ~11 ms for the
//! filesystem scan -- roughly 10,000x faster. The CHANGELOG
//! records the absolute numbers; this bench is the reproducer.

use criterion::{Criterion, criterion_group, criterion_main};
use fontdb::Database;

/// Inter Regular bytes, included when the feature is enabled, used by
/// the embedded-font bench arm.
#[cfg(feature = "embedded-font")]
const EMBEDDED_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/Inter-Regular.ttf");

/// Inter Bold bytes, used by the embedded-font bench arm.
#[cfg(feature = "embedded-font")]
const EMBEDDED_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");

/// Baseline: constructing an empty `Database` costs nothing. This
/// lets the reader subtract any overhead from the loaded-db numbers.
fn bench_db_new(c: &mut Criterion) {
    c.bench_function("db_new_empty", |b| {
        b.iter(|| std::hint::black_box(Database::new()));
    });
}

/// Without the `embedded-font` feature, a fresh `TextMeasurer`
/// eventually calls `db.load_system_fonts()` to resolve any
/// sans-serif query. This bench measures that filesystem scan
/// directly. Each sample creates a fresh `Database`, so the scan
/// pays its full cost every iteration -- exactly the first-render
/// cost the embedded-font feature replaces.
#[cfg(not(feature = "embedded-font"))]
fn bench_db_load_fonts(c: &mut Criterion) {
    c.bench_function("db_load_system_fonts", |b| {
        b.iter_with_large_drop(|| {
            let mut db = Database::new();
            db.load_system_fonts();
            db
        });
    });
}

/// With the `embedded-font` feature, the two bundled font files are
/// pushed into a fresh `Database` via `load_font_source` with
/// `Arc<&'static [u8]>`, which references the static bytes without
/// allocating on the heap. Same scheme as above: each sample
/// constructs fresh state so the number is the cold-path cost per
/// new process.
#[cfg(feature = "embedded-font")]
fn bench_db_load_fonts(c: &mut Criterion) {
    use std::sync::Arc;
    c.bench_function("db_load_embedded_fonts", |b| {
        b.iter_with_large_drop(|| {
            let mut db = Database::new();
            db.load_font_source(fontdb::Source::Binary(Arc::new(EMBEDDED_REGULAR)));
            db.load_font_source(fontdb::Source::Binary(Arc::new(EMBEDDED_BOLD)));
            db
        });
    });
}

criterion_group!(benches, bench_db_new, bench_db_load_fonts);
criterion_main!(benches);
