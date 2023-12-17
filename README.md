_High throughput and simple buffered MPSC channel implementation._

`bufchan` is a MPSC channel implementation that prioritises _sender throughput_ over _receiver
latency_.
It uses local buffers to avoid accessing the shared state, only sending items as a batch once a
threshold is reached.
This implementation prioritises sender **throughput**, for example a heavy computation task where
one wants to keep the compute threads as unblocked as possible.

`bufchan` is also [**very simple**](https://www.infoq.com/presentations/Simple-Made-Easy/).
The channel implementation is around 100 lines of code, and uses no `unsafe`.
I would encourage users to read through the source code, it is heavily commented and a great way to
understand channels!

# Benchmarks

A benchmark setup to measure throughput is constructed where a certain number of tasks do some
non-trivial and non-constant work. The tasks are parallelised, with each result getting sent with
the various channel implementation offerings.
The `int` benchmarks are sending 32-bit integers, the other two benchmarks send an array of various
sizing.

```
bench                 fastest       │ slowest       │ median        │ mean 
├─ int                              │               │               │       
│  ├─ bufchan         36.43 ms      │ 44.15 ms      │ 36.88 ms      │ 37.28 ms
│  ├─ bufchan_buf0    36.5 ms       │ 140.1 ms      │ 38.49 ms      │ 39.6 ms
│  ├─ crossbeam       37.66 ms      │ 42.02 ms      │ 39.14 ms      │ 39.26 ms
│  ├─ flume           122.9 ms      │ 137.9 ms      │ 125.5 ms      │ 127.5 ms
│  ├─ kanal           38.55 ms      │ 49.36 ms      │ 41.54 ms      │ 41.75 ms
│  ╰─ std_mpsc        37.21 ms      │ 44.13 ms      │ 40.43 ms      │ 40.19 ms
├─ non_copy_24bytes                 │               │               │        
│  ├─ bufchan         37.08 ms      │ 137.9 ms      │ 37.96 ms      │ 39.15 ms
│  ├─ bufchan_buf0    36.89 ms      │ 141.1 ms      │ 39.19 ms      │ 41.45 ms
│  ├─ crossbeam       125.9 ms      │ 138.1 ms      │ 128.5 ms      │ 130.5 ms
│  ├─ flume           124.1 ms      │ 144.5 ms      │ 128 ms        │ 130.5 ms
│  ├─ kanal           40.42 ms      │ 46.93 ms      │ 42.78 ms      │ 42.93 ms
│  ╰─ std_mpsc        123.6 ms      │ 144.7 ms      │ 127.7 ms      │ 130.2 ms
╰─ non_copy_256bytes                │               │               │         
   ├─ bufchan         37.4 ms       │ 48.61 ms      │ 38.83 ms      │ 39.14 ms
   ├─ bufchan_buf0    37.84 ms      │ 46.2 ms       │ 40.85 ms      │ 40.78 ms
   ├─ crossbeam       125.7 ms      │ 139.8 ms      │ 128.5 ms      │ 130.5 ms
   ├─ flume           124.6 ms      │ 140.2 ms      │ 126.9 ms      │ 128.8 ms
   ├─ kanal           45.76 ms      │ 58.58 ms      │ 48.37 ms      │ 48.61 ms
   ╰─ std_mpsc        123.7 ms      │ 143.6 ms      │ 126.8 ms      │ 128.7 ms
```

# Caveats

It must be stressed that this is a **buffered** channel.
A single `send` call is likely to **not** trigger the complementing `recv` to wakeup.
All messages are guaranteed to reach `Receiver`, but the _latency_ of individual messages is not
consistent.
For long-lived `Sender`s that are slow to produce, lowering the buffer size, or manually calling
`flush` can ensure messages get to the receiver.
