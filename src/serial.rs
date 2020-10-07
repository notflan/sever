//! Sync operations
use super::*;
use std::{
    fs::{
	self,
	OpenOptions,
    },
    path::{
	Path,
    },
};
use error::{
    Error, ErrorKind,  
};

fn unlink(path: &Path) -> Result<(), Error>
{
    let tmp = temp::TempFile::new_in(path.parent().unwrap());
    fs::copy(path, &tmp).map_err(|e| Error::new(ErrorKind::Copy(e), path.to_owned()))?;
    fs::remove_file(path).map_err(|e| Error::new(ErrorKind::Unlink(e), path.to_owned()))?;
    fs::rename(&tmp, path).map_err(|e| Error::new(ErrorKind::Move(e), path.to_owned()))?;
    tmp.release(); // file no longer exists, so no need to drop;
    Ok(())
}

fn work<P: AsRef<Path>>(apath: P) -> Result<(P, bool), Error>
{
    let path = apath.as_ref();
    let file = OpenOptions::new()
	.read(true)
	.open(path)
	.map_err(|e| (ErrorKind::Open(e), path))?;
    
    let meta = match file.metadata() {
	Ok(meta) => meta,
	Err(err) => {
	    debug!("Failed to stat file: {}", err);
	    warn!("Failed to stat {:?}, skipping", path);
	    return Err((ErrorKind::Stat(err), path).into());
	},
    };
    use std::os::unix::fs::MetadataExt;

    let nlink = meta.nlink();
    trace!("<{:?}> has {} links", path, nlink);
    if nlink > 1 {
	unlink(path)?;
	Ok((apath, true))
    } else {
	Ok((apath, false))
    }
}

pub fn main<I: IntoIterator<Item=String>>(list: I) -> eyre::Result<()>
{
    let list = list.into_iter();
    let mut failures = match list.size_hint() {
	(0, Some(0)) | (0, None) => Vec::new(),
	(_, Some(x)) | (x, None) => Vec::with_capacity(x),
    };
    let mut done =0;
    for file in list
    {
	match work(file) {
	    Ok((path, true)) => info!("<{:?}> OK (processed)", path),
	    Ok((path, false)) => debug!("<{:?}> OK (skipped)", path),
	    Err(kind) if !kind.kind().is_skippable() => {
		failures.push((kind.path().to_owned(), kind.to_string()));
		let fuck = format!("{:?}", kind.path());
		let sug = kind.kind().suggestion();
		let err = Err::<std::convert::Infallible, _>(kind)
		    .wrap_err_with(|| eyre!("<{}> Failed", fuck))
		    .with_section(move || fuck.header("Path was"))
		    .with_suggestion(|| sug)
		    .unwrap_err();
		error!("{}", err);
		debug!("Error: {:?}", err);
	    },
	    Err(k) => {
		failures.push((k.path().to_owned(), k.to_string()));
		warn!("<{:?}> Failed (skipped)", k.path());
	    },
	}
	done+=1;
    }
    
    if failures.len() > 0 {
	return Err(eyre!("{}/{} tasks failed to complete successfullly", failures.len(), done))
	    .with_section(|| failures.into_iter()
			  .map(|(x, err)| format!("{}: {}", x.into_os_string()
						  .into_string()
						  .unwrap_or_else(|os| os.to_string_lossy().into_owned()), err))
			  .join("\n")
			  .header("Failed tasks:"))
	    .with_suggestion(|| "Run with `RUST_LOG=debug` or `RUST_LOG=trace` for verbose error reporting");
    }
    Ok(())
}
