use crate::parser::*;
use crate::types::*;

pub fn yaml_list<'a>(buf: &'a [u8]) -> R<'a, YAMLList<'a>> {
  let (i, _) = start(buf)?;
  let mut buf = i;
  let mut list = YAMLList::new();
  while buf.len() > 0 {
    let (i, _) = list_entry(buf)?;
    let (i, value) = string(i)?;
    let (i, _) = lf(i)?;
    list.push(value);
    buf = i;
  }
  Ok((buf, list))
}

pub fn yaml_map<'a>(buf: &'a [u8]) -> R<'a, YAMLMap<'a>> {
  let (i, _) = start(buf)?;
  let mut buf = i;
  let mut map = YAMLMap::new();
  while buf.len() > 0 {
    let (i, (key, value)) = map_entry(buf)?;
    let (i, _) = lf(i)?;
    map.insert(key, value);
    buf = i;
  }
  Ok((buf, map))
}

fn map_entry<'a>(i: &'a [u8]) -> R<'a, (&'a str, Scalar<'a>)> {
  let (i, key) = string(i)?;
  let (i, _) = chars(i, ": ")?;
  let (i, value) = scalar(i)?;
  Ok((i, (key, value)))
}

fn start<'a>(i: &'a [u8]) -> R<'a, ()> {
  let (i, _) = chars(i, "---")?;
  let (i, _) = lf(i)?;
  Ok((i, ()))
}

fn list_entry<'a>(i: &'a [u8]) -> R<'a, ()> {
  let (buf, _) = chars(i, " - ")?;
  Ok((buf, ()))
}

#[cfg(test)]
mod tests {
  use crate::types::{Scalar, YAMLMap};
  use crate::yaml;

  #[test]
  fn yaml_list() {
    assert_eq!(
      yaml::yaml_list("---\n - hello\n - world\n".as_bytes()),
      Ok(("".as_bytes(), vec!["hello", "world"]))
    );
  }

  #[test]
  fn yaml_map() {
    let mut map = YAMLMap::new();
    map.insert("true", Scalar::Bool(true));
    map.insert("false", Scalar::Bool(false));
    map.insert("int", Scalar::Integer(42));
    map.insert("float", Scalar::Float(3.14));
    map.insert("string", Scalar::String("HelloWorld"));

    assert_eq!(
      yaml::yaml_map(
        "---\ntrue: true\nfalse: false\nint: 42\nfloat: 3.14\nstring: HelloWorld\n".as_bytes()
      ),
      Ok(("".as_bytes(), map))
    );
  }
}
