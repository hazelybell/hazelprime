use crate::big::{*};

use std::ops::Index;
use std::ops::IndexMut;
use std::cmp::Ordering;


pub struct SBig {
    v: Big,
    negative: bool
}

impl SBig {
    pub fn new(sz : BigSize) -> SBig {
        return SBig { 
            v: Big::new(sz),
            negative: false,
            };
    }
    pub fn new_one(sz: BigSize) -> SBig {
        return SBig { 
            v: Big::new_one(sz),
            negative: false,
            };
    }
    pub fn length(&self) -> BigSize { self.v.length() as BigSize }
    pub fn zero(&mut self) { self.v.zero(); self.negative = false; }
}

impl Index<BigSize> for SBig {
    type Output = Limb;
    fn index(&self, i: BigSize) -> &Limb { &self.v[i] }
}

impl IndexMut<BigSize> for SBig {
    fn index_mut(&mut self, i: BigSize) -> &mut Limb { &mut self.v[i] }
}

impl PartialEq for SBig {
    fn eq(&self, other: &SBig) -> bool {
        if self.v == 0 {
            return other.v == 0;
        } else {
            return (self.v == other.v) && (self.negative == other.negative);
        }
    }
}
impl Eq for SBig {}
impl Ord for SBig {
    fn cmp(&self, other: &SBig) -> Ordering {
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
impl PartialOrd for SBig {
    fn partial_cmp(&self, other: &SBig) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[cfg(test)]
mod tests {
    use crate::sbig::{*};
    #[test]
    fn smoke() {
        let a = SBig::new(2);
        assert_eq!(a.length(), 2);
        let mut b = SBig::new_one(2);
        assert_eq!(a.negative, false);
        b.zero();
        assert_eq!(b[0], 0);
        assert!(a.eq(&b));
        assert!(a == b);
        let b = SBig::new_one(2);
        assert!(a < b);
        assert!(b > a);
        assert!(b != a);
    }
}
