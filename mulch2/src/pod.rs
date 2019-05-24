#![warn(rust_2018_idioms)]

use crate::limb::{*};

pub trait Pod {
    fn limbs(&self) -> usize;
    fn get_limb(&self, i: usize) -> &Limb;
    fn as_slice(&self) -> &[Limb];
    fn as_mut_slice(&mut self) -> &mut [Limb];
}

pub trait PodOps {
    fn iter(&self) -> std::slice::Iter<Limb>;
}

impl<T> PodOps for T where T: Pod {
    fn iter(&self) -> std::slice::Iter<Limb> {
        self.as_slice().iter()
    }
}

pub struct PodIterator<'a, T: Pod> {
    pod: &'a T,
    i: usize
}

impl<'a, T> Iterator for PodIterator<'a, T> where T: Pod {
    type Item = &'a Limb;
    fn next(&mut self) -> Option<&'a Limb> {
        let r: Option<&'a Limb>;
        if self.i < self.pod.limbs() {
            r = Some(self.pod.get_limb(self.i));
        } else {
            r = None;
        }
        self.i += 1;
        return r;
    }
}
