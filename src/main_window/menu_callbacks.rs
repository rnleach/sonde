
use gtk::MenuItem;

use super::super::data_context::DataContextPointer;

pub fn open_callback(mi: &MenuItem, dc: &DataContextPointer) {
    // TODO: add code to open a file dialog and load a file.
    println!("Open callback triggered.");
}
