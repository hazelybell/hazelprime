use crate::big::{*};

pub fn fermat(n : BigSize) -> Big {
    let sz = div_up(n+1, LIMB_SIZE);
    let mut f = Big::new_one(sz);
    f <<= n;
    f += 1;
    return f;
}

pub fn mod_fermat(x : &Big, n : BigSize) -> Big {
    let sz = div_up(n, LIMB_SIZE);
    let mut plus = Big::new(sz);
    let mut minus = Big::new(sz);
    let src_bits = x.bitlen();
    let iters = div_up(src_bits, n);
//     println!("src_bits: {}, iters: {}", src_bits, iters);
    for i in 0..iters {
        let piece = x.slice_bits(n*i, n);
//         println!("start: {} len: {} piece: {}", n*i, n, piece);
        if i % 2 == 0 { // even
            plus += &piece;
//             println!("plus: {}", plus)
        } else { // odd
            minus += &piece;
//             println!("minus: {}", minus)
        }
    }
    let f = fermat(n);
    if plus.lt(&minus) {
        plus += &f;
    }
    plus -= &minus;
    if f.lt(&plus) {
        println!("{}<{}", f, plus);
        plus -= &f;
    }
    if f.lt(&plus) {
        panic!("Reducing mod fermat still too big :(");
    }
    return plus;
}


pub fn mul_mod_fermat(a : &Big, b : &Big, n : BigSize) -> Big {
    let p_big = a * b;
    let p = mod_fermat(&p_big, n);
    return p;
}

// pub fn inv_mod_fermat(a: &Big, n: BigSize) -> Big {
//     // extended euclidean algorithm
//     // https://en.wikipedia.org/w/index.php?title=Extended_Euclidean_algorithm&oldid=890036949#Pseudocode
//     let b = fermat(n);
//     let mut s = Big::new(b.length());
//     let s_negative = false;
//     let mut old_s = Big::new_one(b.length());
//     let old_s_negative = false;
//     let mut t = Big::new_one(b.length());
//     let t_negative = false;
//     let mut old_t = Big::new(b.length());
//     let old_t_negative = false;
//     let mut r = b.clone();
//     let mut old_r = a.clone();
//     while !r.is_zero() {
//         let q = div(&old_r, &r);
//         
//         let qr = multiply(&q, &r);
//         assert!(old_r.ge(&qr));
//         let mut new_r = old_r.clone();
//         new_r.decrease_big(&qr);
//         old_r = r;
//         r = new_r;
//         
//         let qs = multiply(&q, &s);
// //         if 
//         
//     }
//     unreachable!();
// }

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::big_mod_f::{*};
    #[test]
    fn mod_fermat_1() {
        let mut a = Big::new(1);
        a[0] = 656;
        let r = mod_fermat(&a, 3);
        assert_eq!(r[0], 8);
        assert_eq!(r.length(), 1);
    }
    #[test]
    fn mod_fermat_2() {
        let mut a = fermat(100);
        assert_eq!(a[0], 1);
        assert_eq!(a[1], 1<<36);
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 0);
        assert_eq!(r[1], 0);
        a[0] = 2;
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 0);
        a[0] = 0xFFFFFFFFFFFFFFFFu64;
        a[1] = 0xFFFFFFFFFFFFFFFFu64;
        let r = mod_fermat(&a, 100);
        assert_eq!(r[0], 0xFFFFFFFFF0000000u64);
        assert_eq!(r[1], 0x0000000FFFFFFFFFu64);
        let r = mod_fermat(&a, 99);
        assert_eq!(r[0], 0xFFFFFFFFE0000000u64);
        assert_eq!(r[1], 0x00000007FFFFFFFFu64);
    }
    #[test]
    fn mul_mod_fermat_1() {
        let mut a = Big::new(1);
        a[0] = 41;
        let mut b = Big::new(1);
        b[0] = 16;
        let r = mul_mod_fermat(&a, &b, 3);
        assert_eq!(r[0], 8);
        assert_eq!(r.length(), 1);
        let r = mul_mod_fermat(&a, &b, 16);
        assert_eq!(r[0], 656);
        assert_eq!(r.length(), 1);
    }
    #[test]
    fn mul_mod_fermat_2() {
        let mut a = Big::new(1);
        a[0] = 0x10000000;
        let mut b = Big::new(1);
        b[0] = 0x10;
        let r = mul_mod_fermat(&a, &b, 32);
        assert_eq!(r[0], 0x100000000);
    }
    #[test]
    fn mod_fermat_3() {
        let mut a = Big::new(1);
        a[0] = 0x100000000;
        let r = mod_fermat(&a, 32);
        assert_eq!(r[0], 0x100000000);
    }
}
