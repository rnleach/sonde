pub use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum SondeError {
    WidgetLoadError(&'static str),
    TextBufferLoadError(&'static str),
    CairoError(gtk::cairo::Error),
    GLibBoolError(gtk::glib::error::BoolError),
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
            CairoError(err) => write!(f, "Error with cairo = {:?}.", err),
            GLibBoolError(err) => write!(f, "Error with glib = {:?}.", err),
            NoMatchingFileType => write!(f, "Unable to find a way to load this file."),
        }
    }
}

impl From<gtk::cairo::Error> for SondeError {
    fn from(err: gtk::cairo::Error) -> Self {
        SondeError::CairoError(err)
    }
}

impl From<gtk::glib::error::BoolError> for SondeError {
    fn from(err: gtk::glib::error::BoolError) -> Self {
        SondeError::GLibBoolError(err)
    }
}

impl Error for SondeError {}
