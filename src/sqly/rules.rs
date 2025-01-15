#[allow(unused_imports)]
use super::*;



/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query, $($t)* } }
}

/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query_as {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query_as, $($t)* } }
}

/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query_scalar {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query_scalar, $($t)* } }
}

/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query_unchecked {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query_unchecked, $($t)* } }
}

/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query_as_unchecked {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query_as_unchecked, $($t)* } }
}

/// Under development.
/// 
/// Does nothing.
#[macro_export]
macro_rules! query_scalar_unchecked {
    { $($t:tt)* } => { $crate::__sqly_query_impl! { query_scalar_unchecked, $($t)* } }
}



#[doc(hidden)]
#[macro_export]
macro_rules! __sqly_query_impl {

{} => {}

}
