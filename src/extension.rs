//! Extension traits.

use std::ffi::CStr;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;

/// Extension trait for [`TokenStream`].
pub trait TokenStreamExt: Sealed + Sized {
    /// Construct a token stream from a single token.
    fn token(token: impl Into<TokenTree>) -> Self;

    /// Returns `true` if the token stream is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Append a token to the token stream.
    fn append<T>(&mut self, token: T)
    where
        T: Into<TokenTree>;
}

impl TokenStreamExt for TokenStream {
    fn token(token: impl Into<TokenTree>) -> Self {
        Self::from(token.into())
    }

    fn equals(&self, other: &Self) -> bool {
        let mut this = self.clone().into_iter();
        let mut other = other.clone().into_iter();

        loop {
            match (this.next(), other.next()) {
                (Some(this), Some(other)) if this.equals(&other) => {}
                (None, None) => return true,
                _ => return false,
            }
        }
    }

    fn append<T>(&mut self, token: T)
    where
        T: Into<TokenTree>,
    {
        self.extend(Some(token.into()));
    }
}

impl Sealed for TokenStream {}

/// Extension trait for [`TokenTree`].
pub trait TokenTreeExt: Sealed + Sized {
    /// Returns `true` if this token tree is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Returns `true` if the token tree is a group.
    fn is_group(&self) -> bool;

    /// Returns `true` if the token tree is an ident.
    fn is_ident(&self) -> bool;

    /// Returns `true` if the token tree is a punct.
    fn is_punct(&self) -> bool;

    /// Returns `true` if the token tree is a literal.
    fn is_literal(&self) -> bool;

    /// Returns `true` if the token tree is a group and the predicate returns
    /// `true`.
    fn is_group_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Group) -> bool;

    /// Returns `true` if the token tree is a ident and the predicate returns
    /// `true`.
    fn is_ident_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Ident) -> bool;

    /// Returns `true` if the token tree is a punct and the predicate returns
    /// `true`.
    fn is_punct_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Punct) -> bool;

    /// Returns `true` if the token tree is a literal and the predicate returns
    /// `true`.
    fn is_literal_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Literal) -> bool;

    /// Attempts to convert the token tree into a group.
    fn into_group(self) -> Result<Group, Self>;

    /// Attempts to convert the token tree into an ident.
    fn into_ident(self) -> Result<Ident, Self>;

    /// Attempts to convert the token tree into a punct.
    fn into_punct(self) -> Result<Punct, Self>;

    /// Attempts to convert the token tree into a literal.
    fn into_literal(self) -> Result<Literal, Self>;

    /// Attempts to convert the token tree into a group if the predicate returns
    /// `true`.
    fn into_group_if<P>(self, predicate: P) -> Result<Group, Self>
    where
        P: FnOnce(&Group) -> bool;

    /// Attempts to convert the token tree into an ident if the predicate
    /// returns `true`.
    fn into_ident_if<P>(self, predicate: P) -> Result<Ident, Self>
    where
        P: FnOnce(&Ident) -> bool;

    /// Attempts to convert the token tree into a punct if the predicate returns
    /// `true`.
    fn into_punct_if<P>(self, predicate: P) -> Result<Punct, Self>
    where
        P: FnOnce(&Punct) -> bool;

    /// Attempts to convert the token tree into a literal if the predicate
    /// returns `true`.
    fn into_literal_if<P>(self, predicate: P) -> Result<Literal, Self>
    where
        P: FnOnce(&Literal) -> bool;

    /// Returns a reference to the group, if it is one.
    fn as_group(&self) -> Option<&Group>;

    /// Returns a reference to the ident, if it is one.
    fn as_ident(&self) -> Option<&Ident>;

    /// Returns a reference to the punct, if it is one.
    fn as_punct(&self) -> Option<&Punct>;

    /// Returns a reference to the literal, if it is one.
    fn as_literal(&self) -> Option<&Literal>;

    /// Returns a mutable reference to the group, if it is one.
    fn as_group_mut(&mut self) -> Option<&mut Group>;

    /// Returns a mutable reference to the ident, if it is one.
    fn as_ident_mut(&mut self) -> Option<&mut Ident>;

    /// Returns a mutable reference to the punct, if it is one.
    fn as_punct_mut(&mut self) -> Option<&mut Punct>;

    /// Returns a mutable reference to the literal, if it is one.
    fn as_literal_mut(&mut self) -> Option<&mut Literal>;
}

