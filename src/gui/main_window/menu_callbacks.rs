use crate::{
    app::AppContextPointer,
    coords::DeviceRect,
    errors::*,
    gui::{plot_context::PlotContext, utility::DrawingArgs, Drawable},
};
use gtk::{
    gio, prelude::*, FileChooserAction, FileChooserDialog, FileFilter, MessageDialog, ResponseType,
    Widget, Window,
};
use std::path::PathBuf;

pub fn open_toolbar_callback(ac: &AppContextPointer, win: &Window) {
    open_files(ac, win);
}

fn open_files(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(
        Some("Open File"),
        Some(win),
        FileChooserAction::Open,
        &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
    );

    dialog.set_select_multiple(true);
    dialog.set_modal(true);

    if let Some(ref fname) = ac.config.borrow().last_open_file {
        dialog.set_file(&gio::File::for_path(fname)).ok();
    }

    let filter_data = [
        ("*.buf", "Bufkit files (*.buf)"),
        ("*.bufr", "Bufr files (*.bufr)"),
        ("*.html", "U of WY HTML(*.html)"),
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

    let ac = ac.clone();
    let win = win.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Ok {
            let paths: Vec<_> = dialog
                .files()
                .into_iter()
                .filter_map(|pb| pb.ok())
                .filter_map(|pb| pb.downcast::<gio::File>().ok())
                .filter_map(|pb| pb.path())
                .filter(|pb| pb.is_file())
                .collect();

            // Remember the last opened file in the config.
            if let Some(ref f0) = paths.get(0) {
                ac.config.borrow_mut().last_open_file = Some(PathBuf::from(f0));
            }

            if let Err(ref err) = crate::app::load_file::load_multiple(&paths, &ac) {
                show_error_dialog(&format!("Error loading file: {}", err), &win);
            } else {
                let da: Widget = ac.fetch_widget("skew_t").unwrap();
                da.grab_focus();
            }
        }

        match response {
            ResponseType::DeleteEvent => {}
            x => dialog.close(),
        }
    });

    dialog.show();
}

// FIXME
/*
pub fn save_image_callback(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Save Image"), Some(win), FileChooserAction::Save, &[("Save", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

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

            dialog.set_current_name(&src_desc);
        }
    }

    if dialog.run() == ResponseType::Ok {
        if let Some(mut filename) = dialog.filename() {
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

    let img =
        gtk::cairo::ImageSurface::create(gtk::cairo::Format::ARgb32, width as i32, height as i32)
            .map_err(SondeError::from)?;

    let cr = &gtk::cairo::Context::new(&img).unwrap();
    cr.transform(ac.skew_t.get_matrix());

    let args = DrawingArgs::new(ac, cr);

    ac.skew_t.draw_background(args);
    ac.skew_t.draw_data_and_legend(args);

    let mut file = std::fs::File::create(path)?;
    img.write_to_png(&mut file)?;

    Ok(())
}
*/

fn show_error_dialog(message: &str, win: &Window) {
    use gtk::{ButtonsType, DialogFlags, MessageType};
    let dialog = MessageDialog::new(
        Some(win),
        DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Error,
        ButtonsType::Ok,
        message,
    );

    dialog.connect_response(|dialog, _response| dialog.close());

    dialog.show();
}

// FIXME
/*
pub fn save_theme(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(
        Some("Save Current Them"),
        Some(win),
        FileChooserAction::Save,
    );

    dialog.add_buttons(&[("Save", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

    let filter = FileFilter::new();
    filter.add_pattern("*.yml");
    filter.set_name(Some("Yaml config files(*.yml)"));
    dialog.add_filter(&filter);

    if dialog.run() == ResponseType::Ok {
        if let Some(mut filename) = dialog.filename() {
            filename.set_extension("yml");
            if let Err(err) = crate::save_config_with_file_name(ac, &filename) {
                show_error_dialog(&format!("Error saving theme: {}", err), win);
            }
        } else {
            show_error_dialog("Could not retrieve file name from dialog.", win);
        }
    }

    dialog.close();
}

pub fn load_theme(ac: &AppContextPointer, win: &Window) {
    let dialog = FileChooserDialog::new(Some("Load Theme"), Some(win), FileChooserAction::Open);

    dialog.add_buttons(&[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]);

    let filter_data = [("*.yml", "Yaml files(*.yml)")];

    // A filter for all supported file types
    let filter = FileFilter::new();
    for &(pattern, _) in &filter_data {
        filter.add_pattern(pattern);
    }
    filter.set_name(Some("All Supported"));
    dialog.add_filter(&filter);

    // Add a (not) filter that lets anything through
    let filter = FileFilter::new();
    filter.add_pattern("*");
    filter.set_name(Some("All Files"));
    dialog.add_filter(&filter);

    if dialog.run() == ResponseType::Ok {
        let path: Option<_> = dialog.filename().into_iter().find(|pb| pb.is_file());

        if let Some(ref f0) = path {
            match crate::load_config_from_file(ac, f0) {
                Ok(()) => {}
                Err(err) => show_error_dialog(
                    &format!("Error loading theme {}: {}", f0.to_string_lossy(), err),
                    win,
                ),
            }
        }
    }

    dialog.close();
}

pub fn load_default_theme(ac: &AppContextPointer, _win: &Window) {
    *ac.config.borrow_mut() = crate::app::config::Config::default();

    ac.mark_background_dirty();
    ac.mark_data_dirty();
    ac.mark_data_dirty();
}
*/
