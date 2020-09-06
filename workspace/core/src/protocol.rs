use crate::parser::*;
use crate::types::*;

#[derive(Debug, PartialEq)]
pub enum Msg<'a> {
  Inserted(u64),
  Buried(Option<u64>),
  Using(&'a str),
  Reserved(u64, &'a [u8]),
  Watching(u64),
  Found(u64, &'a [u8]),
  Kicked(Option<u64>),
  Ok(&'a [u8]),
  Paused,
  Deleted,
  ExpectedCrlf,
  JobTooBig,
  Draining,
  OutOfMemory,
  InternalError,
  BadFormat,
  UnknownCommand,
  DeadlineSoon,
  TimedOut,
  NotFound,
  Released,
  Touched,
  NotIgnored,
}

#[derive(Debug, PartialEq)]
pub enum Cmd<'a> {
  Put {
    pri: u32,
    delay: u32,
    ttr: u32,
    payload: &'a [u8],
  },
  Use(&'a str),
  Reserve,
  ReserveJob(u64),
  ReserveTimeout(u64),
  Delete(u64),
  Release {
    id: u64,
    pri: u32,
    delay: u32,
  },
  Bury {
    id: u64,
    pri: u32,
  },
  Touch(u64),
  Watch(&'a str),
  Ignore(&'a str),
  Peek(u64),
  PeekReady,
  PeekDelayed,
  PeekBuried,
  Kick(u64),
  KickJob(u64),
  StatsJob(u64),
  StatsTube(&'a str),
  Stats,
  ListTubes,
  ListTubeUsed,
  ListTubesWatched,
  Quit,
  PauseTube {
    tube: &'a str,
    delay: u32,
  },
}

static CRLF: &[u8] = "\r\n".as_bytes();

impl From<Cmd<'_>> for Vec<u8> {
  fn from(c: Cmd<'_>) -> Self {
    match c {
      Cmd::Put {
        pri,
        delay,
        ttr,
        payload,
      } => {
        let head = format!("put {} {} {} {}\r\n", pri, delay, ttr, payload.len())
          .as_bytes()
          .to_owned();
        [&head, payload, CRLF].concat()
      }
      _ => panic!("todo"),
    }
  }
}

macro_rules! do_match {
  ($buf:expr, $msgs:expr, $cmd:expr) => {
    if let Ok((i, msg)) = $cmd($buf) {
      $msgs.push(msg);
      $buf = i;
      continue;
    };
  };
}

pub fn parse<'a>(buf: &'a [u8], msgs: &mut Vec<Msg<'a>>) -> R<'a, ()> {
  let mut buf = buf;
  while buf.len() > 0 {
    do_match!(buf, msgs, inserted);
    do_match!(buf, msgs, using);
    do_match!(buf, msgs, reserved);
    do_match!(buf, msgs, out_of_memory);
    do_match!(buf, msgs, internal_error);
    do_match!(buf, msgs, bad_format);
    do_match!(buf, msgs, unknown_command);
    do_match!(buf, msgs, expected_crlf);
    do_match!(buf, msgs, job_too_big);
    do_match!(buf, msgs, draining);
    do_match!(buf, msgs, deadline_soon);
    do_match!(buf, msgs, timed_out);
    do_match!(buf, msgs, not_found);
    do_match!(buf, msgs, deleted);
    do_match!(buf, msgs, released);
    do_match!(buf, msgs, buried);
    do_match!(buf, msgs, touched);
    do_match!(buf, msgs, watching);
    do_match!(buf, msgs, not_ignored);
    do_match!(buf, msgs, found);
    do_match!(buf, msgs, kicked);
    do_match!(buf, msgs, ok);
    do_match!(buf, msgs, paused);
    not_matched!(buf);
  }
  Ok((buf, ()))
}

fn name<'a>(buf: &'a [u8]) -> R<'a, &str> {
  eof!(buf);
  if buf[0] == b'-' {
    // names cannot start with an hyphen
    return Err((buf, ErrorKind::NotMatched));
  }
  let (i, name) = take_until(buf, is_ascii_whitespace)?;
  match std::str::from_utf8(name) {
    Ok(name) => Ok((i, name)),
    Err(e) => Err((buf, ErrorKind::NotUTF8(e))),
  }
}

fn inserted<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "INSERTED")?;
  let (i, _) = space(i)?;
  let (i, id) = u64(i)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Inserted(id)))
}