impl TokenTreeExt for TokenTree {
    fn is_group(&self) -> bool {
        matches!(self, Self::Group(_))
    }

    fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }

    fn is_punct(&self) -> bool {
        matches!(self, Self::Punct(_))
    }

    fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    fn is_group_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Group) -> bool,
    {
        matches!(self, Self::Group(group) if predicate(group))
    }

    fn is_ident_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Ident) -> bool,
    {
        matches!(self, Self::Ident(ident) if predicate(ident))
    }

    fn is_punct_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Punct) -> bool,
    {
        matches!(self, Self::Punct(punct) if predicate(punct))
    }

    fn is_literal_and<P>(&self, predicate: P) -> bool
    where
        P: FnOnce(&Literal) -> bool,
    {
        matches!(self, Self::Literal(literal) if predicate(literal))
    }

    fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Group(this), Self::Group(other)) => this.equals(other),
            (Self::Ident(this), Self::Ident(other)) => this.equals(other),
            (Self::Punct(this), Self::Punct(other)) => this.equals(other),
            (Self::Literal(this), Self::Literal(other)) => this.equals(other),
            _ => false,
        }
    }

    fn into_group(self) -> Result<Group, Self> {
        match self {
            Self::Group(group) => Ok(group),
            _ => Err(self),
        }
    }

    fn into_ident(self) -> Result<Ident, Self> {
        match self {
            Self::Ident(ident) => Ok(ident),
            _ => Err(self),
        }
    }

    fn into_punct(self) -> Result<Punct, Self> {
        match self {
            Self::Punct(punct) => Ok(punct),
            _ => Err(self),
        }
    }

    fn into_literal(self) -> Result<Literal, Self> {
        match self {
            Self::Literal(literal) => Ok(literal),
            _ => Err(self),
        }
    }

    fn into_group_if<P>(self, predicate: P) -> Result<Group, Self>
    where
        P: FnOnce(&Group) -> bool,
    {
        match self {
            Self::Group(group) if predicate(&group) => Ok(group),
            _ => Err(self),
        }
    }

    fn into_ident_if<P>(self, predicate: P) -> Result<Ident, Self>
    where
        P: FnOnce(&Ident) -> bool,
    {
        match self {
            Self::Ident(ident) if predicate(&ident) => Ok(ident),
            _ => Err(self),
        }
    }

    fn into_punct_if<P>(self, predicate: P) -> Result<Punct, Self>
    where
        P: FnOnce(&Punct) -> bool,
    {
        match self {
            Self::Punct(punct) if predicate(&punct) => Ok(punct),
            _ => Err(self),
        }
    }

    fn into_literal_if<P>(self, predicate: P) -> Result<Literal, Self>
    where
        P: FnOnce(&Literal) -> bool,
    {
        match self {
            Self::Literal(literal) if predicate(&literal) => Ok(literal),
            _ => Err(self),
        }
    }

    fn as_group(&self) -> Option<&Group> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }

    fn as_ident(&self) -> Option<&Ident> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn as_punct(&self) -> Option<&Punct> {
        match self {
            Self::Punct(punct) => Some(punct),
            _ => None,
        }
    }

    fn as_literal(&self) -> Option<&Literal> {
        match self {
            Self::Literal(literal) => Some(literal),
            _ => None,
        }
    }

    fn as_group_mut(&mut self) -> Option<&mut Group> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }

    fn as_ident_mut(&mut self) -> Option<&mut Ident> {
        match self {
            Self::Ident(ident) => Some(ident),
            _ => None,
        }
    }

    fn as_punct_mut(&mut self) -> Option<&mut Punct> {
        match self {
            Self::Punct(punct) => Some(punct),
            _ => None,
        }
    }

    fn as_literal_mut(&mut self) -> Option<&mut Literal> {
        match self {
            Self::Literal(literal) => Some(literal),
            _ => None,
        }
    }
}

impl Sealed for TokenTree {}

/// Extension trait for [`Group`].
pub trait GroupExt: Sealed + Sized {
    /// Construct a group with the given span, delimiter, and token stream.
    fn new_spanned(span: Span, delimiter: Delimiter, stream: TokenStream) -> Self;

    /// Returns `true` if this group is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Returns `true` if the group's delimiter matches `delimiter`.
    fn is_delimiter(&self, delimiter: Delimiter) -> bool;

    /// Returns `true` if the delimiter is [`Delimiter::Parenthesis`].
    fn is_parenthesized(&self) -> bool;

