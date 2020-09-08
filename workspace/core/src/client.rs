use crate::types::{ErrorKind, ParseError};
use crate::{protocol, protocol::Cmd, protocol::Msg};
use std::net::TcpStream;
use std::{io, io::Write, io::BufRead, io::BufReader};

#[derive(Debug)]
pub struct BeanstalkClient {
  stream: TcpStream,
}

#[derive(Debug)]
pub enum BeanstalkError {
  IO(io::Error),
  Parse(ErrorKind),
  Msg(Msg),
}

impl<'a> From<io::Error> for BeanstalkError {
  fn from(e: io::Error) -> Self {
    BeanstalkError::IO(e)
  }
}

impl<'a> From<ParseError<'a>> for BeanstalkError {
  fn from(e: ParseError<'a>) -> Self {
    BeanstalkError::Parse(e.1)
  }
}

impl BeanstalkClient {
  pub fn connect(host: &str, port: u16) -> Result<BeanstalkClient, BeanstalkError> {
    let stream = TcpStream::connect(format!("{}:{}", host, port))?;
    stream.set_nonblocking(false)?;
    Ok(BeanstalkClient { stream })
  }

  pub fn put(
    &mut self,
    payload: &[u8],
    pri: Option<u32>,
    delay: Option<u32>,
    ttr: Option<u32>,
  ) -> Result<u64, BeanstalkError> {
    let cmd: Vec<u8> = Cmd::Put {
      pri: pri.unwrap_or(0),
      delay: delay.unwrap_or(0),
      ttr: ttr.unwrap_or(60),
      payload,
    }
    .into();
    self.stream.write(&cmd)?;
    match self.handle_response()? {
      Msg::Inserted(id) => Ok(id),
      m => Err(BeanstalkError::Msg(m)),
    }
  }

  pub fn delete(&mut self, id: u64) -> Result<(), BeanstalkError> {
    let cmd: Vec<u8> = Cmd::Delete(id).into();
    self.stream.write(&cmd)?;
    match self.handle_response()? {
      Msg::Deleted => Ok(()),
      m => Err(BeanstalkError::Msg(m)),
    }
  }

  pub fn reserve(&mut self) -> Result<(u64, Vec<u8>), BeanstalkError> {
    let cmd: Vec<u8> = Cmd::Reserve.into();
    self.stream.write(&cmd)?;
    match self.handle_response()? {
      Msg::Reserved(id, payload) => Ok((id, payload)),
      m => Err(BeanstalkError::Msg(m)),
    }
  }

  pub fn release(
    &mut self,
    id: u64,
    pri: Option<u32>,
    delay: Option<u32>,
  ) -> Result<(), BeanstalkError> {
    let cmd: Vec<u8> = Cmd::Release {
      id,
      pri: pri.unwrap_or(0),
      delay: delay.unwrap_or(0),
    }
    .into();
    self.stream.write(&cmd)?;
    match self.handle_response()? {
      Msg::Released => Ok(()),
      m => Err(BeanstalkError::Msg(m)),
    }
  }

  /// Once you call `quit()` there is no coming-back  
  /// That's why this function takes ownership of `self`
  pub fn quit(mut self) -> Result<(), BeanstalkError> {
    let cmd: Vec<u8> = Cmd::Quit.into();
    self.stream.write(&cmd)?;
    Ok(())
  }

  fn handle_response(&mut self) -> Result<Msg, BeanstalkError> {
    let mut reader = BufReader::new(&self.stream);
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf)?;
    loop {
      match protocol::atomic_parse(&buf) {
        Ok((_, msg)) => {
          return Ok(msg);
        },
        Err(_) => {
          reader.read_until(b'\n', &mut buf)?;
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  mod client {
    use crate::client::BeanstalkClient;
    use crate::client::BeanstalkError;

    #[test]
    fn connect() {
      assert!(BeanstalkClient::connect("localhost", 11300).is_ok());
    }

    #[test]
    fn quit() -> Result<(), BeanstalkError> {
      let client = BeanstalkClient::connect("localhost", 11300)?;
      client.quit()?;
      Ok(())
    }

    #[test]
    fn put_reserve_delete() -> Result<(), BeanstalkError> {
      let mut client = BeanstalkClient::connect("localhost", 11300)?;
      client.put("Hello World".as_bytes(), None, None, None)?;
      let (id, payload) = client.reserve()?;
      // assert_eq!(id, 1);
      assert_eq!(payload, Vec::from("Hello World"));
      client.delete(id)?;
      Ok(())
    }
  }
}
