extern crate gtk;
extern crate cairo;

use gtk::prelude::*;

use gtk::{Window, WindowType, DrawingArea};
use cairo::Context;
use cairo::enums::{FontSlant, FontWeight};

fn draw_fn(darea: &DrawingArea, cr: &Context) -> Inhibit 
{
    cr.set_source_rgb( 0.0, 0.0, 0.0);
    cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    cr.set_font_size(40.0);
    cr.move_to(10.0, 50.0);
    cr.show_text("Hello");
    Inhibit(false)
}

fn main() {

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);

    let darea = Box::new(DrawingArea::new)();
    darea.connect_draw(draw_fn);

    window.add(&darea);

    window.set_title("Sonde");
    window.set_default_size(650, 650);

    window.show_all();

    window.connect_delete_event(|_,_| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
