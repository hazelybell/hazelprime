#![warn(rust_2018_idioms)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::cmp::max;

use crate::limb::{*};
use crate::big::{*};
use crate::big_mod_f::{*};
use crate::ss_simple::{*};

#[derive(Debug)]
pub enum Step {
    SS(Nkn),
    Long
}

pub fn pick_Nkn(p_bits: BigSize) -> Nkn {
    // find a suitable N, k and n
    let N_min = p_bits + 1;
    let N_max = N_min * 2; // I have no clue what to set this to :(
    let k_max: BigSize = 16;
    let k_min: BigSize = 1;
    let mut N = N_min;
    let mut best = Nkn { N: 0, k: 0, n: 0};
    let mut best_waste = BigSize::max_value();
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
                let next_n = get_next_power_of_two(n);
                println!("Next power of two after n: {}", next_n);
                let waste = (next_n - piece_sz) * twok;
                println!("Waste bits: {}", waste);
                let optimal_twok = (N as f64).sqrt();
                println!("Optimal twok={}", optimal_twok);
                if waste <= best_waste {
                    println!("Best so far.");
                    best = Nkn {
                        N: N,
                        k: k,
                        n: n
                    };
                    best_waste = waste;
                }
            }
            println!("");
        }
        N = get_next_power_of_two(N);
    }
    return best;
}

pub fn make_plan(p_bits: BigSize) -> Vec<Step> {
    let mut plan: Vec<Step> = Vec::new();
    let mut c_bits = p_bits;
    while c_bits >= 32768 {
        println!("bits: {}", c_bits);
        let nkn = pick_Nkn(c_bits);
        c_bits = nkn.n;
        plan.push(Step::SS(nkn));
    }
    println!("bits: {}", c_bits);
    plan.push(Step::Long);
    return plan;
}

pub struct Space {
    a: Big,
    b: Big,
    p: Big
}

pub fn make_workspace(plan: Vec<Step>) -> Vec<Space> {
    let workspace: Vec<Space> = Vec::new();
    return workspace;
}
// 
// pub fn rec_multiply(a: Big, b: Big) -> Big {
//     let a_bits = a.bits();
//     let b_bits = b.bits();
//     let a_sz = a.length();
//     let b_sz = b.length();
//     let p_bits = a_bits + b_bits; // number of bits in the product
//     let plan = make_plan();
//     let workspace = make_workspace(plan);
// }
// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::ss_recursive::{*};
    #[test]
    fn pick_Nkn_1() {
        let r = pick_Nkn(3442990);
        println!("{:?}", r);
    }
    #[test]
    fn make_plan_1() {
        let plan = make_plan(3442990);
        println!("{:?}", plan);
        assert!(false);
    }
}
