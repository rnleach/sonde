use crate::{
    app::{AppContext, AppContextPointer},
    errors::SondeError,
};
use gtk::{prelude::*, TextTag, TextView};
use metfor::{Fahrenheit, Feet, Inches, Quantity};
use sounding_analysis::{partition_cape, Analysis};

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

        tb.create_mark("scroll_mark", &tb.get_start_iter(), true);

        Ok(())
    } else {
        Err(SondeError::TextBufferLoadError(TEXT_AREA_ID))
    }
}

pub fn update_indexes_area(ac: &AppContext) {
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
            let anal = &anal.borrow();
            let text = &mut String::with_capacity(4096);

            push_header(
                text,
                anal.sounding().source_description().map(|s| s.to_owned()),
                anal,
            );
            push_profile_indexes(text, anal);
            push_parcel_indexes(text, anal);
            push_fire_indexes(text, anal);

            // Get the scroll position before setting the text
            let old_adj = text_area.get_vadjustment().map(|adj| adj.get_value());

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

#[inline]
fn push_header(buffer: &mut String, source_desc: Option<String>, anal: &Analysis) {
    if let Some(desc) = source_desc {
        buffer.push_str(&desc);
        buffer.push('\n');
    }

    if let Some(vt) = anal.sounding().valid_time() {
        buffer.push_str(&format!("     Valid: {}Z\n", vt));
    }

    let station_info = anal.sounding().station_info();
    if let Some((lat, lon)) = station_info.location() {
        buffer.push_str(&format!("(lat, lon): ({:.6},{:.6})\n", lat, lon));
    }
    if let Some(elev_m) = station_info.elevation().into_option() {
        buffer.push_str(&format!(
            " Elevation: {:.0}m ({:.0}ft)\n",
            elev_m.unpack(),
            Feet::from(elev_m).unpack(),
        ));
    }
}

macro_rules! push_prof {
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                $buf.push_str(&format!($format, val.unpack()));
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, temp, $format2:expr, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                $buf.push_str(&format!($format, val.unpack()));
                $buf.push_str(&format!($format2, Fahrenheit::from(val).unpack().round()));
                Some(())
            })
            .or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
    ($anal: expr, $buf:ident, $name:expr, $selector:tt, $format:expr, mm, $format2:expr, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .$selector()
            .into_option()
            .and_then(|val| {
                $buf.push_str(&format!($format, val.unpack()));
                $buf.push_str(&format!(
                    $format2,
                    (Inches::from(val).unpack() * 100.0).round() / 100.0
                ));
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

    buffer.push_str("Index        Value\n");
    buffer.push_str("----------------------------------------------------\n");
    push_prof!(anal, buffer, "SWeT                ", swet,               "{:>10.0}",                                                 empty_val);
    push_prof!(anal, buffer, "K                   ", k_index,            "{:>8.0}\u{00b0}C",                                         empty_val);
    push_prof!(anal, buffer, "Total Totals        ", total_totals,       "{:>10.0}",                                                 empty_val);
    push_prof!(anal, buffer, "DCAPE               ", dcape,              "{:>5.0} J/kg",                                             empty_val);
    push_prof!(anal, buffer, "PWAT                ", pwat,               "{:>7.0} mm",                  mm,   " ({:>4.2} in)",       empty_val);
    push_prof!(anal, buffer, "Downrush T          ", downrush_t,         "{:>8.0}\u{00b0}C",            temp, " ({:>3.0}\u{00b0}F)", empty_val);
    push_prof!(anal, buffer, "Convective T        ", convective_t,       "{:>8.0}\u{00b0}C",            temp, " ({:>3.0}\u{00b0}F)", empty_val);
    push_prof!(anal, buffer, "3km SR Helicity (RM)", sr_helicity_3k_rm,  "{:>4.0} m\u{00b2}/s\u{00b2}",                              empty_val);
    push_prof!(anal, buffer, "3km SR Helicity (LM)", sr_helicity_3k_lm,  "{:>4.0} m\u{00b2}/s\u{00b2}",                              empty_val);
    push_prof!(anal, buffer, "Eff SR Helicity (RM)", sr_helicity_eff_rm, "{:>4.0} m\u{00b2}/s\u{00b2}",                              empty_val);
    push_prof!(anal, buffer, "Eff SR Helicity (LM)", sr_helicity_eff_lm, "{:>4.0} m\u{00b2}/s\u{00b2}",                              empty_val);
}

#[inline]
#[rustfmt::skip]
fn push_parcel_indexes(buffer: &mut String, anal: &Analysis) {

    buffer.push('\n');

    macro_rules! push_var {
        ($buf:ident, $opt_anal:ident, $selector:tt, $fmt:expr,$empty:expr) => {
            $opt_anal.and_then(|anal| {
                anal.$selector().into_option().and_then(|val|{
                    $buf.push_str(&format!($fmt, val.unpack()));
                    Some(())
                }).or_else(||{
                    $buf.push_str($empty);
                    Some(())
                });
                Some(())
            });
        }
    }

    macro_rules! parcel_index_row {
        ($buf:ident, $pcl_name:expr, $opt_pcl_anal:ident, $empty:expr) => {
            $buf.push_str($pcl_name);

            push_var!($buf, $opt_pcl_anal, cape,         " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, cin,          " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, ncape,        " {:>5.2}", $empty);
            push_var!($buf, $opt_pcl_anal, hail_cape,    " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, lifted_index, " {:>5.1}", $empty);

            $buf.push('\n');
        }
    }

    macro_rules! parcel_level_row {
        ($buf:ident, $pcl_name:expr, $opt_pcl_anal:tt, $empty:expr) => {
            $buf.push_str($pcl_name);

            push_var!($buf, $opt_pcl_anal, lcl_pressure,   " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, lcl_height_agl, " {:>6.0}", $empty);
            push_var!($buf, $opt_pcl_anal, lfc_pressure,   " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, el_pressure,    " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, el_height_asl,  " {:>6.0}", $empty);
            push_var!($buf, $opt_pcl_anal, el_temperature, " {:>5.0}", $empty);

            $buf.push('\n');
        }
    }

    let sfc = anal.surface_parcel_analysis();
    let ml = anal.mixed_layer_parcel_analysis();
    let mu = anal.most_unstable_parcel_analysis();
    let con = anal.convective_parcel_analysis();

    let empty = "     -";
    buffer.push_str("Parcel          CAPE   CIN NCAPE  Hail    LI\n");
    buffer.push_str("                J/Kg  J/Kg        CAPE     C\n");
    buffer.push_str("----------------------------------------------------\n");
    parcel_index_row!(buffer, "Surface       ", sfc, empty);
    parcel_index_row!(buffer, "Mixed Layer   ", ml,  empty);
    parcel_index_row!(buffer, "Most Unstable ", mu,  empty);
    parcel_index_row!(buffer, "Convective    ", con, empty);
    buffer.push('\n');
    buffer.push_str("Parcel           LCL    LCL   LFC    EL     EL    EL\n");
    buffer.push_str("                 hPa  m AGL   hPa   hPa  m ASL     C\n");
    buffer.push_str("----------------------------------------------------\n");
    parcel_level_row!(buffer, "Surface       ", sfc, empty);
    parcel_level_row!(buffer, "Mixed Layer   ", ml,  empty);
    parcel_level_row!(buffer, "Most Unstable ", mu,  empty);
    parcel_level_row!(buffer, "Convective    ", con, empty);
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
    }

    buffer.push('\n');
    buffer.push_str("Fire Weather\n");
    buffer.push_str("----------------------------------------------------\n");

    buffer.push_str("Haines   Low   Mid  High\n");
    buffer.push_str("       ");

    let empty = "  -   ";
    for &hns in [anal.haines_low(), anal.haines_mid(), anal.haines_high()].iter() {
        if let Some(val) = hns.into_option() {
            buffer.push_str(&format!("{:>5.0} ", val));
        } else {
            buffer.push_str(empty);
        }
    }
    buffer.push('\n');
    push_fire_index!(buffer, "HDW         ", anal, hdw, "{:>12.0}\n", empty);

    let empty = " - \n";

    buffer.push_str("\nExperimental\n");
    buffer.push_str("----------------------------------------------------\n");
    push_fire_index!(buffer, "Conv. T def.", anal, convective_deficit,"{:>9.1} \u{00b0}C\n", empty);
    push_fire_index!(buffer, "CAPE ratio  ", anal, cape_ratio,        "{:>12.2}\n", empty);

    if let Some(parcel_anal) = anal.convective_parcel_analysis(){
        if let Ok((dry, wet)) = partition_cape(parcel_anal){
            buffer.push_str(
                &format!("Dry cape    {:>7.0} J/kg\nWet cape    {:>7.0} J/kg\n", 
                    dry.unpack(), 
                    wet.unpack(),
                )
            );
        }
    }
}
