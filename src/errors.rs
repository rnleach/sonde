use sounding_bufkit;

error_chain!{

    links{
        BufkitSounding(sounding_bufkit::Error, sounding_bufkit::ErrorKind);
    }

    foreign_links {
        Glib(::glib::error::BoolError);
    }
}
