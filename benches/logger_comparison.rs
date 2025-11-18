use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use log::LevelFilter;
use log_nonblock::NonBlockingLoggerBuilder;
use std::sync::Once;
use std::thread;

static INIT_LOG_NONBLOCK: Once = Once::new();
static INIT_SIMPLE_LOGGER: Once = Once::new();

// Initialize log_nonblock once
fn init_log_nonblock() {
    INIT_LOG_NONBLOCK.call_once(|| {
        let _logger = NonBlockingLoggerBuilder::new()
            .with_level(LevelFilter::Info)
            .without_timestamps()
            .init()
            .expect("Failed to initialize log_nonblock");
        // Keep logger alive for the duration of the program
        std::mem::forget(_logger);
    });
}

// Initialize simple_logger once
fn init_simple_logger() {
    INIT_SIMPLE_LOGGER.call_once(|| {
        simple_logger::SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .without_timestamps()
            .init()
            .expect("Failed to initialize simple_logger");
    });
}

// Single-threaded benchmarks with different message sizes - log_nonblock
fn bench_log_nonblock_single_thread(c: &mut Criterion) {
    // Initialize logger once for all benchmarks in this group
    init_log_nonblock();

    let message_sizes = vec![
        ("small_100B", "a".repeat(100)),
        ("medium_1KB", "a".repeat(1024)),
        ("large_100KB", "a".repeat(102400)),
    ];

    let mut group = c.benchmark_group("log_nonblock/single_thread");

    for (size_name, message) in message_sizes.iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size_name), message, |b, msg| {
            b.iter(|| {
                log::info!("{}", black_box(msg));
            });
        });
    }

    group.finish();
}

// Single-threaded benchmarks with different message sizes - simple_logger
fn bench_simple_logger_single_thread(c: &mut Criterion) {
    // Initialize logger once for all benchmarks in this group
    init_simple_logger();

    let message_sizes = vec![
        ("small_100B", "a".repeat(100)),
        ("medium_1KB", "a".repeat(1024)),
        ("large_100KB", "a".repeat(102400)),
    ];

    let mut group = c.benchmark_group("simple_logger/single_thread");

    for (size_name, message) in message_sizes.iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size_name), message, |b, msg| {
            b.iter(|| {
                log::info!("{}", black_box(msg));
            });
        });
    }

    group.finish();
}