    /// Returns `true` if the delimiter is [`Delimiter::Brace`].
    fn is_braced(&self) -> bool;

    /// Returns `true` if the delimiter is [`Delimiter::Bracket`].
    fn is_bracketed(&self) -> bool;

    /// Returns `true` if the delimiter is [`Delimiter::None`].
    fn is_undelimited(&self) -> bool;

    /// Returns `true` if there are no tokens delimited in this group.
    fn is_empty(&self) -> bool;
}

impl GroupExt for Group {
    fn new_spanned(span: Span, delimiter: Delimiter, stream: TokenStream) -> Self {
        let mut group = Group::new(delimiter, stream);
        group.set_span(span);
        group
    }

    fn equals(&self, other: &Self) -> bool {
        self.delimiter() == other.delimiter() && self.stream().equals(&other.stream())
    }

    fn is_delimiter(&self, delimiter: Delimiter) -> bool {
        self.delimiter() == delimiter
    }

    fn is_parenthesized(&self) -> bool {
        self.is_delimiter(Delimiter::Parenthesis)
    }

    fn is_braced(&self) -> bool {
        self.is_delimiter(Delimiter::Brace)
    }

    fn is_bracketed(&self) -> bool {
        self.is_delimiter(Delimiter::Bracket)
    }

    fn is_undelimited(&self) -> bool {
        self.is_delimiter(Delimiter::None)
    }

    fn is_empty(&self) -> bool {
        self.stream().is_empty()
    }
}

impl Sealed for Group {}

/// Extension trait for [`Ident`].
pub trait IdentExt: Sealed + Sized {
    /// Returns `true` if this ident is equal to `other`.
    ///
    /// This method is provided for completeness and simply delegates to the
    /// [`PartialEq`] implementation.
    fn equals(&self, other: &Self) -> bool;

    /// Returns `true` if the identifier is prefixed with `r#`.
    fn is_raw(&self) -> bool;

    /// If present, strips the raw identifier prefix `r#`.
    fn unraw(&self) -> Self;
}

impl IdentExt for Ident {
    fn equals(&self, other: &Self) -> bool {
        self == other
    }

    fn is_raw(&self) -> bool {
        self.to_string().starts_with("r#")
    }

    fn unraw(&self) -> Self {
        self.to_string()
            .strip_prefix("r#")
            .map_or_else(|| self.clone(), |stripped| Self::new(stripped, self.span()))
    }
}

impl Sealed for Ident {}

/// Extension trait for [`Punct`].
pub trait PunctExt: Sealed + Sized {
    /// Construct a punct with the given span, character, and spacing.
    fn new_spanned(span: Span, character: char, spacing: Spacing) -> Self;

    /// Returns `true` if this punct is equal to `other`.
    fn equals(&self, other: &Self) -> bool;

    /// Returns `true` if the punct character is `character`.
    fn is_char(&self, character: char) -> bool;

    /// Returns `true` if the spacing matches `spacing`.
    fn is_spacing(&self, spacing: Spacing) -> bool;

    /// Returns `true` if the spacing is [`Spacing::Alone`].
    fn is_alone(&self) -> bool;

    /// Returns `true` if the spacing is [`Spacing::Joint`].
    fn is_joint(&self) -> bool;
}

impl PunctExt for Punct {
    fn new_spanned(span: Span, character: char, spacing: Spacing) -> Self {
        let mut punct = Punct::new(character, spacing);
        punct.set_span(span);
        punct
    }

    fn equals(&self, other: &Self) -> bool {
        self.as_char() == other.as_char() && self.spacing() == other.spacing()
    }

    fn is_char(&self, character: char) -> bool {
        self.as_char() == character
    }

    fn is_spacing(&self, spacing: Spacing) -> bool {
        self.spacing() == spacing
    }

    fn is_alone(&self) -> bool {
        self.is_spacing(Spacing::Alone)
    }

    fn is_joint(&self) -> bool {
        self.is_spacing(Spacing::Joint)
    }
}

impl Sealed for Punct {}

/// Extension trait for [`Literal`].
pub trait LiteralExt: Sealed + Sized {
    /// Construct a suffixed `i8` literal with the given span and integer.
    fn i8_suffixed_spanned(span: Span, integer: i8) -> Self;

    /// Construct an unsuffixed `i8` literal with the given span and integer.
    fn i8_unsuffixed_spanned(span: Span, integer: i8) -> Self;

    /// Construct a suffixed `i16` literal with the given span and integer.
    fn i16_suffixed_spanned(span: Span, integer: i16) -> Self;

