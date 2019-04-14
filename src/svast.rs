use std::ops::Index;
use std::ops::IndexMut;
use std::cmp::Ordering;
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
    v: VastMut<'a>,
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

pub fn add_assign_svast_pod(dest: &mut SVastMut, a: &Pod) {
    let negative: bool;
    if !dest.negative {
        negative = false;
        add_assign_pod(&mut dest.v, a);
    } else {
        let c = cmp_pod(&mut dest.v, a);
        if c == Ordering::Greater {
            negative = true;
            sub_assign_pod(&mut dest.v, a);
        } else {
            negative = false;
            backwards_sub_assign_pod(&mut dest.v, a);
        }
    }
    dest.negative = negative;
}

pub fn sub_assign_svast_pod(dest: &mut SVastMut, a: &Pod) {
    let negative: bool;
    if dest.negative {
        negative = true;
        add_assign_pod(&mut dest.v, a);
    } else { // dest is positive
        let c = cmp_pod(&mut dest.v, a);
        if c == Ordering::Greater {
            negative = false;
            sub_assign_pod(&mut dest.v, a);
        } else {
            negative = true;
            backwards_sub_assign_pod(&mut dest.v, a);
        }
    }
    dest.negative = negative;
}
