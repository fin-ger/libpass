#[macro_export]
macro_rules! try_or {
	  ( $expr:expr , $or:expr ) => {{
		    match { $expr }.into_iter().next() {
			      Some(res) => res,
			      None => { return { $or }; },
		    }
	  }}
}