    /// Construct an unsuffixed `i16` literal with the given span and integer.
    fn i16_unsuffixed_spanned(span: Span, integer: i16) -> Self;

    /// Construct a suffixed `i32` literal with the given span and integer.
    fn i32_suffixed_spanned(span: Span, integer: i32) -> Self;

    /// Construct an unsuffixed `i32` literal with the given span and integer.
    fn i32_unsuffixed_spanned(span: Span, integer: i32) -> Self;

    /// Construct a suffixed `i64` literal with the given span and integer.
    fn i64_suffixed_spanned(span: Span, integer: i64) -> Self;

    /// Construct an unsuffixed `i64` literal with the given span and integer.
    fn i64_unsuffixed_spanned(span: Span, integer: i64) -> Self;

    /// Construct a suffixed `i128` literal with the given span and integer.
    fn i128_suffixed_spanned(span: Span, integer: i128) -> Self;

    /// Construct an unsuffixed `i128` literal with the given span and integer.
    fn i128_unsuffixed_spanned(span: Span, integer: i128) -> Self;

    /// Construct a suffixed `isize` literal with the given span and integer.
    fn isize_suffixed_spanned(span: Span, integer: isize) -> Self;

    /// Construct an unsuffixed `isize` literal with the given span and integer.
    fn isize_unsuffixed_spanned(span: Span, integer: isize) -> Self;

    /// Construct a suffixed `u8` literal with the given span and integer.
    fn u8_suffixed_spanned(span: Span, integer: u8) -> Self;

    /// Construct an unsuffixed `u8` literal with the given span and integer.
    fn u8_unsuffixed_spanned(span: Span, integer: u8) -> Self;

    /// Construct a suffixed `u16` literal with the given span and integer.
    fn u16_suffixed_spanned(span: Span, integer: u16) -> Self;

    /// Construct an unsuffixed `u16` literal with the given span and integer.
    fn u16_unsuffixed_spanned(span: Span, integer: u16) -> Self;

    /// Construct a suffixed `u32` literal with the given span and integer.
    fn u32_suffixed_spanned(span: Span, integer: u32) -> Self;

    /// Construct an unsuffixed `u32` literal with the given span and integer.
    fn u32_unsuffixed_spanned(span: Span, integer: u32) -> Self;

    /// Construct a suffixed `u64` literal with the given span and integer.
    fn u64_suffixed_spanned(span: Span, integer: u64) -> Self;

    /// Construct an unsuffixed `u64` literal with the given span and integer.
    fn u64_unsuffixed_spanned(span: Span, integer: u64) -> Self;

    /// Construct a suffixed `u128` literal with the given span and integer.
    fn u128_suffixed_spanned(span: Span, integer: u128) -> Self;

    /// Construct an unsuffixed `u128` literal with the given span and integer.
    fn u128_unsuffixed_spanned(span: Span, integer: u128) -> Self;

    /// Construct a suffixed `usize` literal with the given span and integer.
    fn usize_suffixed_spanned(span: Span, integer: usize) -> Self;

    /// Construct an unsuffixed `usize` literal with the given span and integer.
    fn usize_unsuffixed_spanned(span: Span, integer: usize) -> Self;

    /// Construct a suffixed `f32` literal with the given span and float.
    fn f32_suffixed_spanned(span: Span, float: f32) -> Self;

    /// Construct an unsuffixed `f32` literal with the given span and float.
    fn f32_unsuffixed_spanned(span: Span, float: f32) -> Self;

    /// Construct a suffixed `f64` literal with the given span and float.
    fn f64_suffixed_spanned(span: Span, float: f64) -> Self;

    /// Construct an unsuffixed `f64` literal with the given span and float.
    fn f64_unsuffixed_spanned(span: Span, float: f64) -> Self;

    /// Construct a string literal with the given span and string.
    fn string_spanned(span: Span, string: &str) -> Self;

    /// Construct a byte string literal with the given span and bytes.
    fn byte_string_spanned(span: Span, bytes: &[u8]) -> Self;

    /// Construct a C-string literal with the given span and c-string.
    fn c_string_spanned(span: Span, string: &CStr) -> Self;

    /// Construct a `char` literal with the given span and character.
    fn character_spanned(span: Span, character: char) -> Self;

    /// Construct a byte character literal with the given span and byte.
    fn byte_character_spanned(span: Span, byte: u8) -> Self;

    /// Returns `true` if this literal is equal to `other`.
    fn equals(&self, other: &Self) -> bool;
}

