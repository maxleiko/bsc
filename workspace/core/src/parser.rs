use crate::types::*;

macro_rules! eof {
  ($buf:expr) => {
    if $buf.len() == 0 {
      return Err(($buf, ErrorKind::EOF));
    }
  };
}

macro_rules! not_matched {
  ($buf:expr) => {
    return Err(($buf, ErrorKind::NotMatched));
  };
}

pub fn char<'a>(buf: &'a [u8], c: char) -> R<'a, char> {
  eof!(buf);

  if buf[0] == c as u8 {
    return Ok((&buf[1..], buf[0].into()));
  }

  not_matched!(buf)
}

pub fn crlf<'a>(buf: &'a [u8]) -> R<'a, ()> {
  let (i, _) = char(buf, '\r')?;
  let (i, _) = char(i, '\n')?;
  Ok((i, ()))
}

pub fn space<'a>(buf: &'a [u8]) -> R<'a, ()> {
  let (i, _) = char(buf, ' ')?;
  Ok((i, ()))
}

pub fn chars<'a>(buf: &'a [u8], value: &str) -> R<'a, &'a [u8]> {
  eof!(buf);

  let len = value.len();

  if buf.len() < len {
    not_matched!(buf);
  }

  if value.as_bytes().eq(&buf[..len]) {
    return Ok((&buf[len..], &buf[..len]));
  }

  not_matched!(buf)
}

pub fn is_ascii_digit(b: u8) -> bool {
  b.is_ascii_digit()
}

pub fn is_ascii_alphabetic(b: u8) -> bool {
  b.is_ascii_alphabetic()
}

pub fn is_ascii_alphanumeric(b: u8) -> bool {
  b.is_ascii_alphanumeric()
}

pub fn is_ascii_whitespace(b: u8) -> bool {
  b.is_ascii_whitespace()
}

pub fn string<'a>(buf: &'a [u8]) -> R<'a, &'a str> {
  let (i, s) = take_while(buf, is_ascii_alphanumeric)?;
  Ok((i, unsafe { std::str::from_utf8_unchecked(s) }))
}

pub fn take_while<'a>(buf: &'a [u8], func: MatchFunc) -> R<'a, &'a [u8]> {
  eof!(buf);

  let mut idx: usize = 0;
  let start = idx;
  while idx < buf.len() && func(buf[idx]) == true {
    idx += 1;
  }

  if start != idx {
    return Ok((&buf[idx..], &buf[..idx]));
  }

  not_matched!(buf)
}

pub fn take_until<'a>(buf: &'a [u8], func: MatchFunc) -> R<'a, &'a [u8]> {
  eof!(buf);

  let mut idx: usize = 0;
  let start = idx;
  while idx < buf.len() && func(buf[idx]) == false {
    idx += 1;
  }

  if start != idx {
    return Ok((&buf[idx..], &buf[..idx]));
  }

  not_matched!(buf)
}

pub fn take<'a>(buf: &'a [u8], len: usize) -> R<'a, &'a [u8]> {
  eof!(buf);

  if buf.len() >= len {
    return Ok((&buf[len..], &buf[..len]));
  }

  not_matched!(buf);
}

pub fn u64<'a>(buf: &'a [u8]) -> R<'a, u64> {
  eof!(buf);

  let mut idx: usize = 0;
  let start = idx;
  while idx < buf.len() && buf[idx].is_ascii_digit() {
    idx += 1;
  }
  if start != idx {
    // we can go unsafe here because we just `is_ascii_digit`-ed them
    let str = unsafe { std::str::from_utf8_unchecked(&buf[..idx]) };
    return match str.parse() {
      Ok(val) => Ok((&buf[idx..], val)),
      Err(e) => Err((buf, ErrorKind::Integer(e))),
    };
  }

  not_matched!(buf)
}

pub fn lf<'a>(buf: &'a [u8]) -> R<'a, ()> {
  let (i, _) = self::char(buf, '\n')?;
  Ok((i, ()))
}

pub fn bool<'a>(buf: &'a [u8]) -> R<'a, bool> {
  if let Ok((i, val)) = chars(buf, "true") {
    Ok((i, true))
  } else if let Ok((i, val)) = chars(buf, "false") {
    Ok((i, false))
  } else {
    Err((buf, ErrorKind::NotMatched))
  }
}

pub fn number<'a>(buf: &'a [u8]) -> R<'a, Scalar> {
  eof!(buf);
  let (i, head) = u64(buf)?;
  if let Ok((i, _)) = self::char(i, '.') {
    let (i, tail) = u64(i)?;
    let mut val = head.to_string();
    val.push('.');
    val.push_str(tail.to_string().as_str());
    match val.parse::<f64>() {
      Ok(val) => Ok((i, Scalar::Float(val))),
      Err(e) => Err((buf, ErrorKind::Float(e)))
    }
  } else {
    Ok((i, Scalar::Integer(head)))
  }
}

