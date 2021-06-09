mod cmd;
mod utils;
mod lib;

use std::{time::Duration, error::Error};

use beanstalkc::Beanstalkc;
use clap::{App, Arg};
use cmd::*;

#[derive(Debug)]
pub struct IOptions {
  pub host: String,
  pub port: u16,
  pub tube: String,
  pub timeout: Option<Duration>,
}

fn main() -> Result<(), Box<dyn Error>>{
  let app = App::new("bsc")
    .version("0.1.4")
    .about("A Beanstalkd client written in Rust")
    .arg(
      Arg::with_name("host")
        .long("host")
        .short("h")
        .env("HOST")
        .default_value("localhost")
        .help("Beanstalkd host"),
    )
    .arg(
      Arg::with_name("port")
        .long("port")
        .short("p")
        .env("PORT")
        .default_value("11300")
        .validator(utils::is_u16)
        .help("Beanstalkd port"),
    )
    .arg(
      Arg::with_name("tube")
        .short("t")
        .long("tube")
        .env("TUBE")
        .default_value("default")
        .help("Beanstalkd tube"),
    )
    .arg(
      Arg::with_name("timeout")
        .long("timeout")
        .env("TIMEOUT")
        .default_value("10")
        .help("Beanstalkd connection timeout"),
    )
    .subcommand(get::create())
    .subcommand(put::create())
    .get_matches();

  let opts = IOptions {
    host: app.value_of("host").unwrap().into(),
    // we can go crazy on unwraps because 'port' is validated by utils::is_u16
    port: app.value_of("port").unwrap().parse::<u16>().unwrap(),
    tube: app.value_of("tube").unwrap().into(),
    timeout: match app.value_of("timeout") {
      Some(t) => Some(Duration::from_secs(t.parse::<u64>()?)),
      None => None,
    }
  };

  let mut client = Beanstalkc::new()
    .host(&opts.host)
    .port(opts.port)
    .connection_timeout(opts.timeout)
    .connect()?;

  match app.subcommand() {
    ("get", Some(args)) => get::get(&mut client, &opts, args),
    ("put", Some(args)) => put::put(&mut client, &opts, args),
    _ => Ok(println!("{}", app.usage())),
  }
}
