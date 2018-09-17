pub use failure::Error;

#[derive(Clone, Copy, Debug, Fail)]
pub enum SondeError {
    #[fail(display = "Could not load widget with id = {}.", _0)]
    WidgetLoadError(&'static str),
    #[fail(
        display = "Could not load buffer for text area with id = {}.",
        _0
    )]
    TextBufferLoadError(&'static str),
    #[fail(display = "Error with logger = {}.", _0)]
    LogError(&'static str),
}
