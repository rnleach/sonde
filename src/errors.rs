error_chain!{

    foreign_links {
        Glib(::glib::error::BoolError);
    }
}
