//! Async operations
use super::*;
use std::{
    num::NonZeroUsize,
    convert::{TryFrom, TryInto,},
    path::Path,
    sync::Arc,
};
use futures::{
    future::{
	OptionFuture,
	FutureExt,
	join_all,
    },
};
use tokio::{
    sync::{
	Semaphore,
    },
    fs::{
	OpenOptions,
	File,
	self,
    },
};
use error::{Error, ErrorKind};

const MAX_WORKERS: Option<NonZeroUsize> = Some(unsafe {NonZeroUsize::new_unchecked(4096)});

fn gensem() -> Option<Arc<Semaphore>>
{
    trace!("Limiting concurrency to {:?}", MAX_WORKERS);
    match MAX_WORKERS {
	Some(nz) => Some(Arc::new(Semaphore::new(nz.into()))),
	None => None,
    }
}

async fn unlink(path: &Path) -> Result<(), Error>
{
    let tmp = temp::TempFile::new_in(path.parent().unwrap());
    fs::copy(path, &tmp).await.map_err(|e| Error::new(ErrorKind::Copy(e), path.to_owned()))?;
    fs::remove_file(path).await.map_err(|e| Error::new(ErrorKind::Unlink(e), path.to_owned()))?;
    fs::rename(&tmp, path).await.map_err(|e| Error::new(ErrorKind::Move(e), path.to_owned()))?;
    tmp.release(); // file no longer exists, so no need to drop;
    Ok(())
}

async fn work<P: AsRef<Path>>(apath: P, sem: Option<Arc<Semaphore>>) -> Result<(P, bool), Error>
{
    let path = apath.as_ref();
    let _lock = OptionFuture::from(sem.map(Semaphore::acquire_owned)).await;
    let file = OpenOptions::new()
	.read(true)
	.open(path).await
	.map_err(|e| (ErrorKind::Open(e), path))?;
    
    let meta = match file.metadata().await {
	Ok(meta) => meta,
	Err(err) => {
	    debug!("Failed to stat file: {}", err);
	    warn!("Failed to stat {:?}, skipping", path);
	    return Err((ErrorKind::Stat(err), path).into());
	},
    };

    use std::os::unix::fs::MetadataExt;

    let nlink = meta.nlink();
    trace!("<{:?}> Links: {}", path, nlink);
    if nlink > 1 {
	//todo work i guess fuck it
	unlink(path).await?;
	Ok((apath, true))
    } else {
	Ok((apath, false))
    }
}

pub async fn main<I: IntoIterator<Item=String>>(list: I) -> eyre::Result<()>
{
    let sem = gensem();
    let mut failures = 0usize;
    for (i, res) in (0usize..).zip(join_all(list.into_iter().map(|file| tokio::spawn(work(file, sem.clone()))))
				   .map(|x| {trace!("--- {} Finished ---", x.len()); x}).await)
    {
	//trace!("Done on {:?}", res);
	match res {
	    Ok(Ok((path, true))) => info!("<{:?}> OK (processed)", path),
	    Ok(Ok((path, false))) => info!("<{:?}> OK (skipped)", path),
	    Err(e) => {
		trace!("child {} cancelled by {}", i, if e.is_panic(){"panic"} else {"cancel"});
		if e.is_panic() {
		    return Err(eyre!("Child {} panic", i))
			.with_error(move || e)
			.with_warning(|| "This suggests a bug in the program");
		} else {
		    warn!("Child {} cancelled", i);
		    failures += 1;
		}
	    },
	    Ok(Err(kind)) if !kind.kind().is_skippable() => { //
		let fuck = format!("{:?}", kind.path());
		let sug = kind.kind().suggestion();
		let err = Err::<std::convert::Infallible, _>(kind)
		    .wrap_err_with(|| eyre!("<{}> Failed", fuck))
		    .with_section(move || fuck.header("Path was"))
		    .with_suggestion(|| sug)
		    .unwrap_err();
		error!("{}", err);
		debug!("Error: {:?}", err);
		failures += 1;
	    },
	    Ok(Err(k)) => {
		trace!("<{:?}> Failed (skipped)", k.path());
		failures+=1;
	    },
	}
    }

    if failures > 0 {
	return Err(eyre!("Some tasks failed to complete successfullly"))
	    .with_section(|| failures.to_string().header("Number of failed tasks"))
	    .with_suggestion(|| "Run with `RUST_LOG=debug` or `RUST_LOG=trace` for verbose error reporting");
    }

    Ok(())
}
