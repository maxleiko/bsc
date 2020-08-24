use crate::utils;
use crate::IOptions;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::time::Duration;

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

pub fn call(opts: &IOptions, cmd: &ArgMatches) {
  let timeout = cmd.value_of("timeout").unwrap().parse::<u64>().unwrap();
  let mut client = utils::beanstalkd_connect(opts);
  if !opts.tube.eq("default") {
    client.watch(&opts.tube).unwrap();
    client.ignore("default").unwrap();
  }
  let result;
  if timeout == 0 {
    eprintln!("reserve on '{}' (no timeout)", opts.tube);
    result = client.reserve();
  } else {
    eprintln!("reserve on '{}' (timeout: {})", opts.tube, timeout);
    result = client.reserve_with_timeout(Duration::from_secs(timeout));
  }
  match result {
    Ok(mut job) => {
      println!("{}", job.body());
      if cmd.is_present("delete") {
        job.delete().unwrap();
      } else {
        job.release_default().unwrap();
      }
    }
    Err(e) => eprintln!("{}", e),
  }
}
