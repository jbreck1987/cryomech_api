pub mod api;
mod packet;

pub use api::{CryomechApiSmdpBuilder, SmdpVersion};
use smdp;

use serialport;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidFormat(String),
    #[error(transparent)]
    Smdp(#[from] smdp::Error),
    #[error(transparent)]
    Serial(#[from] serialport::Error),
}
pub(crate) type CResult<T> = Result<T, Error>;
