//! Argument stuff
use super::*;


/// Name of executable
pub fn program_name() -> &'static str
{
    lazy_static! {
	static ref NAME: String = std::env::args().next().unwrap();
    }
    &NAME[..]
}

/// Print usage
pub fn usage()
{
    println!(r#"Usage: {} <files...>"#, program_name());
}
