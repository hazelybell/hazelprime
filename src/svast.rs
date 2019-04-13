use std::ops::Index;
use std::ops::IndexMut;

use crate::limb::{*};
use crate::vast::{*};

pub struct SVast<'a> {
    v: VastMut<'a>,
    negative: bool
}

impl<'a> From<VastMut<'a>> for SVast<'a> {
    fn from(v: VastMut<'a>) -> SVast<'a> {
        SVast {v: v, negative: false}
    }
}

impl<'a> Index<BigSize> for SVast<'a> {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb {
        &self.v[i]
    }
}

impl<'a> IndexMut<BigSize> for SVast<'a> {
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
