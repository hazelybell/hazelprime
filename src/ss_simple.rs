#![allow(non_snake_case)]
#![allow(unused)]

use std::cmp::max;

use crate::big::{*};
use crate::big_mod_f::{*};

pub fn divides(n : BigSize, d : BigSize) -> bool {
    return (d % n) == 0;
}

#[derive(Debug)]
pub struct Nkn {
    N: BigSize,
    k: BigSize,
    n: BigSize
}

pub fn get_Nkn_unbound(p_bits: BigSize) -> Nkn {
    // find a suitable N, k and n
    let N_min = p_bits + 1;
    let N_max = N_min * 16; // I have no clue what to set this to :(
    let k_max : BigSize = 16;
    let k_min : BigSize = 1;
    for N in N_min..(N_max+1) {
        if (N % (1<<k_min)) != 0 {
            continue;
        }
        println!("Trying N={}", N);
        for k in k_min..(k_max+1) {
            let twok = 1 << k;
            if (twok > p_bits) {
                break;
            }
            if (!divides(twok, N)) {
                // if this 2^k doesnt divide N then a bigger one wont
                break;
            }
            let n_min = 2 * N / twok + k;
            let n_max = N/2;
            println!("Trying k={} twok={} n_min=2N/2^k+k={} n_max={}", k, twok, n_min, n_max);
            let piece_sz = N / twok;
            println!("Piece size: {}", piece_sz);
            if piece_sz % LIMB_SIZE > 0 {
                break;
            }
            let n = div_up(n_min, twok)*twok;
            if n <= n_max {
                assert!(divides(twok, n));
                println!("Satisfied: N={}, k={}, twok={}, n={}", N, k, twok, n);
                let optimal_twok = (N as f64).sqrt();
                println!("Optimal twok={}", optimal_twok);
                let r = Nkn {
                    N: N,
                    k: k,
                    n: n
                };
                return r;
            }
        }
    }
    unreachable!();
}
#[derive(Debug)]
pub struct NknSize {
    Nkn: Nkn,
    sz: BigSize
}

pub fn ss_simple_get_size(p_bits: BigSize) -> NknSize {
     /* room for the modulo which is 2^N+1, and bigger than the biggest
      * value stored in p_bits, so if p_bits is 32, the max valued would
      * be 2^32-1 and we need to use 2^32+1 for the modulo at a minimum,
      * however, 2^32+1 takes 33 bits!
      */
    let min_bits = p_bits + 1;
    let a_Nkn = get_Nkn_unbound(p_bits);
    let N = a_Nkn.N;
    let k = a_Nkn.k;
    let twok = 1 << k;
    println!("k-constraint size: {} bits {}", twok, twok*LIMB_SIZE);
    let nc = div_up(N, LIMB_SIZE);
    println!("n-constraint size: {} bits {}", nc, nc*LIMB_SIZE);
    let sz : BigSize;
    if (twok > nc) {
        println!("Warning: oversized due to k");
        sz = twok;
    } else {
        sz = div_up(nc, twok) * twok;
    }
    println!("using size: {} bits {}", sz, sz*LIMB_SIZE);
    let r = NknSize {
        Nkn: a_Nkn,
        sz: sz
    };
    return r;
    unreachable!();
}

pub fn ss_split(x: Big, number: BigSize, piece_sz: BigSize) -> Vec<Big> {
    let long_sz = x.length();
    assert!(divides(number, long_sz));
    let limbs_each = long_sz / number;
    let mut pieces: Vec<Big> = Vec::with_capacity(number);
    for i in 0..number {
        let mut piece = Big::new(piece_sz);
        let start = i * limbs_each;
        for j in 0..limbs_each {
            let src_j = j + start;
            piece[j] = x[src_j];
        }
        pieces.push(piece);
    }
    return pieces;
}