pub fn scalar<'a>(i: &'a [u8]) -> R<'a, Scalar<'a>> {
  if let Ok((i, val)) = self::bool(i) {
    Ok((i, Scalar::Bool(val)))
  } else if let Ok((i, val)) = number(i) {
    Ok((i, val))
  } else if let Ok((i, val)) = string(i) {
    Ok((i, Scalar::String(val)))
  } else {
    Err((i, ErrorKind::NotMatched))
  }
}

#[cfg(test)]
mod tests {
  use crate::parser;
  use crate::types::ErrorKind;

  #[test]
  fn char() {
    assert_eq!(
      parser::char("hello".as_bytes(), 'h'),
      Ok(("ello".as_bytes(), 'h'))
    );

    assert_eq!(
      parser::char("".as_bytes(), 'a'),
      Err(("".as_bytes(), ErrorKind::EOF))
    );
  }

  #[test]
  fn crlf() {
    assert_eq!(
      parser::crlf("\r\nbar".as_bytes()),
      Ok(("bar".as_bytes(), ()))
    );
    assert_eq!(
      parser::crlf("foo\r\n".as_bytes()),
      Err(("foo\r\n".as_bytes(), ErrorKind::NotMatched))
    );
  }

  #[test]
  fn space() {
    assert_eq!(parser::space(" foo".as_bytes()), Ok(("foo".as_bytes(), ())));
    assert_eq!(
      parser::space("\n  ".as_bytes()),
      Err(("\n  ".as_bytes(), ErrorKind::NotMatched))
    );
  }

  #[test]
  fn chars() {
    assert_eq!(
      parser::chars("foobar".as_bytes(), "foo"),
      Ok(("bar".as_bytes(), "foo".as_bytes()))
    );

    assert_eq!(
      parser::chars("foobar".as_bytes(), "bar"),
      Err(("foobar".as_bytes(), ErrorKind::NotMatched))
    );
  }

  #[test]
  fn u64() {
    assert_eq!(parser::u64("1234".as_bytes()), Ok(("".as_bytes(), 1234)));

    assert_eq!(
      parser::u64("foo".as_bytes()),
      Err(("foo".as_bytes(), ErrorKind::NotMatched))
    );
  }

  #[test]
  fn take() {
    assert_eq!(
      parser::take("Hello World".as_bytes(), 5),
      Ok((" World".as_bytes(), "Hello".as_bytes()))
    );

    {
      let expected = [5u8, 4, 100, 25, 18];
      assert_eq!(
        parser::take(&expected, 3),
        Ok((&expected[3..], &expected[..3]))
      );
    }

    {
      let expected = [5u8, 4, 100];
      assert_eq!(
        parser::take(&expected, 3),
        Ok((&[] as &[u8], &expected[..3]))
      );
    }
  }

  #[test]
  fn take_while() {
    assert_eq!(
      parser::take_while("123456foo".as_bytes(), parser::is_ascii_digit),
      Ok(("foo".as_bytes(), "123456".as_bytes()))
    );

    assert_eq!(
      parser::take_while("foo123456".as_bytes(), parser::is_ascii_alphabetic),
      Ok(("123456".as_bytes(), "foo".as_bytes()))
    );
  }

  #[test]
  fn take_until() {
    assert_eq!(
      parser::take_until("123456foo".as_bytes(), parser::is_ascii_alphabetic),
      Ok(("foo".as_bytes(), "123456".as_bytes()))
    );

    assert_eq!(
      parser::take_until("foo123456".as_bytes(), parser::is_ascii_digit),
      Ok(("123456".as_bytes(), "foo".as_bytes()))
    );

    assert_eq!(
      parser::take_until("hello world".as_bytes(), parser::is_ascii_whitespace),
      Ok((" world".as_bytes(), "hello".as_bytes()))
    );

    assert_eq!(
      parser::take_until("hello\r\n".as_bytes(), parser::is_ascii_whitespace),
      Ok(("\r\n".as_bytes(), "hello".as_bytes()))
    );

    assert_eq!(
      parser::take_until("hello\n".as_bytes(), parser::is_ascii_whitespace),
      Ok(("\n".as_bytes(), "hello".as_bytes()))
    );

    assert_eq!(
      parser::take_until("hello\t".as_bytes(), parser::is_ascii_whitespace),
      Ok(("\t".as_bytes(), "hello".as_bytes()))
    );
  }
}
