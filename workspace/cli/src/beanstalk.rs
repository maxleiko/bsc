use std::net::{Shutdown, TcpStream};
use std::time::Duration;
use std::{io, io::Write, io::Read};

#[derive(Debug)]
pub struct Beanstalk {
  socket: Option<TcpStream>,
}

// FIXME re-read protocol in order to validate primitive types
pub struct PutOptions {
  priority: u64,
  delay: u64,
  ttr: u64,
}

impl Default for PutOptions {
  fn default() -> Self {
    PutOptions {
      priority: 0,
      delay: 0,
      ttr: 60,
    }
  }
}

impl Beanstalk {
  pub fn new() -> Beanstalk {
    Beanstalk { socket: None }
  }

  pub fn connect(&mut self, host: &str, port: u16) -> Result<(), String> {
    let addr = format!("{}:{}", host, port);
    match TcpStream::connect(addr) {
      Ok(socket) => {
        self.socket = Some(socket);
        Ok(())
      }
      Err(e) => Err(format!("{}", e)),
    }
  }

  // pub fn put(&mut self, buf: &[u8], opts: Option<PutOptions>) -> io::Result<u64> {
  //   let opts = opts.unwrap_or(PutOptions::default());
  //   let mut socket = self.socket.unwrap();

  //   Ok(0)
  // }

  // pub fn quit(self) -> io::Result<()> {
  //   let mut socket = self.socket.unwrap();
  //   Ok(())
  // }
}

#[test]
fn connect() {
  let mut socket = TcpStream::connect("localhost:11300").unwrap();
  socket.set_nonblocking(true).unwrap();

  let mut buf: Vec<u8> = Vec::new();
  loop {
    let size =  socket.read_to_end(&mut buf).unwrap();
  }
  
}
