
use std::path::PathBuf;

use gtk::{DialogExt, DialogExtManual, FileChooserAction, FileChooserDialog, FileChooserExt,
          FileFilter, FileFilterExt, MenuItem, ResponseType, WidgetExt, Window};

use sounding_bufkit::BufkitFile;

use app::AppContextPointer;
use errors::*;

pub fn open_callback(_mi: &MenuItem, ac: &AppContextPointer, win: &Window) {

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
            if load_file(&filename, ac).is_ok() {
                let mut ac = ac.borrow_mut();
                ac.fit_to_data();
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

fn load_file(path: &PathBuf, ac: &AppContextPointer) -> Result<()> {
    let mut ac = ac.borrow_mut();

    let file = BufkitFile::load(path).chain_err(|| {
        format!("Error loading file {:?}", path)
    })?;
    let data = file.data()?;

    ac.load_data(&mut data.into_iter())?;

    Ok(())
}
