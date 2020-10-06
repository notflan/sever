//! Splash screen~
use super::*;
use std::borrow::Cow;

macro_rules! feature {
    (in $name:tt, $desc:literal) => {
	cfg_if! {
	    if #[cfg($name)] {
		println!(" +{}\t{}", stringify!($name), $desc);
	    }
	}
    };
    ($name:literal, $desc:literal $($tt:tt)*) => {
	cfg_if! {
	    if #[cfg(feature=$name)] {
		println!(" +{}\t{}", $name, format!($desc $($tt)*));
	    }  else {
		println!(" -{}", $name);
	    }
	}
    };
}

pub fn splash() -> ! {
    arg::usage();

    // splash screen
    println!(r#"
> sever ({}) v{}
>  Coerce hardlinks to new files

For verbose output, set `RUST_LOG` env var to one of the following:
 trace - Most verbose
 debug - Verbose
 info  - Default
 warn  - Only show warnings
 error - Only show errors

Made by {} with <3 (Licensed GPL 3.0 or later)"#, arg::program_name(), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
    println!("\nEnabled extensions: ");
    feature!(in nightly, "\tCompiled with Rust nightly extensions");
    println!();
    feature!("parallel", "\tWill run up to {} operations in parallel", parallel::MAX_WORKERS.map(|x| Cow::Owned(x.to_string())).unwrap_or(Cow::Borrowed("unlimited")));
    feature!("limit-concurrency", "Concurrency is capped");
    feature!("threads", "\tUsing thread-pool");
    std::process::exit(1)
}
