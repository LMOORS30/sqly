mod macros;
mod rules;
mod traits;

/// Documentation.
pub mod docs {
    #[allow(unused_imports)]
    use super::*;
    #[doc = include_str!("docs/documentation.md")]
    pub mod attr {
        #[allow(unused_imports)]
        use super::*;
        #[doc = include_str!("docs/notation.md")]
        pub mod note {}
    }
}

/// A type which has its query checked at compile time.
/// 
/// This trait is not meant to be manually implemented,
/// it will be generated the derive macros
/// unless [`#[sqly(unchecked)]`](docs::attr#dev-attributes) is specified or
/// the `unchecked` [feature](https://github.com/LMOORS30/sqly#features) is enabled.
/// 
/// This trait serves no further purpose.
pub trait Checked {
    fn check(&self);
}

pub use macros::*;
pub use traits::*;
