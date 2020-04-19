use super::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt::{self, Display},
};

pub type Response = Result<Option<HashSet<Snapshot>>, Error>;

#[derive(Deserialize, Serialize)]
pub enum Error {
    NameExists,
    NameNotExists,
    InvalidDuration,
    Generic { message: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NameExists => {
                write!(f, "Timer with this name already exists")
            }
            Self::NameNotExists => {
                write!(f, "Timer with this name does not exist yet")
            }
            Self::InvalidDuration => {
                write!(f, "Provided duration is zero or bigger than max u64")
            }
            Self::Generic { message } => write!(f, "{}", message),
        }
    }
}
