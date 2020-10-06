//! Temp file
use super::*;
use std::{
    ops::Drop,
    path::{
	Path,
	PathBuf,
    },
};

lazy_static! {
    static ref DEFAULT_LOCATION: PathBuf = std::env::temp_dir();
}

fn genname() -> String
{
    uuid::Uuid::new_v4().to_string()
}

#[derive(Debug)]
pub struct TempFile(Option<PathBuf>);

impl TempFile
{
    /// Create a new instance in default temp location
    pub fn new() -> Self
    {
	Self::new_in(&*DEFAULT_LOCATION)
    }
    /// Create a new instance with random name in this location
    pub fn new_in<P: AsRef<Path>>(dir: P) -> Self
    {
	Self::new_path(dir.as_ref().join(genname()))
    }
    /// Create a new instance from a specific path
    pub fn new_path<P: Into<PathBuf>>(path: P) -> Self
    {
	let path = path.into();
	trace!("Creating temp owned path {:?}", path);
	Self(Some(path))
    }

    /// The internal path
    pub fn path(&self) -> &Path
    {
	(&self.0).as_ref().unwrap()
    }

    /// Release ownership of the path, not deleting the file if it exists
    pub fn release(mut self) -> PathBuf
    {
	let d = self.0.take().unwrap();
	std::mem::forget(self);
	d
    }

    #[cfg(feature="parallel")] 
    /// Attempt to remove this temp file async
    pub async fn drop_async(mut self) -> tokio::io::Result<()>
    {
	let res = tokio::fs::remove_file(self.0.take().unwrap()).await;
	std::mem::forget(self);
	res
    }
}

impl AsRef<Path> for TempFile
{
    #[inline] fn as_ref(&self) -> &Path
    {
	self.path()
    }
}

impl Drop for TempFile
{
    fn drop(&mut self)
    {
	if let Err(e) = std::fs::remove_file(self.0.take().unwrap()) {
	    debug!("Failed to remove owned temp file (sync, drop): {}", e);
	}
    }
}
