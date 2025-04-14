mod macros;
mod rules;
mod traits;

pub use macros::*;
pub use traits::*;

#[cfg(feature = "serde")]
pub use serde;
pub use sqlx;

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

/// Utility module for [serde](https://serde.rs).
/// 
/// Only available with the [`serde`](https://github.com/LMOORS30/sqly#features) feature enabled.
/// 
/// Used by [`#[sqly(serde_double_option)]`](docs::attr#serde_double_option).
/// 
/// Intended for [`#[serde(with)]`](https://serde.rs/field-attrs.html#with).
/// 
/// Based on [serde_with](https://docs.rs/serde_with/3.12.0/serde_with/rust/double_option/index.html).
#[cfg(feature = "serde")]
pub mod double_option {
    /// Utility function for [serde](https://serde.rs).
    /// 
    /// Only available with the [`serde`](https://github.com/LMOORS30/sqly#features) feature enabled.
    /// 
    /// Used by [`#[sqly(serde_double_option)]`](super::docs::attr#serde_double_option).
    /// 
    /// Intended for [`#[serde(deserialize_with)]`](https://serde.rs/field-attrs.html#deserialize_with).
    /// 
    /// Based on [serde_with](https://docs.rs/serde_with/3.12.0/serde_with/rust/double_option/index.html).
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: serde::Deserialize<'de>,
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer).map(Some)
    }

    /// Utility function for [serde](https://serde.rs).
    /// 
    /// Only available with the [`serde`](https://github.com/LMOORS30/sqly#features) feature enabled.
    /// 
    /// Used by [`#[sqly(serde_double_option)]`](super::docs::attr#serde_double_option).
    /// 
    /// Intended for [`#[serde(serialize_with)]`](https://serde.rs/field-attrs.html#serialize_with).
    /// 
    /// Based on [serde_with](https://docs.rs/serde_with/3.12.0/serde_with/rust/double_option/index.html).
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: serde::Serialize,
        S: serde::Serializer,
    {
        match value {
            None => serializer.serialize_unit(),
            Some(v) => v.serialize(serializer),
        }
    }
}

/// Utility module for [`#[sqly(dynamic)]`](docs::attr#dynamic).
pub mod dynamic {
    /// Used in code generated for dynamic SQL functions.
    /// 
    /// Stores a value and its index once bound.
    pub struct Bind<T> {
        /// The value to bind,
        /// expected to be a reference ([`Copy`])
        /// implementing [`sqlx::Encode`] and [`sqlx::Type`].
        pub value: T,
        /// The index of the bind,
        /// `None` if not yet bound,
        /// `Some(0)` if bound with an error,
        /// `Some(i)` as a 1-based index otherwise.
        pub index: Option<usize>,
    }

    /// Implements `Bind`.
    impl<T> Bind<T> {
        /// Constructs a new bind with the given value and the index set to `None`.
        pub fn new(value: T) -> Self {
            Self {
                index: None,
                value,
            }
        }

        /// Binds the value to the arguments if not yet bound, returns the index of the bound argument.
        /// 
        /// Sets the arguments to any error that occurs, putting this and all subsequent binds as zero.
        /// 
        /// Assumes the same arguments instance is passed each time.
        pub fn bind<'q, A>(&mut self, args: &mut Result<A, sqlx::error::BoxDynError>) -> usize
        where
            T: 'q + sqlx::Encode<'q, A::Database> + sqlx::Type<A::Database> + Copy,
            A: sqlx::Arguments<'q>,
        {
            *self.index.get_or_insert_with(|| {
                if let Ok(ok) = args {
                    if let Err(err) = ok.add(self.value) {
                        *args = Err(err);
                    }
                }
                args.as_ref().map_or(0, |args| args.len())
            })
        }
    }

    /// Used in code generated for dynamic SQL compile time checks.
    /// 
    /// Emulates an impossible function to get the correct type checking.
    pub trait Rip {
        /// The type which should be checked instead of `Self`.
        type Rip;
        /// The impossible function. Will panic.
        fn rip(&self) -> Self::Rip;
    }

    /// `Option::unwrap` by reference. Impossible. Panics.
    impl<T> Rip for Option<T> {
        /// Wrapped type.
        type Rip = T;
        /// Impossible unwrap. Panics.
        fn rip(&self) -> Self::Rip {
            panic!("not possible")
        }
    }

}
