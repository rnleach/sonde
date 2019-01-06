use std::path::PathBuf;

use cairo;
use gtk::{
    DialogExt, DialogExtManual, FileChooserAction, FileChooserDialog, FileChooserExt, FileFilter,
    FileFilterExt, MenuItem, MessageDialog, ResponseType, WidgetExt, Window,
};

use sounding_bufkit::BufkitFile;

use crate::app::AppContextPointer;
use crate::coords::DeviceRect;
use crate::errors::*;
use crate::gui::{plot_context::PlotContext, utility::DrawingArgs, Drawable};

pub fn open_callback(_mi: &MenuItem, ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Open File"), Some(win), FileChooserAction::Open);

    dialog.add_buttons(&[
        ("Open", ResponseType::Ok.into()),
        ("Cancel", ResponseType::Cancel.into()),
    ]);

    let filter = FileFilter::new();
    filter.add_pattern("*.buf");
    filter.set_name("Bufkit files (*.buf)");
    dialog.add_filter(&filter);

    if ResponseType::from(dialog.run()) == ResponseType::Ok {
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

fn load_file(path: &PathBuf, ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let file = BufkitFile::load(path)?;
    let data = file.data()?;

    ac.load_data(&mut data.into_iter());

    if let Some(name) = path.file_name() {
        let mut src_name = "File: ".to_owned();
        src_name.push_str(&name.to_string_lossy());
        ac.set_source_description(Some(src_name));
    }

    Ok(())
}

pub fn save_image_callback(_mi: &MenuItem, ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Save Image"), Some(win), FileChooserAction::Save);

    dialog.add_buttons(&[
        ("Save", ResponseType::Ok.into()),
        ("Cancel", ResponseType::Cancel.into()),
    ]);

    let filter = FileFilter::new();
    filter.add_pattern("*.png");
    filter.set_name("PNG files (*.png)");
    dialog.add_filter(&filter);

    if let Some(mut src_desc) = ac.get_source_description() {
        src_desc
            .retain(|c| c == '.' || c.is_alphabetic() || c.is_digit(10) || c == '_' || c == ' ');
        let src_desc = src_desc.replace(" ", "_");
        let mut src_desc = src_desc.trim_end_matches(".buf").to_string();
        src_desc.push_str("_skewt");
        if let Some(anal) = ac.get_sounding_for_display() {
            if let Some(lt) = anal.sounding().get_lead_time().into_option() {
                src_desc.push_str(&format!("_f{:03}", lt));
            }
        }
        src_desc.push_str(".png");

        dialog.set_current_name(src_desc);
    }

    if ResponseType::from(dialog.run()) == ResponseType::Ok {
        if let Some(mut filename) = dialog.get_filename() {
            filename.set_extension("png");
            if let Err(err) = save_image(&filename, ac) {
                show_error_dialog(&format!("Error saving image: {}", err), win);
            }
        } else {
            show_error_dialog("Could not retrieve file name from dialog.", win);
        }
    }

    dialog.destroy();
}

fn save_image(path: &PathBuf, ac: &AppContextPointer) -> Result<(), Box<dyn Error>> {
    let DeviceRect { width, height, .. } = ac.skew_t.get_device_rect();

    let img = cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32)
        .map_err(SondeError::from)?;

    let cr = &cairo::Context::new(&img);
    cr.transform(ac.skew_t.get_matrix());

    let args = DrawingArgs::new(ac, cr);

    ac.skew_t.draw_background(args);
    ac.skew_t.draw_data_and_legend(args);

    let mut file = std::fs::File::create(path)?;
    img.write_to_png(&mut file)?;

    Ok(())
}

fn show_error_dialog(message: &str, win: &Window) {
    use gtk::{ButtonsType, DialogFlags, MessageType};
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
