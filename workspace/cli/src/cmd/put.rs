use crate::utils;
use crate::IOptions;
use crate::beanstalk::Beanstalk;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::time::Duration;
use std::io;
// use std::io::prelude::*;

pub fn create() -> App<'static, 'static> {
  SubCommand::with_name("put")
    .display_order(0) // print it before "help"
    .about("Puts the content of stdin to the specified tube")
    .version("0.1.0")
    .arg(
      Arg::with_name("priority")
        .short("p")
        .long("priority")
        .env("PRIORITY")
        .validator(utils::is_u32)
        .default_value("0")
        .help("Beanstalkd priority (the lower the value the higher the priority)"),
    )
    .arg(
      Arg::with_name("delay")
        .short("d")
        .long("delay")
        .env("DELAY")
        .validator(utils::is_u64)
        .default_value("0")
        .help("Beanstalkd put delay in seconds"),
    )
    .arg(
      Arg::with_name("ttr")
        .short("r")
        .long("ttr")
        .env("TTR")
        .validator(utils::is_u64)
        .default_value("10")
        .help("Beanstalkd time-to-run (number of seconds to allow a worker to run this job)"),
    )
}

fn parse(cmd: &ArgMatches) -> (u32, Duration, Duration) {
  (
    cmd.value_of("priority").unwrap().parse::<u32>().unwrap(),
    Duration::from_secs(cmd.value_of("delay").unwrap().parse::<u64>().unwrap()),
    Duration::from_secs(cmd.value_of("ttr").unwrap().parse::<u64>().unwrap()),
  )
}

pub fn call(opts: &IOptions, cmd: &ArgMatches) {
  let (_priority, _delay, _ttr) = parse(cmd);
  let mut client = Beanstalk::new();
  client.connect(&opts.host, opts.port).unwrap();
  // client.use_tube(&opts.tube).unwrap();

  // let stdin = io::stdin();
  // let mut stdin = stdin.lock();
  // let buf = match stdin.fill_buf() {
  //   Ok(buf) => buf,
  //   Err(e) => {
  //     eprintln!("unable to read bytes from stdin (reason: {})", e);
  //     std::process::exit(1);
  //   }
  // };

  // match client.put(buf, priority, delay, ttr) {
  //   Ok(id) => eprintln!("{}", id),
  //   Err(e) => {
  //     eprintln!("unable to put message (reason: {})", e);
  //     std::process::exit(1);
  //   }
  // }
}
