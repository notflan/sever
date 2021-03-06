//! Splash screen~
use super::*;
use std::borrow::Cow;
use recolored::Colorize;

macro_rules! feature {
    (in $name:tt, $desc:literal) => {
	cfg_if! {
	    if #[cfg($name)] {
		println!(" +{}\t{}", stringify!($name).bright_green(), $desc);
	    } else {
		println!(" -{}", stringify!($name));
	    }
	}
    };
    (on $name:literal, $desc:literal $($tt:tt)*) => {
	cfg_if! {
	    if #[cfg(feature=$name)] {
		println!(" +{}\t{}", $name.red(), format!($desc $($tt)*));
	    }  else {
		println!(" -{}", $name.bright_blue());
	    }
	}
    };
    (off $name:literal, $desc:literal $($tt:tt)*) => {
	cfg_if! {
	    if #[cfg(feature=$name)] {
		println!(" +{}\t{}", $name.bright_red(), format!($desc $($tt)*));
	    }  else {
		println!(" -{}", $name.blue());
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
    
    println!("\nFeatures:");

    feature!(on "splash", "\tShow this message");
    feature!(on "parallel", "\tWill run up to {} operations in parallel", parallel::MAX_WORKERS.map(|x| Cow::Owned(x.to_string())).unwrap_or(Cow::Borrowed("unlimited")));
    feature!(on "limit-concurrency", "Concurrency is capped");
    feature!(off "threads", "\tUsing thread-pool");
    feature!(on "recursive", "\tRecursivly process files up to {} directories deep", recurse::MAX_DEPTH);
    feature!(on "limit-recursion", "Recusrion depth is capped");
    feature!(off "paranoid-dedup", "Use SHA256 for argument dedup instead of basic hashing");
    
    std::process::exit(1)
}
