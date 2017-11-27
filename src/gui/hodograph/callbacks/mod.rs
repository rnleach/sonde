
use cairo::Context;
use gtk::prelude::*;

use app::AppContextPointer;

mod drawing;

pub fn draw_hodo(cr: &Context, acp: &AppContextPointer) -> Inhibit {

    let ac = &mut acp.borrow_mut();

    drawing::prepare_to_draw_hodo(cr, ac);
    drawing::draw_hodo_background(cr, ac);
    drawing::draw_hodo_labels(cr, ac);
    drawing::draw_hodo_line(cr, ac);
    drawing::draw_active_readout(cr, ac);

    Inhibit(false)
}
