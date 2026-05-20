//! Token stream parser.

use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::iter;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::extension::{GroupExt, LiteralExt, PunctExt, TokenStreamExt, TokenTreeExt};

/// Parses a [`TokenStream`].
#[derive(Clone)]
pub struct Parser {
    tokens: VecDeque<TokenTree>,
}

impl Parser {
    /// Constructs a parser that will parse the given token stream.
    pub fn new(stream: TokenStream) -> Self {
        Self {
            tokens: stream.into_iter().collect(),
        }
    }

    /// Peeks at the first token. Returns [`None`] if there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::assert_tree_eq;
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// assert_tree_eq!(*parser.first().unwrap(), tree! { number });
    ///
    /// // Advancing the parser will change the first token.
    /// parser.next_tree();
    /// assert_tree_eq!(*parser.first().unwrap(), tree! { < });
    /// ```
    pub fn first(&self) -> Option<&TokenTree> {
        self.tokens.front()
    }

    /// Peeks at the token at the given index. Returns [`None`] if the token
    /// doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::assert_tree_eq;
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// assert_tree_eq!(*parser.peek(1).unwrap(), tree! { < });
    ///
    /// // Advancing the parser will change the peeked token.
    /// parser.next_tree();
    /// assert_tree_eq!(*parser.peek(1).unwrap(), tree! { 10 });
    ///
    /// assert!(parser.peek(2).is_none());
    /// ```
    pub fn peek(&self, index: usize) -> Option<&TokenTree> {
        self.tokens.get(index)
    }

    /// Returns `true` if there are no more tokens to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// assert!(!parser.is_empty());
    ///
    /// // Consume the remaining tokens.
    /// parser.next_trees().count();
    ///
    /// assert!(parser.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns the number of remaining tokens.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns the span of the next token.
    ///
    /// If there are no more tokens, the result of [`Span::call_site`] is
    /// returned.
    pub fn span(&self) -> Span {
        self.first().map_or_else(Span::call_site, TokenTree::span)
    }

    /// Returns a token stream containing [`::core::compile_error!`] with the
    /// span of the next token.
    pub fn error(&mut self, message: impl Display) -> TokenStream {
        let span = self.span();
        let message = message.to_string();

        let mut output = TokenStream::new();

        output.append(Punct::new_spanned(span, ':', Spacing::Joint));
        output.append(Punct::new_spanned(span, ':', Spacing::Alone));
        output.append(Ident::new("core", span));
        output.append(Punct::new_spanned(span, ':', Spacing::Joint));
        output.append(Punct::new_spanned(span, ':', Spacing::Alone));
        output.append(Ident::new("compile_error", span));
        output.append(Punct::new_spanned(span, '!', Spacing::Alone));
        output.append(Group::new_spanned(
            span,
            Delimiter::Brace,
            TokenStream::token(Literal::string_spanned(span, &message)),
        ));

        output
    }

    /// Takes the next tree. Returns [`None`] if there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::assert_tree_eq;
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// assert_tree_eq!(parser.next_tree().unwrap(), tree! { number });
    /// assert_tree_eq!(parser.next_tree().unwrap(), tree! { < });
    /// assert_tree_eq!(parser.next_tree().unwrap(), tree! { 10 });
    /// assert!(parser.next_tree().is_none());
    /// ```
    pub fn next_tree(&mut self) -> Option<TokenTree> {
        self.tokens.pop_front()
    }

