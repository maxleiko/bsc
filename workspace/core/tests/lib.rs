extern crate bsc_core;

use bsc_core::protocol;
use bsc_core::protocol::Msg;
use bsc_core::types::ErrorKind;

#[test]
fn parse_unfinished() {
  let buf = "RESERVED 42 3\r\n123\r\nUSING Hello\r\nUSI".as_bytes();
  let mut msgs = Vec::new();
  let res = protocol::parse(buf, &mut msgs);
  assert_eq!(res, Err(("USI".as_bytes(), ErrorKind::NotMatched)));
  assert_eq!(msgs, Vec::from([
    Msg::Reserved(42, "123".as_bytes()),
    Msg::Using("Hello"),
  ]))
}

#[test]
fn parse_finished() {
  let buf = "RESERVED 42 3\r\n123\r\nUSING Hello\r\n".as_bytes();
  let mut msgs = Vec::new();
  let res = protocol::parse(buf, &mut msgs);
  assert_eq!(res, Ok(("".as_bytes(), ())));
  assert_eq!(msgs, Vec::from([
    Msg::Reserved(42, "123".as_bytes()),
    Msg::Using("Hello"),
  ]))
}

#[test]
fn serialize() {
  
}
