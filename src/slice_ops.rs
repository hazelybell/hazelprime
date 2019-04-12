use crate::big::{*};

pub fn min_length(s: &[Limb]) -> usize {
    for i in (0..s.len()).rev() {
        if s[i] != 0 {
            return i + 1;
        }
    }
    return 0;
}

pub fn zero(s: &mut[Limb]) {
    for i in 0..s.len() {
        s[i] = 0;
    }
}

pub fn multiply_slice(p: &mut[Limb], a: &[Limb], b: &[Limb]) {
    let a_sz = min_length(a);
    let b_sz = min_length(b);
    let p_sz = p.len();
    zero(p);
    assert!(p_sz >= a_sz + b_sz);
    for j in 0..b_sz {
        let mut carry : Limb2 = 0;
        for i in 0..a_sz {
//             println!("i: {} j: {}, i+j: {}", i, j, i + j);
            let mut old = p[i + j] as Limb2;
//             println!("old: {:X} carry: {:X}", old, carry);
            old += carry;
//             println!("a[i]: {:X} b[j]: {:X}", a[i], b[j]);
            let x = (a[i] as Limb2) * (b[j] as Limb2);
            let new = old + x;
//             println!("x: {:X} new: {:X}", x, new);
            if new < x || new < old {
                panic!("Wrapped!");
            }
            carry = new >> LIMB_SHIFT;
            p[i + j] = (new & LIMB_MASK) as Limb;
        }
//         println!("Final carry: {:X}", carry);
        // we don't have anywhere left to put the final carry :(
        assert_eq!(carry & 0xFFFFFFFFFFFFFFFF0000000000000000u128, 0);
        p[a_sz+j] = carry as Limb;
    }
}
