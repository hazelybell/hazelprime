#![warn(rust_2018_idioms)]
#![feature(test)]
extern crate test;
use test::Bencher;
use mulch::fermat::{*};
use mulch::big::{*};
use mulch::vast::{*};
use mulch::limb::{*};

#[bench]
fn bench_1(bencher: &mut Bencher) {
    let n = 136;
    let f = Fermat::new(n);
    let mut big_a = Big::from_hex("9D68E100A50B479104B85497A9BA510639");
    let big_b = Big::from_hex("B6C0259E16F63F000194C4D5BBE3BB3908");
    let mut big_work = Big::new(div_up(n+1, LIMB_SIZE)*2);
    let b = Vast::from(&big_b);
    let closure = || {
        let a = VastMut::from(&mut big_a);
        let work = VastMut::from(&mut big_work);
        Fermat::mul_mod_fermat_assign(
            a,
            &b,
            f,
            work
        );
    };
    bencher.iter(closure);
}
