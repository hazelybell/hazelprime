use std::ops::Index;
use std::ops::IndexMut;
use std::cmp::Ordering;
use std::fmt;
// use std::ops::Add;
// use std::ops::Sub;

use crate::limb::{*};
use crate::pod::{*};
use crate::vast::{*};
use crate::sbig::{SBig};

pub struct SVast<'a> {
    v: Vast<'a>,
    pub negative: bool
}

pub struct SVastMut<'a> {
    pub v: VastMut<'a>,
    pub negative: bool
}

impl<'a> From<Vast<'a>> for SVast<'a> {
    fn from(v: Vast<'a>) -> SVast<'a> {
        SVast {v: v, negative: false}
    }
}

impl<'a> From<SVastMut<'a>> for SVast<'a> {
    fn from(v: SVastMut<'a>) -> SVast<'a> {
        SVast {v: Vast::from(v.v), negative: v.negative}
    }
}

impl<'a> From<&'a SBig> for SVast<'a> {
    fn from(sb: &'a SBig) -> SVast <'a> {
        SVast {v: Vast::from(&sb.v), negative: sb.negative}
    }
}

impl<'a> From<VastMut<'a>> for SVastMut<'a> {
    fn from(v: VastMut<'a>) -> SVastMut<'a> {
        SVastMut {v: v, negative: false}
    }
}


impl<'a> Index<BigSize> for SVast<'a> {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb {
        &self.v[i]
    }
}

impl<'a> Index<BigSize> for SVastMut<'a> {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb {
        &self.v[i]
    }
}

impl<'a> IndexMut<BigSize> for SVastMut<'a> {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb {
        &mut self.v[i]
    }
}

impl<'a> PartialEq for SVast<'a> {
    fn eq(&self, other: &SVast) -> bool {
        if self.v == 0 {
            return other.v == 0;
        } else {
            return (self.v == other.v) && (self.negative == other.negative);
        }
    }
}
impl<'a> Eq for SVast<'a> {}

impl<'a> Ord for SVast<'a> {
    fn cmp(&self, other: &SVast) -> Ordering {
        if self.v == 0 {
            if other.v == 0 {
                return Ordering::Equal;
            } else {
                if other.negative {
                    return Ordering::Greater;
                } else {
                    return Ordering::Less;
                }
            }
        } else {
            if other.v == 0 {
                if self.negative {
                    return Ordering::Less;
                } else {
                    return Ordering::Greater;
                }
            } else {
                // neither are zero
                if self.negative && !other.negative {
                    return Ordering::Less;
                } else if (!self.negative) && other.negative {
                    return Ordering::Greater;
                } else if (!self.negative) && !other.negative {
                    return self.v.cmp(&other.v);
                } else {
                    return self.v.cmp(&other.v).reverse();
                }
            }
        }
    }
}
impl<'a> PartialOrd for SVast<'a> {
    fn partial_cmp(&self, other: &SVast) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> SVastMut<'a> {
    pub fn zero(&mut self) {
        self.v.zero();
        self.negative = false;
    }
    pub fn pod_cmp(&self, other: &PodOps) -> Ordering {
        if self.negative {
            if self.v.pod_eq(&0) {
                if other.pod_eq(&0) {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Less
            }
        } else {
            self.v.pod_cmp(other)
        }
    }
    pub fn pod_add_assign(&mut self, a: &PodOps) {
        let negative: bool;
        if !self.negative {
            negative = false;
            self.v.pod_add_assign(a);
        } else {
            let c = (&self.v).pod_cmp(a);
            if c == Ordering::Greater {
                negative = true;
                self.v.pod_sub_assign(a);
            } else {
                negative = false;
                self.v.pod_backwards_sub_assign(a);
            }
        }
        self.negative = negative;
    }

    pub fn pod_sub_assign(&mut self, a: &PodOps) {
        let negative: bool;
        if self.negative {
            negative = true;
            self.v.pod_add_assign(a);
        } else { // self is positive
            let c = (&self.v).pod_cmp(a);
            if c == Ordering::Greater {
                negative = false;
                self.v.pod_sub_assign(a);
            } else {
                negative = true;
                self.v.pod_backwards_sub_assign(a);
            }
        }
        self.negative = negative;
    }
}

impl<'a> fmt::Display for SVastMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.negative {
            write!(f, "-{}", self.v.to_hex())
        } else {
            write!(f, "+{}", self.v.to_hex())
        }
    }
}

