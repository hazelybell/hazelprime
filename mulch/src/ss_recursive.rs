#![warn(rust_2018_idioms)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::cmp::max;
use std::f64;

use crate::limb::{*};
use crate::big::{*};
use crate::big_mod_f::{*};
use crate::ss_simple::{*};
use crate::fermat::{*};
use crate::vast::{*};
use crate::pod::{*};

struct Plan {
    required_sz: Vec<BigSize>,
}

trait Planner<'a> {
    fn next_goal(&self) -> Goal;
    fn plan(&self) -> Plan;
    fn setup(
        &self,
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a>;
}

pub trait MultiplierOps {
    fn x(&mut self, a: &mut VastMut, b: &Vast);
}

#[derive(PartialEq,Eq)]
enum Goal {
    ModN(BigSize),
    PBits(BigSize),
    Done
}

struct LongPlanner {
    n: BigSize,
    p_bits: BigSize
}

impl LongPlanner {
    fn new(goal: Goal) -> LongPlanner {
        match goal {
            Goal::ModN(n) => {
                return LongPlanner {
                    n: n,
                    p_bits: n*2+2,
                };
            }
            Goal::PBits(b) => {
                return LongPlanner {
                    n: b,
                    p_bits: b+1
                };
            }
            Goal::Done => {
                panic!("Recursion error, planning for done!");
            }
        }
    }
}

struct Long<'a> {
    f: Fermat,
    work: VastMut<'a>,
}

