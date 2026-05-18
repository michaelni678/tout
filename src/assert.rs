//! Assertion macros.

/// Asserts the left [`TokenStream`] is equivalent to the right.
///
/// [`TokenStream`]: proc_macro2::TokenStream
#[macro_export]
#[doc(hidden)]
macro_rules! assert_stream_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::TokenStreamExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_stream_eq;

/// Asserts the left [`TokenStream`] is not equivalent to the right.
///
/// [`TokenStream`]: proc_macro2::TokenStream
#[macro_export]
#[doc(hidden)]
macro_rules! assert_stream_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::TokenStreamExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_stream_ne;

/// Asserts the left [`TokenTree`] is equivalent to the right.
///
/// [`TokenTree`]: proc_macro2::TokenTree
#[macro_export]
#[doc(hidden)]
macro_rules! assert_tree_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::TokenTreeExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_tree_eq;

/// Asserts the left [`TokenTree`] is not equivalent to the right.
///
/// [`TokenTree`]: proc_macro2::TokenTree
#[macro_export]
#[doc(hidden)]
macro_rules! assert_tree_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::TokenTreeExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_tree_ne;

/// Asserts the left [`Group`] is equivalent to the right.
///
/// [`Group`]: proc_macro2::Group
#[macro_export]
#[doc(hidden)]
macro_rules! assert_group_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::GroupExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_group_eq;

/// Asserts the left [`Group`] is not equivalent to the right.
///
/// [`Group`]: proc_macro2::Group
#[macro_export]
#[doc(hidden)]
macro_rules! assert_group_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::GroupExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_group_ne;

/// Asserts the left [`Ident`] is equivalent to the right.
///
/// [`Ident`]: proc_macro2::Ident
#[macro_export]
#[doc(hidden)]
macro_rules! assert_ident_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::IdentExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_ident_eq;

/// Asserts the left [`Ident`] is not equivalent to the right.
///
/// [`Ident`]: proc_macro2::Ident
#[macro_export]
#[doc(hidden)]
macro_rules! assert_ident_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::IdentExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_ident_ne;

/// Asserts the left [`Punct`] is equivalent to the right.
///
/// [`Punct`]: proc_macro2::Punct
#[macro_export]
#[doc(hidden)]
macro_rules! assert_punct_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::PunctExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_punct_eq;

/// Asserts the left [`Punct`] is not equivalent to the right.
///
/// [`Punct`]: proc_macro2::Punct
#[macro_export]
#[doc(hidden)]
macro_rules! assert_punct_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::PunctExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_punct_ne;

/// Asserts the left [`Literal`] is equivalent to the right.
///
/// [`Literal`]: proc_macro2::Literal
#[macro_export]
#[doc(hidden)]
macro_rules! assert_literal_eq {
    ($left:expr, $right:expr) => {
        assert!($crate::extension::LiteralExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_literal_eq;

/// Asserts the left [`Literal`] is not equivalent to the right.
///
/// [`Literal`]: proc_macro2::Literal
#[macro_export]
#[doc(hidden)]
macro_rules! assert_literal_ne {
    ($left:expr, $right:expr) => {
        assert!(!$crate::extension::LiteralExt::equals(&$left, &$right));
    };
}

#[doc(inline)]
pub use assert_literal_ne;
