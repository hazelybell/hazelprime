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
    required_sz: Vec<BigSize>,
    next_bits: BigSize
}

trait Planner<'a> {
    fn get_n(&self, p_bits: BigSize) -> BigSize;
    fn plan(&self, n: BigSize) -> Plan;
    fn setup(
        &self,
        n: BigSize, 
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a>;
}

trait MultiplierOps {
    fn x<'a>(&mut self, a: &mut VastMut<'a>, b: &Vast<'_>);
}

struct LongPlanner {}

struct Long<'a> {
    f: Fermat,
    work: VastMut<'a>,
}

impl<'a> Planner<'a> for LongPlanner {
    fn get_n(&self, p_bits: BigSize) -> BigSize {
        p_bits
    }
    fn plan(&self, n: BigSize) -> Plan {
        let sz = div_up(n+1, LIMB_SIZE);
        let required: Vec<BigSize> = vec![sz];
        return Plan {
            required_sz: required,
            next_bits: 0
        };
    }
    fn setup(
        &self,
        n: BigSize, 
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a> {
        match next {
            Some(_v) => panic!("Long multiplier doesn't need a sub multiplier"),
            None => {}
        }
        return Box::new(Long {
            f: Fermat::new(n),
            work: VastMut::from(&mut workspace[0]),
        });
    }
}

impl<'a> MultiplierOps for Long<'a> {
    fn x<'b>(&mut self, a: &mut VastMut<'b>, b: &Vast<'_>) {
        self.work.pod_assign_mul(a, b);
        Fermat::mod_fermat(a, &Vast::from(&self.work), self.f);
    }
}

pub fn fit_in_power_of_two(x: BigSize) -> BigSize {
    let lz = x.leading_zeros();
    let fz = 64 - lz; // first zero counting from the right
    let r: BigSize = 1 << (fz-1);
//     println!("x: {} lz: {} fz: {} r: {}", x, lz, fz, r);
    if x == r {
        return r; // already is a power of two
    } else {
        return r << 1; // next power of two
    }
    return r as BigSize;
}

struct SSRPlanner {}

struct SSR<'a> {
    f: Fermat,
    k: BigSize,
    n: BigSize,
    x: Box<dyn MultiplierOps + 'a>,
    work: VastMut<'a>
}

impl<'a> Planner<'a> for SSRPlanner {
    fn get_n(&self, p_bits: BigSize) -> BigSize {
        fit_in_power_of_two(p_bits)
    }
    fn plan(&self, n: BigSize) -> Plan {
        let nkn = pick_Nkn(n);
        assert_eq!(nkn.N, n);
        let sz = div_up(n+1, LIMB_SIZE);
        let mut required: Vec<BigSize> = Vec::new();
        required.push(sz);
        return Plan {
            required_sz: required,
            next_bits: nkn.n
        }
    }
    fn setup(
        &self,
        n: BigSize, 
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a> {
        let nkn = pick_Nkn(n);
        assert_eq!(nkn.N, n);
        let ssr = SSR {
            f: Fermat::new(n),
            k: nkn.k,
            n: nkn.n,
            x: next.unwrap(),
            work: VastMut::from(&mut workspace[0])
        };
        return Box::new(ssr);
    }
}

impl<'a> MultiplierOps for SSR<'a> {
    fn x<'b>(&mut self, a: &mut VastMut<'b>, b: &Vast<'_>) {
        panic!("unimplemented");
    }
}

fn pick_multiplier<'a>(bits: BigSize) -> Box<dyn Planner<'a>> {
    if bits > 32768 {
        return Box::new(SSRPlanner {});
    } else {
        return Box::new(LongPlanner {});
    }
}

pub fn play(a: &mut VastMut, b: &Vast) {
    let p_bits = a.bits() + b.bits();
    let mut workspaces: Vec<Vec<Big>> = Vec::new();
    let mut planners: Vec<Box<dyn Planner>> = Vec::new();
    planners.push(pick_multiplier(p_bits));
    let n = planners[0].get_n(p_bits);
    
    let mut plans: Vec<Plan> = Vec::new();
    plans.push(planners[0].plan(p_bits));
    
    let mut c_plan = &plans[0];
    while c_plan.next_bits > 0 {
        planners.push(pick_multiplier(c_plan.next_bits));
        plans.push(planners[planners.len()-1].plan(c_plan.next_bits));
        c_plan = &plans[planners.len()-1];
    }
    
    for pi in 0..planners.len() {
        let plan = &plans[pi];
        let mut workspace: Vec<Big> = Vec::new();
        for i in 0..plan.required_sz.len() {
            workspace.push(Big::new(plan.required_sz[i]));
        }
        workspaces.push(workspace);
    }
    
    let mut mults: Vec<Box<dyn MultiplierOps>> = Vec::new();
    let mut last: Option<Box<dyn MultiplierOps>> = None;
    let mut pi = planners.len() - 1;
    for workspace in workspaces.iter_mut().rev() {
        let planner = &planners[pi];
        let up_n: BigSize;
//         let workspace: &mut Vec<Big> = &mut workspaces[pi];
        if pi == 0 {
            up_n = n;
        } else {
            up_n = plans[pi-1].next_bits;
        }
        let mut next = planner.setup(up_n, workspace, last);
        last = Some(next);
        pi -= 1;
        // warning: mults will be backwards from planners, plans, workspaces
    }
    let mut l = last.unwrap();
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
    #[test]
    fn fit_in_power_of_two_() {
        let r = fit_in_power_of_two(512);
        assert_eq!(r, 512);
        let r = fit_in_power_of_two(513);
        assert_eq!(r, 1024);
    }
//     #[test]
//     fn make_plan_1() {
//         let plan = make_plan(3442990);
//         println!("{:?}", plan);
// //         assert!(false);
//     }
}
