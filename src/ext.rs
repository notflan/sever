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
