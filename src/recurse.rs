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

impl Recursion {
    pub fn can_recurse(&self, depth: usize) -> bool
    {
	match self {
	    Recursion::All => true,
	    Recursion::N(n) if depth < usize::from(*n) => true,
	    _ => {
		warn!("Depth {} exceeds max recursion depth of {}, ignoring", depth, self);
		false
	    },
	}
    }
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
	    #[allow(unreachable_patterns)] _ => write!(f, "no"),
	}
    }
}

#[cfg(not(feature="parallel"))] 
mod iter
{
    use super::*;
    use std::{
	path::Path,
	collections::VecDeque,
	fs,
	iter::{
	    Fuse,
	},
    };
    
    #[derive(Debug)]
    pub struct DirWalker<I>{
	paths: VecDeque<(String, usize)>,
	iter: Fuse<I>
    }

    /// Walk any amount of directories
    pub fn walk_dirs<I: IntoIterator>(iter: I) -> DirWalker<I::IntoIter>
    where I::Item: Into<String>
    {
	let iter = iter.into_iter();
	let paths = match iter.size_hint() {
	    (0, Some(0)) | (0, None) => VecDeque::new(),
	    (x, None) | (_, Some(x)) => VecDeque::with_capacity(x),
	};
	DirWalker {
	    paths,
	    iter: iter.fuse(),
	}
    }

    impl<I: Iterator> Iterator for DirWalker<I>
    where I::Item: Into<String>
    {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item>
	{
	    fn process(inserter: &mut VecDeque<(String, usize)>, p: String, depth: usize)
	    {
		match fs::read_dir(&p) {
		    Ok(dir) => {
			for file in dir {
			    match file {
				Ok(file) =>  match file.path().into_os_string().into_string() {
				    Ok(string) => inserter.push_front((string, depth)),
				    Err(err) => error!("Couldn't process file {:?} because it contains invalid UTF-8", err),
				},
				Err(err) => error!("Failed to enumerate dir {:?}: {}", p, err),
			    }
			}
		    },
		    Err(err) => error!("Walking dir {:?} failed: {}", p, err),
		}
	    }
	    if let Some(next) = self.iter.next() {
		self.paths.push_front((next.into(), 0));
	    }
	    if let Some((path, depth)) = self.paths.pop_back() {
		if Path::new(&path).is_dir() {
		    if MAX_DEPTH.can_recurse(depth) {
			process(&mut self.paths, path, depth+1);
		    }
		    self.next()
		} else {
		    Some(path)
		}
	    } else {
		None
	    }
	}
    }
}

#[cfg(not(feature="parallel"))] 
pub use iter::*;
