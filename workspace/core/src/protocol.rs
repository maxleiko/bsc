use crate::parser::*;
use crate::types::*;

#[derive(Debug, PartialEq)]
pub enum Msg {
  Inserted(u64),
  Buried(Option<u64>),
  Using(String),
  Reserved(u64, Vec<u8>),
  Watching(u64),
  Found(u64, Vec<u8>),
  Kicked(Option<u64>),
  Ok(Vec<u8>),
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
      Cmd::Use(tube) => format!("use {}\r\n", tube).into(),
      Cmd::Reserve => "reserve\r\n".into(),
      Cmd::ReserveJob(id) => format!("reserve-job {}\r\n", id).into(),
      Cmd::ReserveTimeout(secs) => format!("reserve-with-timeout {}\r\n", secs).into(),
      Cmd::Delete(id) => format!("delete {}\r\n", id).into(),
      Cmd::Release { pri, delay, id } => format!("release {} {} {}\r\n", id, pri, delay).into(),
      Cmd::Bury { id, pri } => format!("bury {} {}\r\n", id, pri).into(),
      Cmd::Touch(id) => format!("touch {}\r\n", id).into(),
      Cmd::Watch(tube) => format!("watch {}\r\n", tube).into(),
      Cmd::Ignore(tube) => format!("ignore {}\r\n", tube).into(),
      Cmd::Peek(id) => format!("peek {}\r\n", id).into(),
      Cmd::PeekReady => "peek-ready\r\n".into(),
      Cmd::PeekDelayed => "peek-delayed\r\n".into(),
      Cmd::PeekBuried => "peek-buried\r\n".into(),
      Cmd::Kick(bound) => format!("kick {}\r\n", bound).into(),
      Cmd::KickJob(id) => format!("kick-job {}\r\n", id).into(),
      Cmd::StatsJob(id) => format!("stats-job {}\r\n", id).into(),
      Cmd::StatsTube(tube) => format!("stats-tube {}\r\n", tube).into(),
      Cmd::Stats => format!("stats\r\n").into(),
      Cmd::ListTubes => "list-tubes\r\n".into(),
      Cmd::ListTubeUsed => "list-tube-used\r\n".into(),
      Cmd::ListTubesWatched => "list-tubes-watched\r\n".into(),
      Cmd::Quit => "quit\r\n".into(),
      Cmd::PauseTube { tube, delay } => format!("pause-tube {} {}\r\n", tube, delay).into(),
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

macro_rules! try_parse {
  ($buf:expr, $cmd:expr) => {
    if let Ok((i, msg)) = $cmd($buf) {
      return Ok((i, msg));
    }
  };
}

pub fn parse<'a>(buf: &'a [u8], msgs: &mut Vec<Msg>) -> R<'a, ()> {
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

pub fn atomic_parse<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  try_parse!(buf, inserted);
  try_parse!(buf, using);
  try_parse!(buf, reserved);
  try_parse!(buf, out_of_memory);
  try_parse!(buf, internal_error);
  try_parse!(buf, bad_format);
  try_parse!(buf, unknown_command);
  try_parse!(buf, expected_crlf);
  try_parse!(buf, job_too_big);
  try_parse!(buf, draining);
  try_parse!(buf, deadline_soon);
  try_parse!(buf, timed_out);
  try_parse!(buf, not_found);
  try_parse!(buf, deleted);
  try_parse!(buf, released);
  try_parse!(buf, buried);
  try_parse!(buf, touched);
  try_parse!(buf, watching);
  try_parse!(buf, not_ignored);
  try_parse!(buf, found);
  try_parse!(buf, kicked);
  try_parse!(buf, ok);
  try_parse!(buf, paused);
  not_matched!(buf);
}

fn name<'a>(buf: &'a [u8]) -> R<'a, &str> {
  eof!(buf);
  if buf[0] == b'-' {
    // names cannot start with an hyphen
    return Err((buf, ErrorKind::NotMatched));
  }
  let (i, name) = take_until(buf, is_ascii_whitespace)?;
  // FIXME investigate the unsafeness of this further more.
  // But as the beanstalkd protocol states: everything other
  // than the payload is supposed to be ASCII, UTF-8 being
  // a superset of ASCII, we should be good to go
  Ok((i, unsafe { std::str::from_utf8_unchecked(name) }))
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
  Ok((i, Msg::Using(name.into())))
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
  Ok((i, Msg::Reserved(id, data.into())))
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
  Ok((i, Msg::Found(id, data.into())))
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
  Ok((i, Msg::Ok(data.into())))
}

fn paused<'a>(buf: &'a [u8]) -> R<'a, Msg> {
  let (i, _) = chars(buf, "PAUSED")?;
  let (i, _) = crlf(i)?;
  Ok((i, Msg::Paused))
}

