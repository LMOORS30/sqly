mod macros;
mod rules;
mod traits;

pub use macros::*;
pub use traits::*;

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



/// Utility module for error reporting in generated code.
#[doc(hidden)]
pub mod require {
    use super::*;

    /// Requires the given type to exist and implement [`Table`](Table).
    /// 
    /// Used in generated code to raise errors for incorrect attribute paths.
    /// 
    /// ```compile_fail
    /// # mod arg { pub struct Type; }
    /// const _: () = sqly::require::table::<arg::Type>();
    /// ```
    pub const fn table<T: Table>() {}

    /// Requires the given type to exist and implement [`DeleteImpl`](DeleteImpl).
    /// 
    /// Used in generated code to raise errors for incorrect attribute paths.
    /// 
    /// ```compile_fail
    /// # mod arg { pub struct Type; }
    /// const _: () = sqly::require::delete::<arg::Type>();
    /// ```
    pub const fn delete<T: DeleteImpl>() {}

    /// Requires the given type to exist and implement [`InsertImpl`](InsertImpl).
    /// 
    /// Used in generated code to raise errors for incorrect attribute paths.
    /// 
    /// ```compile_fail
    /// # mod arg { pub struct Type; }
    /// const _: () = sqly::require::insert::<arg::Type>();
    /// ```
    pub const fn insert<T: InsertImpl>() {}

    /// Requires the given type to exist and implement [`SelectImpl`](SelectImpl).
    /// 
    /// Used in generated code to raise errors for incorrect attribute paths.
    /// 
    /// ```compile_fail
    /// # mod arg { pub struct Type; }
    /// const _: () = sqly::require::select::<arg::Type>();
    /// ```
    pub const fn select<T: SelectImpl>() {}

    /// Requires the given type to exist and implement [`UpdateImpl`](UpdateImpl).
    /// 
    /// Used in generated code to raise errors for incorrect attribute paths.
    /// 
    /// ```compile_fail
    /// # mod arg { pub struct Type; }
    /// const _: () = sqly::require::update::<arg::Type>();
    /// ```
    pub const fn update<T: UpdateImpl>() {}
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
#[doc(hidden)]
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
            Self { value, index: None }
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



/// Utility module for [`#[sqly(try_from_flat)]`](docs::attr#flat).
/// 
/// Based on [`sqlx::spec_error`](https://docs.rs/crate/sqlx/0.8.0/source/src/spec_error.rs).
#[doc(hidden)]
pub mod spec_error {
    /// Builds a `Box<dyn Error + Send + Sync + 'static>` from any expression.
    /// 
    /// Used by [`#[sqly(try_from_flat)]`](docs::attr#flat).
    #[doc(hidden)]
    #[macro_export]
    macro_rules! __spec_error {
        ($e:expr) => {{
            use $crate::spec_error::SpecError as _;
            let wrapper = $crate::spec_error::SpecErrorWrapper($e);
            (&&&&wrapper).__sqly_spec_error()(wrapper.0)
        }};
    }

    /// Wrapper to perform autoderef specialization on.
    /// 
    /// Used by [`__spec_error!`](__spec_error!).
    pub struct SpecErrorWrapper<E>(pub E);

    /// Trait to perform autoderef specialization with.
    /// 
    /// Used by [`__spec_error!`](__spec_error!).
    pub trait SpecError<E> {
        /// Function to perform autoderef specialization with.
        /// 
        /// Used by [`__spec_error!`](__spec_error!).
        fn __sqly_spec_error(&self) -> fn(E) -> sqlx::error::BoxDynError;
    }

    impl<E: std::error::Error + Send + Sync + 'static> SpecError<E> for &&&SpecErrorWrapper<E> {
        fn __sqly_spec_error(&self) -> fn(E) -> sqlx::error::BoxDynError {
            |e| Box::new(e)
        }
    }
    impl<E: std::fmt::Display> SpecError<E> for &&SpecErrorWrapper<E> {
        fn __sqly_spec_error(&self) -> fn(E) -> sqlx::error::BoxDynError {
            |e| e.to_string().into()
        }
    }
    impl<E: std::fmt::Debug> SpecError<E> for &SpecErrorWrapper<E> {
        fn __sqly_spec_error(&self) -> fn(E) -> sqlx::error::BoxDynError {
            |e| format!("{:?}", e).into()
        }
    }
    impl<E> SpecError<E> for SpecErrorWrapper<E> {
        fn __sqly_spec_error(&self) -> fn(E) -> sqlx::error::BoxDynError {
            |_| format!("unprintable error: {}", std::any::type_name::<E>()).into()
        }
    }

    #[test]
    fn spec_error() {
        type E = sqlx::error::BoxDynError;
        let e: E = __spec_error!(std::io::Error::from(std::io::ErrorKind::Other));
        assert_eq!(format!("{:?}", e), "Kind(Other)");
        assert_eq!(format!("{}", e), "other error");
        let e: E = __spec_error!("display");
        assert_eq!(format!("{:?}", e), "\"display\"");
        let e: E = __spec_error!(&["debug"]);
        assert_eq!(format!("{}", e), "[\"debug\"]");
        let _: E = __spec_error!(SpecErrorWrapper(&e));
    }
}
