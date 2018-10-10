use gtk::prelude::*;
use gtk::{TextTag, TextView};

use app::{AppContext, AppContextPointer};
use errors::SondeError;

macro_rules! make_default_tag {
    ($tb:ident) => {
        if let Some(tag_table) = $tb.get_tag_table() {
            let tag = TextTag::new("default");

            tag.set_property_font(Some("courier bold 12"));

            let success = tag_table.add(&tag);
            debug_assert!(success, "Failed to add tag to text tag table");
        }
    };
}

macro_rules! set_text {
    ($tb:ident, $txt:expr) => {
        $tb.set_text($txt);
        let start = $tb.get_start_iter();
        let end = $tb.get_end_iter();
        $tb.apply_tag_by_name("default", &start, &end);
    };
}

const TEXT_AREA_ID: &str = "indexes_text_area";

pub fn set_up_indexes_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    let text_area: TextView = acp.fetch_widget(TEXT_AREA_ID)?;

    if let Some(tb) = text_area.get_buffer() {
        make_default_tag!(tb);
        set_text!(tb, "No data loaded");

        if let Some(tag_table) = tb.get_tag_table() {
            let red_tag = TextTag::new("red");
            let yellow_tag = TextTag::new("yellow");

            let mut success = tag_table.add(&red_tag);
            debug_assert!(success, "Failed to add red tag");
            success = tag_table.add(&yellow_tag);
            debug_assert!(success, "Failed to add yellow tag");
        }

        tb.create_mark("scroll_mark", &tb.get_start_iter(), true);

        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(TEXT_AREA_ID))
    }
}

macro_rules! push_profile_index {
    ($anal: expr, $buf:ident, $name:expr, $selector:expr, $format:expr, $empty_val:ident) => (
        $buf.push_str($name);
        $anal.get_profile_index($selector)
            .and_then(|val| {
                $buf.push_str(&format!($format, val));
                Some(())
            }).or_else(||{
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    )
}

pub fn update_indexes_area(ac: &AppContext) {
    use sounding_analysis::ProfileIndex::*;

    let text_area: TextView = if let Ok(ta) = ac.fetch_widget(TEXT_AREA_ID) {
        ta
    } else {
        return;
    };

    if !text_area.is_visible() {
        return;
    }

    if let Some(tb) = text_area.get_buffer() {
        if let Some(anal) = ac.get_sounding_for_display() {
            let mut text = String::with_capacity(4096);
            let empty_val = "    -    ";

            text.push_str("Convection\n");

            push_profile_index!(anal, text, "SWeT        ", SWeT, "{:>9.0}", empty_val);
            push_profile_index!(anal, text, "K           ", K, "{:>9.0}", empty_val);
            push_profile_index!(anal, text, "Total Totals", TotalTotals, "{:>9.0}", empty_val);
            push_profile_index!(anal, text, "DCAPE       ", DCAPE, "{:>9.0}", empty_val);
            push_profile_index!(anal, text, "Downrush T  ", DownrushT, "{:>6.0} \u{00b0}C", empty_val);
            push_profile_index!(anal, text, "PWAT        ", PWAT, "{:>6.0} mm", empty_val);
            

            text.push_str("\nFire\n");
            push_profile_index!(anal, text, "Haines      ", Haines, "{:>9.0}", empty_val);
            
            set_text!(tb, &text);
        }
    }
}