#[cfg(test)]
mod tests {
  mod msg {
    use crate::protocol;
    use crate::protocol::Msg;

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
        Ok(("".as_bytes(), Msg::Reserved(42, Vec::from("123456")))),
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
        Ok(("".as_bytes(), Msg::Using(String::from("foo")))),
      );
    }
  }

  mod cmd {
    use crate::protocol::Cmd;

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

    #[test]
    fn use_() {
      let cmd: Vec<u8> = Cmd::Use("tube").into();
      assert_eq!(cmd, Vec::from("use tube\r\n".as_bytes()));
    }

    #[test]
    fn reserve() {
      let cmd: Vec<u8> = Cmd::Reserve.into();
      assert_eq!(cmd, Vec::from("reserve\r\n".as_bytes()));
    }

    #[test]
    fn reserve_job() {
      let cmd: Vec<u8> = Cmd::ReserveJob(42).into();
      assert_eq!(cmd, Vec::from("reserve-job 42\r\n".as_bytes()));
    }

    #[test]
    fn reserve_timeout() {
      let cmd: Vec<u8> = Cmd::ReserveTimeout(42).into();
      assert_eq!(cmd, Vec::from("reserve-with-timeout 42\r\n".as_bytes()));
    }

    #[test]
    fn delete() {
      let cmd: Vec<u8> = Cmd::Delete(42).into();
      assert_eq!(cmd, Vec::from("delete 42\r\n".as_bytes()));
    }

    #[test]
    fn release() {
      let cmd: Vec<u8> = Cmd::Release {
        id: 42,
        pri: 2,
        delay: 3,
      }
      .into();
      assert_eq!(cmd, Vec::from("release 42 2 3\r\n".as_bytes()));
    }

    #[test]
    fn bury() {
      let cmd: Vec<u8> = Cmd::Bury { id: 42, pri: 2 }.into();
      assert_eq!(cmd, Vec::from("bury 42 2\r\n".as_bytes()));
    }

    #[test]
    fn touch() {
      let cmd: Vec<u8> = Cmd::Touch(42).into();
      assert_eq!(cmd, Vec::from("touch 42\r\n".as_bytes()));
    }

    #[test]
    fn watch() {
      let cmd: Vec<u8> = Cmd::Watch("tube").into();
      assert_eq!(cmd, Vec::from("watch tube\r\n".as_bytes()));
    }

    #[test]
    fn ignore() {
      let cmd: Vec<u8> = Cmd::Ignore("tube").into();
      assert_eq!(cmd, Vec::from("ignore tube\r\n".as_bytes()));
    }

    #[test]
    fn peek() {
      let cmd: Vec<u8> = Cmd::Peek(42).into();
      assert_eq!(cmd, Vec::from("peek 42\r\n".as_bytes()));
    }

    #[test]
    fn peek_ready() {
      let cmd: Vec<u8> = Cmd::PeekReady.into();
      assert_eq!(cmd, Vec::from("peek-ready\r\n".as_bytes()));
    }

    #[test]
    fn peek_delayed() {
      let cmd: Vec<u8> = Cmd::PeekDelayed.into();
      assert_eq!(cmd, Vec::from("peek-delayed\r\n".as_bytes()));
    }

    #[test]
    fn peek_buried() {
      let cmd: Vec<u8> = Cmd::PeekBuried.into();
      assert_eq!(cmd, Vec::from("peek-buried\r\n".as_bytes()));
    }

    #[test]
    fn kick() {
      let cmd: Vec<u8> = Cmd::Kick(42).into();
      assert_eq!(cmd, Vec::from("kick 42\r\n".as_bytes()));
    }

    #[test]
    fn kick_job() {
      let cmd: Vec<u8> = Cmd::KickJob(42).into();
      assert_eq!(cmd, Vec::from("kick-job 42\r\n".as_bytes()));
    }

    #[test]
    fn stats_job() {
      let cmd: Vec<u8> = Cmd::StatsJob(42).into();
      assert_eq!(cmd, Vec::from("stats-job 42\r\n".as_bytes()));
    }

    #[test]
    fn stats_tube() {
      let cmd: Vec<u8> = Cmd::StatsTube("tube").into();
      assert_eq!(cmd, Vec::from("stats-tube tube\r\n".as_bytes()));
    }

    #[test]
    fn stats() {
      let cmd: Vec<u8> = Cmd::Stats.into();
      assert_eq!(cmd, Vec::from("stats\r\n".as_bytes()));
    }

    #[test]
    fn list_tubes() {
      let cmd: Vec<u8> = Cmd::ListTubes.into();
      assert_eq!(cmd, Vec::from("list-tubes\r\n".as_bytes()));
    }

    #[test]
    fn list_tube_used() {
      let cmd: Vec<u8> = Cmd::ListTubeUsed.into();
      assert_eq!(cmd, Vec::from("list-tube-used\r\n".as_bytes()));
    }

    #[test]
    fn list_tubes_watched() {
      let cmd: Vec<u8> = Cmd::ListTubesWatched.into();
      assert_eq!(cmd, Vec::from("list-tubes-watched\r\n".as_bytes()));
    }

    #[test]
    fn quit() {
      let cmd: Vec<u8> = Cmd::Quit.into();
      assert_eq!(cmd, Vec::from("quit\r\n".as_bytes()));
    }

    #[test]
    fn pause_tube() {
      let cmd: Vec<u8> = Cmd::PauseTube {
        tube: "tube",
        delay: 42,
      }
      .into();
      assert_eq!(cmd, Vec::from("pause-tube tube 42\r\n".as_bytes()));
    }
  }
}
