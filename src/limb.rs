#![warn(rust_2018_idioms)]

use std::fmt;
use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Sub;

pub type BigSize = isize;
// pub const SIZE_SHIFT : usize = 63;
pub type Limb = u64;
pub type Limb2 = u128;
pub const LIMB_SHIFT : usize = 64;
pub const LIMB_SIZE : BigSize = LIMB_SHIFT as BigSize;
pub const LIMB_MASK : Limb2 = 0xFFFFFFFFFFFFFFFFu128;

pub fn div_up(n : BigSize, d : BigSize) -> BigSize {
    let mut r = n / d;
    if r * d < n {
        r = r + 1;
    }
    return r;
}

#[derive(Clone,Copy)]
pub struct SLimb {
    v: Limb,
    negative: bool
}

impl SLimb {
}

impl From<Limb> for SLimb {
    fn from(l: Limb) -> Self {
        SLimb {v: l, negative: false}
    }
}

impl PartialEq for SLimb {
    fn eq(&self, other: &SLimb) -> bool {
        if self.v == 0 {
            return other.v == 0;
        } else {
            return (self.v == other.v) && (self.negative == other.negative);
        }
    }
}
impl Eq for SLimb {}

impl fmt::Debug for SLimb {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        if self.negative {
            write!(f, "SLimb {{-{}}}", self.v)
        } else {
            write!(f, "SLimb {{+{}}}", self.v)
        }
    }
}

impl Ord for SLimb {
    fn cmp(&self, other: &SLimb) -> Ordering {
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
                if self.negative {
                    if other.negative {
                        return other.v.cmp(&self.v);
                    } else {
                        return Ordering::Less;
                    }
                } else {
                    if other.negative {
                        return Ordering::Greater;
                    } else {
                        return self.v.cmp(&other.v);
                    }
                }
            }
        }
    }
}
impl PartialOrd for SLimb {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add<SLimb> for SLimb {
    type Output = SLimb;
    fn add(self, a: Self) -> Self {
        if self.negative {
            if a.negative {
                SLimb {v: self.v + a.v, negative: true}
            } else {
                if self.v >= a.v {
                    SLimb {v: self.v - a.v, negative: true}
                } else {
                    SLimb {v: a.v - self.v, negative: false}
                }
            }
        } else {
            if a.negative {
                if self.v >= a.v {
                    SLimb {v: self.v - a.v, negative: false}
                } else {
                    SLimb {v: a.v - self.v, negative: true}
                }
            } else {
                SLimb {v: self.v + a.v, negative: false}
            }
        }
    }
}

impl Sub<SLimb> for SLimb {
    type Output = SLimb;
    fn sub (self, a: Self) -> Self {
        if self.negative {
            if a.negative {
                if self.v >= a.v {
                    SLimb {v: self.v - a.v, negative: true}
                } else {
                    SLimb {v: a.v - self.v, negative: false}
                }
            } else {
                SLimb {v: self.v + a.v, negative: true}
            }
        } else {
            if a.negative {
                SLimb {v: self.v + a.v, negative: false}
            } else {
                if self.v >= a.v {
                    SLimb {v: self.v - a.v, negative: false}
                } else {
                    SLimb {v: a.v - self.v, negative: true}
                }
            }
        }
    }
}
// **************************************************************************
// * tests                                                                  *
// **************************************************************************
#[cfg(test)]
mod tests {
    use crate::limb::{*};
    #[test]
    fn smoke() {
        let zero: SLimb = SLimb::from(0);
        assert_eq!(zero.v, 0);
        assert_eq!(zero.negative, false);
        let m = SLimb::from(0);
        assert_eq!(zero, m);
        let one = SLimb::from(1);
        assert!(one > zero);
        assert!(zero < one);
        let minus_one = zero - one;
        assert!(one > minus_one);
        assert!(zero > minus_one);
        assert!(minus_one < zero);
        assert!(minus_one < one);
    }
}
