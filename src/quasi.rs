//! Quasi-quotation macros.

/// [`TokenStream`] quasi-quotation macro.
///
/// This is not a replacement for [`quote::quote!`]. It's slower, does not
/// support variable interpolation, and possibly lossy. Its intended use is in
/// test cases that require tokens to be parsed verbatim, without interpolation.
///
/// # Examples
///
/// ```
/// # use tout::assert::assert_stream_eq;
/// use quote::quote;
/// use tout::quasi::stream;
///
/// let a = stream! {
///     impl MyTrait for MyStruct {}
/// };
///
/// let b = quote! {
///     impl MyTrait for MyStruct {}
/// };
///
/// assert_stream_eq!(a, b);
/// ```
///
/// When `#` is followed by an ident, `quote!` performs variable
/// interpolation, but `stream!` does not.
///
/// ```
/// # use tout::assert::assert_stream_ne;
/// use quote::quote;
/// use tout::quasi::stream;
///
/// let variable = 5;
///
/// let a = stream! { # variable };
/// let b = quote! { # variable };
///
/// // Not equal, `stream!` doesn't interpolate!
/// assert_stream_ne!(a, b);
/// ```
///
/// [`TokenStream`]: proc_macro2::TokenStream
/// [`quote::quote!`]: https://docs.rs/quote/latest/quote/macro.quote.html
#[macro_export]
#[doc(hidden)]
macro_rules! stream {
    ($($tt:tt)*) => {
        ::core::stringify!($($tt)*)
            .parse::<::proc_macro2::TokenStream>()
            .expect("couldn't parse tokens")
    };
}

#[doc(inline)]
pub use stream;

/// [`TokenTree`] quasi-quotation macro.
///
/// [`TokenTree`]: proc_macro2::TokenTree
#[macro_export]
#[doc(hidden)]
macro_rules! tree {
    ($tt:tt) => {
        $crate::stream!($tt)
            .into_iter()
            .next()
            .expect("expected a token")
    };
}

#[doc(inline)]
pub use tree;

/// [`Group`] quasi-quotation macro.
///
/// # Examples
///
/// ```
/// # use tout::assert::assert_stream_eq;
/// # use tout::extension::{GroupExt, TokenStreamExt};
/// # use tout::quasi::stream;
/// use tout::quasi::group;
///
/// let group = group! { [1, 2, 3] };
///
/// assert!(group.is_bracketed());
/// assert_stream_eq!(group.stream(), stream! { 1, 2, 3 });
/// ```
///
/// # Panics
///
/// Panics if the token isn't a group.
///
/// ```should_panic
/// use tout::quasi::group;
///
/// group! { "tout" };
/// ```
///
/// [`Group`]: proc_macro2::Group
#[macro_export]
#[doc(hidden)]
macro_rules! group {
    ($tt:tt) => {
        $crate::extension::TokenTreeExt::into_group($crate::tree!($tt)).expect("expected a group")
    };
}

#[doc(inline)]
pub use group;

/// [`Ident`] quasi-quotation macro.
///
/// # Examples
///
/// ```
/// use tout::quasi::ident;
///
/// let ident = ident! { variable };
///
/// assert_eq!(ident, "variable");
/// ```
///
/// # Panics
///
/// Panics if the token isn't an ident.
///
/// ```should_panic
/// use tout::quasi::ident;
///
/// ident! { [1, 2, 3] };
/// ```
///
/// [`Ident`]: proc_macro2::Ident
#[macro_export]
#[doc(hidden)]
macro_rules! ident {
    ($tt:tt) => {
        $crate::extension::TokenTreeExt::into_ident($crate::tree!($tt)).expect("expected an ident")
    };
}

#[doc(inline)]
pub use ident;

/// [`Punct`] quasi-quotation macro.
///
/// # Examples
///
/// ```
/// # use tout::extension::PunctExt;
/// use tout::quasi::punct;
///
/// let punct = punct! { < };
///
/// assert!(punct.is_char('<'));
/// assert!(punct.is_alone());
/// ```
///
/// # Panics
///
/// Panics if the token isn't a punct.
///
/// ```should_panic
/// use tout::quasi::punct;
///
/// punct! { variable };
/// ```
///
/// [`Punct`]: proc_macro2::Punct
#[macro_export]
#[doc(hidden)]
macro_rules! punct {
    ($tt:tt) => {
        $crate::extension::TokenTreeExt::into_punct($crate::tree!($tt)).expect("expected a punct")
    };
}

#[doc(inline)]
pub use punct;

/// [`Literal`] quasi-quotation macro.
///
/// # Examples
///
/// ```
/// use tout::quasi::literal;
///
/// let literal = literal! { "tout" };
///
/// assert_eq!(literal.to_string(), "\"tout\"");
/// ```
///
/// # Panics
///
/// Panics if the token isn't a literal.
///
/// ```should_panic
/// use tout::quasi::literal;
///
/// literal! { < };
/// ```
///
/// [`Literal`]: proc_macro2::Literal
#[macro_export]
#[doc(hidden)]
macro_rules! literal {
    ($tt:tt) => {
        $crate::extension::TokenTreeExt::into_literal($crate::tree!($tt))
            .expect("expected a literal")
    };
}

#[doc(inline)]
pub use literal;
