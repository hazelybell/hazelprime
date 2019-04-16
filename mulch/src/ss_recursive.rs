#![warn(rust_2018_idioms)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::cmp::max;
use std::rc::Rc;

use crate::limb::{*};
use crate::big::{*};
use crate::big_mod_f::{*};
use crate::ss_simple::{*};
use crate::fermat::{*};
use crate::vast::{*};
use crate::pod::{*};

struct Plan {
    required_sz: BigSize,
    next_bits: BigSize
}

trait Planned {
    fn plan(n: BigSize) -> Plan;
}

trait MultiplierOps {
    fn x<'a>(&mut self, a: &mut VastMut<'a>, b: &Vast<'_>);
}

trait MultiplierFactory<'a> {
    fn setup(n: BigSize, big_work: &'a mut Big) -> Box<dyn MultiplierOps + 'a>;
}

struct Long<'a> {
    f: Fermat,
    work: VastMut<'a>,
}

impl<'a> Planned for Long<'a> {
    fn plan(n: BigSize) -> Plan {
        let sz = div_up(n+1, LIMB_SIZE);
        return Plan {
            required_sz: sz,
            next_bits: 0
        }
    }
}

impl<'a> MultiplierOps for Long<'a> {
    fn x<'b>(&mut self, a: &mut VastMut<'b>, b: &Vast<'_>) {
        self.work.pod_assign_mul(a, b);
        Fermat::mod_fermat(a, &Vast::from(&self.work), self.f);
    }
}

impl<'a> MultiplierFactory<'a> for Long<'a> {
    fn setup(n: BigSize, big_work: &'a mut Big) -> Box<dyn MultiplierOps + 'a> {
        Box::new(Long {
            f: Fermat::new(n),
            work: VastMut::from(big_work),
        })
    }
}

struct Long2<'a> {
    f: Fermat,
    work: VastMut<'a>,
}

impl<'a> Planned for Long2<'a> {
    fn plan(n: BigSize) -> Plan {
        let sz = div_up(n+1, LIMB_SIZE);
        return Plan {
            required_sz: sz,
            next_bits: 0
        }
    }
}

impl<'a> MultiplierOps for Long2<'a> {
    fn x<'b>(&mut self, a: &mut VastMut<'b>, b: &Vast<'_>) {
        self.work.pod_assign_mul(a, b);
        Fermat::mod_fermat(a, &Vast::from(&self.work), self.f);
    }
}

impl<'a> MultiplierFactory<'a> for Long2<'a> {
    fn setup(n: BigSize, big_work: &'a mut Big) -> Box<dyn MultiplierOps + 'a> {
        Box::new(Long2 {
            f: Fermat::new(n),
            work: VastMut::from(big_work),
        })
    }
}

fn setup_long12<'a>(n: BigSize, big_work: &'a mut Big) -> Box<dyn MultiplierOps + 'a> {
    if (n % 2) == 0 {
        Long::setup(n, big_work)
    } else {
        Long2::setup(n, big_work)
    }
}

struct SSR<'a> {
    f: Fermat,
    k: BigSize,
    n: BigSize,
    work: VastMut<'a>
}

impl<'a> MultiplierOps for SSR<'a> {
    fn x<'b>(&mut self, a: &mut VastMut<'b>, b: &Vast<'_>) {
        panic!("unimplemented");
    }
}

pub fn play(a: &mut VastMut, b: &Vast) {
    let n = a.bits() + b.bits();
    let sz = div_up(n+1, LIMB_SIZE);
    let mut big_work = Big::new(sz);
    let mut l = setup_long12(n, &mut big_work);
    l.x(a, b);
}

pub fn pick_Nkn(n: BigSize) -> Nkn {
    // find a suitable N, k and n
    let N_min = n + 1;
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
            if (twok > n) {
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

// pub fn make_plan(n: BigSize) -> Vec<Step> {
//     let mut plan: Vec<Step> = Vec::new();
//     let mut c_bits = n;
//     while c_bits >= 32768 {
//         println!("bits: {}", c_bits);
//         let nkn = pick_Nkn(c_bits);
//         c_bits = nkn.n;
//         plan.push(Step::SS(nkn));
//     }
//     println!("bits: {}", c_bits);
//     plan.push(Step::Long);
//     return plan;
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
//     #[test]
//     fn make_plan_1() {
//         let plan = make_plan(3442990);
//         println!("{:?}", plan);
// //         assert!(false);
//     }
}
