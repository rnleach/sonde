pub use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum SondeError {
    WidgetLoadError(&'static str),
    TextBufferLoadError(&'static str),
    CairoError(cairo::Status),
    NoMatchingFileType,
}

impl Display for SondeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use crate::SondeError::*;
        match self {
            WidgetLoadError(id) => write!(f, "Could not load widget with id = {}.", id),
            TextBufferLoadError(id) => {
                write!(f, "Could not load buffer for text area with id = {}.", id)
            }
            CairoError(status) => write!(f, "Error with cairo = {:?}.", status),
            NoMatchingFileType => write!(f, "Unable to find a way to load this file."),
        }
    }
}

impl From<cairo::Status> for SondeError {
    fn from(status: cairo::Status) -> Self {
        SondeError::CairoError(status)
    }
}

impl Error for SondeError {}
