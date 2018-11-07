pub use std::error::Error;
use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum SondeError {
    WidgetLoadError(&'static str),
    TextBufferLoadError(&'static str),
    LogError(&'static str),
}

impl Display for SondeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use SondeError::*;
        match self {
            WidgetLoadError(id) => write!(f, "Could not load widget with id = {}.", id),
            TextBufferLoadError(id) => write!(f, "Could not load buffer for text area with id = {}.", id),
            LogError(msg) => write!(f, "Error with logger = {}.", msg),
        }
    }
}

impl Error for SondeError {}