
use gtk::{TextView, ScrollablePolicy, TextTag};
use gtk::prelude::*;

use app::config;

use sounding_base::Sounding;

use app::AppContextPointer;

macro_rules! make_default_tag {
    ($tb:ident) => {
        if let Some(tag_table) = $tb.get_tag_table(){
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

pub fn set_up_text_area(text_area: &TextView, _acp: &AppContextPointer) {

    text_area.set_hexpand(true);
    text_area.set_vexpand(true);
    text_area.set_editable(false);
    text_area.set_property_margin(config::WIDGET_MARGIN);
    text_area.set_vscroll_policy(ScrollablePolicy::Minimum);
    text_area.set_hscroll_policy(ScrollablePolicy::Minimum);

    if let Some(tb) = text_area.get_buffer(){
        make_default_tag!(tb);
        set_text!(tb, "No data loaded");
    }
}

pub fn update_text_area(text_area: &TextView, snd: Option<&Sounding>) {
    use app::config;

    macro_rules! unwrap_to_str {
        ($opt_val:expr, $fmt:expr) => {
            if $opt_val.as_option().is_some() {
                format!($fmt, $opt_val.unwrap())
            } else {
                "".to_owned()
            }
        };
        ($opt_val:expr, $fmt:expr, $multiplier:expr) => {
            if $opt_val.as_option().is_some() {
                format!($fmt, $opt_val.unwrap() * $multiplier)
            } else {
                "".to_owned()
            }
        };
    }

    if let Some(tb) = text_area.get_buffer() {
        if let Some(snd) = snd {
            let mut text = String::with_capacity(4096);

            for row in snd.top_down() {
                if row.pressure.unwrap() < config::MINP {
                    continue;
                }
                text.push_str(&format!(
                    "{:>4} {:>5} {:>5} {:>5} {:>5} {:>6} {:>4} {:>4} {:>5} {:>4}\n",
                    unwrap_to_str!(row.pressure, "{:.0}"),
                    unwrap_to_str!(row.height, "{:.0}"),
                    unwrap_to_str!(row.temperature, "{:.1}"),
                    unwrap_to_str!(row.wet_bulb, "{:.1}"),
                    unwrap_to_str!(row.dew_point, "{:.1}"),
                    unwrap_to_str!(row.theta_e, "{:.0}"),
                    unwrap_to_str!(row.direction, "{:.0}"),
                    unwrap_to_str!(row.speed, "{:.0}"),
                    unwrap_to_str!(row.omega, "{:.1}", 10.0),
                    unwrap_to_str!(row.cloud_fraction, "{:.0}"),
                ));
            }

            // Get the scroll position before setting the text
            let old_adj;
            if let Some(adj) = text_area.get_vadjustment() {
                old_adj = Some(adj.get_value());
            } else {
                old_adj = None;
            }

            // tb.set_text(&text);
            set_text!(tb, &text);

            // I don't totally understand this, but after quite a lot of experimentation this works
            // well at keeping the scroll of the text view in the same area as you step through
            // time.
            if let Some(adj) = text_area.get_vadjustment() {
                if let Some(val) = old_adj {
                    let val = if val.round() < (adj.get_upper() - adj.get_page_size()).round() {
                        val.round()
                    } else {
                        (adj.get_upper() - adj.get_page_size() - 1.0).round()
                    };
                    adj.set_value(val);
                }
            }
        }
    }
}

pub fn make_header_text_area() -> TextView {
    let header = TextView::new();

    header.set_hexpand(true);
    header.set_vexpand(false);
    header.set_editable(false);
    header.set_property_margin(config::WIDGET_MARGIN);
    header.set_margin_bottom(0);
    header.set_hscroll_policy(ScrollablePolicy::Minimum);

    
    if let Some(tb) = header.get_buffer() {
        let mut text = String::with_capacity(512);

        text.push_str(&format!(
                "{:^4} {:^5} {:^5} {:^5} {:^5} {:^6} {:^4} {:^4} {:^5} {:^4}\n",
                "Pres",
                "Hgt",
                "T",
                "WB(C)",
                "DP(C)",
                "EPT(K)",
                "DIR",
                "SPD",
                "\u{03C9}",
                "CLD",
            ));
        text.push_str(&format!(
                "{:^4} {:^5} {:^5} {:^5} {:^5} {:^6} {:^4} {:^4} {:^5} {:^4}",
                "hPa",
                "m",
                "\u{00b0}C",
                "\u{00b0}C",
                "\u{00b0}C",
                "\u{00b0}K",
                "deg",
                "KT",
                "hPa/s",
                "%",
            ));

        make_default_tag!(tb);
        set_text!(tb, &text);
    }

    header
}
