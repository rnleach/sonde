use cairo;
pub use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum SondeError {
    WidgetLoadError(&'static str),
    TextBufferLoadError(&'static str),
    LogError(&'static str),
    CairoError(cairo::Status),
}

impl Display for SondeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use crate::SondeError::*;
        match self {
            WidgetLoadError(id) => write!(f, "Could not load widget with id = {}.", id),
            TextBufferLoadError(id) => {
                write!(f, "Could not load buffer for text area with id = {}.", id)
            }
            LogError(msg) => write!(f, "Error with logger = {}.", msg),
            CairoError(status) => write!(f, "Error with cairo = {:?}.", status),
        }
    }
}

impl From<cairo::Status> for SondeError {
    fn from(status: cairo::Status) -> Self {
        SondeError::CairoError(status)
    }
}

impl Error for SondeError {}
