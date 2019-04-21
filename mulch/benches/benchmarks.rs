#![warn(rust_2018_idioms)]
#![feature(test)]
extern crate test;
use test::Bencher;
use mulch::fermat::{*};
use mulch::big::{*};
use mulch::vast::{*};
use mulch::limb::{*};
use mulch::ss_recursive::{*};
use mulch::pod::{*};

#[bench]
fn bench_mul_mod_fermat_assign(bencher: &mut Bencher) {
    let n = 136;
    let f = Fermat::new(n);
    let mut big_a = Big::from_hex("9D68E100A50B479104B85497A9BA510639");
    let big_b = Big::from_hex("B6C0259E16F63F000194C4D5BBE3BB3908");
    let mut big_work = Big::new(div_up(n+1, LIMB_SIZE)*2);
    let b = Vast::from(&big_b);
    let closure = || {
        let mut a = VastMut::from(&mut big_a);
        let mut work = VastMut::from(&mut big_work);
        Fermat::mul_mod_fermat_assign(
            &mut a,
            &b,
            f,
            &mut work
        );
    };
    bencher.iter(closure);
}

#[bench]
fn bench_multiply_2048(bencher: &mut Bencher) {
    let bo = Big::from_hex(
            "B954E7DFEE6CCE82F19BC30B53E6B6E15081CD494DD1652CEA6A30D134316E1\
            452C5BB2012B0889BB5A148093ED8CA2DDA1FA3E09D4473C6EAA90FC7809247C\
            FB7FE805D7095BD679653E016B74FFA844E7401BBE68BB7B25754B87F0D07AD0\
            72DBBEAB6F3E9B7C94ED93B8665FEBEE18091EB2BDFB021A5DA9DDC981F23E12"
    );
    let a_sz = bo.limbs()*2;
    let mut ba = Big::new(a_sz);
    let bb = Big::from_hex(
            "45BAA2EE705DDC4BDB71C3B963B612EC2CFE3B14E836C9988D260410DC9CF4C\
            B11C1E091B2EE874887BFBFBB5FD136859D2E887D96F43D0328C0FF3BAFDF67C\
            E3C71874F014F0C076109C3112C9C051F88B60F929967758F58E5041728C98B5\
            0B099D03817A54400BB065726B0D5D8DB328957083535EF65229F3FC0C65F691"
    );
    let mut a = VastMut::from(&mut ba);
    let b = Vast::from(&bb);
    let p_bits = b.bits() + bo.bits();
    println!("{} {} {}", p_bits, bo.bits(), b.bits());
    let mut workspaces: Vec<Vec<Big>> = Vec::new();
    let mut mult = recursive_setup(p_bits, &mut workspaces);
    let closure = || {
        a.pod_assign(&bo);
        mult.x(&mut a, &b);
    };
    bencher.iter(closure);
}