impl<'a> Planner<'a> for LongPlanner {
    fn next_goal(&self) -> Goal {
        Goal::Done
    }
    fn plan(&self) -> Plan {
        let sz = div_up(self.p_bits, LIMB_SIZE);
        println!("Requesting {} bits", sz * LIMB_SIZE);
        let required: Vec<BigSize> = vec![sz];
        return Plan {
            required_sz: required,
        };
    }
    fn setup(
        &self,
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a> {
        match next {
            Some(_v) => panic!("Long multiplier doesn't need a sub multiplier"),
            None => {}
        }
        return Box::new(Long {
            f: Fermat::new(self.n),
            work: VastMut::from(&mut workspace[0]),
        });
    }
}

impl<'a> MultiplierOps for Long<'a> {
    fn x(&mut self, a: &mut VastMut, b: &Vast) {
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

#[derive(Clone,Copy)]
struct SSRPlanner {
    N: BigSize,
    k: BigSize,
    n: BigSize,
    long_sz: BigSize,
    longer_sz: BigSize,
    twok: BigSize,
    piece_sz: BigSize,
    limbs_each: BigSize,
    piece_work_sz: BigSize,
    sum_sz: BigSize,
}

impl SSRPlanner {
    fn new(goal: Goal) -> SSRPlanner {
        let nkn: Nkn;
        match goal {
            Goal::ModN(n) => {
                nkn = pick_Nkn(n);
            }
            Goal::PBits(b) => {
                nkn = pick_Nkn(b);
            }
            Goal::Done => {
                panic!("Recursion error, planning for done!");
            }
        }
        assert_ne!(nkn.N, 0);
        assert_ne!(nkn.k, 0);
        assert_ne!(nkn.n, 0);
        let longer_sz = div_up(nkn.N+1, LIMB_SIZE);
        let long_sz = div_up(nkn.N, LIMB_SIZE);
        let twok: BigSize = 1 << nkn.k;
        assert!(divides(twok, long_sz));
        let piece_sz = div_up(nkn.n+1, LIMB_SIZE);
        let limbs_each = long_sz / twok;
        let piece_work_sz = piece_sz * 4;
        let sum_sz = long_sz + (piece_sz - 1);
        return SSRPlanner {
            N: nkn.N,
            k: nkn.k,
            n: nkn.n,
            long_sz: long_sz,
            longer_sz: longer_sz,
            twok: twok,
            piece_sz: piece_sz,
            limbs_each: limbs_each,
            piece_work_sz: piece_work_sz,
            sum_sz: sum_sz,
        }
    }
    fn dft_matrix<'a>(&self) -> Vec<BigSize> {
        let shift = 2 * self.n / self.twok;
        let mut DS: Vec<BigSize> = Vec::new();
        for i in 0..self.twok {
            for j in 0..self.twok {
                let e = i * j;
                let ex = e % self.twok;
                let s = ex * shift;
                DS.push(s);
            }
        }
//         let check = ss_dft_matrix(self.k, self.n);
//         let mut work = Big::new(self.piece_sz * 3);
//         for i in 0..(self.twok*self.twok) {
//             work.zero();
//             work[0] = 1;
//             work <<= DS[i as usize];
//             let de = mod_fermat(&work, self.n);
//             assert_eq!(de, check[i as usize]);
//         }
        return DS;
    }
    fn idft_matrix(&self) -> Vec<BigSize> {
        let shift = 2 * self.n / self.twok;
        let mut DS: Vec<BigSize> = Vec::new();
        for i in 0..self.twok {
            for j in 0..self.twok {
                let e = i * j;
                let mut ex = (e * -1 + self.twok) % self.twok;
                if ex < 0 {
                    ex += self.twok;
                }
                let s = ex * shift;
                DS.push(s);
            }
        }
//         let check = ss_idft_matrix(self.k, self.n);
//         let mut work = Big::new(self.piece_sz * 4);
//         let mut twok_big = Big::new(self.piece_sz);
//         twok_big[0] = self.twok as Limb;
//         println!("twok_big: {}", twok_big);
//         let mut itwok = inv_mod_fermat(&twok_big, self.n);
//         println!("itwok: {}", itwok);
//         for i in 0..(self.twok*self.twok) {
//             println!("{},{}: {}", i % self.twok, i / self.twok, DS[i as usize]);
//             work.zero();
//             work[0] = 1;
//             work <<= DS[i as usize];
//             let de = mod_fermat(&((&work) * (&itwok)), self.n);
//             assert_eq!(de, check[i as usize]);
//         }
        return DS;
    }
}

struct SSR<'a> {
    params: SSRPlanner,
    F: Fermat,
    f: Fermat,
    x: Box<dyn MultiplierOps + 'a>,
    a_split: Vec<VastMut<'a>>,
    b_split: Vec<VastMut<'a>>,
    piece_work: VastMut<'a>,
    D: Vec<BigSize>,
    Di: Vec<BigSize>,
    a_dft: Vec<VastMut<'a>>,
    b_dft: Vec<VastMut<'a>>,
    dft_work: VastMut<'a>,
    itwok: Vast<'a>,
    Ci: Vec<Vast<'a>>,
    sum: VastMut<'a>,
}

impl<'a> SSR<'a> {
    #[cfg(debug_assertions)]
    fn print_ab(&self) {
        println!("A:");
        for i in 0..self.params.twok {
            println!("{:?}", self.a_split[i as usize]);
        }
        println!("B:");
        for i in 0..self.params.twok {
            println!("{:?}", self.b_split[i as usize]);
        }
    }
    #[cfg(not(debug_assertions))]
    fn print_ab(&self) {}
    #[cfg(debug_assertions)]
    fn print_dft_ab(&self) {
        println!("A:");
        for i in 0..self.params.twok {
            println!("{:?}", self.a_dft[i as usize]);
        }
        println!("B:");
        for i in 0..self.params.twok {
            println!("{:?}", self.b_dft[i as usize]);
        }
    }
    #[cfg(not(debug_assertions))]
    fn print_dft_ab(&self) {}
    #[cfg(debug_assertions)]
    fn print_dft_a(&self) {
        println!("A:");
        for i in 0..self.params.twok {
            println!("{:?}", self.a_dft[i as usize]);
        }
    }
    #[cfg(not(debug_assertions))]
    fn print_dft_a(&self) {}
    #[cfg(debug_assertions)]
    fn print_a(&self) {
        println!("A:");
        for i in 0..self.params.twok {
            println!("{:?}", self.a_split[i as usize]);
        }
    }
    #[cfg(not(debug_assertions))]
    fn print_a(&self) {}
}

impl<'a> Planner<'a> for SSRPlanner {
    fn next_goal(&self) -> Goal {
        Goal::ModN(self.n)
    }
    fn plan(&self) -> Plan {
        let k = self.k;
        let N = self.N;
        let mut required: Vec<BigSize> = Vec::new();
        
        for i in 0..self.twok { // a_split
            required.push(self.piece_sz);
        }
        
        for i in 0..self.twok { // b_split
            required.push(self.piece_sz);
        }
        
        required.push(self.piece_work_sz); // piece_work
        
        for i in 0..self.twok { // a_dft
            required.push(self.piece_sz);
        }
        
        for i in 0..self.twok { // b_dft
            required.push(self.piece_sz);
        }
        
        required.push(self.piece_work_sz); // dft_work
        
        required.push(self.piece_sz); // itwok
        
        for i in 0..self.twok { // Ci
            required.push(self.piece_sz);
        }
        
        required.push(self.sum_sz); // sum for carrying
        
        return Plan {
            required_sz: required,
        }
    }
    fn setup(
        &self,
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a> {
        let mut worki = workspace.into_iter();
        
        let mut a_split: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..self.twok {
            a_split.push(VastMut::from(worki.next().unwrap()));
        }
        
        let mut b_split: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..self.twok {
            b_split.push(VastMut::from(worki.next().unwrap()));
        }
        
        let piece_work: VastMut<'a> =VastMut::from(worki.next().unwrap());
        
        let mut a_dft: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..self.twok {
            a_dft.push(VastMut::from(worki.next().unwrap()));
        }
        
        let mut b_dft: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..self.twok {
            b_dft.push(VastMut::from(worki.next().unwrap()));
        }
        
        let D = self.dft_matrix();
        let Di = self.idft_matrix();
        
        let dft_work: VastMut<'a> = VastMut::from(worki.next().unwrap());
        
        let mut itwok: VastMut<'a> = VastMut::from(worki.next().unwrap());
        {
            let mut twok_big = Big::new(self.piece_sz);
            twok_big[0] = self.twok as Limb;
            println!("twok_big: {}", twok_big);
            let mut itwok_big = inv_mod_fermat(&twok_big, self.n);
            println!("itwok: {}", itwok_big);
            let should_be_one = mul_mod_fermat(&twok_big, &itwok_big, self.n);
            assert!(should_be_one== 1);
            for i in 0..itwok_big.limbs() {
                itwok[i] = itwok_big[i];
            }
        }
        
        let mut Ci: Vec<Vast<'a>> = Vec::new(); 
        {
            for j in 0..self.twok {
                let shift = j * self.n / self.twok;
                let mut coeff = Big::new_one(self.piece_sz);
                coeff <<= shift;
                let invcoeff = inv_mod_fermat(&coeff, self.n);
                let mut ci: VastMut<'a> = VastMut::from(worki.next().unwrap());
                for i in 0..invcoeff.limbs() {
                    ci[i] = invcoeff[i];
                }
                Ci.push(Vast::from(ci));
            }
        }
        
        let sum: VastMut<'a> =VastMut::from(worki.next().unwrap());
        
        let ssr = SSR {
            params: *self,
            F: Fermat::new(self.N),
            f: Fermat::new(self.n),
            x: next.unwrap(),
            a_split: a_split,
            b_split: b_split,
            piece_work: piece_work,
            D: D,
            Di: Di,
            a_dft: a_dft,
            b_dft: b_dft,
            dft_work: dft_work,
            itwok: Vast::from(itwok),
            Ci: Ci,
            sum: sum,
        };
        return Box::new(ssr);
    }
}

fn split<'a, 'b>(
    into: &mut Vec<VastMut<'a>>,
    from: &Vast,
    long_sz: BigSize
) {
    let from_sz = from.limbs();
    let piece_sz = into[0].limbs();
    let number = into.len() as BigSize;
    assert!(divides(number, long_sz));
    let limbs_each = long_sz / number;
    let mut i = 0;
    for piece in into {
        let start = i * limbs_each;
        for j in 0..limbs_each {
            let src_j = j + start;
            let l: Limb;
            if src_j < from_sz {
                l = from[src_j];
            } else {
                l = 0;
            }
            piece[j] = l;
        }
        for j in limbs_each..piece_sz {
            piece[j] = 0;
        }
        i += 1;
    }
}

impl<'a> MultiplierOps for SSR<'a> {
    fn x(&mut self, a: &mut VastMut, b: &Vast) {
        let n = self.params.n;
        let long_sz = self.params.long_sz;
        let twok = self.params.twok;
        // split
        split(&mut self.a_split, &Vast::from(&*a), long_sz);
        split(&mut self.b_split, b, long_sz);
        self.print_ab();
        // weight
        for j in 0..twok {
            let shift = j * n / twok;
            self.piece_work.pod_assign_shl(
                &self.a_split[j as usize],
                shift
            );
            Fermat::mod_fermat(
                &mut self.a_split[j as usize], 
                &Vast::from(&self.piece_work), 
                self.f
            );
            self.piece_work.pod_assign_shl(
                &self.b_split[j as usize],
                shift
            );
            Fermat::mod_fermat(
                &mut self.b_split[j as usize], 
                &Vast::from(&self.piece_work), 
                self.f
            );
        }
        self.print_ab();
        // dft
        for i in 0..twok {
            self.piece_work.zero();
            for j in 0..twok {
                let didx = i + j * twok;
                let shift = self.D[didx as usize];
                self.dft_work.zero();
                self.dft_work.pod_assign_shl(
                    &self.a_split[j as usize],
                    shift
                );
                self.piece_work.pod_add_assign(&self.dft_work);
            }
            Fermat::mod_fermat(
                &mut self.a_dft[i as usize],
                &Vast::from(&self.piece_work),
                self.f
            );
        }
        for i in 0..twok {
            self.piece_work.zero();
            for j in 0..twok {
                let didx = i + j * twok;
                let shift = self.D[didx as usize];
                self.dft_work.zero();
                self.dft_work.pod_assign_shl(
                    &self.b_split[j as usize],
                    shift
                );
//                 println!("{},{} {} {}", i, j, shift, self.b_split[j as usize].to_hex());
//                 println!("{},{} {} {}", i, j, shift, self.dft_work.to_hex());
                self.piece_work.pod_add_assign(&self.dft_work);
//                 println!("{}", self.piece_work.to_hex());
            }
//             println!("Before modf {}", self.piece_work.to_hex());
            Fermat::mod_fermat(
                &mut self.b_dft[i as usize],
                &Vast::from(&self.piece_work),
                self.f
            );
//             println!("After modf {}", self.b_dft[i as usize].to_hex());
            if i == 5 {
                assert_eq!(self.b_dft[i as usize].get_limb(0),
                    0x9065FBFDEE10FAC3
                )
            }
        }
        self.print_dft_ab();
        // dot product and recurse!
        for i in 0..twok {
            self.x.x(
                &mut self.a_dft[i as usize],
                &Vast::from(&self.b_dft[i as usize])
            );
        }
        self.print_dft_a();
        // Inverse dft
        for i in 0..twok {
            self.piece_work.zero();
            for j in 0..twok {
                let didx = i + j * twok;
                let shift = self.Di[didx as usize];
                self.dft_work.zero();
                self.dft_work.pod_assign_shl(
                    &self.a_dft[j as usize],
                    shift
                );
//                 println!("{},{} {} {}", i, j, shift, self.dft_work.to_hex());
                self.piece_work.pod_add_assign(&self.dft_work);
//                 println!("{}", self.piece_work.to_hex());
            }
//             println!("Before modf {}", self.piece_work.to_hex());
            self.a_split[i as usize].zero();
            Fermat::mod_fermat(
                &mut self.a_split[i as usize],
                &Vast::from(&self.piece_work),
                self.f
            );
//             println!("After modf {}", self.a_split[i as usize].to_hex());
        }
//         println!("Before *itwok");
        self.print_a();
        for i in 0..twok {
            self.x.x(
                &mut self.a_split[i as usize],
                &self.itwok
            );
        }
        self.print_a();
        // unweight
        for i in 0..twok {
            self.x.x(
                &mut self.a_split[i as usize],
                &self.Ci[i as usize]
            );
        }
        self.print_a();
        // do carrying
        for i in (1..twok).rev() {
            self.sum.pod_add_assign(&self.a_split[i as usize]);
            self.sum.pod_shl_assign(LIMB_SIZE * self.params.limbs_each);
        }
        self.sum.pod_add_assign(&self.a_split[0]);
//         println!("{:?}", self.sum);
        a.zero();
        Fermat::mod_fermat(
            a,
            &Vast::from(&self.sum),
            self.F
        );
//         println!("{:?}", a);
    }
}

fn pick_multiplier<'a>(goal: Goal) -> Box<dyn Planner<'a>> {
    let bits: BigSize;
    match goal {
        Goal::ModN(n) => { bits = n; }
        Goal::PBits(b) => { bits = b; }
        Goal::Done => { panic!("Recursion error, planning for done!"); }
    }
    if bits > 512 {
        return Box::new(SSRPlanner::new(goal));
    } else {
        return Box::new(LongPlanner::new(goal));
    }
}

pub fn recursive_setup<'a>(
    p_bits: BigSize,
    mut workspaces: &'a mut Vec<Vec<Big>>
) -> Box<dyn MultiplierOps + 'a> {
    let mut planners: Vec<Box<dyn Planner>> = Vec::new();
    planners.push(pick_multiplier(Goal::PBits(p_bits)));
    
