//! Recursion stuffs
use super::*;
use std::{
    num::NonZeroUsize,
    fmt,
};

#[derive(Debug)]
pub enum Recursion
{
    All,
    N(NonZeroUsize),
}

cfg_if!{
    if #[cfg(feature="limit-recursion")] {
	pub const MAX_DEPTH: Recursion = Recursion::N(unsafe{NonZeroUsize::new_unchecked(256)});
    } else {
	pub const MAX_DEPTH: Recursion = Recursion::All;
    }
}

impl fmt::Display for Recursion
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	match self {
	    Self::N(n) => write!(f, "{}", n),
	    Self::All => write!(f, "unlimited"),
	    _ => write!(f, "no"),
	}
    }
}
