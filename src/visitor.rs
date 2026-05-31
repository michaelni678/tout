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
///                 .next_if_map_ident(|ident| self.0.get(&ident).ok_or(ident))
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

type StreamHook<'a> = Box<dyn FnMut(&mut TokenStream, TokenStream) -> Option<TokenStream> + 'a>;

type TokenHook<'a, T> = Box<dyn FnMut(&mut TokenStream, T, &mut Parser) -> Option<T> + 'a>;

/// A built-in [`Visitor`] that can be configured with closures.
///
/// Each hook corresponds to a visitor method. When a hook is missing or returns
/// [`Some`], the corresponding default visit function is called. Returning
/// [`None`] indicates that the token has been consumed and should not be
/// processed further.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// #
/// # use proc_macro2::{TokenStream, Punct};
/// # use quote::quote;
/// # use tout::assert::assert_stream_eq;
/// # use tout::extension::PunctExt;
/// # use tout::quasi::ident;
/// use tout::parser::Parser;
/// use tout::visitor::{Hooks, Visitor};
///
/// let input = quote! {
///     fn $method() {
///         println!($print);
///     }
/// };
///
/// let replacements = HashMap::from([
///     // Replace `$method` with `duck`.
///     (ident! { method }, quote! { duck }),
///     // Replace `$print` with `"quack"`.
///     (ident! { print }, quote! { "quack" }),
/// ]);
///
/// let mut output = TokenStream::new();
///
/// Hooks::new()
///     .punct(
///         |output: &mut TokenStream, punct: Punct, parser: &mut Parser| {
///             if punct.is_char('$')
///                 && let Some(replacement) = parser
///                     .next_if_map_ident(|ident| replacements.get(&ident).ok_or(ident))
///                     .cloned()
///             {
///                 output.extend(replacement);
///                 return None;
///             }
///
///             Some(punct)
///         },
///     )
///     .visit_stream(&mut output, input);
///
/// let expected = quote! {
///     fn duck() {
///         println!("quack");
///     }
/// };
///
/// assert_stream_eq!(output, expected);
/// ```
#[derive(Default)]
pub struct Hooks<'a> {
    stream_hook: Option<StreamHook<'a>>,
    tree_hook: Option<TokenHook<'a, TokenTree>>,
    group_hook: Option<TokenHook<'a, Group>>,
    ident_hook: Option<TokenHook<'a, Ident>>,
    punct_hook: Option<TokenHook<'a, Punct>>,
    literal_hook: Option<TokenHook<'a, Literal>>,
}

impl<'a> Hooks<'a> {
    /// Constructs a new hook-based visitor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`TokenStream`] hook.
    pub fn stream<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, TokenStream) -> Option<TokenStream> + 'a,
    {
        self.stream_hook = Some(Box::new(hook));
        self
    }

    /// Sets the [`TokenTree`] hook.
    pub fn tree<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, TokenTree, &mut Parser) -> Option<TokenTree> + 'a,
    {
        self.tree_hook = Some(Box::new(hook));
        self
    }

    /// Sets the [`Group`] hook.
    pub fn group<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, Group, &mut Parser) -> Option<Group> + 'a,
    {
        self.group_hook = Some(Box::new(hook));
        self
    }

    /// Sets the [`Ident`] hook.
    pub fn ident<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, Ident, &mut Parser) -> Option<Ident> + 'a,
    {
        self.ident_hook = Some(Box::new(hook));
        self
    }

    /// Sets the [`Punct`] hook.
    pub fn punct<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, Punct, &mut Parser) -> Option<Punct> + 'a,
    {
        self.punct_hook = Some(Box::new(hook));
        self
    }

    /// Sets the [`Literal`] hook.
    pub fn literal<F>(mut self, hook: F) -> Self
    where
        F: FnMut(&mut TokenStream, Literal, &mut Parser) -> Option<Literal> + 'a,
    {
        self.literal_hook = Some(Box::new(hook));
        self
    }
}

impl Visitor for Hooks<'_> {
    fn visit_stream(&mut self, output: &mut TokenStream, stream: TokenStream) {
        let Some(hook) = &mut self.stream_hook else {
            visit_stream(self, output, stream);
            return;
        };

        if let Some(stream) = hook(output, stream) {
            visit_stream(self, output, stream);
        }
    }

    fn visit_tree(&mut self, output: &mut TokenStream, tree: TokenTree, parser: &mut Parser) {
        let Some(hook) = &mut self.tree_hook else {
            visit_tree(self, output, tree, parser);
            return;
        };

        if let Some(tree) = hook(output, tree, parser) {
            visit_tree(self, output, tree, parser);
        }
    }

    fn visit_group(&mut self, output: &mut TokenStream, group: Group, parser: &mut Parser) {
        let Some(hook) = &mut self.group_hook else {
            visit_group(self, output, group, parser);
            return;
        };

        if let Some(group) = hook(output, group, parser) {
            visit_group(self, output, group, parser);
        }
    }

    fn visit_ident(&mut self, output: &mut TokenStream, ident: Ident, parser: &mut Parser) {
        let Some(hook) = &mut self.ident_hook else {
            visit_ident(self, output, ident, parser);
            return;
        };

        if let Some(ident) = hook(output, ident, parser) {
            visit_ident(self, output, ident, parser);
        }
    }

    fn visit_punct(&mut self, output: &mut TokenStream, punct: Punct, parser: &mut Parser) {
        let Some(hook) = &mut self.punct_hook else {
            visit_punct(self, output, punct, parser);
            return;
        };

        if let Some(punct) = hook(output, punct, parser) {
            visit_punct(self, output, punct, parser);
        }
    }

    fn visit_literal(&mut self, output: &mut TokenStream, literal: Literal, parser: &mut Parser) {
        let Some(hook) = &mut self.literal_hook else {
            visit_literal(self, output, literal, parser);
            return;
        };

        if let Some(literal) = hook(output, literal, parser) {
            visit_literal(self, output, literal, parser);
        }
    }
}
