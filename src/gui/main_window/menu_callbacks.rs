use crate::{
    app::AppContextPointer,
    coords::DeviceRect,
    errors::*,
    gui::{plot_context::PlotContext, utility::DrawingArgs, Drawable},
};
use cairo;
use gtk::{
    DialogExt, DialogExtManual, FileChooserAction, FileChooserDialog, FileChooserExt, FileFilter,
    MenuItem, MessageDialog, ResponseType, WidgetExt, Window,
};
use std::path::PathBuf;

mod load_file;

pub fn open_callback(mi: &MenuItem, ac: &AppContextPointer, win: &Window) {
    open_files(mi, ac, win, false);
}

pub fn open_many_callback(mi: &MenuItem, ac: &AppContextPointer, win: &Window) {
    open_files(mi, ac, win, true);
}

fn open_files(_mi: &MenuItem, ac: &AppContextPointer, win: &Window, open_multiple: bool) {
    let dialog = FileChooserDialog::new(Some("Open File"), Some(win), FileChooserAction::Open);

    dialog.add_buttons(&[
        ("Open", ResponseType::Ok.into()),
        ("Cancel", ResponseType::Cancel.into()),
    ]);

    dialog.set_select_multiple(open_multiple);

    let filter_data = [
        ("*.buf", "Bufkit files (*.buf)"),
        ("*.bufr", "Bufr files (*.bufr)"),
    ];

    // A filter for all supported file types
    let filter = FileFilter::new();
    for &(pattern, _) in &filter_data {
        filter.add_pattern(pattern);
    }
    filter.set_name(Some("All Supported"));
    dialog.add_filter(&filter);

    // Add a filter for each supported type individually
    for &(pattern, name) in &filter_data {
        let filter = FileFilter::new();
        filter.add_pattern(pattern);
        filter.set_name(Some(name));
        dialog.add_filter(&filter);
    }

    // Add a (not) filter that lets anything through
    let filter = FileFilter::new();
    filter.add_pattern("*");
    filter.set_name(Some("All Files"));
    dialog.add_filter(&filter);

    if ResponseType::from(dialog.run()) == ResponseType::Ok {
        if open_multiple {
            let paths: Vec<_> = dialog
                .get_filenames()
                .into_iter()
                .filter(|pb| pb.is_file())
                .collect();
            if let Err(ref err) = load_file::load_multiple(&paths, ac) {
                show_error_dialog(&format!("Error loading file: {}", err), win);
            }
        } else {
            if let Some(filename) = dialog.get_filename() {
                if let Err(ref err) = load_file::load_file(&filename, ac) {
                    show_error_dialog(&format!("Error loading file: {}", err), win);
                }
            } else {
                show_error_dialog("Could not retrieve file name from dialog.", win);
            }
        }
    }

    dialog.destroy();
}

pub fn save_image_callback(_mi: &MenuItem, ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Save Image"), Some(win), FileChooserAction::Save);

    dialog.add_buttons(&[
        ("Save", ResponseType::Ok.into()),
        ("Cancel", ResponseType::Cancel.into()),
    ]);

    let filter = FileFilter::new();
    filter.add_pattern("*.png");
    filter.set_name(Some("PNG files (*.png)"));
    dialog.add_filter(&filter);

    if let Some(anal) = ac.get_sounding_for_display() {
        if let Some(mut src_desc) = anal
            .borrow()
            .sounding()
            .source_description()
            .map(|desc| desc.to_owned())
        {
            src_desc.retain(|c| {
                c == '.' || c.is_alphabetic() || c.is_digit(10) || c == '_' || c == ' '
            });
            let src_desc = src_desc.replace(" ", "_");
            let mut src_desc = src_desc.trim_end_matches(".buf").to_string();
            src_desc.push_str("_skewt");
            if let Some(anal) = ac.get_sounding_for_display() {
                if let Some(lt) = anal.borrow().sounding().lead_time().into_option() {
                    src_desc.push_str(&format!("_f{:03}", lt));
                }
            }
            src_desc.push_str(".png");

            dialog.set_current_name(src_desc);
        }
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
