use beanstalkc::Beanstalkc;
use std::time::Duration;
use crate::IOptions;

// FIXME change is_XXX to macros

pub fn is_u16(v: String) -> Result<(), String> {
  match v.parse::<u16>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}

pub fn is_u64(v: String) -> Result<(), String> {
  match v.parse::<u64>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}

pub fn is_u32(v: String) -> Result<(), String> {
  match v.parse::<u32>() {
    Ok(_) => Ok(()),
    Err(e) => Err(format!("{}", e)),
  }
}

pub fn beanstalkd_connect(opts: &IOptions) -> beanstalkc::Beanstalkc {
  let client = Beanstalkc::new()
    .host(&opts.host)
    .port(opts.port)
    .connection_timeout(Some(Duration::from_secs(5))); // fixme define that in options ?

  match client.connect() {
    Ok(client) => client,
    Err(e) => {
      eprintln!("Unable to connect to beanstalkd: {}", e);
      std::process::exit(1);
    }
  }
}
