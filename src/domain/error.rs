use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::sync;

/// Wraps errors so the lib exposes a common error type.
#[derive(Debug)]
pub enum GuestlistError {
    Io(io::Error),
    Poison(GuestlistPoisonError),
}

#[derive(Debug)]
pub struct GuestlistPoisonError;

impl fmt::Display for GuestlistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::GuestlistError::*;
        match *self {
            Io(ref err) => write!(f, "IO error: {}", err),
            Poison(ref err) => write!(f, "{}", err),
        }
    }
}

impl fmt::Display for GuestlistPoisonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "GuestlistPoisonError: {}", self.description())
    }
}

impl error::Error for GuestlistError {
    fn description(&self) -> &str {
        use self::GuestlistError::*;
        match *self {
            Io(ref err) => err.description(),
            Poison(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::GuestlistError::*;
        match *self {
            Io(ref err) => Some(err),
            Poison(ref err) => Some(err),
        }
    }
}

impl error::Error for GuestlistPoisonError {
    fn description(&self) -> &str {
        "One of the threads accessing the list of nodes panicked"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<io::Error> for GuestlistError {
    fn from(error: io::Error) -> Self {
        GuestlistError::Io(error)
    }
}

impl<T> From<sync::PoisonError<T>> for GuestlistError {
    fn from(_error: sync::PoisonError<T>) -> Self {
        GuestlistError::Poison(GuestlistPoisonError)
    }
}
