//! Token stream parser.

use std::{collections::VecDeque, iter};

use proc_macro2::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};

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
    pub fn first(&self) -> Option<&TokenTree> {
        self.tokens.front()
    }

    /// Peeks at the token at the given index. Returns [`None`] if the token
    /// doesn't exist.
    pub fn peek(&self, index: usize) -> Option<&TokenTree> {
        self.tokens.get(index)
    }

    /// Returns `true` if there are no more tokens to parse.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns the span of the next token.
    ///
    /// If there are no more tokens, the result of [`Span::call_site`] is
    /// returned.
    pub fn span(&self) -> Span {
        self.first().map_or_else(Span::call_site, TokenTree::span)
    }

    /// Takes the next tree. Returns [`None`] if there are no more tokens.
    pub fn next_tree(&mut self) -> Option<TokenTree> {
        self.tokens.pop_front()
    }

    /// Takes the next tree and applies the function `map` to it. If the closure
    /// returns [`Ok`], the result is returned. Otherwise, the value is put back
    /// for the next iteration.
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

    /// Takes the next tree if it matches the given predicate. Returns [`None`]
    /// if the predicate returns `false` or if there are no more tokens.
    pub fn next_tree_if<P>(&mut self, predicate: P) -> Option<TokenTree>
    where
        P: FnOnce(&TokenTree) -> bool,
    {
        self.tokens.pop_front_if(|token| predicate(token))
    }

    /// Takes the next tree and applies the function `map` to it. If the closure
    /// returns [`Ok`] and the predicate returns `true`, the value is returned.
    /// Otherwise, the value is put back for the next iteration.
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

    /// Returns an iterator over the rest of the tokens.
    pub fn next_trees(&mut self) -> impl Iterator<Item = TokenTree> {
        self.tokens.drain(..)
    }

    /// Returns an iterator over consecutive trees that satisfy the given
    /// predicate.
    pub fn next_trees_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = TokenTree>
    where
        P: FnMut(&TokenTree) -> bool,
    {
        iter::from_fn(move || self.next_tree_if(&mut predicate))
    }

    /// Takes the next token if it is a group. Returns [`None`] if the next
    /// token is not a group or there are no more tokens.
    pub fn next_group(&mut self) -> Option<Group> {
        self.next_tree_if_map(TokenTree::into_group)
    }

    /// Takes the next token if it is a group and it matches the given
    /// predicate. Returns [`None`] if the next token is not a group, if the
    /// predicate returns `false`, or if there are no more tokens.
    pub fn next_group_if<P>(&mut self, predicate: P) -> Option<Group>
    where
        P: FnOnce(&Group) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_group, predicate)
    }

    /// Returns an iterator over consecutive groups.
    pub fn next_groups(&mut self) -> impl Iterator<Item = Group> {
        iter::from_fn(|| self.next_group())
    }

    /// Returns an iterator over consecutive groups that satisfy the given
    /// predicate.
    pub fn next_groups_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Group>
    where
        P: FnMut(&Group) -> bool,
    {
        iter::from_fn(move || self.next_group_if(&mut predicate))
    }

    /// Takes the next token if it is an ident. Returns [`None`] if the next
    /// token is not an ident or there are no more tokens.
    pub fn next_ident(&mut self) -> Option<Ident> {
        self.next_tree_if_map(TokenTree::into_ident)
    }

    /// Takes the next token if it is an ident and it matches the given
    /// predicate. Returns [`None`] if the next token is not an ident, if the
    /// predicate returns `false`, or if there are no more tokens.
    pub fn next_ident_if<P>(&mut self, predicate: P) -> Option<Ident>
    where
        P: FnOnce(&Ident) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_ident, predicate)
    }

    /// Returns an iterator over consecutive idents.
    pub fn next_idents(&mut self) -> impl Iterator<Item = Ident> {
        iter::from_fn(|| self.next_ident())
    }

    /// Returns an iterator over consecutive idents that satisfy the given
    /// predicate.
    pub fn next_idents_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Ident>
    where
        P: FnMut(&Ident) -> bool,
    {
        iter::from_fn(move || self.next_ident_if(&mut predicate))
    }

    /// Takes the next token if it is a punct. Returns [`None`] if the next
    /// token is not a punct or there are no more tokens.
    pub fn next_punct(&mut self) -> Option<Punct> {
        self.next_tree_if_map(TokenTree::into_punct)
    }

    /// Takes the next token if it is a punct and it matches the given
    /// predicate. Returns [`None`] if the next token is not a punct, if the
    /// predicate returns `false`, or if there are no more tokens.
    pub fn next_punct_if<P>(&mut self, predicate: P) -> Option<Punct>
    where
        P: FnOnce(&Punct) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_punct, predicate)
    }

    /// Returns an iterator over consecutive puncts.
    pub fn next_puncts(&mut self) -> impl Iterator<Item = Punct> {
        iter::from_fn(|| self.next_punct())
    }

    /// Returns an iterator over consecutive puncts that satisfy the given
    /// predicate.
    pub fn next_puncts_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Punct>
    where
        P: FnMut(&Punct) -> bool,
    {
        iter::from_fn(move || self.next_punct_if(&mut predicate))
    }

    /// Takes the next token if it is a literal. Returns [`None`] if the next
    /// token is not a literal or there are no more tokens.
    pub fn next_literal(&mut self) -> Option<Literal> {
        self.next_tree_if_map(TokenTree::into_literal)
    }

    /// Takes the next token if it is a literal and it matches the given
    /// predicate. Returns [`None`] if the next token is not a literal, if the
    /// predicate returns `false`, or if there are no more tokens.
    pub fn next_literal_if<P>(&mut self, predicate: P) -> Option<Literal>
    where
        P: FnOnce(&Literal) -> bool,
    {
        self.next_tree_if_map_and(TokenTree::into_literal, predicate)
    }

    /// Returns an iterator over consecutive literals.
    pub fn next_literals(&mut self) -> impl Iterator<Item = Literal> {
        iter::from_fn(|| self.next_literal())
    }

    /// Returns an iterator over consecutive literals that satisfy the given
    /// predicate.
    pub fn next_literals_while<P>(&mut self, mut predicate: P) -> impl Iterator<Item = Literal>
    where
        P: FnMut(&Literal) -> bool,
    {
        iter::from_fn(move || self.next_literal_if(&mut predicate))
    }
}
