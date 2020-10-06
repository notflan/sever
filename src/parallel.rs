//! Async operations
use super::*;
use std::{
    num::NonZeroUsize,
    convert::{TryFrom, TryInto,},
    path::Path,
    sync::Arc,
    iter,
};
use futures::{
    future::{
	Future,
	OptionFuture,
	FutureExt,
	BoxFuture,
	join_all,
    },
    stream::{
	self,
	Stream,
	StreamExt,
    },
};
use tokio::{
    sync::{
	Semaphore,
	mpsc,
    },
    fs::{
	OpenOptions,
	File,
	self,
    },
};
use error::{Error, ErrorKind};

cfg_if!{
    if #[cfg(feature="limit-concurrency")] {
	pub const MAX_WORKERS: Option<NonZeroUsize> = Some(unsafe {NonZeroUsize::new_unchecked(4096)});
    } else {
	pub const MAX_WORKERS: Option<NonZeroUsize> = None;
    }
}

#[cfg(feature="recursive")] use recurse::{MAX_DEPTH, Recursion};

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
    debug!("<{:?}> has {} links", path, nlink);
    if nlink > 1 {
	//todo work i guess fuck it
	unlink(path).await?;
	Ok((apath, true))
    } else {
	Ok((apath, false))
    }
}

async fn join_stream<I: Stream>(stream: I) -> impl Iterator<Item=<I::Item as Future>::Output> + ExactSizeIterator
where I::Item: Future
{
    //gotta be a better way than heap allocating here, right?
    stream.then(|x| async move { x.await }).collect::<Vec<_>>().await.into_iter()
}

pub async fn main<I: Stream<Item=String>>(list: I) -> eyre::Result<()>
{
    let sem = gensem();
    //let list = list.into_iter();
    let mut failures = match list.size_hint() {
	(0, Some(0)) | (0, None) => Vec::new(),
	(x, None) | (_, Some(x)) => Vec::with_capacity(x),
    };
    let mut done = 0usize;
    for (i, res) in (0usize..).zip(join_stream(list.map(|file| tokio::spawn(work(file, sem.clone()))))
				   .map(|x| {trace!("--- {} Finished ---", x.len()); x}).await)
    {
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
		    return Ok(());
		}
	    },
	    Ok(Err(kind)) if !kind.kind().is_skippable() => {
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
	    Ok(Err(k)) => {
		failures.push((k.path().to_owned(), k.to_string()));
		trace!("<{:?}> Failed (skipped)", k.path());
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

#[cfg(feature="recursive")] 
fn push_dir<'a>(path: &'a Path, depth: usize, to: mpsc::Sender<String>) -> BoxFuture<'a, tokio::io::Result<()>>
{
    async move {
	let mut dir = fs::read_dir(path).await?;
	let mut workers = match dir.size_hint() {
	    (0, Some(0)) | (0, None) => Vec::new(),
	    (x, None) | (_, Some(x)) => Vec::with_capacity(x),
	};
	let can_recurse = match MAX_DEPTH {
	    Recursion::All => true,
	    Recursion::N(n) if depth < usize::from(n) => true,
	    _ => false,
	};
	while let Some(item) = dir.next_entry().await? {
	    let mut to = to.clone();
	    workers.push(async move {
		match path.join(item.file_name()).into_os_string().into_string() {
		    Ok(name) => {
			if item.file_type().await?.is_dir() {
			    if can_recurse {
				if let Err(e) = push_dir(name.as_ref(), depth+1, to).await {
				    error!("Walking dir {:?} failed: {}", item.file_name(), e);
				}
			    }
			} else {
			    to.send(name).await.unwrap();
			}
		    },
		    Err(err) => {
			error!("Couldn't process file {:?} because it contains invalid UTF-8", err);
		    },
		}
		Ok::<_, std::io::Error>(())
	    });
	}
	join_all(workers).await;
	Ok(())
    }.boxed()
}

pub async fn expand_dir(p: String) -> impl Stream<Item=String>
{
    cfg_if!{
	if #[cfg(feature="recursive")]  {
	    let (mut tx, rx) = mpsc::channel(16);
	    tokio::spawn(async move {
		let path = Path::new(&p);
		if path.is_dir() {
		    if let Err(err) = push_dir(path, 0, tx).await {
			error!("Walking dir {:?} failed: {}", path, err);
		    }
		} else {
		    tx.send(p).await.unwrap();
		}
	    });
	    rx	
	} else {
	    stream::iter(iter::once(p).filter_map(|p| {
		if Path::new(&p).is_dir() {
		    warn!("{:?} is a directory, skipping", p);
		    None
		} else {
		    Some(p)
		}
	    }))
	}
    }
}
