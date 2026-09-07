use std::fmt;
use std::num::ParseIntError;
use std::ops::{Bound, RangeBounds};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LenRange {
    lo: u8,
    hi: u8,
}

impl LenRange {
    pub const FULL: Self = Self { lo: 0, hi: u8::MAX };
    pub const EMPTY: Self = Self { lo: u8::MAX, hi: 0 };

    #[inline]
    pub const fn new_inclusive(lo: u8, hi: u8) -> Self {
        Self { lo, hi }
    }

    #[inline(always)]
    pub const fn contains(self, x: u8) -> bool {
        self.lo <= x && x <= self.hi
    }

    #[inline(always)]
    pub const fn contains_len(self, len: usize) -> bool {
        let x = if len > u8::MAX as usize {
            u8::MAX
        } else {
            len as u8
        };
        self.contains(x)
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.lo > self.hi
    }

    #[inline]
    pub const fn start(self) -> u8 {
        self.lo
    }

    #[inline]
    pub const fn end(self) -> u8 {
        self.hi
    }

    #[inline]
    pub fn intersect(self, other: Self) -> Self {
        Self {
            lo: self.lo.max(other.lo),
            hi: self.hi.min(other.hi),
        }
    }
}

impl Default for LenRange {
    fn default() -> Self {
        Self::FULL
    }
}

impl RangeBounds<u8> for LenRange {
    fn start_bound(&self) -> Bound<&u8> {
        Bound::Included(&self.lo)
    }

    fn end_bound(&self) -> Bound<&u8> {
        if self.is_empty() {
            Bound::Excluded(&self.lo)
        } else {
            Bound::Included(&self.hi)
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseRangeError {
    #[error("expected a range like `a..b`, `a..=b`, `..b`, `a..` or `..`")]
    NotARange,
    #[error("`..=` requires an upper bound")]
    MissingInclusiveEnd,
    #[error("invalid lower bound: {0}")]
    Start(#[source] ParseIntError),
    #[error("invalid upper bound: {0}")]
    End(#[source] ParseIntError),
}

impl FromStr for LenRange {
    type Err = ParseRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (lo_s, hi_s, inclusive) = if let Some((a, b)) = s.split_once("..=") {
            (a, b, true)
        } else if let Some((a, b)) = s.split_once("..") {
            (a, b, false)
        } else {
            let n: u8 = s.parse().map_err(|_| ParseRangeError::NotARange)?;
            return Ok(Self { lo: n, hi: n });
        };

        let lo_s = lo_s.trim();
        let hi_s = hi_s.trim();

        let lo = if lo_s.is_empty() {
            0
        } else {
            lo_s.parse().map_err(ParseRangeError::Start)?
        };

        let hi = if hi_s.is_empty() {
            if inclusive {
                return Err(ParseRangeError::MissingInclusiveEnd);
            }
            u8::MAX
        } else {
            let e: u8 = hi_s.parse().map_err(ParseRangeError::End)?;
            if inclusive {
                e
            } else {
                match e.checked_sub(1) {
                    Some(h) => h,
                    None => return Ok(Self::EMPTY),
                }
            }
        };

        Ok(Self { lo, hi })
    }
}

impl fmt::Display for LenRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "{}..{}", self.lo, self.lo);
        }
        if self.lo != 0 {
            write!(f, "{}", self.lo)?;
        }
        if self.hi == u8::MAX {
            write!(f, "..")
        } else {
            write!(f, "..={}", self.hi)
        }
    }
}
