use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{glib::translate::IntoGlib, prelude::*, EventControllerKey, Inhibit, TextTag, TextView};
use std::{fmt::Write, rc::Rc};

const TEXT_AREA_ID: &str = "provider_data_text";

macro_rules! make_default_tag {
    ($tb:ident, $acp:ident) => {
        let tag_table = $tb.tag_table();
        let config = $acp.config.borrow();
        let font = &config.font_name;
        let font_size = config.text_area_font_size_points;

        let tag = TextTag::builder()
            .name("default")
            .family(font)
            .size_points(font_size)
            .weight(gtk::pango::Weight::Bold.into_glib())
            .build();

        let success = tag_table.add(&tag);
        debug_assert!(success, "Failed to add tag to text tag table");
    };
}

macro_rules! set_text {
    ($tb:ident, $txt:expr) => {
        $tb.set_text($txt);
        let start = $tb.start_iter();
        let end = $tb.end_iter();
        $tb.apply_tag_by_name("default", &start, &end);
    };
}

pub fn set_up_provider_text_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    use gtk::gdk::Key;

    let text_area: TextView = acp.fetch_widget(TEXT_AREA_ID)?;

    let key_press = EventControllerKey::new();
    let ac = Rc::clone(acp);
    key_press.connect_key_pressed(move |_key_press, key, _code, _key_modifier| {
        if key == Key::KP_Right || key == Key::Right {
            ac.display_next();
        } else if key == Key::KP_Left || key == Key::Left {
            ac.display_previous();
        }
        Inhibit(true)
    });
    text_area.add_controller(key_press);

    let tb = text_area.buffer();
    make_default_tag!(tb, acp);
    set_text!(tb, "No data loaded");

    Ok(())
}

pub fn update_text_area(ac: &AppContext) {
    let text_area: TextView = if let Ok(ta) = ac.fetch_widget(TEXT_AREA_ID) {
        ta
    } else {
        return;
    };

    if let Some(anal) = ac.get_sounding_for_display() {
        let tb = text_area.buffer();
        let anal = anal.borrow();
        let mut text = String::with_capacity(4096);

        let mut provider_data = anal.provider_analysis().iter().collect::<Vec<(_, _)>>();
        provider_data.sort_by_key(|kv| kv.0);

        for (k, v) in provider_data.into_iter() {
            writeln!(text, "{:-35} : {:9.3}", k, v).unwrap();
        }

        set_text!(tb, &text);
    }
}
