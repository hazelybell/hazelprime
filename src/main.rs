use rug::Integer;
use rug::Assign;
use gmp_mpfr_sys::gmp;
use gmp_mpfr_sys::gmp::{limb_t, size_t};

extern crate clap; 
use clap::{Arg, App};

#[macro_use]
extern crate nom;
use nom::digit;
use nom::types::CompleteStr;
use std::str::FromStr;
use std::num::ParseIntError;

fn u32_from_cstr(input: CompleteStr) -> Result<u32, ParseIntError> {
    u32::from_str(input.as_ref())
}

#[derive(Debug, Copy, Clone)]
pub struct Proth {
    pub t: u32,
    pub e: u32, 
}

named!(uint32<CompleteStr, u32>,
    map_res!(digit, u32_from_cstr)
);

named!(times<CompleteStr, CompleteStr>,
    alt!(tag!("*") | tag!("x") | tag!("."))
);

named!(two<CompleteStr, CompleteStr>,
    tag!("2")
);

named!(to_the<CompleteStr, CompleteStr>,
    alt!( tag!("^") | tag!("e") )
);

named!(plus<CompleteStr, CompleteStr>,
    tag!("+")
);

named!(one<CompleteStr, CompleteStr>,
    tag!("1")
);


named!(parse_proth<CompleteStr, Proth>,
    do_parse!(
        t: uint32 >>
        times >>
        two >>
        to_the >>
        e: uint32 >>
        plus >>
        one >>
        (Proth { t, e })
    )
);

fn proth_gmp_simple(n : Proth) -> (Integer, Integer) {
    let two_to_the_e : Integer = Integer::from(Integer::u_pow_u(2, n.e));
    let n_full : Integer = two_to_the_e * n.t + 1;
    let n_minus_one_over_two : Integer = Integer::from(&n_full - 1) / 2;
    println!("{:?}", n_full);
    let a = Integer::from(3);
    let n_full_ptr = n_full.as_raw();
    let n_minus_one_over_two_ptr = n_minus_one_over_two.as_raw();
    let a_ptr = a.as_raw();
    let mut r = Integer::with_capacity((n_full.significant_bits() + a.significant_bits()) as usize);
    let r_ptr = r.as_raw_mut();
    println!("Start powm");
    unsafe {
        gmp::mpz_powm(r_ptr, a_ptr, n_minus_one_over_two_ptr, n_full_ptr);
    }
    println!("Done powm");
    let r_minus_p : Integer = &r - n_full;
    println!("{:?}", r_minus_p);
    if r_minus_p == -1 {
        println!("Prime");
    } else {
        println!("Not prime");
    }
    return (r, r_minus_p);
}

fn proth_gmp(n : Proth) -> (Integer, Integer) {
    let two_to_the_e : Integer = Integer::from(Integer::u_pow_u(2, n.e));
    let n_full : Integer = two_to_the_e * n.t + 1;
    let n_minus_one_over_two : Integer = Integer::from(Integer::from(&n_full - 1) / 2);
    println!("n: {:?} bts", n_full.significant_bits());
    let n_full_ptr = n_full.as_raw();
    let mut rr = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let rr_ptr = rr.as_raw_mut();
    let mut ai = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let ai_ptr = ai.as_raw_mut();
    let original_capacity = ai.capacity();
    ai.assign(3);
    rr.assign(1);
    assert_eq!(ai.capacity(), original_capacity);
    println!("{}", rr);
    let mut i : u32 = 0;
    let bits : u32 = n_minus_one_over_two.significant_bits();
    println!("n_minus_one_over_two: {} bits", n_minus_one_over_two.significant_bits());
    while i < bits {
        let bit = n_minus_one_over_two.get_bit(i);
        unsafe {
            if bit {
                gmp::mpz_mul(rr_ptr, rr_ptr, ai_ptr);
                gmp::mpz_mod(rr_ptr, rr_ptr, n_full_ptr);
            }
            // square ai
            gmp::mpz_mul(ai_ptr, ai_ptr, ai_ptr);
            gmp::mpz_mod(ai_ptr, ai_ptr, n_full_ptr);
        }
        if i % 100 == 0 {
            println!("{}/{} {}", i, bits, (i as f32)/(bits as f32));
        }
        i += 1;
    }
    println!("done");
    let r_minus_p : Integer = &rr - n_full;
    let r : Integer = Integer::from(&rr);
    println!("{:?}", r_minus_p);
    return (r, r_minus_p);
}

