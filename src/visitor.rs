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
/// # use tout::quasi::ident;
/// use tout::parser::Parser;
/// use tout::visitor::{Visitor, visit_punct};
///
/// pub struct ReplaceVariables(HashMap<Ident, TokenStream>);
///
/// impl Visitor for ReplaceVariables {
///     fn visit_punct(&mut self, output: &mut TokenStream, punct: Punct, parser: &mut Parser) {
///         if punct.is_char('$')
///             && let Some(replacement) = parser
///                 .next_ident_if_map(|ident| self.0.get(&ident).ok_or_else(|| ident))
///                 .cloned()
///         {
///             output.extend(replacement);
///             return;
///         }
///
///         visit_punct(self, output, punct, parser);
///     }
/// }
///
/// let input = quote! {
///     fn $method() {
///         println!($print);
///     }
/// };
///
/// let mut visitor = ReplaceVariables(HashMap::from([
///     // Replace `$method` with `duck`.
///     (ident! { method }, quote! { duck }),
///     // Replace `$print` with `"quack"`.
///     (ident! { print }, quote! { "quack" }),
/// ]));
///
/// let mut output = TokenStream::new();
/// visitor.visit_stream(&mut output, input);
///
/// let expected = quote! {
///     fn duck() {
///         println!("quack");
///     }
/// };
///
/// assert_stream_eq!(output, expected);
/// ```
pub trait Visitor {
    /// Invoked when a [`TokenStream`] is encountered.
    fn visit_stream(&mut self, output: &mut TokenStream, stream: TokenStream) {
        visit_stream(self, output, stream)
    }

    /// Invoked when a [`TokenTree`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_tree(&mut self, output: &mut TokenStream, tree: TokenTree, parser: &mut Parser) {
        visit_tree(self, output, tree, parser)
    }

    /// Invoked when a [`Group`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_group(&mut self, output: &mut TokenStream, group: Group, parser: &mut Parser) {
        visit_group(self, output, group, parser)
    }

    /// Invoked when an [`Ident`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_ident(&mut self, output: &mut TokenStream, ident: Ident, parser: &mut Parser) {
        visit_ident(self, output, ident, parser)
    }

    /// Invoked when a [`Punct`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_punct(&mut self, output: &mut TokenStream, punct: Punct, parser: &mut Parser) {
        visit_punct(self, output, punct, parser)
    }

    /// Invoked when a [`Literal`] is encountered.
    ///
    /// The remaining unvisited tokens are available in `parser`.
    fn visit_literal(&mut self, output: &mut TokenStream, literal: Literal, parser: &mut Parser) {
        visit_literal(self, output, literal, parser)
    }
}

/// Default function invoked when a [`TokenStream`] is encountered.
pub fn visit_stream<V>(visitor: &mut V, output: &mut TokenStream, stream: TokenStream)
where
    V: Visitor + ?Sized,
{
    Parser::new(stream).visit(visitor, output)
}

/// Default function invoked when a [`TokenTree`] is encountered.
pub fn visit_tree<V>(
    visitor: &mut V,
    output: &mut TokenStream,
    tree: TokenTree,
    parser: &mut Parser,
) where
    V: Visitor + ?Sized,
{
    match tree {
        TokenTree::Group(group) => visitor.visit_group(output, group, parser),
        TokenTree::Ident(ident) => visitor.visit_ident(output, ident, parser),
        TokenTree::Punct(punct) => visitor.visit_punct(output, punct, parser),
        TokenTree::Literal(literal) => visitor.visit_literal(output, literal, parser),
    }
}

/// Default function invoked when a [`Group`] is encountered.
pub fn visit_group<V>(visitor: &mut V, output: &mut TokenStream, group: Group, _parser: &mut Parser)
where
    V: Visitor + ?Sized,
{
    let mut inner = TokenStream::new();
    visitor.visit_stream(&mut inner, group.stream());

    let group = Group::new_spanned(group.span(), group.delimiter(), inner);

    output.append(group);
}

/// Default function invoked when an [`Ident`] is encountered.
pub fn visit_ident<V>(
    _visitor: &mut V,
    output: &mut TokenStream,
    ident: Ident,
    _parser: &mut Parser,
) where
    V: Visitor + ?Sized,
{
    output.append(ident);
}

/// Default function invoked when a [`Punct`] is encountered.
pub fn visit_punct<V>(
    _visitor: &mut V,
    output: &mut TokenStream,
    punct: Punct,
    _parser: &mut Parser,
) where
    V: Visitor + ?Sized,
{
    output.append(punct);
}

/// Default function invoked when a [`Literal`] is encountered.
pub fn visit_literal<V>(
    _visitor: &mut V,
    output: &mut TokenStream,
    literal: Literal,
    _parser: &mut Parser,
) where
    V: Visitor + ?Sized,
{
    output.append(literal);
}
