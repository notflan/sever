//! Map iter
use super::*;
use std::{
    iter,
    collections::HashSet,
    hash::Hash,
};

//TODO: Feature flag for SHA256 hashing
type HashOutput = u64;

fn compute<H: Hash>(what: &H) -> HashOutput
{
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
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
	    (x, None) | (_, Some(x)) => HashSet::with_capacity(x),
	};
	DedupIter(self, set)
    }
}
