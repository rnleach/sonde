use sounding_base;
use sounding_bufkit;

error_chain!{

    links{
        BufkitSounding(sounding_bufkit::Error, sounding_bufkit::ErrorKind);
        BaseSounding(sounding_base::Error, sounding_base::ErrorKind);
    }

    foreign_links {
        Glib(::glib::error::BoolError);
    }
}
