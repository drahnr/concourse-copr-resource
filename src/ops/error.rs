

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ResponseError {
    InvalidRequest,
    AuthentificationFailure,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResponseError::InvalidRequest => write!(f, "Invalid request"),
            ResponseError::AuthentificationFailure => write!(f, "Authentification failed"),
        }
    }
}

impl error::Error for ResponseError {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            ResponseError::InvalidRequest => "BadRequest/400",
            ResponseError::AuthentificationFailure => "Forbidden/403",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ResponseError::InvalidRequest => None,
            ResponseError::AuthentificationFailure => None,
        }
    }
}

#[derive(Debug)]
pub enum MiscError {
    X,
}

impl fmt::Display for MiscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MiscError::X => write!(f, "X"),
        }
    }
}

impl error::Error for MiscError {
    fn description(&self) -> &str {
        // Both underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            MiscError::X => "X",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            MiscError::X => None,
        }
    }
}
