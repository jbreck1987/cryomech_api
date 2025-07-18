pub mod api;
mod packet;

pub use api::{CryomechApiSmdpBuilder, SmdpVersion};
use smdp;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    InvalidFormat(String),
    #[error("{0}")]
    Smdp(String),
}
impl Error {
    // Small helper to propagate IO errors to caller
    fn propagate_smdp_io(e: smdp::Error) -> Self {
        if e.is_io() {
            return Self::Io(e.to_string());
        } else {
            return Self::Smdp(e.to_string());
        }
    }
}
pub(crate) type CResult<T> = Result<T, Error>;
