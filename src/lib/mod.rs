#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Invalid name '{0}'. Names should be ASCII strings and  `[A-Za-z0-9+/;?$_()][A-Za-z0-9-+/;?$_()]*`")]
  InvalidName(String),
}

#[derive(Debug)]
enum Command {
  /// Tells the server that this client wants to quit
  Quit,
  /// The "put" command is for any process that wants to insert a job into the queue.
  Put { pri: u32, delay: u32, ttr: u32, bytes: u64 },
}

#[derive(Debug)]
enum Response {
  Success(SuccessResponse),
  Error(ErrorResponse),
}

#[derive(Debug)]
enum ErrorResponse {
  /// The server cannot allocate enough memory for the job.  
  /// The client should try again later.
  OutOfMemory,
  /// This indicates a bug in the server. It should never
  /// happen. If it does happen, please report it at
  /// http://groups.google.com/group/beanstalk-talk.
  InternalError,
  /// The client sent a command line that was not well-formed.
  /// This can happen if the line's length exceeds 224 bytes including \r\n,
  /// if the name of a tube exceeds 200 bytes, if non-numeric
  /// characters occur where an integer is expected, if the wrong number of
  /// arguments are present, or if the command line is mal-formed in any other
  /// way.
  BadFormat,
  /// The client sent a command that the server does not know.
  UnknownCommand,
  /// The job body must be followed by a CR-LF pair, that is,
  /// "\r\n". These two bytes are not counted in the job size given by the client
  /// in the put command line.
  ExpectedCrlf,
  /// The client has requested to put a job with a body larger
  /// than max-job-size bytes.
  JobTooBig,
  /// This means that the server has been put into "drain mode" and
  /// is no longer accepting new jobs. The client should try another server or
  /// disconnect and try again later. To put the server in drain mode, send the
  /// SIGUSR1 signal to the process.
  Draining,
}

#[derive(Debug)]
enum SuccessResponse {
  Inserted { id: u64 },
  Buried { id: u64 },
  
}

#[derive(Debug)]
enum JobState {
  Ready,
  Reserved,
  Delayed,
  Buried,
}



#[inline(always)]
fn is_alpha(c: char) -> bool {
  ('A'..='Z').contains(&c) || ('a'..='z').contains(&c)
}

#[inline(always)]
fn is_special_start(c: char) -> bool {
  matches!(c, '+' | '/' | ';' | '.' | '$' | '_' | '(' | ')')
}

#[inline(always)]
fn is_name_start(c: char) -> bool {
  is_alpha(c) || is_special_start(c)
}

#[inline(always)]
fn is_name_char(c: char) -> bool {
  is_name_start(c) || matches!(c, '-')
}

fn is_name(name: &str) -> bool {
  if name.is_empty() {
    return false;
  }

  let mut chars = name.chars();
  if let Some(c) = chars.next() {
    if !is_name_start(c) {
      return false;
    }
  }

  for c in chars {
    if !is_name_char(c) {
      return false;
    }
  }

  return true;
}

pub(crate) fn validate_name(name: impl AsRef<str>) -> Result<(), Error> {
  let name = name.as_ref();
  if is_name(name) {
    return Ok(())
  }
  Err(Error::InvalidName(String::from(name)))
}
