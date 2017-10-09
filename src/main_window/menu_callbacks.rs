
use std::path::PathBuf;

use gtk::{DialogExt, DialogExtManual, FileChooserAction, FileChooserDialog, FileChooserExt,
          FileFilter, FileFilterExt, MenuItem, ResponseType, WidgetExt, Window};

use sounding_bufkit::BufkitFile;

use data_context::DataContextPointer;
use sounding::sounding_context::SoundingContextPointer;
use errors::*;

pub fn open_callback(
    _mi: &MenuItem,
    dc: &DataContextPointer,
    win: &Window,
    sc: &SoundingContextPointer,
) {

    let dialog = FileChooserDialog::new(Some("Open File"), Some(win), FileChooserAction::Open);

    dialog.add_buttons(
        &[
            ("Open", ResponseType::Ok.into()),
            ("Cancel", ResponseType::Cancel.into()),
        ],
    );

    let filter = FileFilter::new();
    filter.add_pattern("*.buf");
    filter.set_name("Bufkit files (*.buf)");
    dialog.add_filter(&filter);

    if dialog.run() == ResponseType::Ok.into() {

        if let Some(filename) = dialog.get_filename() {
            if load_file(&filename, dc).is_ok() {
                let mut sc = sc.borrow_mut();
                let dc = dc.borrow();
                sc.fit_to(dc.get_lower_left(), dc.get_upper_right());
            } else {
                // TODO: Show error dialog
            }

        } else {
            // TODO: Show error dialog
        }
    } else {
        // TODO: Show error dialog
    }

    dialog.destroy();
}

fn load_file(path: &PathBuf, dc: &DataContextPointer) -> Result<()> {
    let mut dc = dc.borrow_mut();

    let file = BufkitFile::load(path).chain_err(|| {
        format!("Error loading file {:?}", path)
    })?;
    let data = file.data()?;

    dc.load_data(&mut data.into_iter())?;

    Ok(())
}
