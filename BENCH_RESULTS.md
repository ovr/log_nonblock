### Benchmarks

# `simple_logger`

`cargo run --example blocking_test simple_logger`

Results:

``
Results:
  Total time: 0.217s
  Throughput: 46079.12 messages/sec
  Throughput: 45.00 MB/sec
``

# `log_nonblock`

`cargo run --example blocking_test log_nonblock`

Results:

``
Results:
  Logging time: 0.017s
  Flush time: 0.129s
  Total time: 0.147s
  Throughput (logging): 576817.70 messages/sec
  Throughput (logging): 563.30 MB/sec
  Throughput (total): 68102.96 messages/sec
  Throughput (total): 66.51 MB/sec
``

### Results

As you can see, `log_nonblock` is much faster than `simple_logger`.
`0.017s` vs `0.217s`, at throughput of `563.30 MB/sec` vs `45.00 MB/sec`.

It's the result, that `log_nonblock` logs asynchronously by using a dedicated thread and channel.
