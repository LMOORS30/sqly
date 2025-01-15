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

pub use macros::*;
pub use traits::*;
