mod beanstalk;
mod error;
mod stats;

pub use error::*;
pub use beanstalk::*;
pub use stats::*;

pub(crate) type Result<T, E = crate::Error> = std::result::Result<T, E>;