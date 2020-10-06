#![allow(dead_code)]

#[macro_use] extern crate log;
#[macro_use] mod macros;

use color_eyre::{
    eyre::{self, eyre, WrapErr},
    Help, SectionExt,
};

fn init() -> eyre::Result<()>
{
    color_eyre::install()?;
    pretty_env_logger::init(); //TODO: Change to builder
    trace!("Initialised");
    Ok(())
}

mod error;

#[cfg(feature="parallel")]
mod parallel;

#[cfg(feature="parallel")]
#[cfg_attr(feature="parallel", tokio::main)]
async fn main() -> eyre::Result<()> {
    reyre!(init(), "Failed to initialise")?;
    
    reyre!(parallel::main(std::env::args().skip(1)).await, "Jobs failed")
}

#[cfg(not(feature="parallel"))]
mod serial;

#[cfg(not(feature="parallel"))]
fn main() -> eyre::Result<()> {
    reyre!(init(), "Failed to initialise")?;
    todo!("Sync unimplemented")
}
