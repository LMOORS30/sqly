mod macros;
mod rules;
mod traits;

/// Imports.
pub mod derive {
    pub use super::Table;
    pub use super::Delete;
    pub use super::DeleteImpl;
    pub use super::Insert;
    pub use super::InsertImpl;
    pub use super::Select;
    pub use super::SelectImpl;
    pub use super::Update;
    pub use super::UpdateImpl;
}

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

pub use sqlx;