fn proth_gmp2(n : Proth) -> (Integer, Integer) {
    let two_to_the_e : Integer = Integer::from(Integer::u_pow_u(2, n.e));
    let n_full : Integer = two_to_the_e * n.t + 1;
    let n_minus_one_over_two : Integer = Integer::from(Integer::from(&n_full - 1) / 2);
    println!("n: {:?} bts", n_full.significant_bits());
    let n_full_ptr = n_full.as_raw();
    let mut rr = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let rr_ptr = rr.as_raw_mut();
    let mut rrt = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let rrt_ptr = rrt.as_raw_mut();
    let mut ai = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let ai_ptr = ai.as_raw_mut();
    let mut aj = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let aj_ptr = aj.as_raw_mut();
    let limb_bits : usize;
    {
        let t = Integer::from(3);
        let t_ptr = t.as_raw();
        unsafe {
            assert_eq!(gmp::mpz_size(t_ptr), 1);
        }
        limb_bits = t.capacity();
        
    }
    let mut q = Integer::with_capacity((n_full.significant_bits()  as usize) + limb_bits);
    let q_ptr = q.as_raw_mut();
    let mut i : u32 = 0;
    let bits : u32 = n_minus_one_over_two.significant_bits();
    println!("n_minus_one_over_two: {} bits", n_minus_one_over_two.significant_bits());
    let ai_0 : *mut limb_t ;
    let aj_0 : *mut limb_t ;
    let mut ax_0 : *mut limb_t ;
    let mut ay_0 : *mut limb_t ;
    let q_0 : *mut limb_t ;
    let rr_0 : *mut limb_t ;
    let rrt_0 : *mut limb_t ;
    let n_0 : *const limb_t;
    let n_sz : size_t ;
    let a_sz : size_t ;
    let q_sz : size_t ;
    let rr_sz :  size_t ;
    let rrt_sz : size_t ;
    let double_sz : size_t ;
    unsafe {
        n_sz = gmp::mpz_size(n_full_ptr) as size_t;
        double_sz = n_sz * 2;
        println!("n size: {} double: {}", n_sz, double_sz);
        a_sz = double_sz * 4;
        q_sz = double_sz + 2;
        rr_sz = double_sz * 4;
        rrt_sz = double_sz * 4;
        
        
        ai_0 = gmp::mpz_limbs_modify(ai_ptr, a_sz);
        aj_0 = gmp::mpz_limbs_modify(aj_ptr, a_sz);
        q_0 = gmp::mpz_limbs_modify(q_ptr, q_sz);
        rr_0 = gmp::mpz_limbs_modify(rr_ptr, rr_sz);
        rrt_0 = gmp::mpz_limbs_modify(rrt_ptr, rrt_sz);
        n_0 = gmp::mpz_limbs_read(n_full_ptr);
        
        println!("{}", *n_0.offset(n_sz as isize - 1));
        assert_ne!(*(n_0.offset(n_sz as isize - 1)), 0);
        
        gmp::mpn_zero(ai_0, a_sz);
        gmp::mpn_zero(aj_0, a_sz);
        gmp::mpn_zero(q_0, q_sz);
        gmp::mpn_zero(rr_0, rr_sz);
        gmp::mpn_zero(rrt_0, rrt_sz);
        
        *rr_0 = 1;
        *ai_0 = 3;
    }
    let double_sz = n_sz * 2;
    while i < bits {
        let bit = n_minus_one_over_two.get_bit(i);
        unsafe {
            if (i % 2 == 0) {
                ax_0 = ai_0;
                ay_0 = aj_0;
            } else {
                ax_0 = aj_0;
                ay_0 = ai_0;
            }
//             println!("rr_0: {}", *rr_0);
            if bit {
                gmp::mpn_zero(rrt_0, rrt_sz);
//                 println!("rr_0: {} rrt_0: {} ax_0: {}", *rr_0, *rrt_0, *ax_0);
                gmp::mpn_mul(rrt_0, rr_0, n_sz, ax_0, n_sz);
//                 println!("rr_0: {} rrt_0: {} ax_0: {}", *rr_0, *rrt_0, *ax_0);
                gmp::mpn_zero(q_0, q_sz);
                gmp::mpn_zero(rr_0, rr_sz);
//                 println!("rr_0: {} rrt_0: {} n_0: {}", *rr_0, *rrt_0, *n_0);
                gmp::mpn_tdiv_qr(q_0, rr_0, 0, rrt_0, double_sz, n_0, n_sz);
//                 println!("rr_0: {} rrt_0: {} n_0: {}", *rr_0, *rrt_0, *n_0);
            }
            // square
//             println!("ax_0: {}", *ax_0);
            gmp::mpn_zero(ay_0, a_sz);
            gmp::mpn_sqr(ay_0, ax_0, n_sz);
//             println!("ay_0: {}", *ay_0);
            gmp::mpn_zero(q_0, q_sz);
//             println!("ay_0: {}", *ay_0);
            gmp::mpn_tdiv_qr(q_0, ay_0, 0, ay_0, double_sz, n_0, n_sz);
//             println!("ay_0: {}", *ay_0);
        }
        if i % 100 == 0 || i < 100 {
            println!("{}/{} {}", i, bits, (i as f32)/(bits as f32));
        }
        i += 1;
    }
    unsafe {
        gmp::mpz_limbs_finish(ai_ptr, a_sz);
        gmp::mpz_limbs_finish(aj_ptr, a_sz);
        gmp::mpz_limbs_finish(q_ptr, q_sz);
        gmp::mpn_zero(rr_0.offset(n_sz as isize), rr_sz - n_sz);
        gmp::mpz_limbs_finish(rr_ptr, rr_sz);
        gmp::mpz_limbs_finish(rrt_ptr, rrt_sz);
    }
    println!("done");
//     let r : Integer;
//     unsafe {
//         r = Integer::from(Integer::from_raw(*rr_ptr));
//     }
    println!("X");
    let r_minus_p : Integer = Integer::from(&rr - n_full);
    println!("Y");
    println!("{:?}", r_minus_p);
    return (rr, r_minus_p);
}

