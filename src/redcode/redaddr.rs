use crate::redcode;
use std::ops;

// this can be changed to a different size
// as long as it is unsigned and can hold redcode::CORESIZE
type RedAddrInternal = usize;

#[derive(Copy, Clone, Debug, Eq, Default)]
pub struct RedAddr {
    val: RedAddrInternal,
}

impl PartialEq for RedAddr {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl ops::Add<RedAddr> for RedAddr {
    type Output = RedAddr;
    fn add(self, rhs: RedAddr) -> RedAddr {
        RedAddr {
            val: normalize(self.val + rhs.val),
        }
    }
}

impl ops::Sub<RedAddr> for RedAddr {
    type Output = RedAddr;
    fn sub(self, rhs: RedAddr) -> RedAddr {
        RedAddr {
            val: normalize(self.val + (redcode::CORESIZE - rhs.val)),
        }
    }
}

impl RedAddr {
    pub fn new(a: RedAddrInternal) -> RedAddr {
        RedAddr { val: a }
    }
    pub fn from_i32(a: i32) -> RedAddr {
        let mut a = a;
        while a < 0 {
            a += redcode::CORESIZE as i32;
        }
        RedAddr {
            val: (a as RedAddrInternal) % redcode::CORESIZE,
        }
    }

    pub fn value(self) -> usize {
        self.val as usize
    }
}

fn normalize(val: RedAddrInternal) -> RedAddrInternal {
    // they are already Internal which is unsigned
    val % redcode::CORESIZE
}
