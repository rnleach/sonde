use gtk::prelude::*;
use gtk::{TextTag, TextView};
use sounding_analysis::{partition_cape, Analysis};

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
        if let Some(ref anal) = ac.get_sounding_for_display() {
            let text = &mut String::with_capacity(4096);

            push_header(text, ac.get_source_description(), anal);
            push_profile_indexes(text, anal);
            push_parcel_indexes(text, anal);
            push_fire_indexes(text, anal);

            set_text!(tb, &text);
        }
    }
}

#[inline]
fn push_header(buffer: &mut String, source_desc: Option<String>, anal: &Analysis) {
    if let Some(desc) = source_desc {
        buffer.push_str(&desc);
        buffer.push('\n');
    }

    if let Some(vt) = anal.sounding().get_valid_time() {
        buffer.push_str(&format!("     Valid: {}Z\n", vt));
    }

    let station_info = anal.sounding().get_station_info();
    if let Some((lat, lon)) = station_info.location() {
        buffer.push_str(&format!("(lat, lon): ({:.6},{:.6})\n", lat, lon));
    }
    if let Some(elev_m) = station_info.elevation().into_option() {
        buffer.push_str(&format!(
            " Elevation: {:.0}m ({:.0}ft)\n",
            elev_m,
            elev_m * 3.28084
        ));
    }
}

macro_rules! push_profile_index {
    ($anal: expr, $buf:ident, $name:expr, $selector:expr, $format:expr, $empty_val:expr) => {
        $buf.push_str($name);
        $anal
            .get_profile_index($selector)
            .and_then(|val| {
                $buf.push_str(&format!($format, val));
                Some(())
            }).or_else(|| {
                $buf.push_str($empty_val);
                Some(())
            });
        $buf.push('\n');
    };
}

#[inline]
#[rustfmt::skip]
fn push_profile_indexes(buffer: &mut String, anal: &Analysis){
    use sounding_analysis::ProfileIndex::*;
    let empty_val = "    -    ";

    buffer.push('\n');
    buffer.push('\n');

    buffer.push_str("Index        Value\n");
    buffer.push_str("-----------------------\n");
    push_profile_index!(anal, buffer, "SWeT         ", SWeT,        "{:>10.0}",          empty_val);
    push_profile_index!(anal, buffer, "K            ", K,           "{:>7.0} \u{00b0}C", empty_val);
    push_profile_index!(anal, buffer, "Total Totals ", TotalTotals, "{:>10.0}",          empty_val);
    push_profile_index!(anal, buffer, "DCAPE        ", DCAPE,       "{:>5.0} J/kg",      empty_val);
    push_profile_index!(anal, buffer, "PWAT         ", PWAT,        "{:>7.0} mm",        empty_val);
    push_profile_index!(anal, buffer, "Downrush T   ", DownrushT,   "{:>7.0} \u{00b0}C", empty_val);
    push_profile_index!(anal, buffer, "Convective T ", ConvectiveT, "{:>7.0} \u{00b0}C", empty_val);
}

#[inline]
#[rustfmt::skip]
fn push_parcel_indexes(buffer: &mut String, anal: &Analysis) {
    use sounding_analysis::ParcelIndex::*;

    buffer.push('\n');
    buffer.push('\n');

    macro_rules! push_var {
        ($buf:ident, $opt_anal:ident, $selector:ident, $fmt:expr,$empty:expr) => {
            $opt_anal.and_then(|anal| {
                anal.get_index($selector).and_then(|val|{
                    $buf.push_str(&format!($fmt, val));
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

            push_var!($buf, $opt_pcl_anal, CAPE,        " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, CIN,         " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, NCAPE,       " {:>5.2}", $empty);
            push_var!($buf, $opt_pcl_anal, CAPEHail,    " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, LI,          " {:>5.1}", $empty);

            $buf.push('\n');
        }
    }

    macro_rules! parcel_level_row {
        ($buf:ident, $pcl_name:expr, $opt_pcl_anal:ident, $empty:expr) => {
            $buf.push_str($pcl_name);

            push_var!($buf, $opt_pcl_anal, LCLPressure,   " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, LCLHeightAGL,  " {:>6.0}", $empty);
            push_var!($buf, $opt_pcl_anal, LFC,           " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, ELPressure,    " {:>5.0}", $empty);
            push_var!($buf, $opt_pcl_anal, ELHeightASL,   " {:>6.0}", $empty);
            push_var!($buf, $opt_pcl_anal, ELTemperature, " {:>5.0}", $empty);

            $buf.push('\n');
        }
    }

    let sfc = anal.get_surface_parcel_analysis();
    let ml = anal.get_mixed_layer_parcel_analysis();
    let mu = anal.get_most_unstable_parcel_analysis();
    let con = anal.get_convective_parcel_analysis();

    let empty = "     -";
    buffer.push_str("Parcel          CAPE   CIN NCAPE  Hail    LI\n");
    buffer.push_str("                J/Kg  J/Kg        CAPE     C\n");
    buffer.push_str("--------------------------------------------\n");
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
    use sounding_analysis::ProfileIndex::*;

    macro_rules! push_fire_index {
        ($buf:ident, $label:expr, $anal:ident, $selector:ident, $fmt:expr, $empty:expr) => {
            $buf.push_str($label);
            if let Some(val) = $anal.get_profile_index($selector) {
                $buf.push_str(&format!($fmt, val));
            } else {
                $buf.push_str($empty);
            }
        };
    }

    buffer.push('\n');
    buffer.push('\n');
    buffer.push_str("Fire Weather\n");
    buffer.push_str("------------------------\n\n");

    buffer.push_str("Haines   Low   Mid  High\n");
    buffer.push_str("       ");

    let empty = "  -   ";
    for &hns in [HainesLow, HainesMid, HainesHigh].into_iter() {
        if let Some(val) = anal.get_profile_index(hns) {
            buffer.push_str(&format!("{:>5.0} ", val));
        } else {
            buffer.push_str(empty);
        }
    }
    buffer.push('\n');
    push_fire_index!(buffer, "HDW         ", anal, Hdw, "{:>12.0}\n\n", empty);

    let empty = " - \n";

    buffer.push_str("Experimental\n");
    buffer.push_str("------------------------\n\n");
    push_fire_index!(buffer, "Conv. T def.", anal, ConvectiveDeficit, "{:>9.1} \u{00b0}C\n", empty);
    push_fire_index!(buffer, "CAPE ratio  ", anal, CapeRatio,         "{:>12.2}\n", empty);

    if let Some(parcel_anal) = anal.get_convective_parcel_analysis(){
        if let Ok((dry, wet)) = partition_cape(parcel_anal){
            buffer.push_str(&format!("Dry cape    {:>7.0} J/kg\nWet cape    {:>7.0} J/kg\n", dry, wet));
        }
    }
}
