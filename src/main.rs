#![allow(dead_code)]

#[macro_use] extern crate log;
#[macro_use] mod macros;

use color_eyre::{
    eyre::{self, eyre, WrapErr},
    Help, SectionExt,
};
use lazy_static::lazy_static;
use cfg_if::cfg_if;

fn init() -> eyre::Result<()>
{
    color_eyre::install()?;
    if let None = std::env::var_os("RUST_LOG") {
	std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    trace!("Initialised");
    Ok(())
}

mod ext;
use ext::*;
mod map;
use map::*;
mod temp;
mod error;
mod arg;

cfg_if!{
    if #[cfg(feature="splash")] {
	mod splash;
    } else {
	mod splash {
	    #[inline(always)] pub fn splash() -> ! {
		super::arg::usage();
		std::process::exit(1)
	    }
	}
    }
}

#[cfg(feature="parallel")]
mod parallel;

fn args_or_out<T: ExactSizeIterator>(i: T, low: usize) -> T
{
    if i.len() < low {
	splash::splash();
    } else {
	i
    }
}

#[cfg(feature="parallel")]
#[cfg_attr(feature="parallel", tokio::main)]
async fn main() -> eyre::Result<()> {
    reyre!(init(), "Failed to initialise")?;
    
    reyre!(parallel::main(futures::stream::iter(args_or_out(std::env::args(), 2)
						.skip(1)
						.dedup())).await,
	   "Jobs failed")
}

#[cfg(not(feature="parallel"))]
mod serial;

#[cfg(not(feature="parallel"))]
fn main() -> eyre::Result<()> {
    reyre!(init(), "Failed to initialise")?;
    let args = args_or_out(std::env::args(), 2).skip(1).dedup();
    todo!("Sync unimplemented")
}