fn using<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "USING")?;
  let (i, _) = space(i)?;
  let (i, name) = name(i)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Using(name)))
}

fn reserved<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "RESERVED")?;
  let (i, _) = space(i)?;
  let (i, id) = u64(i)?;
  let (i, _) = space(i)?;
  let (i, len) = u64(i)?;
  let (i, _) = crlf(i)?;
  let (i, data) = take(i, len as usize)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Reserved(id, data)))
}

fn out_of_memory<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "OUT_OF_MEMORY")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::OutOfMemory))
}

fn internal_error<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "INTERNAL_ERROR")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::InternalError))
}

fn bad_format<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "BAD_FORMAT")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::BadFormat))
}

fn unknown_command<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "UNKNOWN_COMMAND")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::UnknownCommand))
}

fn expected_crlf<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "EXPECTED_CRLF")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::ExpectedCrlf))
}

fn job_too_big<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "JOB_TOO_BIG")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::JobTooBig))
}

fn draining<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "DRAINING")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Draining))
}

fn deadline_soon<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "DEADLINE_SOON")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::DeadlineSoon))
}

fn timed_out<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "TIMED_OUT")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::TimedOut))
}

fn not_found<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "NOT_FOUND")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::NotFound))
}

fn deleted<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "DELETED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Deleted))
}

fn released<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "RELEASED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Released))
}

fn buried<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "BURIED")?;
  if let Ok((i, _)) = space(i) {
    let (i, id) = u64(i)?;
    let (i, _) = crlf(i)?;
    Ok((i, Msg::Buried(Some(id))))
  } else {
    let (i, _) = crlf(i)?;
    Ok((i, Msg::Buried(None)))
  }
}

fn touched<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "TOUCHED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Touched))
}

fn watching<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "TOUCHED")?;
  let (i, _) = space(i)?;
  let (i, count) = u64(i)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Watching(count)))
}

fn not_ignored<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "NOT_IGNORED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::NotIgnored))
}

fn found<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "FOUND")?;
  let (i, id) = u64(i)?;
  let (i, _) = space(i)?;
  let (i, len) = u64(i)?;
  let (i, _) = crlf(i)?;
  let (i, data) = take(i, len as usize)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Found(id, data)))
}

fn kicked<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "KICKED")?;
  if let Ok((i, _)) = space(i) {
    let (i, count) = u64(i)?;
    let (i, _) = crlf(i)?;
    Ok((i, Msg::Kicked(Some(count))))
  } else {
    let (i, _) = crlf(i)?;
    Ok((i, Msg::Kicked(None)))
  }
}

fn ok<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "OK")?;
  let (i, _) = space(i)?;
  let (i, len) = u64(i)?;
  let (i, _) = crlf(i)?;
  let (i, data) = take(i, len as usize)?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Ok(data)))
}

fn paused<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "PAUSED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Paused))
}

#[cfg(test)]
mod tests {
  use crate::protocol;
  use crate::protocol::{Cmd, Msg};

  #[test]
  fn inserted() {
    assert_eq!(
      protocol::inserted("INSERTED 42\r\n".as_bytes()),
      Ok(("".as_bytes(), Msg::Inserted(42))),
    );
  }

  #[test]
  fn reserved() {
    assert_eq!(
      protocol::reserved("RESERVED 42 6\r\n123456\r\n".as_bytes()),
      Ok(("".as_bytes(), Msg::Reserved(42, "123456".as_bytes()))),
    );
  }

  #[test]
  fn kicked_none() {
    assert_eq!(
      protocol::kicked("KICKED\r\n".as_bytes()),
      Ok(("".as_bytes(), Msg::Kicked(None))),
    );
  }

  #[test]
  fn kicked_some() {
    assert_eq!(
      protocol::kicked("KICKED 42\r\n".as_bytes()),
      Ok(("".as_bytes(), Msg::Kicked(Some(42)))),
    );
  }

  #[test]
  fn using() {
    assert_eq!(
      protocol::using("USING foo\r\n".as_bytes()),
      Ok(("".as_bytes(), Msg::Using("foo"))),
    );
  }

  #[test]
  fn put() {
    let cmd: Vec<u8> = Cmd::Put {
      pri: 1,
      delay: 2,
      ttr: 3,
      payload: "123456".as_bytes(),
    }
    .into();
    assert_eq!(cmd, Vec::from("put 1 2 3 6\r\n123456\r\n".as_bytes()),);
  }
}