impl LiteralExt for Literal {
    fn i8_suffixed_spanned(span: Span, integer: i8) -> Self {
        let mut literal = Self::i8_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i8_unsuffixed_spanned(span: Span, integer: i8) -> Self {
        let mut literal = Self::i8_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i16_suffixed_spanned(span: Span, integer: i16) -> Self {
        let mut literal = Self::i16_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i16_unsuffixed_spanned(span: Span, integer: i16) -> Self {
        let mut literal = Self::i16_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i32_suffixed_spanned(span: Span, integer: i32) -> Self {
        let mut literal = Self::i32_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i32_unsuffixed_spanned(span: Span, integer: i32) -> Self {
        let mut literal = Self::i32_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i64_suffixed_spanned(span: Span, integer: i64) -> Self {
        let mut literal = Self::i64_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i64_unsuffixed_spanned(span: Span, integer: i64) -> Self {
        let mut literal = Self::i64_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i128_suffixed_spanned(span: Span, integer: i128) -> Self {
        let mut literal = Self::i128_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn i128_unsuffixed_spanned(span: Span, integer: i128) -> Self {
        let mut literal = Self::i128_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn isize_suffixed_spanned(span: Span, integer: isize) -> Self {
        let mut literal = Self::isize_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn isize_unsuffixed_spanned(span: Span, integer: isize) -> Self {
        let mut literal = Self::isize_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u8_suffixed_spanned(span: Span, integer: u8) -> Self {
        let mut literal = Self::u8_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u8_unsuffixed_spanned(span: Span, integer: u8) -> Self {
        let mut literal = Self::u8_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u16_suffixed_spanned(span: Span, integer: u16) -> Self {
        let mut literal = Self::u16_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u16_unsuffixed_spanned(span: Span, integer: u16) -> Self {
        let mut literal = Self::u16_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u32_suffixed_spanned(span: Span, integer: u32) -> Self {
        let mut literal = Self::u32_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u32_unsuffixed_spanned(span: Span, integer: u32) -> Self {
        let mut literal = Self::u32_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u64_suffixed_spanned(span: Span, integer: u64) -> Self {
        let mut literal = Self::u64_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u64_unsuffixed_spanned(span: Span, integer: u64) -> Self {
        let mut literal = Self::u64_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u128_suffixed_spanned(span: Span, integer: u128) -> Self {
        let mut literal = Self::u128_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn u128_unsuffixed_spanned(span: Span, integer: u128) -> Self {
        let mut literal = Self::u128_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn usize_suffixed_spanned(span: Span, integer: usize) -> Self {
        let mut literal = Self::usize_suffixed(integer);
        literal.set_span(span);
        literal
    }

    fn usize_unsuffixed_spanned(span: Span, integer: usize) -> Self {
        let mut literal = Self::usize_unsuffixed(integer);
        literal.set_span(span);
        literal
    }

    fn f32_suffixed_spanned(span: Span, float: f32) -> Self {
        let mut literal = Self::f32_suffixed(float);
        literal.set_span(span);
        literal
    }

    fn f32_unsuffixed_spanned(span: Span, float: f32) -> Self {
        let mut literal = Self::f32_unsuffixed(float);
        literal.set_span(span);
        literal
    }

    fn f64_suffixed_spanned(span: Span, float: f64) -> Self {
        let mut literal = Self::f64_suffixed(float);
        literal.set_span(span);
        literal
    }

    fn f64_unsuffixed_spanned(span: Span, float: f64) -> Self {
        let mut literal = Self::f64_unsuffixed(float);
        literal.set_span(span);
        literal
    }

    fn string_spanned(span: Span, string: &str) -> Self {
        let mut literal = Self::string(string);
        literal.set_span(span);
        literal
    }

    fn byte_string_spanned(span: Span, bytes: &[u8]) -> Self {
        let mut literal = Self::byte_string(bytes);
        literal.set_span(span);
        literal
    }

    fn c_string_spanned(span: Span, string: &CStr) -> Self {
        let mut literal = Self::c_string(string);
        literal.set_span(span);
        literal
    }

    fn character_spanned(span: Span, character: char) -> Self {
        let mut literal = Self::character(character);
        literal.set_span(span);
        literal
    }

    fn byte_character_spanned(span: Span, byte: u8) -> Self {
        let mut literal = Self::byte_character(byte);
        literal.set_span(span);
        literal
    }

    fn equals(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Sealed for Literal {}
