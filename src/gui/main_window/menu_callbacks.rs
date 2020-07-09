use crate::{
    app::AppContextPointer,
    coords::DeviceRect,
    errors::*,
    gui::{plot_context::PlotContext, utility::DrawingArgs, Drawable},
};
use gtk::{
    prelude::DialogExtManual, DialogExt, FileChooserAction, FileChooserDialog, FileChooserExt,
    FileFilter, GtkWindowExt, MessageDialog, ResponseType, Widget, WidgetExt, Window,
};
use std::path::PathBuf;

mod load_file;

pub fn open_toolbar_callback(ac: &AppContextPointer, win: &Window) {
    open_files(ac, win);
}

fn open_files(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Open File"), Some(win), FileChooserAction::Open);

    dialog.add_buttons(&[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);
    dialog.set_select_multiple(true);

    if let Some(ref fname) = ac.config.borrow().last_open_file {
        dialog.set_filename(fname);
    }

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

    if dialog.run() == ResponseType::Ok {
        let paths: Vec<_> = dialog
            .get_filenames()
            .into_iter()
            .filter(|pb| pb.is_file())
            .collect();

        // Remember the last opened file in the config.
        if let Some(ref f0) = paths.get(0) {
            ac.config.borrow_mut().last_open_file = Some(PathBuf::from(f0));
        }

        if let Err(ref err) = load_file::load_multiple(&paths, ac) {
            show_error_dialog(&format!("Error loading file: {}", err), win);
        } else {
            let da: Widget = ac.fetch_widget("skew_t").unwrap();
            da.grab_focus();
        }
    }

    dialog.close();
}

pub fn save_image_callback(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Save Image"), Some(win), FileChooserAction::Save);

    dialog.add_buttons(&[("Save", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

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

    if dialog.run() == ResponseType::Ok {
        if let Some(mut filename) = dialog.get_filename() {
            filename.set_extension("png");
            if let Err(err) = save_image(&filename, ac) {
                show_error_dialog(&format!("Error saving image: {}", err), win);
            }
        } else {
            show_error_dialog("Could not retrieve file name from dialog.", win);
        }
    }

    dialog.close();
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
    dialog.close();
}