fn proth_gmp3(n : Proth) -> (Integer, Integer) {
    let two_to_the_e : Integer = Integer::from(Integer::u_pow_u(2, n.e));
    let n_full : Integer = two_to_the_e * n.t + 1;
    let n_minus_one_over_two : Integer = Integer::from(Integer::from(&n_full - 1) / 2);
    println!("n: {:?} bts", n_full.significant_bits());
    let n_full_ptr = n_full.as_raw();
    let mut rr = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let rr_ptr = rr.as_raw_mut();
    let mut rrt = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let rrt_ptr = rrt.as_raw_mut();
    let mut ai = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let ai_ptr = ai.as_raw_mut();
    let mut aj = Integer::with_capacity((n_full.significant_bits() * 2) as usize);
    let aj_ptr = aj.as_raw_mut();
    let limb_bits : usize;
    {
        let t = Integer::from(3);
        let t_ptr = t.as_raw();
        unsafe {
            assert_eq!(gmp::mpz_size(t_ptr), 1);
        }
        limb_bits = t.capacity();
        
    }
    let mut q = Integer::with_capacity((n_full.significant_bits()  as usize) + limb_bits);
    let q_ptr = q.as_raw_mut();
    let mut i : u32 = 0;
    let bits : u32 = n_minus_one_over_two.significant_bits();
    println!("n_minus_one_over_two: {} bits", n_minus_one_over_two.significant_bits());
    let ai_0 : *mut limb_t ;
    let aj_0 : *mut limb_t ;
    let mut ax_0 : *mut limb_t ;
    let mut ay_0 : *mut limb_t ;
    let q_0 : *mut limb_t ;
    let rr_0 : *mut limb_t ;
    let rrt_0 : *mut limb_t ;
    let n_0 : *const limb_t;
    let n_sz : size_t ;
    let a_sz : size_t ;
    let q_sz : size_t ;
    let rr_sz :  size_t ;
    let rrt_sz : size_t ;
    let double_sz : size_t ;
    unsafe {
        n_sz = gmp::mpz_size(n_full_ptr) as size_t;
        double_sz = n_sz * 2;
        println!("n size: {} double: {}", n_sz, double_sz);
        a_sz = double_sz * 4;
        q_sz = double_sz + 2;
        rr_sz = double_sz * 4;
        rrt_sz = double_sz * 4;
        
        
        ai_0 = gmp::mpz_limbs_modify(ai_ptr, a_sz);
        aj_0 = gmp::mpz_limbs_modify(aj_ptr, a_sz);
        q_0 = gmp::mpz_limbs_modify(q_ptr, q_sz);
        rr_0 = gmp::mpz_limbs_modify(rr_ptr, rr_sz);
        rrt_0 = gmp::mpz_limbs_modify(rrt_ptr, rrt_sz);
        n_0 = gmp::mpz_limbs_read(n_full_ptr);
        
        println!("{}", *n_0.offset(n_sz as isize - 1));
        assert_ne!(*(n_0.offset(n_sz as isize - 1)), 0);
        
        gmp::mpn_zero(ai_0, a_sz);
        gmp::mpn_zero(aj_0, a_sz);
        gmp::mpn_zero(q_0, q_sz);
        gmp::mpn_zero(rr_0, rr_sz);
        gmp::mpn_zero(rrt_0, rrt_sz);
        
        *rr_0 = 1;
        *ai_0 = 3;
    }
    let double_sz = n_sz * 2;
    while i < bits {
        let bit = n_minus_one_over_two.get_bit(i);
        unsafe {
            if (i % 2 == 0) {
                ax_0 = ai_0;
                ay_0 = aj_0;
            } else {
                ax_0 = aj_0;
                ay_0 = ai_0;
            }
//             println!("rr_0: {}", *rr_0);
            if bit {
//                 gmp::mpn_zero(rrt_0, rrt_sz);
//                 println!("rr_0: {} rrt_0: {} ax_0: {}", *rr_0, *rrt_0, *ax_0);
                gmp::mpn_mul(rrt_0, rr_0, n_sz, ax_0, n_sz);
//                 println!("rr_0: {} rrt_0: {} ax_0: {}", *rr_0, *rrt_0, *ax_0);
//                 gmp::mpn_zero(q_0, q_sz);
//                 gmp::mpn_zero(rr_0, rr_sz);
//                 println!("rr_0: {} rrt_0: {} n_0: {}", *rr_0, *rrt_0, *n_0);
                gmp::mpn_tdiv_qr(q_0, rr_0, 0, rrt_0, double_sz, n_0, n_sz);
//                 println!("rr_0: {} rrt_0: {} n_0: {}", *rr_0, *rrt_0, *n_0);
            }
            // square
//             println!("ax_0: {}", *ax_0);
//             gmp::mpn_zero(ay_0, a_sz);
            gmp::mpn_sqr(ay_0, ax_0, n_sz);
//             println!("ay_0: {}", *ay_0);
//             gmp::mpn_zero(q_0, q_sz);
//             println!("ay_0: {}", *ay_0);
            gmp::mpn_tdiv_qr(q_0, ay_0, 0, ay_0, double_sz, n_0, n_sz);
//             println!("ay_0: {}", *ay_0);
        }
        if i % 100 == 0 || i < 100 {
            println!("{}/{} {}", i, bits, (i as f32)/(bits as f32));
        }
        i += 1;
    }
    unsafe {
        gmp::mpz_limbs_finish(ai_ptr, a_sz);
        gmp::mpz_limbs_finish(aj_ptr, a_sz);
        gmp::mpz_limbs_finish(q_ptr, q_sz);
        gmp::mpn_zero(rr_0.offset(n_sz as isize), rr_sz - n_sz);
        gmp::mpz_limbs_finish(rr_ptr, rr_sz);
        gmp::mpz_limbs_finish(rrt_ptr, rrt_sz);
    }
    println!("done");
//     let r : Integer;
//     unsafe {
//         r = Integer::from(Integer::from_raw(*rr_ptr));
//     }
    println!("X");
    let r_minus_p : Integer = Integer::from(&rr - n_full);
    println!("Y");
    println!("{:?}", r_minus_p);
    return (rr, r_minus_p);
}


