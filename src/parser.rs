//! Token stream parser.

use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::iter;

use proc_macro2::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

use crate::diagnostic::error;
use crate::extension::TokenTreeExt;

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
    /// parser.stream();
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

    /// Collects the remaining tokens and returns the token stream.
    pub fn stream(&mut self) -> TokenStream {
        self.next_trees().collect()
    }

    /// Returns a token stream containing [`::core::compile_error!`] with the
    /// span of the next token.
    pub fn error(&mut self, message: impl Display) -> TokenStream {
        error(self.span(), message)
    }

    /// Takes the next tree and applies the function `map` to it. If the closure
    /// returns [`Ok`], the result is returned. Otherwise, the token is added
    /// back to the parser.
    ///
    /// When mapping to [`TokenTree`] variants, consider using
    /// [`Self::next_group`], [`Self::next_ident`], [`Self::next_punct`], or
    /// [`Self::next_literal`] instead.
    pub fn next_if_map<T, M>(&mut self, map: M) -> Option<T>
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
    ///
    /// When mapping to [`TokenTree`] variants, consider using
    /// [`Self::next_group_if`], [`Self::next_ident_if`],
    /// [`Self::next_punct_if`], or [`Self::next_literal_if`] instead.
    pub fn next_if_map_and<T, M, P>(&mut self, map: M, predicate: P) -> Option<T>
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

    /// Takes the next two trees and applies the mapping functions. If they
    /// return [`Ok`], the tokens are returned. Otherwise, they're added back to
    /// the parser.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::TokenTree;
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_ident_eq, assert_stream_eq};
    /// # use tout::extension::TokenTreeExt;
    /// # use tout::quasi::{group, ident};
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { data[3].x });
    ///
    /// let (ident, group) = parser
    ///     .next2_if_map(TokenTree::into_ident, TokenTree::into_group)
    ///     .unwrap();
    ///
    /// assert_ident_eq!(ident, ident! { data });
    /// assert_group_eq!(group, group! { [3] });
    /// assert_stream_eq!(parser.stream(), quote! { .x });
    /// ```
    ///
    /// If a mapping function returns [`Err`], this function returns [`None`].
    ///
    /// ```
    /// # use proc_macro2::TokenTree;
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_ident_eq, assert_stream_eq};
    /// # use tout::extension::TokenTreeExt;
    /// # use tout::quasi::{group, ident};
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { data.x[3] });
    ///
    /// let parsed = parser.next2_if_map(
    ///     TokenTree::into_ident,
    ///     TokenTree::into_group, // Fails because token `.` is not a group.
    /// );
    ///
    /// assert!(parsed.is_none());
    /// assert_stream_eq!(parser.stream(), quote! { data.x[3] });
    /// ```
    pub fn next2_if_map<T1, M1, T2, M2>(&mut self, map1: M1, map2: M2) -> Option<(T1, T2)>
    where
        T1: Into<TokenTree>,
        M1: FnOnce(TokenTree) -> Result<T1, TokenTree>,
        M2: FnOnce(TokenTree) -> Result<T2, TokenTree>,
    {
        let first = self.next_if_map(map1)?;

        match self.next_if_map(map2) {
            Some(second) => Some((first, second)),
            None => {
                self.tokens.push_front(first.into());
                None
            }
        }
    }

    /// Takes the next two trees and applies the mapping functions. If they
    /// return [`Ok`] and the predicates return `true`, the tokens are returned.
    /// Otherwise, they're added back to the parser.
    ///
    /// # Examples
    ///
    /// See [`Self::next3_if_map`].
    pub fn next2_if_map_and<T1, M1, P1, T2, M2, P2>(
        &mut self,
        map1: M1,
        predicate1: P1,
        map2: M2,
        predicate2: P2,
    ) -> Option<(T1, T2)>
    where
        T1: Into<TokenTree>,
        M1: FnOnce(TokenTree) -> Result<T1, TokenTree>,
        P1: FnOnce(&T1) -> bool,
        T2: Into<TokenTree>,
        M2: FnOnce(TokenTree) -> Result<T2, TokenTree>,
        P2: FnOnce(&T2) -> bool,
    {
        let first = self.next_if_map_and(map1, predicate1)?;

        match self.next_if_map_and(map2, predicate2) {
            Some(second) => Some((first, second)),
            None => {
                self.tokens.push_front(first.into());
                None
            }
        }
    }

    /// Takes the next three trees and applies the mapping functions. If they
    /// return [`Ok`], the tokens are returned. Otherwise, they're added back to
    /// the parser.
    ///
    /// # Examples
    ///
    /// See [`Self::next2_if_map`].
    pub fn next3_if_map<T1, M1, T2, M2, T3, M3>(
        &mut self,
        map1: M1,
        map2: M2,
        map3: M3,
    ) -> Option<(T1, T2, T3)>
    where
        T1: Into<TokenTree>,
        M1: FnOnce(TokenTree) -> Result<T1, TokenTree>,
        T2: Into<TokenTree>,
        M2: FnOnce(TokenTree) -> Result<T2, TokenTree>,
        M3: FnOnce(TokenTree) -> Result<T3, TokenTree>,
    {
        let (first, second) = self.next2_if_map(map1, map2)?;

        match self.next_if_map(map3) {
            Some(third) => Some((first, second, third)),
            None => {
                self.tokens.push_front(second.into());
                self.tokens.push_front(first.into());
                None
            }
        }
    }

    /// Takes the next three trees and applies the mapping functions. If they
    /// return [`Ok`] and the predicates return `true`, the tokens are returned.
    /// Otherwise, they're added back to the parser.
    ///
    /// # Examples
    ///
    /// ```
    /// # use proc_macro2::{Group, TokenTree};
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_punct_eq, assert_stream_eq};
    /// # use tout::extension::{GroupExt, PunctExt, TokenTreeExt};
    /// # use tout::quasi::{group, punct};
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { $(,)? % });
    ///
    /// let (punct1, group, punct2) = parser
    ///     .next3_if_map_and(
    ///         TokenTree::into_punct,
    ///         |punct| punct.is_char('$'),
    ///         TokenTree::into_group,
    ///         Group::is_parenthesized,
    ///         TokenTree::into_punct,
    ///         |punct| punct.is_char('?'),
    ///     )
    ///     .unwrap();
    ///
    /// assert_punct_eq!(punct1, punct! { $ });
    /// assert_group_eq!(group, group! { (,) });
    /// assert_punct_eq!(punct2, punct! { ? });
    /// assert_stream_eq!(parser.stream(), quote! { % });
    /// ```
    ///
    /// If a mapping function returns [`Err`] or a predicate returns `false`,
    /// this function returns [`None`].
    ///
    /// ```
    /// # use proc_macro2::{Group, TokenTree};
    /// # use quote::quote;
    /// # use tout::assert::{assert_group_eq, assert_punct_eq, assert_stream_eq};
    /// # use tout::extension::{GroupExt, PunctExt, TokenTreeExt};
    /// # use tout::quasi::{group, punct};
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { $(,)* % });
    ///
    /// let parsed = parser.next3_if_map_and(
    ///     TokenTree::into_punct,
    ///     |punct| punct.is_char('$'),
    ///     TokenTree::into_group,
    ///     Group::is_parenthesized,
    ///     TokenTree::into_punct,
    ///     |punct| punct.is_char('?'), // Fails because token `*` is not `?`.
    /// );
    ///
    /// assert!(parsed.is_none());
    /// assert_stream_eq!(parser.stream(), quote! { $(,)* % });
    /// ```
    pub fn next3_if_map_and<T1, M1, P1, T2, M2, P2, T3, M3, P3>(
        &mut self,
        map1: M1,
        predicate1: P1,
        map2: M2,
        predicate2: P2,
        map3: M3,
        predicate3: P3,
    ) -> Option<(T1, T2, T3)>
    where
        T1: Into<TokenTree>,
        M1: FnOnce(TokenTree) -> Result<T1, TokenTree>,
        P1: FnOnce(&T1) -> bool,
        T2: Into<TokenTree>,
        M2: FnOnce(TokenTree) -> Result<T2, TokenTree>,
        P2: FnOnce(&T2) -> bool,
        T3: Into<TokenTree>,
        M3: FnOnce(TokenTree) -> Result<T3, TokenTree>,
        P3: FnOnce(&T3) -> bool,
    {
        let (first, second) = self.next2_if_map_and(map1, predicate1, map2, predicate2)?;

        match self.next_if_map_and(map3, predicate3) {
            Some(third) => Some((first, second, third)),
            None => {
                self.tokens.push_front(second.into());
                self.tokens.push_front(first.into());
                None
            }
        }
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
    /// assert_stream_eq!(parser.stream(), quote! { ; });
    /// ```
    pub fn next_group(&mut self) -> Option<Group> {
        self.next_if_map(TokenTree::into_group)
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
    /// assert_stream_eq!(parser.stream(), quote! { = 5; });
    /// ```
    pub fn next_ident(&mut self) -> Option<Ident> {
        self.next_if_map(TokenTree::into_ident)
    }

    /// Takes the next token if it is a punct. Returns [`None`] if the next
    /// token is not a punct or there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Self::next_group`] and [`Self::next_ident`].
    pub fn next_punct(&mut self) -> Option<Punct> {
        self.next_if_map(TokenTree::into_punct)
    }

    /// Takes the next token if it is a literal. Returns [`None`] if the next
    /// token is not a literal or there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Self::next_group`] and [`Self::next_ident`].
    pub fn next_literal(&mut self) -> Option<Literal> {
        self.next_if_map(TokenTree::into_literal)
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
    /// assert_stream_eq!(parser.stream(), quote! { < 10 });
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
    /// assert_stream_eq!(parser.stream(), quote! { (); });
    /// ```
    pub fn next_group_if<P>(&mut self, predicate: P) -> Option<Group>
    where
        P: FnOnce(&Group) -> bool,
    {
        self.next_if_map_and(TokenTree::into_group, predicate)
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
    /// assert_stream_eq!(parser.stream(), quote! { x = 5; });
    /// ```
    pub fn next_ident_if<P>(&mut self, predicate: P) -> Option<Ident>
    where
        P: FnOnce(&Ident) -> bool,
    {
        self.next_if_map_and(TokenTree::into_ident, predicate)
    }

    /// Takes the next token if it is a punct and it matches the given
    /// predicate. Returns [`None`] if the next token is not a punct, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Self::next_group_if`] and [`Self::next_ident_if`].
    pub fn next_punct_if<P>(&mut self, predicate: P) -> Option<Punct>
    where
        P: FnOnce(&Punct) -> bool,
    {
        self.next_if_map_and(TokenTree::into_punct, predicate)
    }

    /// Takes the next token if it is a literal and it matches the given
    /// predicate. Returns [`None`] if the next token is not a literal, if the
    /// predicate returns `false`, or if there are no more tokens.
    ///
    /// # Examples
    ///
    /// See [`Self::next_group_if`] and [`Self::next_ident_if`].
    pub fn next_literal_if<P>(&mut self, predicate: P) -> Option<Literal>
    where
        P: FnOnce(&Literal) -> bool,
    {
        self.next_if_map_and(TokenTree::into_literal, predicate)
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
    /// assert_stream_eq!(parser.stream(), quote! { 10 });
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
    /// assert_stream_eq!(parser.stream(), quote! { ; });
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
    /// assert_stream_eq!(parser.stream(), quote! { = 5; });
    /// ```
    pub fn next_idents(&mut self) -> impl Iterator<Item = Ident> {
        iter::from_fn(|| self.next_ident())
    }

    /// Returns an iterator over consecutive puncts.
    ///
    /// # Examples
    ///
    /// See [`Self::next_groups`] and [`Self::next_idents`].
    pub fn next_puncts(&mut self) -> impl Iterator<Item = Punct> {
        iter::from_fn(|| self.next_punct())
    }

    /// Returns an iterator over consecutive literals.
    ///
    /// # Examples
    ///
    /// See [`Self::next_groups`] and [`Self::next_idents`].
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
    /// assert_stream_eq!(parser.stream(), quote! { 10 });
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
    /// assert_stream_eq!(parser.stream(), quote! { (); });
    /// ```
    ///
    /// See [`Self::next_idents_while`] for another example.
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
    /// assert_stream_eq!(parser.stream(), quote! { = 5; });
    /// ```
    ///
    /// See [`Self::next_groups_while`] for another example.
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
    /// See [`Self::next_groups_while`] and [`Self::next_idents_while`].
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
    /// See [`Self::next_groups_while`] and [`Self::next_idents_while`].
    pub fn next_literals_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Literal>
    where
        P: FnMut(&Literal) -> bool,
    {
        iter::from_fn(move || self.next_literal_if(&mut predicate))
    }

    /// Returns `true` if the next token is a group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use quote::quote;
    /// use tout::parser::Parser;
    ///
    /// let mut parser = Parser::new(quote! { data[x] });
    ///
    /// assert!(!parser.is_next_group());
    ///
    /// parser.skip_tree();
    ///
    /// assert!(parser.is_next_group());
    /// ```
    pub fn is_next_group(&self) -> bool {
        self.first().map(TokenTree::is_group).unwrap_or(false)
    }

    /// Returns `true` if the next token is an ident.
    ///
    /// # Examples
    ///
    /// See [`Self::is_next_group`].
    pub fn is_next_ident(&self) -> bool {
        self.first().map(TokenTree::is_ident).unwrap_or(false)
    }

    /// Returns `true` if the next token is a punct.
    ///
    /// # Examples
    ///
    /// See [`Self::is_next_group`].
    pub fn is_next_punct(&self) -> bool {
        self.first().map(TokenTree::is_punct).unwrap_or(false)
    }

    /// Returns `true` if the next token is a literal.
    ///
    /// # Examples
    ///
    /// See [`Self::is_next_group`].
    pub fn is_next_literal(&self) -> bool {
        self.first().map(TokenTree::is_literal).unwrap_or(false)
    }

    /// Returns `true` and skips the next tree if the parser isn't empty.
    pub fn skip_tree(&mut self) -> bool {
        self.next_tree().is_some()
    }

    /// Returns `true` and skips the next tree if it's a group.
    pub fn skip_group(&mut self) -> bool {
        self.next_group().is_some()
    }

    /// Returns `true` and skips the next tree if it's an ident.
    pub fn skip_ident(&mut self) -> bool {
        self.next_ident().is_some()
    }

    /// Returns `true` and skips the next tree if it's a punct.
    pub fn skip_punct(&mut self) -> bool {
        self.next_punct().is_some()
    }

    /// Returns `true` and skips the next tree if it's a literal.
    pub fn skip_literal(&mut self) -> bool {
        self.next_literal().is_some()
    }

    /// Returns `true` and skips the next tree if the parser isn't empty and the
    /// predicate returns `true`.
    pub fn skip_tree_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&TokenTree) -> bool,
    {
        self.next_tree_if(predicate).is_some()
    }

    /// Returns `true` and skips the next tree if it's a group and the predicate
    /// returns `true`.
    pub fn skip_group_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&Group) -> bool,
    {
        self.next_group_if(predicate).is_some()
    }

    /// Returns `true` and skips the next tree if it's an ident and the
    /// predicate returns `true`.
    pub fn skip_ident_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&Ident) -> bool,
    {
        self.next_ident_if(predicate).is_some()
    }

    /// Returns `true` and skips the next tree if it's a punct and the predicate
    /// returns `true`.
    pub fn skip_punct_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&Punct) -> bool,
    {
        self.next_punct_if(predicate).is_some()
    }

    /// Returns `true` and skips the next tree if it's a literal and the
    /// predicate returns `true`.
    pub fn skip_literal_if<P>(&mut self, predicate: P) -> bool
    where
        P: FnOnce(&Literal) -> bool,
    {
        self.next_literal_if(predicate).is_some()
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
        let stream: TokenStream = self.clone().stream();
        Display::fmt(&stream, f)
    }
}
