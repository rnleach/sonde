use gtk::{idle_add, TextBufferExt, TextTag, TextTagExt, TextTagTableExt, TextView, TextViewExt};
use log::{self, Level, LevelFilter, Log, Metadata, Record};
use std::rc::Rc;
use std::sync::mpsc::{sync_channel, TryRecvError};

use app::AppContextPointer;
use errors::SondeError;

const LOG_LEVEL: Level = Level::Trace;

const TEXT_AREA_ID: &str = "console_log";

pub fn set_up_console_log(acp: &AppContextPointer) -> Result<(), SondeError> {
    if let Ok(text_area) = acp.fetch_widget::<TextView>(TEXT_AREA_ID) {
        if let Some(buf) = text_area.get_buffer() {
            if let Some(tag_table) = buf.get_tag_table() {
                let tag = TextTag::new("default");

                tag.set_property_font(Some("courier bold 9"));

                let success = tag_table.add(&tag);
                debug_assert!(success, "Failed to add tag to text tag table");
            }
        }
    }

    let (tx, rx) = sync_channel::<String>(256);

    let logger = Box::new(AppLogger(tx));
    let acp2 = Rc::clone(acp);

    idle_add(move || {
        match rx.try_recv() {
            Ok(ref msg) => log_msg(&acp2, msg),
            Err(TryRecvError::Disconnected) => log_msg(&acp2, "\n\nLOGGER DISCONNECTED\n\n"),
            Err(TryRecvError::Empty) => {}
        }

        ::glib::source::Continue(true)
    });

    log::set_boxed_logger(logger).map_err(|_| SondeError::LogError("Error setting logger"))?;
    log::set_max_level(LevelFilter::max());

    Ok(())
}

struct AppLogger(::std::sync::mpsc::SyncSender<String>);

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LOG_LEVEL
    }

    fn log(&self, record: &Record) {
        eprintln!(
            "{} - {} - {}\n",
            record.level(),
            record.args(),
            self.enabled(record.metadata())
        );
        if self.enabled(record.metadata()) {
            self.0
                .send(format!("{} - {}\n", record.level(), record.args()))
                .ok();
        }
    }

    fn flush(&self) {}
}

fn log_msg(acp: &AppContextPointer, msg: &str) {
    if let Ok(text_area) = acp.fetch_widget::<TextView>(TEXT_AREA_ID) {
        if let Some(buf) = text_area.get_buffer() {
            let end = &mut buf.get_end_iter();
            buf.insert(end, msg);

            let start = &buf.get_start_iter();
            buf.apply_tag_by_name("default", start, end);
            text_area.scroll_to_iter(end, 0.0, false, 1.0, 1.0);
        }
    }
}