    let mut next_goal = planners[0].next_goal();
    while next_goal != Goal::Done {
        planners.push(pick_multiplier(next_goal));
        next_goal = planners[planners.len()-1].next_goal();
    }
    
    for planner in (&mut planners) {
        let plan = planner.plan();
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
        let mut next = planner.setup(workspace, last);
        last = Some(next);
        if pi > 0 {
            pi -= 1;
        }
        // warning: mults will be backwards from planners, plans, workspaces
    }
    let mut l = last.unwrap();
    return l;
}

pub fn recursive_multiply(a: &mut VastMut, b: &Vast) {
    let mut workspaces: Vec<Vec<Big>> = Vec::new();
    let p_bits = a.bits() + b.bits();
    println!("p_bits: {}", p_bits);
    if p_bits > a.limbs() * LIMB_SIZE {
        panic!("a not big enough to hold result!")
    }
    let mut mult = recursive_setup(p_bits, &mut workspaces);
    mult.x(a, b);
}

pub struct KN {
    k: BigSize,
    n: BigSize,
}

pub fn pick_kn(N: BigSize, k: BigSize) -> KN {
    let mut best = KN {k: 0, n: 0};
    let twok = 1 << k;
    if (!divides(twok, N)) {
        // if this 2^k doesnt divide N then a bigger one wont
        return best;
    }
    let n_min = 2 * N / twok + k;
    let n_max = N/2;
    println!("    Trying k={} twok={} n_min=2N/2^k+k={} n_max={}", k, twok, n_min, n_max);
    let piece_sz = N / twok;
    println!("    Piece size: {}", piece_sz);
    if piece_sz % LIMB_SIZE > 0 {
        return best;
    }
    let n = div_up(n_min, twok)*twok;
    if n <= n_max {
        assert!(divides(twok, n));
        println!("    Satisfied: N={}, k={}, twok={}, n={}", N, k, twok, n);
        let next_n = pick_Nkn(n).N;
        println!("    Next n: {}", next_n);
        let waste_bits = (next_n - piece_sz) * twok;
        println!("    Waste bits: {}", waste_bits);
        return KN {
            k: k,
            n: n,
        };
    }
    return best;
}

