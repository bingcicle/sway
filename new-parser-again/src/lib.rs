mod priv_prelude;
mod span;
mod ident;
mod literal;
mod token;
pub mod parser;
pub mod parse;
pub mod keywords;
pub mod program;
pub mod dependency;
mod item;
pub mod brackets;
pub mod punctuated;
pub mod ty;
pub mod expr;
pub mod pattern;
pub mod path;
pub mod generics;
pub mod statement;
pub mod assignable;

pub use crate::{
    span::Span,
    ident::Ident,
    token::lex,
    parser::Parser,
    parse::Parse,
    program::Program,
    item::{
        item_use::{ItemUse, UseTree},
        item_struct::ItemStruct,
        item_enum::ItemEnum,
    },
};

use std::sync::Arc;

pub fn lex_and_parse<T>(src: &Arc<str>, start: usize, end: usize) -> T
where
    T: Parse,
{
    let token_stream = lex(src, start, end).unwrap();
    let mut parser = Parser::new(&token_stream);
    let ret = parser.parse().unwrap();
    if !parser.is_empty() {
        panic!("not all tokens consumed");
    }
    ret
}