    /// Takes the next token if it is a group. Returns [`None`] if the next
    /// token is not a group or there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_stream_eq};
    /// # use tout::quasi::group;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { (self.function)(); });
    ///
    /// assert_group_eq!(parser.next_group().unwrap(), group! { (self.function) });
    /// assert_group_eq!(parser.next_group().unwrap(), group! { () });
    /// assert!(parser.next_group().is_none());
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { ; });
    /// ```
    pub fn next_group(&mut self) -> Option<Group> {
        self.next_tree_if_map(TokenTree::into_group)
    }

    /// Takes the next token if it is an ident. Returns [`None`] if the next
    /// token is not an ident or there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::{assert_ident_eq, assert_stream_eq};
    /// # use tout::quasi::ident;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { let x = 5; });
    ///
    /// assert_ident_eq!(parser.next_ident().unwrap(), ident! { let });
    /// assert_ident_eq!(parser.next_ident().unwrap(), ident! { x });
    /// assert!(parser.next_ident().is_none());
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { = 5; });
    /// ```
    pub fn next_ident(&mut self) -> Option<Ident> {
        self.next_tree_if_map(TokenTree::into_ident)
    }

    /// Takes the next token if it is a punct. Returns [`None`] if the next
    /// token is not a punct or there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_group`] and [`Parser::next_ident`].
    pub fn next_punct(&mut self) -> Option<Punct> {
        self.next_tree_if_map(TokenTree::into_punct)
    }

    /// Takes the next token if it is a literal. Returns [`None`] if the next
    /// token is not a literal or there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_group`] and [`Parser::next_ident`].
    pub fn next_literal(&mut self) -> Option<Literal> {
        self.next_tree_if_map(TokenTree::into_literal)
    }

    /// Takes the next tree and applies the function `map` to it. If the closure
    /// returns [`Ok`], the result is returned. Otherwise, the token is added
    /// back to the parser.
    pub fn next_tree_if_map<T, M>(&mut self, map: M) -> Option<T>
    where
        M: FnOnce(TokenTree) -> Result<T, TokenTree>,
    {
        match self.next_tree().map(map)? {
            Ok(token) => Some(token),
            Err(other) => {
                self.tokens.push_front(other);
                None
            }
        }
    }

    /// Takes the next tree and applies the function `map` to it. If the closure
    /// returns [`Ok`] and the predicate returns `true`, the token is returned.
    /// Otherwise, the token is added back to the parser.
    pub fn next_tree_if_map_and<T, M, P>(&mut self, map: M, predicate: P) -> Option<T>
    where
        T: Into<TokenTree>,
        M: FnOnce(TokenTree) -> Result<T, TokenTree>,
        P: FnOnce(&T) -> bool,
    {
        let other = match self.next_tree().map(map)? {
            Ok(token) if predicate(&token) => return Some(token),
            Ok(token) => token.into(),
            Err(other) => other,
        };

        self.tokens.push_front(other);
        None
    }

    /// Takes the next tree if it matches the given predicate. Returns [`None`]
    /// if the predicate returns `false` or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::TokenTree;
    /// # use quote::quote;
    /// # use tout::assert::{assert_stream_eq, assert_tree_eq};
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// let tree = parser.next_tree_if(|tree| matches!(tree, TokenTree::Ident(_)));
    /// assert_tree_eq!(tree.unwrap(), tree! { number });
    ///
    /// // `None` is returned because `<` isn't a group.
    /// let tree = parser.next_tree_if(|tree| matches!(tree, TokenTree::Group(_)));
    /// assert!(tree.is_none());
    ///
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { < 10 });
    /// ```
    pub fn next_tree_if<P>(&mut self, predicate: P) -> Option<TokenTree>
    where
        P: FnOnce(&TokenTree) -> bool,
    {
        self.tokens.pop_front_if(|token| predicate(token))
    }

    /// Takes the next token if it is a group and it matches the given
    /// predicate. Returns [`None`] if the next token is not a group, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::Group;
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_stream_eq};
    /// # use tout::extension::GroupExt;
    /// # use tout::quasi::group;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { (self.function)(); });
    ///
    /// let group = parser.next_group_if(Group::is_parenthesized);
    /// assert_group_eq!(group.unwrap(), group! { (self.function) });
    ///
    /// // `None` is returned because `()` isn't delimited with braces.
    /// let group = parser.next_group_if(Group::is_braced);
    /// assert!(group.is_none());
    ///
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { (); });
    /// ```
    pub fn next_group_if<P>(&mut self, predicate: P) -> Option<Group>
    where
        P: FnOnce(&Group) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_group, predicate)
    }

    /// Takes the next token if it is an ident and it matches the given
    /// predicate. Returns [`None`] if the next token is not an ident, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// # use tout::assert::{assert_ident_eq, assert_stream_eq};
    /// # use tout::quasi::ident;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { let x = 5; });
    ///
    /// let ident = parser.next_ident_if(|ident| ident == "let");
    /// assert_ident_eq!(ident.unwrap(), ident! { let });
    ///
    /// // `None` is returned because `x` isn't `y`.
    /// let ident = parser.next_ident_if(|ident| ident == "y");
    /// assert!(ident.is_none());
    ///
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { x = 5; });
    /// ```
    pub fn next_ident_if<P>(&mut self, predicate: P) -> Option<Ident>
    where
        P: FnOnce(&Ident) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_ident, predicate)
    }

    /// Takes the next token if it is a punct and it matches the given
    /// predicate. Returns [`None`] if the next token is not a punct, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_group_if`] and [`Parser::next_ident_if`].
    pub fn next_punct_if<P>(&mut self, predicate: P) -> Option<Punct>
    where
        P: FnOnce(&Punct) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_punct, predicate)
    }

    /// Takes the next token if it is a literal and it matches the given
    /// predicate. Returns [`None`] if the next token is not a literal, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_group_if`] and [`Parser::next_ident_if`].
    pub fn next_literal_if<P>(&mut self, predicate: P) -> Option<Literal>
    where
        P: FnOnce(&Literal) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_literal, predicate)
    }

    /// Returns an iterator over the rest of the tokens.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::TokenTree;
    /// # use quote::quote;
    /// # use tout::assert::{assert_stream_eq, assert_tree_eq};
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// let trees: Vec<TokenTree> = parser.next_trees().take(2).collect();
    ///
    /// assert_tree_eq!(trees[0], tree! { number });
    /// assert_tree_eq!(trees[1], tree! { < });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { 10 });
    /// ```
    pub fn next_trees(&mut self) -> impl Iterator<Item = TokenTree> {
        iter::from_fn(|| self.tokens.pop_front())
    }

    /// Returns an iterator over consecutive groups.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::Group;
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_stream_eq};
    /// # use tout::quasi::group;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { (self.function)(); });
    ///
    /// let groups: Vec<Group> = parser.next_groups().collect();
    ///
    /// assert_group_eq!(groups[0], group! { (self.function) });
    /// assert_group_eq!(groups[1], group! { () });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { ; });
    /// ```
    pub fn next_groups(&mut self) -> impl Iterator<Item = Group> {
        iter::from_fn(|| self.next_group())
    }

    /// Returns an iterator over consecutive idents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::Ident;
    /// # use quote::quote;
    /// # use tout::assert::{assert_ident_eq, assert_stream_eq};
    /// # use tout::quasi::ident;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { let x = 5; });
    ///
    /// let idents: Vec<Ident> = parser.next_idents().collect();
    ///
    /// assert_ident_eq!(idents[0], ident! { let });
    /// assert_ident_eq!(idents[1], ident! { x });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { = 5; });
    /// ```
    pub fn next_idents(&mut self) -> impl Iterator<Item = Ident> {
        iter::from_fn(|| self.next_ident())
    }

    /// Returns an iterator over consecutive puncts.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_groups`] and [`Parser::next_idents`].
    pub fn next_puncts(&mut self) -> impl Iterator<Item = Punct> {
        iter::from_fn(|| self.next_punct())
    }

    /// Returns an iterator over consecutive literals.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_groups`] and [`Parser::next_idents`].
    pub fn next_literals(&mut self) -> impl Iterator<Item = Literal> {
        iter::from_fn(|| self.next_literal())
    }

    /// Returns an iterator over consecutive trees that satisfy the given
    /// predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::TokenTree;
    /// # use quote::quote;
    /// # use tout::assert::{assert_stream_eq, assert_tree_eq};
    /// # use tout::quasi::tree;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { number < 10 });
    ///
    /// let trees: Vec<TokenTree> = parser
    ///     .next_trees_while(|tree| matches!(tree, TokenTree::Ident(_) | TokenTree::Punct(_)))
    ///     .collect();
    ///
    /// assert_tree_eq!(trees[0], tree! { number });
    /// assert_tree_eq!(trees[1], tree! { < });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { 10 });
    /// ```
    pub fn next_trees_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = TokenTree>
    where
        P: FnMut(&TokenTree) -> bool,
    {
        iter::from_fn(move || self.next_tree_if(&mut predicate))
    }

    /// Returns an iterator over consecutive groups that satisfy the given
    /// predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::Group;
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_stream_eq};
    /// # use tout::extension::GroupExt;
    /// # use tout::quasi::group;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { (self.function)(); });
    ///
    /// let groups: Vec<Group> = parser
    ///     .next_groups_while(|group| !group.is_empty())
    ///     .collect();
    ///
    /// assert_group_eq!(groups[0], group! { (self.function) });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { (); });
    /// ```
    ///
    /// See [`Parser::next_idents_while`] for another example.
    pub fn next_groups_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Group>
    where
        P: FnMut(&Group) -> bool,
    {
        iter::from_fn(move || self.next_group_if(&mut predicate))
    }

    /// Returns an iterator over consecutive idents that satisfy the given
    /// predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::Ident;
    /// # use quote::quote;
    /// # use tout::assert::{assert_ident_eq, assert_stream_eq};
    /// # use tout::extension::IdentExt;
    /// # use tout::quasi::ident;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { let x = 5; });
    ///
    /// let idents: Vec<Ident> = parser.next_idents_while(|ident| !ident.is_raw()).collect();
    ///
    /// assert_ident_eq!(idents[0], ident! { let });
    /// assert_ident_eq!(idents[1], ident! { x });
    /// assert_stream_eq!(parser.next_trees().collect(), quote! { = 5; });
    /// ```
    ///
    /// See [`Parser::next_groups_while`] for another example.
    pub fn next_idents_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Ident>
    where
        P: FnMut(&Ident) -> bool,
    {
        iter::from_fn(move || self.next_ident_if(&mut predicate))
    }

    /// Returns an iterator over consecutive puncts that satisfy the given
    /// predicate.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_groups_while`] and [`Parser::next_idents_while`].
    pub fn next_puncts_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Punct>
    where
        P: FnMut(&Punct) -> bool,
    {
        iter::from_fn(move || self.next_punct_if(&mut predicate))
    }

    /// Returns an iterator over consecutive literals that satisfy the given
    /// predicate.
    ///
    /// # Examples
    ///
    /// See [`Parser::next_groups_while`] and [`Parser::next_idents_while`].
    pub fn next_literals_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Literal>
    where
        P: FnMut(&Literal) -> bool,
    {
        iter::from_fn(move || self.next_literal_if(&mut predicate))
    }
}

impl Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Parser ")?;
        f.debug_list().entries(self.clone().next_trees()).finish()
    }
}

impl Display for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stream: TokenStream = self.clone().next_trees().collect();
        Display::fmt(&stream, f)
    }
}
