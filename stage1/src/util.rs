use std::{
    fmt::{Debug, Display},
    num::NonZero,
};

use crate::packed_stream::DefaultPackable;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonMaxU32(NonZero<u32>);

impl NonMaxU32 {
    pub const fn new(value: u32) -> Option<Self> {
        if value < u32::MAX {
            let inner = unsafe { NonZero::new_unchecked(value + 1) };
            Some(Self(inner))
        } else {
            None
        }
    }

    pub const unsafe fn new_unchecked(value: u32) -> Self {
        let inner = unsafe { NonZero::new_unchecked(value + 1) };
        Self(inner)
    }

    pub const fn get(self) -> u32 {
        self.0.get() - 1
    }

    pub const fn checked_add(self, rhs: u32) -> Option<Self> {
        match self.0.checked_add(rhs) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    pub const fn checked_sub(self, rhs: u32) -> Option<Self> {
        match self.0.get().checked_sub(rhs) {
            Some(0) | None => None,
            Some(x) => unsafe { Some(Self(NonZero::new_unchecked(x))) },
        }
    }
}

impl Debug for NonMaxU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NonMaxU32").field(&self.get()).finish()
    }
}

impl Display for NonMaxU32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl DefaultPackable for NonMaxU32 {}