pub fn ss_dft_matrix(k: BigSize, n: BigSize) -> Vec<Big> {
    let twok : BigSize = 1 << k;
    let piece_sz = div_up(n, LIMB_SIZE);
    // DFT/NTT/Fermat Number Transform
    let mut prou = Big::new_one(piece_sz);
    prou <<= (2 * n / twok); // the twokth primitive root of unity
    let prou = prou;
    println!("prou: {}", prou);
    
    let mut prou_powers = Vec::with_capacity(twok as usize);
    let mut prou_n = Big::new_one(piece_sz);
    for i in 0..twok {
        println!("prou^{} = {:?}", i, prou_n);
        prou_powers.push(prou_n.clone());
        prou_n = mul_mod_fermat(&prou_n, &prou, n);
    }
    println!("prou^{} = {:?}.", twok, prou_n);
    assert_eq!(prou_n, 1);
    
    let modf = fermat(n);
    println!("2^n+1: {}", modf);
    let a_elts = twok * twok;
    // DFT matrix
    let mut a : Vec<Big> = Vec::with_capacity(a_elts as usize);
    let one = Big::new_one(piece_sz);
    let mut aa = one.clone();
    for i in 0..twok {
        a.push(one.clone()); // fill the first col with ones
        print!("{} ", 1);
        let mut aaa = aa.clone();
        for j in 1..twok {
            a.push(aaa.clone());
            print!("{} ", aaa);
            aaa = mul_mod_fermat(&aaa, &aa, n);
            assert!(aaa.lt(&modf));
            assert!(aaa != 0);
        }
        aa = mul_mod_fermat(&aa, &prou, n);
        println!("");
    }
    return a;
}

pub fn ss_idft_matrix(k: BigSize, n: BigSize) -> Vec<Big> {
    let twok : BigSize = 1 << k;
    let piece_sz = div_up(n, LIMB_SIZE);
    // DFT/NTT/Fermat Number Transform
    let mut prou = Big::new_one(piece_sz);
    prou <<= (2 * n / twok); // the twokth primitive root of unity
    let prou = prou;
    println!("prou: {}", prou);
    let mut iprou = Big::new_one(piece_sz);
    for i in 0..twok-1 {
        iprou = mul_mod_fermat(&prou, &iprou, n);
    }
    println!("iprou: {}", iprou);
    let should_be_one = mul_mod_fermat(&prou, &iprou, n);
    assert!(should_be_one == 1);
    
    let mut twok_big = Big::new(piece_sz);
    twok_big[0] = twok as Limb;
    println!("twok_big: {}", twok_big);
    let mut itwok = inv_mod_fermat(&twok_big, n);
    println!("itwok: {}", itwok);
    let should_be_one = mul_mod_fermat(&twok_big, &itwok, n);
    assert!(should_be_one== 1);

    let modf = fermat(n);
    println!("2^n+1: {}", modf);
    let a_elts = twok * twok;
    // DFT matrix
    let mut a : Vec<Big> = Vec::with_capacity(a_elts as usize);
    let one = Big::new_one(piece_sz);
    let mut aa = one.clone();
    for i in 0..twok {
        a.push(itwok.clone()); // fill the first col with 1/twok
        print!("{} ", itwok);
        let mut aaa = aa.clone();
        for j in 1..twok {
            let aaa_over_twok = mul_mod_fermat(&aaa, &itwok, n);
            a.push(aaa_over_twok.clone());
            print!("{} ", aaa_over_twok);
            aaa = mul_mod_fermat(&aaa, &aa, n);
            assert!(aaa.lt(&modf));
            assert!(aaa != 0);
        }
        aa = mul_mod_fermat(&aa, &iprou, n);
        println!("");
    }
    return a;
}


