#[doc(hidden)]
#[macro_export]
/// Matches the provided expression and returns the value if the [`Ok`]
/// variant was produced, otherwise logs the error and returns the function
///
/// [`Ok`]: Result::Ok
macro_rules! tri {
     ($e: expr) => {
         $crate::tri!(@inner $e => "An error occurred:")
     };

    ($e: expr => $msg: literal) => {
        $crate::tri!(@inner $e => $msg)
    };

    (@inner $e: expr => $msg: literal) => {
        match $e {
             Ok(a) => a,
             Err(err) => {
                 ::tracing::error!(concat!($msg, " {err:?}"), err = err);
                 return;
             }
        }
    }
 }