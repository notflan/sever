use super::*;
use std::{
    error, fmt,
    io,
    path::{PathBuf, Path,}
};

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind
{
    Stat(io::Error),
    Open(io::Error),
    Copy(io::Error),
    Unlink(io::Error),
    Move(io::Error),
}

impl ErrorKind
{
    pub fn is_skippable(&self) -> bool
    {
	match self {
	    Self::Stat(_) => true,
	    _ => false,
	}
    }

    pub fn suggestion(&self) -> &'static str
    {
	//TODO: Maybe?
	match self {
	    _ => "Are you sure file exists, and you have write privilages here?"
	}
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    path: PathBuf,
}

impl<P: AsRef<Path>> From<(ErrorKind, P)> for Error
{
    fn from((kind, p): (ErrorKind, P)) -> Self
    {
	Self::new(kind, p.as_ref())
    }
}


impl Error
{
    pub fn new(kind: ErrorKind, path: impl Into<PathBuf>) -> Self
    {
	Self {
	    kind,
	    path: path.into()
	}
    }
    pub fn path(&self) -> &Path
    {
	self.path.as_ref()
    }
    pub fn kind(&self) -> &ErrorKind
    {
	&self.kind
    }
}

impl std::error::Error for Error
{
    fn source(&self) -> Option<&(dyn error::Error + 'static)>
    {
	Some(match &self.kind {
	    ErrorKind::Stat(io) => io,
	    ErrorKind::Open(io) => io,
	    ErrorKind::Copy(io) => io,
	    ErrorKind::Unlink(io) => io,
	    ErrorKind::Move(io) => io,
	   // _ => return None,
	})
    }
}
impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	match &self.kind {
	    ErrorKind::Stat(_) => write!(f, "Failed to stat file {:?}", self.path),
	    ErrorKind::Open(_) => write!(f, "Failed to open file {:?}", self.path),
	    ErrorKind::Copy(_) => write!(f, "Failed to create copy of file to temp location {:?}", self.path),
	    ErrorKind::Unlink(_) => write!(f, "Failed to unlink file {:?}", self.path),
	    ErrorKind::Move(_) => write!(f, "Failed to move new file into place {:?}", self.path),
	}
    }
}