pub fn ss_multiply2(a: Big, b: Big, params: Nkn) {
    let N = params.N;
    let k = params.k;
    let n = params.n;
    let twok : BigSize = 1 << k;
    let piece_sz = div_up(n, LIMB_SIZE);
    let mut a_split = ss_split(a, twok, piece_sz);
    let mut b_split = ss_split(b, twok, piece_sz);
    let pieces = a_split.len();
    let nf = fermat(n);
    println!("pieces: {} limbs_each: {}", pieces, piece_sz);
    // weight
    println!("a_split: {:?}", a_split);
    for j in 0..pieces {
        let js = j as BigSize;
        let shift = js * n / twok;
        println!("shift: {}", shift);
        a_split[j] <<= shift;
        assert!(a_split[j].lt(&nf));
        b_split[j] <<= shift;
        assert!(b_split[j].lt(&nf));
    }
    // DFT
    let D = ss_dft_matrix(k, n);
    for i in 0..twok {
        {
            let ai = a_split[i as usize].clone();
            let mut new_ai = Big::new(piece_sz);
            for j in 0..twok {
                new_ai += &mul_mod_fermat(
                    &ai,
                    &D[(i + j * twok) as usize],
                    n
                );
            }
            a_split[i as usize] = mod_fermat(&new_ai, n);
        }
        {
            let bi = b_split[i as usize].clone();
            let mut new_bi = Big::new(piece_sz);
            for j in 0..twok {
                new_bi += &mul_mod_fermat(
                    &bi, 
                    &D[(i + j * twok) as usize],
                    n
                );
            }
            b_split[i as usize] = mod_fermat(&new_bi, n);
        }
    }
    // dot product
    let mut c_split : Vec<Big> = Vec::with_capacity(pieces);
    // inverse DFT
    let Dinv = ss_idft_matrix(k, n);
    
}

pub fn ss_multiply(a: Big, b: Big) -> Big {
    let mut target_sz : BigSize = 1;
    let a_bits = a.bits();
    let b_bits = b.bits();
    let p_bits = a_bits + b_bits; // number of bits in the product
    let params = ss_simple_get_size(p_bits);
    let sz = params.sz;
    // we need space for the product which can be a+b long
    let a2 = big_extend(a, sz); 
    let b2 = big_extend(b, sz);
    ss_multiply2(a2, b2, params.Nkn);
    return Big::new(1); // shut up the compiler
}


// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::ss_simple::{*};
    #[test]
    fn divides_() {
        assert!(divides(64, 128));
        assert!(divides(16, 16));
    }
    #[test]
    fn ss_simple_get_size_() {
        let r = ss_simple_get_size(150000);
        println!("{:?}", r);
//         assert!(false)
    }
    #[test]
    fn ss_simple_get_size_2() {
        let r = ss_simple_get_size(120);
        println!("{:?}", r);
    }
    #[test]
    fn div_up_() {
        let r = div_up(3, 4);
        assert_eq!(r, 1);
    }
    #[test]
    fn get_Nkn_unbound_1() {
        let r = get_Nkn_unbound(16000);
        println!("{:?}", r);
    }
    #[test]
    fn ss_multiply_() {
        let mut a = Big::new(2);
        let mut b = Big::new(2);
        a[1] = 0x00FFFFFFFFFFFFFFu64;
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        b[1] = 0x0u64;
        b[0] = 0x10u64;
        ss_multiply(a, b);
        assert!(false);
    }
    #[test]
    fn dft_idft_1() {
        let n: BigSize = 136;
        let k: BigSize = 3;
        let dim = 1 << k;
        let nf = fermat(n);
        let sz = nf.length();
        let A = ss_dft_matrix(k, n);
        let B = ss_idft_matrix(k, n);
        for ai in 0..dim {
            for bj in 0..dim {
                let mut e = Big::new(sz);
                for aj in 0..dim {
                    let bi = aj;
                    let a = &A[ai + aj * dim];
                    let b = &B[bi + bj * dim];
                    let ab = mul_mod_fermat(a, b, n);
                    e += &ab;
//                     println!("e: {:?}", e);
                }
                e = mod_fermat(&e, n);
                print!("{}", e);
                if ai == bj { // diagonal
                    assert_eq!(e, 1);
                } else {
                    assert_eq!(e, 0);
                }
            }
            println!("");
        }
    }
}