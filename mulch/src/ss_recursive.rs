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
    next_n: BigSize
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

pub trait MultiplierOps {
    fn x(&mut self, a: &mut VastMut, b: &Vast);
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
        println!("Requesting {} bits", sz * LIMB_SIZE);
        let required: Vec<BigSize> = vec![sz];
        return Plan {
            required_sz: required,
            next_n: 0
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

struct SSRPlanner {}

struct SSR<'a> {
    f: Fermat,
    k: BigSize,
    n: BigSize,
    x: Box<dyn MultiplierOps + 'a>,
    a_split: Vec<VastMut<'a>>,
    b_split: Vec<VastMut<'a>>,
}

impl<'a> Planner<'a> for SSRPlanner {
    fn get_n(&self, p_bits: BigSize) -> BigSize {
        fit_in_power_of_two(p_bits)
    }
    fn plan(&self, N: BigSize) -> Plan {
        let nkn = pick_Nkn(N);
        assert_eq!(nkn.N, N);
        let k = nkn.k;
        let longer_sz = div_up(N+1, LIMB_SIZE);
        let long_sz = div_up(N, LIMB_SIZE);
        let mut required: Vec<BigSize> = Vec::new();
        let twok: BigSize = 1 << k;
        let pieces = twok as usize;
        assert!(divides(twok, long_sz));
        let piece_sz = long_sz / twok;
        for i in 0..pieces { // a_split
            required.push(piece_sz);
        }
        for i in 0..pieces { // b_split
            required.push(piece_sz);
        }
        return Plan {
            required_sz: required,
            next_n: nkn.n
        }
    }
    fn setup(
        &self,
        N: BigSize, 
        workspace: &'a mut Vec<Big>,
        next: Option<Box<dyn MultiplierOps + 'a>>
    ) -> Box<dyn MultiplierOps + 'a> {
        let nkn = pick_Nkn(N);
        assert_eq!(nkn.N, N);
        let k = nkn.k;
        let twok = 1 << k;
        let mut worki = workspace.into_iter();
        let mut a_split: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..twok {
            a_split.push(VastMut::from(worki.next().unwrap()));
        }
        let mut b_split: Vec<VastMut<'a>> = Vec::new();
        for _i in 0..twok {
            b_split.push(VastMut::from(worki.next().unwrap()));
        }
        let ssr = SSR {
            f: Fermat::new(N),
            k: nkn.k,
            n: nkn.n,
            x: next.unwrap(),
            a_split: a_split,
            b_split: b_split
        };
        return Box::new(ssr);
    }
}

fn split<'a, 'b>(into: &mut Vec<VastMut<'a>>, from: &Vast) {
    let long_sz = from.limbs();
    let piece_sz = into[0].limbs();
    let number = into.len() as BigSize;
    assert!(divides(number, long_sz));
    let limbs_each = long_sz / number;
    let mut i = 0;
    for piece in into {
        let start = i * limbs_each;
        for j in 0..limbs_each {
            let src_j = j + start;
            piece[j] = from[src_j];
        }
        i += 1;
    }
}

impl<'a> MultiplierOps for SSR<'a> {
    fn x(&mut self, a: &mut VastMut, b: &Vast) {
        // might need to pad these first
        split(&mut self.a_split, &Vast::from(&*a));
        split(&mut self.b_split, b);
        panic!("unimplemented");
    }
}

fn pick_multiplier<'a>(bits: BigSize) -> Box<dyn Planner<'a>> {
    if bits > 512 {
        return Box::new(SSRPlanner {});
    } else {
        return Box::new(LongPlanner {});
    }
}

pub fn recursive_setup<'a>(
    p_bits: BigSize,
    mut workspaces: &'a mut Vec<Vec<Big>>
) -> Box<dyn MultiplierOps + 'a> {
    let mut planners: Vec<Box<dyn Planner>> = Vec::new();
    planners.push(pick_multiplier(p_bits));
    let n = planners[0].get_n(p_bits);
    
    let mut plans: Vec<Plan> = Vec::new();
    plans.push(planners[0].plan(n));
    
    let mut c_plan = &plans[0];
    while c_plan.next_n > 0 {
        planners.push(pick_multiplier(c_plan.next_n));
        plans.push(planners[planners.len()-1].plan(c_plan.next_n));
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
            up_n = plans[pi-1].next_n;
        }
        let mut next = planner.setup(up_n, workspace, last);
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
    waste: BigSize
}

pub fn pick_kn(N: BigSize) -> KN {
    let k_max: BigSize = 16;
    let k_min: BigSize = 1;
    let mut best = KN {k: 0, n: 0, waste: BigSize::max_value()};
    for k in k_min..(k_max+1) {
        let twok = 1 << k;
        if (!divides(twok, N)) {
            // if this 2^k doesnt divide N then a bigger one wont
            break;
        }
        let n_min = 2 * N / twok + k;
        let n_max = N/2;
        println!("    Trying k={} twok={} n_min=2N/2^k+k={} n_max={}", k, twok, n_min, n_max);
        let piece_sz = N / twok;
        println!("    Piece size: {}", piece_sz);
        if piece_sz % LIMB_SIZE > 0 {
            break;
        }
        let n = div_up(n_min, twok)*twok;
        if n <= n_max {
            assert!(divides(twok, n));
            println!("    Satisfied: N={}, k={}, twok={}, n={}", N, k, twok, n);
            let next_n = get_next_power_of_two(n);
            println!("    Next power of two after n: {}", next_n);
            let waste = (next_n - piece_sz) * twok;
            println!("    Waste bits: {}", waste);
            let optimal_twok = (N as f64).sqrt();
            println!("    Optimal twok={}", optimal_twok);
            if waste <= best.waste {
                println!("    Best so far.");
                best = KN {
                    k: k,
                    n: n,
                    waste: waste
                };
            }
        }
        println!("");
    }
    return best;
}

pub fn pick_Nkn(N_start: BigSize) -> Nkn {
    // find a suitable N, k and n
    let N_max = N_start * 2; // I have no clue what to set this to :(
    let mut N = N_start;
    let mut best = Nkn { N: 0, k: 0, n: 0};
    let mut best_waste = BigSize::max_value();
    while N < N_max {
        println!("Trying N={}", N);
        let kn = pick_kn(N);
        println!("Best so far: N={} k={} n={} waste={}", N, kn.k, kn.n, kn.waste);
        if kn.waste < best_waste {
            best = Nkn {
                N: N,
                k: kn.k,
                n: kn.n
            }
        }
        N = get_next_power_of_two(N);
    }
    assert_ne!(best.N, 0);
    assert_ne!(best.k, 0);
    assert_ne!(best.n, 0);
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
        let r = pick_Nkn(400);
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
