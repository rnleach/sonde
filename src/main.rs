#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

fn main() {
    #[cfg(target_os = "windows")]
    {
        let key = "GDK_PIXBUF_MODULEDIR";
        let value = "lib/gdk-pixbuf-2.0/2.10.0/loaders";
        std::env::set_var(key, value)
    }

    if let Err(e) = sonde::run() {
        println!("error: {}", e);

        let mut err = &*e;

        while let Some(cause) = err.source() {
            println!("caused by: {}", cause);

            err = cause;
        }

        ::std::process::exit(1);
    }
}
