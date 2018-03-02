extern crate failure;
extern crate sonde;

use failure::Fail;

fn main() {
    if let Err(ref e) = sonde::run() {
        println!("error: {}", e);

        let mut fail: &Fail = e.cause();

        while let Some(cause) = fail.cause() {
            println!("caused by: {}", cause);

            if let Some(backtrace) = cause.backtrace() {
                println!("backtrace: {}\n\n\n", backtrace);
            }

            fail = cause;
        }

        ::std::process::exit(1);
    }
}
