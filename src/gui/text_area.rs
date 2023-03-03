use crate::{
    app::{sample::Sample, AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{glib::translate::IntoGlib, prelude::*, EventControllerKey, Inhibit, TextTag, TextView};
use metfor::{HectoPascal, Quantity};
use sounding_analysis::DataRow;
use std::{fmt::Write, rc::Rc};

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

pub fn set_up_text_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    use gtk::gdk::Key;

    const TEXT_AREA_ID: &str = "text_area";
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

    fill_header_text_area(acp)?;

    let tb = text_area.buffer();
    make_default_tag!(tb, acp);
    set_text!(tb, "No data loaded");

    let tag_table = tb.tag_table();
    let above_tag = TextTag::new(Some("highlight_above"));
    let below_tag = TextTag::new(Some("highlight_below"));

    let mut success = tag_table.add(&above_tag);
    debug_assert!(success, "Failed to add highlight_above tag");
    success = tag_table.add(&below_tag);
    debug_assert!(success, "Failed to add highlight_below tag");

    tb.create_mark(Some("scroll_mark"), &tb.start_iter(), true);

    Ok(())
}

pub fn update_text_area(ac: &AppContext) {
    use crate::app::config;

    let text_area: TextView = if let Ok(ta) = ac.fetch_widget("text_area") {
        ta
    } else {
        return;
    };

    if let Some(anal) = ac.get_sounding_for_display() {
        let tb = text_area.buffer();
        let anal = anal.borrow();
        let mut text = String::with_capacity(4096);

        anal.sounding()
            .top_down()
            .filter(|row| row.pressure.map(|p| p > config::MINP).unwrap_or(false))
            .for_each(|row| {
                write_row(&mut text, row);
            });

        // Get the scroll position before setting the text
        let old_adj = text_area.vadjustment().map(|adj| adj.value());

        set_text!(tb, &text);

        // I don't totally understand this, but after quite a lot of experimentation this works
        // well at keeping the scroll of the text view in the same area as you step through
        // time.
        if !ac.config.borrow().show_active_readout {
            if let Some(adj) = text_area.vadjustment() {
                if let Some(val) = old_adj {
                    let val = if val.round() < (adj.upper() - adj.page_size()).round() {
                        val.round()
                    } else {
                        (adj.upper() - adj.page_size() - 1.0).round()
                    };
                    adj.set_value(val);
                }
            }
        }
    }

    macro_rules! write_opt {
        ($opt_val:expr, $num_fmt: expr, $width_fmt: expr, $buf:ident) => {
            if $opt_val.is_some() {
                // Should never panic because $buf is a String.
                write!($buf, $num_fmt, $opt_val.unpack().unpack()).unwrap();
            } else {
                write!($buf, $width_fmt, "").unwrap();
            }
        };
    }

    fn write_row(buf: &mut String, row: DataRow) {
        write_opt!(row.pressure, "{:5.0}", "{:5}", buf);
        write_opt!(row.height, "{:6.0}", "{:6}", buf);
        write_opt!(row.temperature, "{:6.1}", "{:6}", buf);
        write_opt!(row.wet_bulb, "{:6.1}", "{:6}", buf);
        write_opt!(row.dew_point, "{:6.1}", "{:6}", buf);
        write_opt!(row.theta_e, "{:^8.0}", "{:^8}", buf);
        write_opt!(row.wind.map_t(|wnd| wnd.direction), "{:5.0}", "{:5}", buf);
        write_opt!(row.wind.map_t(|wnd| wnd.speed), "{:4.0}", "{:4}", buf);
        write_opt!(row.pvv, "{:6.1}", "{:6}", buf);
        write_opt!(row.cloud_fraction, "{:6.0}", "{:6}", buf);
        writeln!(buf).unwrap();
    }
}

pub fn fill_header_text_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    const HEADER_ID: &str = "text_header";
    let header: TextView = acp.fetch_widget(HEADER_ID)?;

    let tb = header.buffer();
    let mut text = String::with_capacity(512);

    text.push_str(&format!(
        " {:^4} {:^5} {:^5} {:^5} {:^5} {:^7}  {:^3} {:^4} {:^5}  {:^3}\n",
        "Pres", "Hgt", "T", "WB", "DP", "Theta E", "DIR", "SPD", "\u{03C9}", "CLD",
    ));
    text.push_str(&format!(
        " {:^4} {:^5} {:^5} {:^5} {:^5} {:^7}  {:^3} {:^4} {:^5}  {:^3}",
        "hPa", "m", "\u{00b0}C", "\u{00b0}C", "\u{00b0}C", "\u{00b0}K", "deg", "KT", "Pa/s", "%",
    ));

    make_default_tag!(tb, acp);
    set_text!(tb, &text);

    Ok(())
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

    if !text_area.is_realized() {
        return;
    }

    let tp = match *ac.get_sample() {
        Sample::Sounding { data, .. } => {
            if let Some(pr) = data.pressure.into_option() {
                pr
            } else {
                return;
            }
        }
        _ => return,
    };

    let tb = text_area.buffer();
    let start = tb.start_iter();
    let end = tb.end_iter();
    tb.remove_tag_by_name("highlight_above", &start, &end);
    tb.remove_tag_by_name("highlight_below", &start, &end);

    let lines = tb.line_count();
    for i in 0..(lines - 1) {
        if let (Some(start_above), Some(start_below)) = (tb.iter_at_line(i), tb.iter_at_line(i + 1))
        {
            let mut end_above = start_above;
            end_above.forward_chars(5);
            let above_val: HectoPascal =
                f64::from_str(tb.text(&start_above, &end_above, false).trim())
                    .map(HectoPascal)
                    .unwrap_or(HectoPascal(0.0));

            let mut end_below = start_below;
            end_below.forward_chars(5);
            let below_val: HectoPascal =
                f64::from_str(tb.text(&start_below, &end_below, false).trim())
                    .map(HectoPascal)
                    .unwrap_or(HectoPascal(0.0));

            if tp > above_val && tp <= below_val {
                let tt = tb.tag_table();
                // Set line colors
                let rgba = config.active_readout_line_rgba;
                let range = below_val - above_val;
                let alpha_below = (tp - above_val) / range;
                let alpha_above = 1.0 - alpha_below;

                let rgba_below = gtk::gdk::RGBA::new(
                    rgba.0 as f32,
                    rgba.1 as f32,
                    rgba.2 as f32,
                    alpha_below as f32,
                );
                let rgba_above = gtk::gdk::RGBA::new(
                    rgba.0 as f32,
                    rgba.1 as f32,
                    rgba.2 as f32,
                    alpha_above as f32,
                );

                if let Some(below_tag) = tt.lookup("highlight_below") {
                    below_tag.set_background_rgba(Some(&rgba_below));
                    end_below.forward_line();
                    tb.apply_tag(&below_tag, &start_below, &end_below);
                }
                if let Some(above_tag) = tt.lookup("highlight_above") {
                    above_tag.set_background_rgba(Some(&rgba_above));
                    end_above.forward_line();
                    tb.apply_tag(&above_tag, &start_above, &end_above);
                }

                // Scroll the view to this point.
                if let Some(ref mark) = tb.mark("scroll_mark") {
                    tb.move_mark(mark, &end_above);
                    text_area.scroll_to_mark(mark, 0.2, true, 0.0, 0.5);
                }

                break;
            }
        }
    }
}
