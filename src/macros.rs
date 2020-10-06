#![allow(unused_macros)]

/// Run something as async or or depending on feature flag `parallel`
macro_rules! sync
{
    (if {$($if:tt)*} else {$($else:tt)*}) => {
	cfg_if::cfg_if! {
	    if #[cfg(feature="parallel")] {
		$($if)*
	    } else {
		$($else)*
	    }
	}
    };
    
    (if {$($if:tt)*}) => {
	cfg_if::cfg_if! {
	    if #[cfg(feature="parallel")] {
		$($if)*
	    }
	}
    };
    
    (else {$($if:tt)*}) => {
	cfg_if::cfg_if! {
	    if #[cfg(not(eature="parallel"))] {
		$($if)*
	    }
	}
    };
}

#[macro_export] macro_rules! reyre {
    (m {$($body:tt)*} $lit:literal $($tt:tt)*) => {
	{
	    let cls = move || {
		$($body)*
	    };
	    $crate::reyre!{
		cls(), $lit $($tt)*
	    }
	}
    };
    ({$($body:tt)*} $lit:literal $($tt:tt)*) => {
	{
	    let cls = || {
		$($body)*
	    };
	    $crate::reyre!{
		cls(), $lit $($tt)*
	    }
	}
    };
    ($expr:expr, $lit:literal $($tt:tt)*) => {
	{
	    use ::color_eyre::eyre::WrapErr;
	    $expr.wrap_err_with(|| ::color_eyre::eyre::eyre!($lit $($tt)*))
	}
    }
}
