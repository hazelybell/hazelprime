use rug::Integer;
use rug::Assign;
use gmp_mpfr_sys::gmp;
use gmp_mpfr_sys::gmp::{limb_t, size_t};
use std::mem::size_of;

#[derive(Debug, Copy, Clone)]
pub struct Proth {
    pub t: u32,
    pub e: u32, 
}

pub fn simple(n : Proth) -> (Integer, Integer) {
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

pub fn medium(n : Proth) -> (Integer, Integer) {
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

pub fn low(n : Proth) -> (Integer, Integer) {
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
        a_sz = double_sz;
        q_sz = a_sz - n_sz + 1;
        rr_sz = double_sz;
        rrt_sz = double_sz;
        
        
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
            if i % 2 == 0 {
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

fn find_m(n : & Integer) -> (Integer, size_t) {
    // find 1/n mod 2^(64*w) ?
    // euclidean algorithm?
    assert!(n.is_odd());
    // since n is odd
    let n_copy = Integer::from(n);
    let mut two_to_k = Integer::from(1);

    let n_ptr = n.as_raw();
    let n_sz : size_t;
    unsafe {
        n_sz = gmp::mpz_size(n_ptr) as size_t;
    }
    
//     let bits : i32 =  (size_of::<limb_t>() as i32) * 8 * (w as i32);
    let target_sz : i32 = (n_sz as i32) * 2;
    let bits : i32 = target_sz * (size_of::<limb_t>() as i32) * 8;
    two_to_k = two_to_k << bits; // compute 2^bits
//     println!("shifted: {:X}", two_to_k);
//     println!("{:X}", n_copy);
    let two_to_k2 = Integer::from(&two_to_k);
    let (mut m, _r) = two_to_k2.div_rem_floor(n_copy);
//     println!("{:X}", m);
    m.shrink_to_fit();
    let m_ptr = m.as_raw();
    let two_to_k_ptr = two_to_k.as_raw();
    
    let m_sz : size_t;
    let two_to_k_sz : size_t;
    unsafe {
        m_sz = gmp::mpz_size(m_ptr) as size_t;
        two_to_k_sz = gmp::mpz_size(two_to_k_ptr) as size_t;
    }
    println!("n_sz: {} double: {} two_to_k_sz: {} m_sz: {}", n_sz, n_sz * 2, two_to_k_sz, m_sz);
    return (m, bits as size_t);
}


pub fn barrett(n : Proth) -> (Integer, Integer) {
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
    }
    double_sz = n_sz * 2;
    println!("n size: {} double: {}", n_sz, double_sz);
    a_sz = double_sz;
    q_sz = a_sz - n_sz + 1;
    rr_sz = double_sz;
    rrt_sz = double_sz;
        
    unsafe {
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
    
    let (m, m_shift) = find_m(&n_full);
    let m = Integer::from(&m);
    let m_ptr = m.as_raw();
    let m_0 : *const limb_t;
    let m_sz : size_t;
    unsafe {
        m_0 = gmp::mpz_limbs_read(m_ptr);
        m_sz = gmp::mpz_size(m_ptr) as size_t;
    }
    
    let mut q2 = Integer::from(0);
    let q2_ptr = q2.as_raw_mut();
    let q2_0 : *mut limb_t ;
    let q2_sz : size_t = m_sz + a_sz;
    let q2_shifted : *mut limb_t ;
    
    unsafe {
        q2_0 = gmp::mpz_limbs_modify(q2_ptr, q2_sz);
        gmp::mpn_zero(q2_0, q2_sz);
    }
    
    let limb_sz : size_t = (size_of::<limb_t>() as size_t) * 8;
    assert_eq!(m_shift % limb_sz, 0);
    let m_shift_limbs = m_shift / limb_sz;
    let q2_shifted_sz = q2_sz - m_shift_limbs;
    unsafe {
        q2_shifted = q2_0.offset(m_shift_limbs as isize);
    }
    
    println!("q2_sz: {} m_shift_limbs: {} q2_shifted_sz: {}", q2_sz, m_shift_limbs, q2_shifted_sz);
    
    let mut qn = Integer::from(0);
    let qn_ptr = qn.as_raw_mut();
    let qn_0 : *mut limb_t ;
    let qn_sz : size_t = q2_shifted_sz + n_sz;
    unsafe {
        qn_0 = gmp::mpz_limbs_modify(qn_ptr, qn_sz);
        gmp::mpn_zero(qn_0, qn_sz);
    }
    
    println!("qn_sz: {} ", q2_shifted_sz + n_sz);

    while i < bits {
        let bit = n_minus_one_over_two.get_bit(i);
        unsafe {
            if i % 2 == 0 {
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
//             gmp::mpn_tdiv_qr(q_0, ay_0, 0, ay_0, double_sz, n_0, n_sz);
            // div by n = multiply by 1/n
            // q := (a * m) in barret reduction
            gmp::mpn_mul(q2_0, ay_0, a_sz, m_0, m_sz);
            let q2_last = *(q2_0.offset((q2_sz as isize) - 1));
            assert_eq!(q2_last, 0);
            let q2_last = *(q2_0.offset((q2_sz as isize) - 2));
            let n_last = *(n_0.offset((n_sz as isize) - 1));
            assert!(n_last > q2_last);
            
            // >> k in barret reduction
            // compute q * n

            gmp::mpn_mul(qn_0, n_0, n_sz, q2_shifted, q2_shifted_sz);
            let qn_last = *(qn_0.offset((qn_sz as isize) - 1));
            assert_eq!(qn_last, 0);
            gmp::mpn_sub(ay_0, ay_0, a_sz, qn_0, qn_sz-1);
            for o in n_sz..a_sz {
                assert_eq!(*(ay_0.offset(o as isize)), 0);
            }
            let v = gmp::mpn_cmp(ay_0, n_0, n_sz);
            assert!(v < 0);
            
//             println!("ay_0: {}", *ay_0);
//             panic!("stop");
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

