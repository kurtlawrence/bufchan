use std::{collections::HashSet, thread::spawn};

use crate::*;

#[quickcheck]
fn channel_works(xs: Vec<isize>, ys: Vec<isize>) -> bool {
    let len = xs.len() + ys.len();
    let set = xs.iter().chain(&ys).copied().collect::<HashSet<_>>();
    let (mut txa, rx) = unbounded();
    let mut txb = txa.clone();
    spawn(move || xs.into_iter().for_each(|x| txa.send(x)));
    spawn(move || ys.into_iter().for_each(|x| txb.send(x)));

    let rx = rx.into_iter().collect::<Vec<_>>();
    assert_eq!(rx.len(), len);
    let rx = HashSet::from_iter(rx);
    rx == set
}
