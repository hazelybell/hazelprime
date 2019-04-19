#![warn(rust_2018_idioms)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::cmp::max;

use crate::limb::{*};
use crate::pod::{*};
use crate::big::{*};
use crate::big_mod_f::{*};

pub fn divides(n : BigSize, d : BigSize) -> bool {
    return (d % n) == 0;
}

#[derive(Debug)]
pub struct Nkn {
    pub N: BigSize,
    pub k: BigSize,
    pub n: BigSize
}

pub fn get_next_power_of_two(x: BigSize) -> BigSize {
    let lz = x.leading_zeros();
    let fz = 64 - lz; // first zero counting from the right
    let r = 1 << fz;
//     println!("x: {} lz: {} fz: {} r: {}", x, lz, fz, r);
    return r as BigSize;
}

pub fn get_Nkn_unbound(p_bits: BigSize) -> Nkn {
    // find a suitable N, k and n
    let N_min = p_bits + 1;
    let N_max = N_min * 16; // I have no clue what to set this to :(
    let k_max : BigSize = 16;
    let k_min : BigSize = 1;
    let mut N = N_min;
    while N < N_max {
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
        N = get_next_power_of_two(N);
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
    let mut pieces: Vec<Big> = Vec::with_capacity(number as usize);
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
    let piece_sz = div_up(n+1, LIMB_SIZE);
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
    let piece_sz = div_up(n+1, LIMB_SIZE);
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

pub fn vec_times_mat(v: Vec<Big>, m: &Vec<Big>, n: BigSize) -> Vec<Big> {
    let mut r: Vec<Big> = Vec::with_capacity(v.len());
    let dim = v.len();
    for i in 0..dim {
        let mut ri = Big::new(v[i].length());
        for j in 0..dim {
            ri += &mul_mod_fermat(
                &v[j],
                &m[(i + j * dim)],
                n
            );
        }
        let ri = mod_fermat(&ri, n);
        println!("{} {:?}", i, ri);
        r.push(ri);
    }
    return r;
}

pub fn ss_multiply2(a: Big, b: Big, params: Nkn) -> Big {
    let N = params.N;
    let k = params.k;
    let n = params.n;
    let twok : BigSize = 1 << k;
    let piece_sz = div_up(n+1, LIMB_SIZE);
    let orig_sz = a.length();
    let mut a_split = ss_split(a, twok, piece_sz);
    let mut b_split = ss_split(b, twok, piece_sz);
    let pieces = a_split.len();
    let orig_limbs_each = orig_sz / twok;
    let sum_sz = orig_limbs_each * twok + (piece_sz - 1);
    let nf = fermat(n);
    println!("pieces: {} limbs_each: {}", pieces, piece_sz);
    // weight
    println!("a_split: {:?}", a_split);
    for j in 0..pieces {
        let js = j as BigSize;
        let shift = js * n / twok;
        println!("shift: {}", shift);
        println!("A{}: {:?}", j, a_split[j]);
        a_split[j] <<= shift;
        println!("A{}: {:?}", j, a_split[j]);
        assert!(a_split[j].lt(&nf));
        b_split[j] <<= shift;
        assert!(b_split[j].lt(&nf));
    }
    // DFT
    let D = ss_dft_matrix(k, n);
    
    println!("A:");
    let a_dft = vec_times_mat(a_split, &D, n);
    println!("B:");
    let b_dft = vec_times_mat(b_split, &D, n);
    
    // dot product
    let mut c_dft : Vec<Big> = Vec::with_capacity(pieces as usize);
    for i in 0..pieces {
        let ci = mul_mod_fermat(&a_dft[i], &b_dft[i], n);
        c_dft.push(ci);
    }
    // inverse DFT
    let Dinv = ss_idft_matrix(k, n);
    println!("C:");
    let mut c_idft = vec_times_mat(c_dft, &Dinv, n);
    // unshift
    for j in 0..pieces {
        let js = j as BigSize;
        let shift = js * n / twok;
        let mut coeff = Big::new_one(piece_sz);
        coeff <<= shift;
        let invcoeff = inv_mod_fermat(&coeff, n);
        let unshifted = mul_mod_fermat(&invcoeff, &c_idft[j], n);
        c_idft[j] = unshifted;
        println!("C{}: {:?}", j, c_idft[j]);
    }
    // do carrying
    let mut sum: Big = Big::new(sum_sz);
    for i in (1..pieces).rev() {
        sum += &c_idft[i];
        sum <<= LIMB_SIZE * orig_limbs_each;
    }
    sum += &c_idft[0];
    println!("{:?}", sum);
    let p = mod_fermat(&sum, N);
    println!("{:?}", p);
    return p;
}

pub fn ss_multiply(a: Big, b: Big) -> Big {
    let mut target_sz : BigSize = 1;
    let a_bits = a.bits();
    let b_bits = b.bits();
    let a_sz = a.length();
    let b_sz = b.length();
    let p_bits = a_bits + b_bits; // number of bits in the product
    let params = ss_simple_get_size(p_bits);
    let sz = params.sz;
    // we need space for the product which can be a+b long
    let a2 = big_extend(a, sz); 
    let b2 = big_extend(b, sz);
    let p = ss_multiply2(a2, b2, params.Nkn);
    let p2 = p.downsized(a_sz + b_sz);
    return p2;
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
    fn ss_simple_get_size_1() {
        let r = ss_simple_get_size(150000);
        println!("{:?}", r);
//         assert!(false)
    }
    #[test]
    fn ss_simple_get_size_2() {
        let r = ss_simple_get_size(120);
        println!("{:?}", r);
//         assert!(false);
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
    fn ss_multiply_1() {
        let a = Big::from_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        let b = Big::from_hex("10");
        let p = ss_multiply(a, b);
        assert_eq!(p.hex_str(), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0");
    }
    #[test]
    fn ss_multiply_2() {
        let a = Big::from_hex("B85497A9BA510638");
        let b = Big::from_hex("68E100A50B479104");
        let p = ss_multiply(a, b);
        assert_eq!(p.hex_str(), "4B84606D1682968773BEAB03EF51D0E0");
    }
    #[test]
    fn ss_multiply_128() {
        let a = Big::from_hex("5C068A34E30288DED00B063876877E9D");
        let b = Big::from_hex("C0259E16F63F000194C4D5BBE3BB3907");
        let p = ss_multiply(a, b);
        assert_eq!(p.hex_str(), "45126D6DEE4829175D96FEE6FAF7CA84D2CAD2D35BED3265C68E95DD1C946B4B");
    }
    #[test]
    fn ss_multiply_256() {
        let a = Big::from_hex("B27CC95B7B89BF33DDCE184822C1376CF99527E2862042DBB66313F44C4C47B6");
        let b = Big::from_hex("5D94E89EF3FBA74A9314E05B5D1533B48AE9F0C710ED2A2C8885CAD9F5757B8F");
        let p = ss_multiply(a, b);
        assert_eq!(p.hex_str(), "413F277A8E5F8CA21ECA155F55015643AD0E5FFD1FF5F3F566D0556C650D3C9278081C242052F867408AE0018570DE663FED010592A91E083666CAE3393E80AA");
    }
    #[test]
    fn ss_multiply_512() {
        let a = Big::from_hex("F99527E2862042DBB66313F44C4C47B6C0259E16F63F000194C4D5BBE3BB39075C068A34E30288DED00B063876877E9D68E100A50B479104B85497A9BA510638");
        let b = Big::from_hex("D517B4B082CB3651E1CEE7FF12C1F985D94E89EF3FBA74A9314E05B5D1533B48AE9F0C710ED2A2C8885CAD9F5757B8FB27CC95B7B89BF33DDCE184822C1376C");
        let p = ss_multiply(a, b);
        assert_eq!(p.hex_str(), "CFC036BF050D730EA92C3A8E66BF44B94319958CC3C0E8FD8570CC61A7CD39CD66EFBE891948DD59F4AF2FCFC7CB63B8682B9660B3AC2142DF54E37DA1A4EDF3D0962A14463B0E5CDE726E2FD903B8FFA53AC9E2ECCCDB93B0D4078912B98887A54AA1782704F6E7AF894DA712689FDFCCDFCF33B91DB702A68AC4B22BCA7A0");
    }
    #[test]
    fn ss_multiply_2048() {
        let a = Big::from_hex(
             "B954E7DFEE6CCE82F19BC30B53E6B6E15081CD494DD1652CEA6A30D134316E1\
             452C5BB2012B0889BB5A148093ED8CA2DDA1FA3E09D4473C6EAA90FC7809247C\
             FB7FE805D7095BD679653E016B74FFA844E7401BBE68BB7B25754B87F0D07AD0\
             72DBBEAB6F3E9B7C94ED93B8665FEBEE18091EB2BDFB021A5DA9DDC981F23E12"
        );
        let b = Big::from_hex(
             "45BAA2EE705DDC4BDB71C3B963B612EC2CFE3B14E836C9988D260410DC9CF4C\
             B11C1E091B2EE874887BFBFBB5FD136859D2E887D96F43D0328C0FF3BAFDF67C\
             E3C71874F014F0C076109C3112C9C051F88B60F929967758F58E5041728C98B5\
             0B099D03817A54400BB065726B0D5D8DB328957083535EF65229F3FC0C65F691"
        );
        let p = ss_multiply(a, b);
        assert_eq!(p.to_hex(), 
              "327B00242CFAEE8DF0C4F7486CADB351CEABFBDCF340A119E34DC3BEFD209D\
             6408553EA56FEC93DED68F3FFB9BABB60E3E0C03FF652DB955AACE4F05576796\
             3E8DA37B7C7FD5C35A29AE814656217397F562B3E5527F49DFAC585F32E8B905\
             ADCB3C58F3C0F4D3511A1E02A357EBE095371FAEC2F1616595CBA68029323FF8\
             916FB9E7792750B8309B1322E8A1B8038881CE87B99F241A1C475629ACF29077\
             A8A06FED983FF02114C3E7D57CFF99EAB76323E2B356E24A0CC49618BE216A2A\
             C97DB6185B92275311C91B2B337B38F6839960047A9971BFE776668CEB0802DC\
             3E1F7310289C6E4AF589914E6FCAC46673D036908906B308CB301134B6F47432"
            );
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
    #[test]
    fn get_next_power_of_two_() {
        let r = get_next_power_of_two(1022);
        assert_eq!(r, 1024);
    }
}