// Multi-threaded benchmarks - log_nonblock
fn bench_log_nonblock_multi_thread(c: &mut Criterion) {
    init_log_nonblock();

    let thread_counts = vec![2, 4, 8];

    let mut group = c.benchmark_group("log_nonblock/multi_thread");

    for thread_count in thread_counts {
        group.throughput(Throughput::Elements(thread_count as u64 * 100));

        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &tc| {
                b.iter(|| {
                    let handles: Vec<_> = (0..tc)
                        .map(|i| {
                            thread::spawn(move || {
                                for j in 0..100 {
                                    log::info!("Thread {} message {}", i, black_box(j));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Multi-threaded benchmarks - simple_logger
fn bench_simple_logger_multi_thread(c: &mut Criterion) {
    init_simple_logger();

    let thread_counts = vec![2, 4, 8];

    let mut group = c.benchmark_group("simple_logger/multi_thread");

    for thread_count in thread_counts {
        group.throughput(Throughput::Elements(thread_count as u64 * 100));

        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &tc| {
                b.iter(|| {
                    let handles: Vec<_> = (0..tc)
                        .map(|i| {
                            thread::spawn(move || {
                                for j in 0..100 {
                                    log::info!("Thread {} message {}", i, black_box(j));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

// Benchmark with mixed log levels - log_nonblock
fn bench_log_nonblock_mixed_levels(c: &mut Criterion) {
    init_log_nonblock();

    let mut group = c.benchmark_group("log_nonblock/mixed_levels");
    group.throughput(Throughput::Elements(4));

    group.bench_function("filtered", |b| {
        b.iter(|| {
            log::debug!("This should be filtered out");
            log::info!("This is an info message");
            log::warn!("This is a warning message");
            log::error!("This is an error message");
        });
    });

    group.finish();
}

// Benchmark with mixed log levels - simple_logger
fn bench_simple_logger_mixed_levels(c: &mut Criterion) {
    init_simple_logger();

    let mut group = c.benchmark_group("simple_logger/mixed_levels");
    group.throughput(Throughput::Elements(4));

    group.bench_function("filtered", |b| {
        b.iter(|| {
            log::debug!("This should be filtered out");
            log::info!("This is an info message");
            log::warn!("This is a warning message");
            log::error!("This is an error message");
        });
    });

    group.finish();
}

// Benchmark large messages - log_nonblock
fn bench_log_nonblock_large_messages(c: &mut Criterion) {
    init_log_nonblock();

    let mut group = c.benchmark_group("log_nonblock/large_messages");
    group.sample_size(10);

    // Test with 1MB message
    let large_message = "x".repeat(1024 * 1024);
    group.throughput(Throughput::Bytes(large_message.len() as u64));

    group.bench_function("1MB", |b| {
        b.iter(|| {
            log::info!("{}", black_box(&large_message));
            log::logger().flush();
        });
    });

    // Test with 10MB message
    let huge_message = "x".repeat(10 * 1024 * 1024);
    group.throughput(Throughput::Bytes(huge_message.len() as u64));

    group.bench_function("10MB", |b| {
        b.iter(|| {
            log::info!("{}", black_box(&huge_message));
            log::logger().flush();
        });
    });

    group.finish();
}

// Benchmark large messages - simple_logger
fn bench_simple_logger_large_messages(c: &mut Criterion) {
    init_simple_logger();

    let mut group = c.benchmark_group("simple_logger/large_messages");
    group.sample_size(10);

    // Test with 1MB message
    let large_message = "x".repeat(1024 * 1024);
    group.throughput(Throughput::Bytes(large_message.len() as u64));

    group.bench_function("1MB", |b| {
        b.iter(|| {
            log::info!("{}", black_box(&large_message));
        });
    });

    // Test with 10MB message
    let huge_message = "x".repeat(10 * 1024 * 1024);
    group.throughput(Throughput::Bytes(huge_message.len() as u64));

    group.bench_function("10MB", |b| {
        b.iter(|| {
            log::info!("{}", black_box(&huge_message));
        });
    });

    group.finish();
}

// Benchmark overhead of log calls - log_nonblock
fn bench_log_nonblock_overhead(c: &mut Criterion) {
    init_log_nonblock();

    let mut group = c.benchmark_group("log_nonblock/overhead");
    group.throughput(Throughput::Elements(100));

    group.bench_function("100_calls", |b| {
        b.iter(|| {
            for i in 0..100 {
                log::info!("Message {}", black_box(i));
            }
        });
    });

    group.finish();
}

// Benchmark overhead of log calls - simple_logger
fn bench_simple_logger_overhead(c: &mut Criterion) {
    init_simple_logger();

    let mut group = c.benchmark_group("simple_logger/overhead");
    group.throughput(Throughput::Elements(100));

    group.bench_function("100_calls", |b| {
        b.iter(|| {
            for i in 0..100 {
                log::info!("Message {}", black_box(i));
            }
        });
    });

    group.finish();
}

criterion_group!(
    log_nonblock_benches,
    bench_log_nonblock_single_thread,
    bench_log_nonblock_multi_thread,
    bench_log_nonblock_mixed_levels,
    bench_log_nonblock_large_messages,
    bench_log_nonblock_overhead
);

criterion_group!(
    simple_logger_benches,
    bench_simple_logger_single_thread,
    bench_simple_logger_multi_thread,
    bench_simple_logger_mixed_levels,
    bench_simple_logger_large_messages,
    bench_simple_logger_overhead
);

criterion_main!(log_nonblock_benches, simple_logger_benches);
