//! Diagnostic messages.

use std::fmt::Display;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream};

use crate::extension::{GroupExt, LiteralExt, PunctExt, TokenStreamExt};

/// Returns a token stream containing [`::core::compile_error!`] with the given
/// span.
pub fn error(span: Span, message: impl Display) -> TokenStream {
    error_ranged(span, span, message)
}

/// Returns a token stream containing [`::core::compile_error!`] spanning
/// `start` and `end`.
pub fn error_ranged(start: Span, end: Span, message: impl Display) -> TokenStream {
    let mut output = TokenStream::new();

    output.append(Punct::new_spanned(start, ':', Spacing::Joint));
    output.append(Punct::new_spanned(start, ':', Spacing::Alone));
    output.append(Ident::new("core", start));
    output.append(Punct::new_spanned(start, ':', Spacing::Joint));
    output.append(Punct::new_spanned(start, ':', Spacing::Alone));
    output.append(Ident::new("compile_error", start));
    output.append(Punct::new_spanned(start, '!', Spacing::Alone));
    output.append(Group::new_spanned(
        end,
        Delimiter::Brace,
        TokenStream::token(Literal::string_spanned(end, &message.to_string())),
    ));

    output
}
