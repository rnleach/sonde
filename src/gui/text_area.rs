use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{prelude::*, TextTag, TextView};
use metfor::{HectoPascal, Quantity};

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

pub fn set_up_text_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    const TEXT_AREA_ID: &str = "text_area";
    let text_area: TextView = acp.fetch_widget(TEXT_AREA_ID)?;

    fill_header_text_area(acp)?;

    if let Some(tb) = text_area.get_buffer() {
        make_default_tag!(tb);
        set_text!(tb, "No data loaded");

        if let Some(tag_table) = tb.get_tag_table() {
            let above_tag = TextTag::new("highlight_above");
            let below_tag = TextTag::new("highlight_below");

            let mut success = tag_table.add(&above_tag);
            debug_assert!(success, "Failed to add highlight_above tag");
            success = tag_table.add(&below_tag);
            debug_assert!(success, "Failed to add highlight_below tag");
        }

        tb.create_mark("scroll_mark", &tb.get_start_iter(), true);

        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(TEXT_AREA_ID))
    }
}

pub fn update_text_area(ac: &AppContext) {
    use crate::app::config;

    macro_rules! unwrap_to_str {
        ($opt_val:expr, $fmt:expr) => {
            if let Some(val) = $opt_val.into_option() {
                format!($fmt, val.unpack())
            } else {
                "".to_owned()
            }
        };
        ($opt_val:expr, $fmt:expr, $multiplier:expr) => {
            if let Some(val) = $opt_val {
                format!($fmt, val * $multiplier)
            } else {
                "".to_owned()
            }
        };
    }

    let text_area: TextView = if let Ok(ta) = ac.fetch_widget("text_area") {
        ta
    } else {
        return;
    };

    if !text_area.is_visible() {
        return;
    }

    if let Some(tb) = text_area.get_buffer() {
        if let Some(snd) = ac.get_sounding_for_display() {
            let mut text = String::with_capacity(4096);

            for row in snd.sounding().top_down() {
                if let Some(p) = row.pressure.into_option() {
                    if p < config::MINP {
                        continue;
                    }
                } else {
                    continue;
                }
                text.push_str(&format!(
                    " {:>4} {:>5} {:>5} {:>5} {:>5} {:^7}  {:>3}{:>4} {:>5}  {:>3}\n",
                    unwrap_to_str!(row.pressure, "{:.0}"),
                    unwrap_to_str!(row.height, "{:.0}"),
                    unwrap_to_str!(row.temperature, "{:.1}"),
                    unwrap_to_str!(row.wet_bulb, "{:.1}"),
                    unwrap_to_str!(row.dew_point, "{:.1}"),
                    unwrap_to_str!(row.theta_e, "{:.0}"),
                    unwrap_to_str!(row.wind.map_t(|wnd| wnd.direction), "{:.0}"),
                    unwrap_to_str!(row.wind.map_t(|wnd| wnd.speed), "{:.0}"),
                    unwrap_to_str!(row.pvv, "{:.1}"),
                    unwrap_to_str!(row.cloud_fraction, "{:.0}"),
                ));
            }

            // Get the scroll position before setting the text
            let old_adj = text_area.get_vadjustment().map(|adj| adj.get_value());

            set_text!(tb, &text);

            // I don't totally understand this, but after quite a lot of experimentation this works
            // well at keeping the scroll of the text view in the same area as you step through
            // time.
            if !ac.config.borrow().show_active_readout {
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
}

pub fn fill_header_text_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    const HEADER_ID: &str = "text_header";
    let header: TextView = acp.fetch_widget(HEADER_ID)?;

    if let Some(tb) = header.get_buffer() {
        let mut text = String::with_capacity(512);

        text.push_str(&format!(
            " {:^4} {:^5} {:^5} {:^5} {:^5} {:^7}  {:^3} {:^4} {:^5}  {:^3}\n",
            "Pres", "Hgt", "T", "WB", "DP", "Theta E", "DIR", "SPD", "\u{03C9}", "CLD",
        ));
        text.push_str(&format!(
            " {:^4} {:^5} {:^5} {:^5} {:^5} {:^7}  {:^3} {:^4} {:^5}  {:^3}",
            "hPa",
            "m",
            "\u{00b0}C",
            "\u{00b0}C",
            "\u{00b0}C",
            "\u{00b0}K",
            "deg",
            "KT",
            "Pa/s",
            "%",
        ));

        make_default_tag!(tb);
        set_text!(tb, &text);

        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(HEADER_ID))
    }
}

pub fn update_text_highlight(ac: &AppContext) {
    use std::str::FromStr;
    let config = ac.config.borrow();

    if !config.show_active_readout {
        return;
    }

    let text_area: TextView = if let Ok(ta) = ac.fetch_widget("text_area") {
        ta
    } else {
        return;
    };

    if !text_area.is_visible() {
        return;
    }

    let tp = if let Some(sample) = ac.get_sample() {
        if let Some(tp) = sample.pressure.into_option() {
            tp
        } else {
            return;
        }
    } else {
        return;
    };

    if let Some(tb) = text_area.get_buffer() {
        let start = tb.get_start_iter();
        let end = tb.get_end_iter();
        tb.remove_tag_by_name("highlight_above", &start, &end);
        tb.remove_tag_by_name("highlight_below", &start, &end);

        let lines = tb.get_line_count();
        for i in 0..(lines - 1) {
            let start_above = tb.get_iter_at_line(i);
            let mut end_above = start_above.clone();
            end_above.forward_chars(5);
            let above_val: HectoPascal = f64::from_str(
                tb.get_text(&start_above, &end_above, false)
                    .unwrap_or_else(|| "0.0".to_owned())
                    .trim(),
            )
            .map(HectoPascal)
            .unwrap_or(HectoPascal(0.0));

            let start_below = tb.get_iter_at_line(i + 1);
            let mut end_below = start_below.clone();
            end_below.forward_chars(5);
            let below_val: HectoPascal = f64::from_str(
                tb.get_text(&start_below, &end_below, false)
                    .unwrap_or_else(|| "0.0".to_owned())
                    .trim(),
            )
            .map(HectoPascal)
            .unwrap_or(HectoPascal(0.0));

            if tp > above_val && tp <= below_val {
                if let Some(tt) = tb.get_tag_table() {
                    // Set line colors
                    let rgba = config.active_readout_line_rgba;
                    let range = below_val - above_val;
                    let alpha_below = (tp - above_val) / range;
                    let alpha_above = 1.0 - alpha_below;
                    let rgba_below = ::gdk::RGBA {
                        red: rgba.0,
                        green: rgba.1,
                        blue: rgba.2,
                        alpha: alpha_below,
                    };
                    let rgba_above = ::gdk::RGBA {
                        red: rgba.0,
                        green: rgba.1,
                        blue: rgba.2,
                        alpha: alpha_above,
                    };
                    if let Some(below_tag) = tt.lookup("highlight_below") {
                        below_tag.set_property_background_rgba(Some(&rgba_below));
                        end_below.forward_line();
                        tb.apply_tag(&below_tag, &start_below, &end_below);
                    }
                    if let Some(above_tag) = tt.lookup("highlight_above") {
                        above_tag.set_property_background_rgba(Some(&rgba_above));
                        end_above.forward_line();
                        tb.apply_tag(&above_tag, &start_above, &end_above);
                    }

                    // Scroll the view to this point.
                    if let Some(ref mark) = tb.get_mark("scroll_mark") {
                        tb.move_mark(mark, &end_above);
                        text_area.scroll_to_mark(mark, 0.0, true, 0.0, 0.5);
                    }
                }
                break;
            }
        }
    }
}
