use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{prelude::*, TextTag, TextView};
use std::{fmt::Write, rc::Rc};

const TEXT_AREA_ID: &str = "provider_data_text";

macro_rules! make_default_tag {
    ($tb:ident) => {
        if let Some(tag_table) = $tb.tag_table() {
            let tag = TextTag::new(Some("default"));

            tag.set_font(Some("courier bold 12"));

            let success = tag_table.add(&tag);
            debug_assert!(success, "Failed to add tag to text tag table");
        }
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
    use gdk::keys::constants::{KP_Left, KP_Right, Left, Right};

    let text_area: TextView = acp.fetch_widget(TEXT_AREA_ID)?;

    let ac1 = Rc::clone(acp);
    text_area.connect_key_press_event(move |_ta, event| {
        let keyval = event.keyval();
        if keyval == KP_Right || keyval == Right {
            ac1.display_next();
        } else if keyval == KP_Left || keyval == Left {
            ac1.display_previous();
        }
        Inhibit(true)
    });

    if let Some(tb) = text_area.buffer() {
        make_default_tag!(tb);
        set_text!(tb, "No data loaded");

        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(TEXT_AREA_ID))
    }
}

pub fn update_text_area(ac: &AppContext) {
    let text_area: TextView = if let Ok(ta) = ac.fetch_widget(TEXT_AREA_ID) {
        ta
    } else {
        return;
    };

    if let (Some(tb), Some(anal)) = (text_area.buffer(), ac.get_sounding_for_display()) {
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