pub fn pick_Nkn(N_start: BigSize) -> Nkn {
    // find a suitable N, k and n
    if N_start <= 512 {
        println!("Do other multiplication...");
        return Nkn {N: N_start, k: 0, n: 0};
    }
    let N_max = N_start * 2; // I have no clue what to set this to :(
    let mut N = N_start;
    let mut best = Nkn { N: 0, k: 0, n: 0};
    let optimal_twok = (N_start as f64).sqrt();
    let optimal_k = optimal_twok.log2();
    let k: BigSize = optimal_k.floor() as BigSize;
    println!(
        "Optimal twok={} k={} k={}",
        optimal_twok, optimal_k, k);
    while N < N_max {
        println!("Trying N={}", N);
        let kn = pick_kn(N, k);
        if kn.n != 0 {
            println!("Best so far: N={} k={} n={}", N, kn.k, kn.n);
            best = Nkn {
                N: N,
                k: kn.k,
                n: kn.n
            };
            break;
        }
        N = (N / 512 + 1) * 512;
    }
    println!("Final: N={} k={} n={}", best.N, best.k, best.n);
    return best;
}

// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::ss_recursive::{*};
    #[test]
    fn pick_Nkn_1() {
        let r = pick_Nkn(5000000);
        assert_ne!(r.N, 0);
        assert_ne!(r.k, 0);
        assert_ne!(r.n, 0);
        println!("{:?}", r);
    }
    #[test]
    fn fit_in_power_of_two_() {
        let r = fit_in_power_of_two(512);
        assert_eq!(r, 512);
        let r = fit_in_power_of_two(513);
        assert_eq!(r, 1024);
    }
    #[test]
    fn multiply_2048() {
        let mut ba = Big::from_hex(
             "B954E7DFEE6CCE82F19BC30B53E6B6E15081CD494DD1652CEA6A30D134316E1\
             452C5BB2012B0889BB5A148093ED8CA2DDA1FA3E09D4473C6EAA90FC7809247C\
             FB7FE805D7095BD679653E016B74FFA844E7401BBE68BB7B25754B87F0D07AD0\
             72DBBEAB6F3E9B7C94ED93B8665FEBEE18091EB2BDFB021A5DA9DDC981F23E12"
        );
        let a_sz = ba.limbs()*2;
        let mut ba = big_extend(ba, a_sz);
        let bb = Big::from_hex(
             "45BAA2EE705DDC4BDB71C3B963B612EC2CFE3B14E836C9988D260410DC9CF4C\
             B11C1E091B2EE874887BFBFBB5FD136859D2E887D96F43D0328C0FF3BAFDF67C\
             E3C71874F014F0C076109C3112C9C051F88B60F929967758F58E5041728C98B5\
             0B099D03817A54400BB065726B0D5D8DB328957083535EF65229F3FC0C65F691"
        );
        let mut a: VastMut = VastMut::from(&mut ba);
        let b: Vast = Vast::from(&bb);
        recursive_multiply(&mut a, &b);
        assert_eq!(a.to_hex(), 
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
    fn multiply_2048_direct() {
        let mut ba = Big::from_hex(
             "B954E7DFEE6CCE82F19BC30B53E6B6E15081CD494DD1652CEA6A30D134316E1\
             452C5BB2012B0889BB5A148093ED8CA2DDA1FA3E09D4473C6EAA90FC7809247C\
             FB7FE805D7095BD679653E016B74FFA844E7401BBE68BB7B25754B87F0D07AD0\
             72DBBEAB6F3E9B7C94ED93B8665FEBEE18091EB2BDFB021A5DA9DDC981F23E12"
        );
        let a_sz = ba.limbs()*2;
        let mut ba = big_extend(ba, a_sz);
        let bb = Big::from_hex(
             "45BAA2EE705DDC4BDB71C3B963B612EC2CFE3B14E836C9988D260410DC9CF4C\
             B11C1E091B2EE874887BFBFBB5FD136859D2E887D96F43D0328C0FF3BAFDF67C\
             E3C71874F014F0C076109C3112C9C051F88B60F929967758F58E5041728C98B5\
             0B099D03817A54400BB065726B0D5D8DB328957083535EF65229F3FC0C65F691"
        );
        let mut a: VastMut = VastMut::from(&mut ba);
        let b: Vast = Vast::from(&bb);
        let p_bits = a.bits() + b.bits();
        let mut workspaces: Vec<Vec<Big>> = Vec::new();
        let mut mult = recursive_setup(p_bits, &mut workspaces);
        mult.x(&mut a, &b);
        assert_eq!(a.to_hex(), 
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
//     #[test]
//     fn make_plan_1() {
//         let plan = make_plan(3442990);
//         println!("{:?}", plan);
// //         assert!(false);
//     }
}
