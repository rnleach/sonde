
use std::path::PathBuf;

use gtk::{DialogExt, DialogExtManual, FileChooserAction, FileChooserDialog, FileChooserExt,
          FileFilter, FileFilterExt, MenuItem, MessageDialog, ResponseType, WidgetExt, Window};

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
            if let Err(ref err) = load_file(&filename, ac) {
                show_error_dialog(&format!("Error loading file: {}", err), win);
            }
        } else {
            show_error_dialog("Could not retrieve file name from dialog.", win);
        }
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

    if let Some(name) = path.file_name() {
        let mut src_name = "File: ".to_owned();
        src_name.push_str(&name.to_string_lossy());
        ac.set_source_description(Some(src_name));
    }

    Ok(())
}

fn show_error_dialog(message: &str, win: &Window) {
    use gtk::{MessageType, ButtonsType, DialogFlags};
    let dialog = MessageDialog::new(
        Some(win),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Error,
        ButtonsType::Ok,
        message,
    );
    dialog.run();
    dialog.destroy();
}
