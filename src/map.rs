//! Map iter
use super::*;
use std::{
    iter,
    collections::HashSet,
    hash::Hash,
};

cfg_if!{
    if #[cfg(feature="paranoid-dedup")] {
	pub const HASH_SIZE: usize  = 256;
	pub type HashOutput = [u8; HASH_SIZE];
    } else {
	pub const HASH_SIZE: usize = 8;
	pub type HashOutput = u64;
    }

}
pub fn compute<H: Hash>(what: &H) -> HashOutput
{
    use std::hash::Hasher;

    let mut hasher = {
	cfg_if!{
            if #[cfg(feature="paranoid-dedup")] {
		use sha2::{Sha256, Digest,};
		struct Sha256Hasher(Sha256);
		impl Hasher for Sha256Hasher
		{
		    fn write(&mut self, bytes: &[u8])
		    {
			self.0.update(bytes);
		    }
		    fn finish(&self) -> u64
		    {
			unimplemented!("This shouldn't really be called tbh")
		    }
		}

		impl Sha256Hasher
		{
		    fn finish(self) -> HashOutput
		    {
			let mut output = [0u8; HASH_SIZE];
			let finish = self.0.finalize();
			for (d, s) in output.iter_mut().zip(finish.into_iter())
			{
			    *d = s;
			}
			output
		    }
		}

		Sha256Hasher(Sha256::new())
	    } else {
		std::collections::hash_map::DefaultHasher::new()
	    }
	}
    };
    what.hash(&mut hasher);
    hasher.finish()
}

pub struct DedupIter<I: Iterator>(I, HashSet<HashOutput>)
where <I as Iterator>::Item: Hash;

impl<I: Iterator> Iterator for DedupIter<I>
where I::Item: Hash
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item>
    {
	if let Some(next) = self.0.next() {
	    let hash = compute(&next);
	    if self.1.insert(hash) {
		Some(next)
	    } else {
		return self.next();
	    }
	} else {
	    None
	}
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
	let (low, high) = self.0.size_hint();

	(if low < 1 {0} else {1}, high)
    }
}

pub trait DedupIterExt: Iterator + Sized
where Self::Item: Hash
{
    fn dedup(self) -> DedupIter<Self>;
}

impl<I: Iterator> DedupIterExt for I
where I::Item: Hash
{
    fn dedup(self) -> DedupIter<Self>
    {
	let set = match self.size_hint() {
	    (0, Some(0)) | (0, None) => HashSet::new(),
	    (_, Some(x)) | (x, None) => HashSet::with_capacity(x),
	};
	DedupIter(self, set)
    }
}
