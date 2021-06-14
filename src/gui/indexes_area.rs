use crate::{
    analysis::Analysis,
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{prelude::*, TextBuffer, TextTag, TextView};
use metfor::{Fahrenheit, Inches, Quantity};
use std::{fmt::Write, rc::Rc};

const TEXT_AREA_ID: &str = "indexes_text_area";
const HEADER_LINE: &str = "----------------------------------------------------\n";

pub fn set_up_indexes_area(acp: &AppContextPointer) -> Result<(), SondeError> {
    use gdk::keys::constants::{KP_Left, KP_Right, Left, Right};

    let text_area: TextView = acp.fetch_widget(TEXT_AREA_ID)?;

    let ac1 = Rc::clone(acp);
    text_area.connect_key_press_event(move |_ta, event| {
        let keyval = event.get_keyval();
        if keyval == KP_Right || keyval == Right {
            ac1.display_next();
        } else if keyval == KP_Left || keyval == Left {
            ac1.display_previous();
        }
        Inhibit(true)
    });

    if let Some(text_buffer) = text_area.get_buffer() {
        set_up_tags(&text_buffer, acp);
        set_text(&text_buffer, "No data, loaded");
        text_buffer.create_mark(Some("scroll_mark"), &text_buffer.get_start_iter(), true);
        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(TEXT_AREA_ID))
    }
}

