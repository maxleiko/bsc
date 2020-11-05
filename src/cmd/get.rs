use std::{error::Error, io::stdout};
use std::time::Duration;
use std::io::Write;

use beanstalkc::Beanstalkc;
use clap::{App, Arg, ArgMatches, SubCommand};

use crate::utils;
use crate::IOptions;


pub fn create() -> App<'static, 'static> {
  SubCommand::with_name("get")
    .display_order(0) // print it before "help"
    .about("Gets the latest message on the specified tube")
    .version("0.1.0")
    .arg(
      Arg::with_name("delete")
        .long("delete")
        .short("d")
        .default_value("false")
        .takes_value(false)
        .help("Whether or not to delete the message from the queue"),
    )
    .arg(
      Arg::with_name("timeout")
        .long("timeout")
        .env("TIMEOUT")
        .validator(utils::is_u64)
        .default_value("0")
        .help("Beanstalkd reserve timeout in seconds (set to 0 to disable timeout)"),
    )
}

pub fn get(client: &mut Beanstalkc, opts: &IOptions, cmd: &ArgMatches) -> Result<(), Box<dyn Error>> {
  let timeout: Option<Duration> = match cmd.value_of("timeout") {
    Some(t) => Some(Duration::from_secs(t.parse::<u64>()?)),
    None => None,
  };

  if !opts.tube.eq("default") {
    client.ignore("default")?;
    client.watch(&opts.tube)?;
  }

  let mut job = match timeout {
    Some(t) => client.reserve_with_timeout(t)?,
    None => client.reserve()?,
  };

  if let Err(e) = stdout().write_all(job.body()) {
    eprintln!("Something went wrong {}", e);
  }

  if cmd.is_present("delete") {
    job.delete()?;
  } else {
    job.release_default()?;
  }

  Ok(())
}
