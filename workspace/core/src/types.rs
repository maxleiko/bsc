use std::collections::HashMap;
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
  Integer(std::num::ParseIntError),
  Float(std::num::ParseFloatError),
  NotUTF8(std::str::Utf8Error),
  NotMatched,
  EOF,
}

#[derive(Debug, PartialEq)]
pub enum Scalar<'a> {
  String(&'a str),
  Integer(u64),
  Float(f64),
  Bool(bool),
}

pub type ParseError<'a> = (&'a [u8], ErrorKind);
pub type R<'a, O> = Result<(&'a [u8], O), ParseError<'a>>;
pub type MatchFunc = fn(u8) -> bool;
pub type YAMLMap<'a> = HashMap<&'a str, Scalar<'a>>;
pub type YAMLList<'a> = Vec<&'a str>;
