use rayon::prelude::*;

fn main() {
    divan::main();
}

const TASKS: u32 = 40_000;

fn run<Tx, Rx, S>(channel: (Tx, Rx), send: S)
where
    Tx: Clone + Send + 'static,
    S: Fn(&mut Tx, u32) + Send + Sync + 'static,
    Rx: IntoIterator,
{
    let (tx, rx) = channel;
    std::thread::spawn(|| {
        // ensure we are on our own thread pool
        rayon::ThreadPoolBuilder::new()
            .build()
            .unwrap()
            .install(|| {
                std::thread::spawn(move || {
                    (0..TASKS).into_par_iter().for_each_with(tx, |tx, n| {
                        let x = (0..n).fold(0u32, |a, b| divan::black_box(a.overflowing_add(b).0));
                        send(tx, x)
                    });
                });
            });
    });
    assert_eq!(rx.into_iter().count(), TASKS as usize);
}

mod int {
    use super::*;

    #[divan::bench]
    fn bufchan() {
        run(bufchan::unbounded(), |tx, x| tx.send(x));
    }

    #[divan::bench]
    fn bufchan_buf0() {
        run(bufchan::unbounded_with_buffer(0), |tx, x| tx.send(x));
    }

    #[divan::bench]
    fn std_mpsc() {
        run(std::sync::mpsc::channel(), |tx, x| tx.send(x).unwrap());
    }

    #[divan::bench]
    fn flume() {
        run(flume::unbounded(), |tx, x| tx.send(x).unwrap());
    }

    #[divan::bench]
    fn crossbeam() {
        run(crossbeam::channel::unbounded(), |tx, x| tx.send(x).unwrap());
    }

    #[divan::bench]
    fn kanal() {
        run(kanal::unbounded(), |tx, x| tx.send(x).unwrap());
    }
}

mod non_copy_256bytes {
    use super::*;

    struct Big {
        arr: [u32; 64],
    }

    fn f(x: u32) -> Big {
        let mut arr = [0u32; 64];
        arr.fill(x);
        Big { arr }
    }

    #[divan::bench]
    fn bufchan() {
        run(bufchan::unbounded(), |tx, x| tx.send(f(x)));
    }

    #[divan::bench]
    fn bufchan_buf0() {
        run(bufchan::unbounded_with_buffer(0), |tx, x| tx.send(f(x)));
    }

    #[divan::bench]
    fn std_mpsc() {
        run(std::sync::mpsc::channel(), |tx, x| tx.send(f(x)).unwrap());
    }

    #[divan::bench]
    fn flume() {
        run(flume::unbounded(), |tx, x| tx.send(f(x)).unwrap());
    }

    #[divan::bench]
    fn crossbeam() {
        run(crossbeam::channel::unbounded(), |tx, x| {
            tx.send(f(x)).unwrap()
        });
    }

    #[divan::bench]
    fn kanal() {
        run(kanal::unbounded(), |tx, x| tx.send(f(x)).unwrap());
    }
}

mod non_copy_24bytes {
    use super::*;

    struct Big {
        arr: [u32; 6],
    }

    fn f(x: u32) -> Big {
        let mut arr = [0u32; 6];
        arr.fill(x);
        Big { arr }
    }

    #[divan::bench]
    fn bufchan() {
        run(bufchan::unbounded(), |tx, x| tx.send(f(x)));
    }

    #[divan::bench]
    fn bufchan_buf0() {
        run(bufchan::unbounded_with_buffer(0), |tx, x| tx.send(f(x)));
    }

    #[divan::bench]
    fn std_mpsc() {
        run(std::sync::mpsc::channel(), |tx, x| tx.send(f(x)).unwrap());
    }

    #[divan::bench]
    fn flume() {
        run(flume::unbounded(), |tx, x| tx.send(f(x)).unwrap());
    }

    #[divan::bench]
    fn crossbeam() {
        run(crossbeam::channel::unbounded(), |tx, x| {
            tx.send(f(x)).unwrap()
        });
    }

    #[divan::bench]
    fn kanal() {
        run(kanal::unbounded(), |tx, x| tx.send(f(x)).unwrap());
    }
}
