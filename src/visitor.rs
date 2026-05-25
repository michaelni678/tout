//! Visit tokens.

use proc_macro2::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

use crate::extension::{GroupExt, TokenStreamExt};
use crate::parser::Parser;

/// Traverses tokens, calling the associated visit method for each one.
///
/// # Examples
///
/// In the example below, each `$variable` is replaced by the mapped token
/// stream.
///
/// ```
/// # use std::collections::HashMap;
/// #
/// # use proc_macro2::{Ident, Punct, TokenStream};
/// # use quote::quote;
/// # use tout::assert::assert_stream_eq;
/// # use tout::extension::PunctExt;
/// # use tout::parser::Parser;
/// # use tout::quasi::ident;
/// use tout::visitor::{Visitor, visit_punct};
///
/// pub struct ReplaceVariables(HashMap<Ident, TokenStream>);
///
/// impl Visitor for ReplaceVariables {
///     fn visit_punct(&mut self, punct: Punct, parser: &mut Parser) -> TokenStream {
///         if punct.is_char('$')
///             && let Some(ident) = parser.next_ident_if(|ident| self.0.contains_key(ident))
///         {
///             return self.0[&ident].clone();
///         }
///
///         visit_punct(self, punct, parser)
///     }
/// }
///
/// let input = quote! {
///     fn $method() {
///         println!($print);
///     }
/// };
///
/// let mut replace_variables = ReplaceVariables(HashMap::from([
///     // Replace `$method` with `duck`.
///     (ident! { method }, quote! { duck }),
///     // Replace `$print` with "quack".
///     (ident! { print }, quote! { "quack" }),
/// ]));
///
/// let expected = quote! {
///     fn duck() {
///         println!("quack");
///     }
/// };
///
/// assert_stream_eq!(replace_variables.visit_stream(input), expected);
/// ```
pub trait Visitor {
    /// Invoked when a [`TokenStream`] is encountered.
    fn visit_stream(&mut self, stream: TokenStream) -> TokenStream {
        visit_stream(self, stream)
    }

    /// Invoked when a [`TokenTree`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_tree(&mut self, tree: TokenTree, parser: &mut Parser) -> TokenStream {
        visit_tree(self, tree, parser)
    }

    /// Invoked when a [`Group`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_group(&mut self, group: Group, parser: &mut Parser) -> TokenStream {
        visit_group(self, group, parser)
    }

    /// Invoked when an [`Ident`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_ident(&mut self, ident: Ident, parser: &mut Parser) -> TokenStream {
        visit_ident(self, ident, parser)
    }

    /// Invoked when a [`Punct`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_punct(&mut self, punct: Punct, parser: &mut Parser) -> TokenStream {
        visit_punct(self, punct, parser)
    }

    /// Invoked when a [`Literal`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_literal(&mut self, literal: Literal, parser: &mut Parser) -> TokenStream {
        visit_literal(self, literal, parser)
    }
}

/// Default function invoked when a [`TokenStream`] is encountered.
pub fn visit_stream<V>(visitor: &mut V, stream: TokenStream) -> TokenStream
where
    V: Visitor + ?Sized,
{
    let mut parser = Parser::new(stream);
    let mut output = TokenStream::new();

    while let Some(tree) = parser.next_tree() {
        output.extend(visitor.visit_tree(tree, &mut parser));
    }

    output
}

/// Default function invoked when a [`TokenTree`] is encountered.
pub fn visit_tree<V>(visitor: &mut V, tree: TokenTree, parser: &mut Parser) -> TokenStream
where
    V: Visitor + ?Sized,
{
    match tree {
        TokenTree::Group(group) => visitor.visit_group(group, parser),
        TokenTree::Ident(ident) => visitor.visit_ident(ident, parser),
        TokenTree::Punct(punct) => visitor.visit_punct(punct, parser),
        TokenTree::Literal(literal) => visitor.visit_literal(literal, parser),
    }
}

/// Default function invoked when a [`Group`] is encountered.
pub fn visit_group<V>(visitor: &mut V, group: Group, _parser: &mut Parser) -> TokenStream
where
    V: Visitor + ?Sized,
{
    let group = Group::new_spanned(
        group.span(),
        group.delimiter(),
        visitor.visit_stream(group.stream()),
    );

    TokenStream::token(group)
}

/// Default function invoked when an [`Ident`] is encountered.
pub fn visit_ident<V>(_visitor: &mut V, ident: Ident, _parser: &mut Parser) -> TokenStream
where
    V: Visitor + ?Sized,
{
    TokenStream::token(ident)
}

/// Default function invoked when a [`Punct`] is encountered.
pub fn visit_punct<V>(_visitor: &mut V, punct: Punct, _parser: &mut Parser) -> TokenStream
where
    V: Visitor + ?Sized,
{
    TokenStream::token(punct)
}

/// Default function invoked when a [`Literal`] is encountered.
pub fn visit_literal<V>(_visitor: &mut V, literal: Literal, _parser: &mut Parser) -> TokenStream
where
    V: Visitor + ?Sized,
{
    TokenStream::token(literal)
}
