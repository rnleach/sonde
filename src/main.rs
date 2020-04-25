
fn main() {
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
