
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

#[cfg(feature="parallel")]
#[cfg_attr(feature="parallel", tokio::main)]
async fn main() {
    println!("Hello world!")
}


#[cfg(not(feature="parallel"))]
fn main() {
    println!("Hello world! sync")
}