pub fn update_indexes_area(ac: &AppContext) {
    let text_area: TextView = match ac.fetch_widget::<TextView>(TEXT_AREA_ID) {
        Ok(ta) => ta,
        Err(_) => return,
    };

    let text_buffer = match text_area.get_buffer() {
        Some(tb) => tb,
        None => return,
    };

    let anal = match ac.get_sounding_for_display() {
        Some(anal) => anal,
        None => return,
    };

    let anal = &anal.borrow();
    let text = &mut String::with_capacity(4096);

    push_profile_indexes(text, anal);
    push_parcel_indexes(text, anal);
    push_fire_indexes(text, anal);

    // Get the scroll position before setting the text
    let old_adj = text_area.get_vadjustment().map(|adj| adj.get_value());

    set_text(&text_buffer, &text);

    highlight_parcel(&text_buffer, ac);

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

fn set_up_tags(tb: &TextBuffer, ac: &AppContext) {
    if let Some(tag_table) = tb.get_tag_table() {
        let default_tag = TextTag::new(Some("default"));

        default_tag.set_property_font(Some("courier bold 12"));

        let success = tag_table.add(&default_tag);
        debug_assert!(success, "Failed to add tag to text tag table");

        let rgba = ac.config.borrow().parcel_indexes_highlight;
        let parcel_tag = TextTag::new(Some("parcel"));
        parcel_tag.set_property_background_rgba(Some(&gdk::RGBA {
            red: rgba.0,
            green: rgba.1,
            blue: rgba.2,
            alpha: rgba.3,
        }));

        let success = tag_table.add(&parcel_tag);
        debug_assert!(success, "Failed to add tag to text tag table");
    }
}

fn set_text(tb: &TextBuffer, txt: &str) {
    tb.set_text(txt);
    let start = tb.get_start_iter();
    let end = tb.get_end_iter();
    tb.apply_tag_by_name("default", &start, &end);
}

macro_rules! push_prof {
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                write!($buf, $format, val.unpack()).unwrap();
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, temp, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                write!(
                    $buf,
                    $format,
                    val.unpack(),
                    Fahrenheit::from(val).unpack().round()
                )
                .unwrap();
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, mm, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                let val_inches = Inches::from(val);
                if val_inches < Inches(0.01) && val_inches > Inches(0.0) {
                    write!($buf, "         T").unwrap();
                } else {
                    write!(
                        $buf,
                        $format,
                        (val.unpack() * 10.0).round() / 10.0,
                        (val_inches.unpack() * 100.0).round() / 100.0,
                    )
                    .unwrap();
                }
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, cape, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                let mps = (val.unpack() * 2.0).sqrt();
                let mph = mps * 2.23694;
                write!($buf, $format, val.unpack(), mps.round(), mph.round()).unwrap();
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
}

#[inline]
#[rustfmt::skip]
fn push_profile_indexes(buffer: &mut String, anal: &Analysis){
    let empty_val = "    -    ";

    buffer.push('\n');

    buffer.push_str("Index                Value\n");
    buffer.push_str(HEADER_LINE);
    push_prof!(anal, buffer, "1-hour Precip       ", provider_1hr_precip, "{:>7.1} mm ({:>4.2} in)",                           mm,   empty_val);
    push_prof!(anal, buffer, "DCAPE               ", dcape,               "{:>5.0} J/kg ({:>3.0} m/s {:>3.0} m/h)",            cape, empty_val);
    push_prof!(anal, buffer, "PWAT                ", pwat,                "{:>7.0} mm ({:>4.2} in)",                           mm,   empty_val);
    push_prof!(anal, buffer, "Downrush T          ", downrush_t,          "{:>8.0}\u{00b0}C ({:>3.0}\u{00b0}F)",               temp, empty_val);
    push_prof!(anal, buffer, "Convective T        ", convective_t,        "{:>8.0}\u{00b0}C ({:>3.0}\u{00b0}F)              ", temp, empty_val);
    push_prof!(anal, buffer, "3km SR Helicity (RM)", sr_helicity_3k_rm,   "{:>4.0} m\u{00b2}/s\u{00b2}",                             empty_val);
    push_prof!(anal, buffer, "3km SR Helicity (LM)", sr_helicity_3k_lm,   "{:>4.0} m\u{00b2}/s\u{00b2}",                             empty_val);
    push_prof!(anal, buffer, "Eff SR Helicity (RM)", sr_helicity_eff_rm,  "{:>4.0} m\u{00b2}/s\u{00b2}",                             empty_val);
    push_prof!(anal, buffer, "Eff SR Helicity (LM)", sr_helicity_eff_lm,  "{:>4.0} m\u{00b2}/s\u{00b2}",                             empty_val);
}

#[inline]
#[rustfmt::skip]
fn push_parcel_indexes(buffer: &mut String, anal: &Analysis) {
    buffer.push('\n');

    macro_rules! push_var {
        ($buf:ident, $anal:ident, $selector:tt, $fmt:expr,$empty:expr) => {
            $anal.$selector().into_option().and_then(|val|{
                $buf.push_str(&format!($fmt, val.unpack()));
                Some(())
            }).or_else(||{
                $buf.push_str($empty);
                Some(())
            });
        }
    }

    macro_rules! parcel_index_row {
        ($buf:ident, $pcl_name:expr, $opt_pcl_anal:ident, $empty:expr) => {
            $buf.push_str($pcl_name);
            if let Some(anal) = $opt_pcl_anal {

                push_var!($buf, anal, cape,         " {:>5.0}", $empty);
                push_var!($buf, anal, cin,          " {:>5.0}", $empty);
                push_var!($buf, anal, ncape,        " {:>5.2}", $empty);
                push_var!($buf, anal, hail_cape,    " {:>5.0}", $empty);
                $buf.push_str("              ");
            } else {
                $buf.push_str("         -- No Parcel --              ");
            }

            $buf.push('\n');
        }
    }

    macro_rules! parcel_level_row {
        ($buf:ident, $pcl_name:expr, $opt_pcl_anal:tt, $empty:expr) => {
            $buf.push_str($pcl_name);

            if let Some(anal) = $opt_pcl_anal {
                push_var!($buf, anal, lcl_pressure,   " {:>5.0}", $empty);
                push_var!($buf, anal, lcl_height_agl, " {:>6.0}", $empty);
                push_var!($buf, anal, lfc_pressure,   " {:>5.0}", $empty);
                push_var!($buf, anal, el_pressure,    " {:>5.0}", $empty);
                push_var!($buf, anal, el_height_asl,  " {:>6.0}", $empty);
                push_var!($buf, anal, el_temperature, " {:>5.0}", $empty);
            } else {
                $buf.push_str("         -- No Parcel --              ");
            }

            $buf.push('\n');
        }
    }

    let sfc = anal.surface_parcel_analysis();
    let ml = anal.mixed_layer_parcel_analysis();
    let mu = anal.most_unstable_parcel_analysis();
    let con = anal.convective_parcel_analysis();
    let eff = anal.effective_parcel_analysis();

    let empty = "     -";
    buffer.push_str("Parcel          CAPE   CIN NCAPE  Hail\n");
    buffer.push_str("                J/Kg  J/Kg        CAPE\n");
    buffer.push_str(HEADER_LINE);
    parcel_index_row!(buffer, "Surface       ", sfc, empty);
    parcel_index_row!(buffer, "Mixed Layer   ", ml,  empty);
    parcel_index_row!(buffer, "Most Unstable ", mu,  empty);
    parcel_index_row!(buffer, "Convective    ", con, empty);
    parcel_index_row!(buffer, "Effective     ", eff, empty);
    buffer.push('\n');
    buffer.push_str("Parcel           LCL    LCL   LFC    EL     EL    EL\n");
    buffer.push_str("                 hPa  m AGL   hPa   hPa  m ASL     C\n");
    buffer.push_str(HEADER_LINE);
    parcel_level_row!(buffer, "Surface       ", sfc, empty);
    parcel_level_row!(buffer, "Mixed Layer   ", ml,  empty);
    parcel_level_row!(buffer, "Most Unstable ", mu,  empty);
    parcel_level_row!(buffer, "Convective    ", con, empty);
    parcel_level_row!(buffer, "Effective     ", eff, empty);
}

#[inline]
#[rustfmt::skip]
fn push_fire_indexes(buffer: &mut String, anal: &Analysis) {
    
    macro_rules! push_fire_index {
        ($buf:ident, $label:expr, $anal:ident, $selector:tt, $fmt:expr, $empty:expr) => {
            $buf.push_str($label);
            if let Some(val) = $anal.$selector().into_option() {
                $buf.push_str(&format!($fmt, val.unpack()));
            } else {
                $buf.push_str($empty);
            }
        };
        ($buf:ident, $label:expr, $anal:ident, $selector_low:tt, $selector_high:tt, $fmt:expr, $empty:expr) => {
            $buf.push_str($label);
            if let (Some(val_low), Some(val_high)) = 
                ($anal.$selector_low().into_option(),$anal.$selector_high().into_option()) {
                    $buf.push_str(&format!($fmt, val_low.unpack(), val_high.unpack()));
            } else {
                $buf.push_str($empty);
            }
        };
    }

    buffer.push('\n');
    buffer.push_str("Fire Weather\n");
    buffer.push_str(HEADER_LINE);

    buffer.push_str("Haines     Low   Mid  High\n");
    buffer.push_str("         ");

    let empty = "  -   ";
    for &hns in [anal.haines_low(), anal.haines_mid(), anal.haines_high()].iter() {
        if let Some(val) = hns.into_option() {
            buffer.push_str(&format!("{:>5.0} ", val));
        } else {
            buffer.push_str(empty);
        }
    }
    buffer.push('\n');

    let empty = "  -   \n";
    push_fire_index!(buffer, "HDW           ", anal, hdw, "{:>12.0}\n", empty);
    buffer.push_str("PFT           ");
    if let Some(pft_anal) = anal.pft(){
            buffer.push_str(&format!("{:>10.0}GW\n", pft_anal.pft.unpack()));
    } else {
        buffer.push_str(empty);
    }

    let empty = " - \n";

    buffer.push_str("\nExperimental\n");
    buffer.push_str(HEADER_LINE);
    push_fire_index!(buffer, "Cloud ∆T         ", anal, lcl_dt_low,                                                    "{:>5.1}\u{00b0}C\n\n",                  empty);
    push_fire_index!(buffer, "Blow Up ∆T (LMIB)  ", anal, el_blow_up_dt_low, el_blow_up_dt_high,                       "{:>5.1}\u{00b0}C - {:>4.1}\u{00b0}C\n", empty);
    push_fire_index!(buffer, "Blow Up Hgt (LMIB) ", anal, el_blow_up_height_change_low, el_blow_up_height_change_high, "{:>6.0}m - {:>4.0}m\n",                 empty);

}

fn highlight_parcel(tb: &TextBuffer, ac: &AppContext) {
    use crate::app::config::ParcelType;

    let config = ac.config.borrow();

    if !config.show_parcel_profile {
        return;
    }

    let tag = match tb.get_tag_table().and_then(|tt| tt.lookup("parcel")) {
        Some(tag) => tag,
        None => return,
    };
    let rgba = config.parcel_indexes_highlight;
    tag.set_property_background_rgba(Some(&gdk::RGBA {
        red: rgba.0,
        green: rgba.1,
        blue: rgba.2,
        alpha: rgba.3,
    }));

    let pcl_label: &'static str = match config.parcel_type {
        ParcelType::Surface => "Surface",
        ParcelType::MixedLayer => "Mixed",
        ParcelType::MostUnstable => "Most",
        ParcelType::Effective => "Effective",
        ParcelType::Convective => "Convective",
    };

    let lines = tb.get_line_count();
    for i in 0..lines {
        let start = tb.get_iter_at_line(i);
        let mut end = start.clone();
        end.forward_line();

        tb.get_text(&start, &end, false)
            .map(|gstr| gstr.as_str().starts_with(pcl_label))
            .into_iter()
            .filter(|starts_with_parcel| *starts_with_parcel)
            .for_each(|_| tb.apply_tag(&tag, &start, &end));
    }
}
