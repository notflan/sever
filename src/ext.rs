use super::*;
use std::iter;

pub trait StringJoinExt: Sized
{
    fn join<P: AsRef<str>>(self, sep: P) -> String;
}

impl<I,T> StringJoinExt for I
where I: IntoIterator<Item=T>,
      T: AsRef<str>
{
    fn join<P: AsRef<str>>(self, sep: P) -> String
    {
	let mut string = String::new();
	for (first, s) in iter::successors(Some(true), |_| Some(false)).zip(self.into_iter())
	{
	    if !first {
		string.push_str(sep.as_ref());
	    }
	    string.push_str(s.as_ref());
	}
	string
    }
}

#[cfg(feature="parallel")] 
mod para
{
    use super::*;
    use std::{
	collections::HashSet,
	task::{Poll, Context,},
	pin::Pin,
	marker::PhantomData,
	hash::Hash,
    };
    use futures::{
	stream::{
	    Stream,
	},
    };

    #[pin_project]
    pub struct DedupStream<I, T>(#[pin] I, HashSet<map::HashOutput>, PhantomData<T>);

    impl<I: Stream<Item=T>, T: Hash> Stream for DedupStream<I, T>
    {
	type Item = T;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
	    let this = self.as_mut().project();
	    match this.0.poll_next(cx) {
		Poll::Ready(Some(x)) => {
		    if this.1.insert(map::compute(&x)) {
			Poll::Ready(Some(x))
		    } else {
			self.poll_next(cx)
		    }
		},
		Poll::Ready(None) => Poll::Ready(None),
		Poll::Pending =>  Poll::Pending,
	    }
	}
    }

    pub trait DedupStreamExt: Stream+ Sized
    {
	fn dedup(self) -> DedupStream<Self, Self::Item>;
    }

    impl<T: Stream> DedupStreamExt for T
    where T::Item: Hash
    {
	fn dedup(self) -> DedupStream<Self, Self::Item>
	{
	    DedupStream(self, HashSet::new(), PhantomData)
	}
    }

}
pub use para::*;