fn main() {
    let matches = App::new("Hazel's Primality Tester")
        .version("0.1.0")
        .author("Hazel Victoria Campbell")
        .about("Tests Proth numbers for primality")
        .arg(Arg::with_name("number")
            .index(1)
            .required(true)
            .help("A proth number of the format 943*2^3442990+1")
        )
        .arg(Arg::with_name("method")
            .short("m")
            .long("method")
            .value_name("METHOD")
            .help("Select primality testing algorithm")
            .takes_value(true)
            .default_value("gmp2")
        )
        .get_matches();
    assert!(matches.is_present("number"));
    let number_s : &str = matches.value_of("number").expect("What");
    let number_cs = CompleteStr(number_s);
    let number_parsed = parse_proth(number_cs);
    println!("{:?}", number_parsed);
    let n : Proth =  number_parsed.expect("You must provide numbers in the format 943*2^3442990+1").1;
    println!("{:?}", n);
    let method : &str = matches.value_of("method").expect("What");
    match method {
        "gmp_simple" => proth_gmp_simple(n),
        "gmp" => proth_gmp(n),
        "gmp2" => proth_gmp2(n),
        "gmp3" => proth_gmp3(n),
        _ => panic!("You must select a valid method: gmp_simple, gmp, or gmp2")
    };
    println!("exit");
}

// tests

#[cfg(test)]
mod tests {
    use crate::{Proth, proth_gmp, proth_gmp_simple};
    
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn test_proth_gmp() {
        let five = Proth { t: 1, e: 2 };
        assert_eq!((proth_gmp(five)).1, -1);
    }
    #[test]
    fn test_proth_gmp_2() {
        let five_26607 = Proth { t: 5, e: 26607 };
        assert_eq!((proth_gmp(five_26607)).1, -1);
    }
    #[test]
    fn test_proth_gmp_3() {
        let five_26606 = Proth { t: 5, e: 26606 };
        let r = proth_gmp(five_26606);
        let r_simple = proth_gmp_simple(five_26606);
        assert_ne!(r.1, -1);
        assert_eq!(r.0, r_simple.0);
        assert_eq!(r.1, r_simple.1);
    }
}
