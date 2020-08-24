extern crate beanstalkc;
extern crate clap;

use clap::{App, Arg};
mod cmd;
mod utils;

#[derive(Debug)]
pub struct IOptions {
  pub host: String,
  pub port: u16,
  pub tube: String,
}

fn main() {
  let app = App::new("bsc")
    .version("0.1.0")
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
    .subcommand(cmd::get::create())
    .subcommand(cmd::put::create())
    .get_matches();

  let opts = IOptions {
    host: app.value_of("host").unwrap().into(),
    // we can go crazy on unwraps because 'port' is validated by utils::is_u16
    port: app.value_of("port").unwrap().parse::<u16>().unwrap(),
    tube: app.value_of("tube").unwrap().into(),
  };

  match app.subcommand() {
    ("get", Some(args)) => cmd::get::call(&opts, args),
    ("put", Some(args)) => cmd::put::call(&opts, args),
    _ => println!("{}", app.usage()),
  }
}
