use crate::{app::AppContextPointer, errors::SondeError};
use gtk::{
    idle_add, timeout_add_seconds, TextBufferExt, TextTag, TextTagExt, TextTagTableExt, TextView,
    TextViewExt,
};
use log::{self, Level, LevelFilter, Log, Metadata, Record};
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::{
        mpsc::{sync_channel, TryRecvError, TrySendError},
        Mutex,
    },
};

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

    let logger = Box::new(AppLogger::new(tx));
    let acp2 = Rc::clone(acp);

    timeout_add_seconds(1, move || {
        loop {
            match rx.try_recv() {
                Ok(ref msg) => log_msg(&acp2, msg),
                Err(TryRecvError::Disconnected) => log_msg(&acp2, "\n\nLOGGER DISCONNECTED\n\n"),
                Err(TryRecvError::Empty) => {
                    break;
                }
            }
        }

        ::glib::source::Continue(true)
    });

    log::set_boxed_logger(logger).map_err(|_| SondeError::LogError("Error setting logger"))?;
    log::set_max_level(LevelFilter::max());

    Ok(())
}

struct AppLogger {
    tx: ::std::sync::mpsc::SyncSender<String>,
    overflow: Mutex<RefCell<VecDeque<String>>>,
}

impl AppLogger {
    fn new(tx: ::std::sync::mpsc::SyncSender<String>) -> Self {
        AppLogger {
            tx,
            overflow: Mutex::new(RefCell::new(VecDeque::new())),
        }
    }
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= LOG_LEVEL
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            // put it on the overflow, and then try to empty
            let overflow = self.overflow.lock().unwrap();
            let mut overflow = overflow.borrow_mut();
            overflow.push_back(format!("{} - {}\n", record.level(), record.args()));

            // empty the overflow
            while let Some(msg) = overflow.pop_front() {
                match self.tx.try_send(msg) {
                    Ok(_) => {}
                    Err(TrySendError::Full(msg_back)) => {
                        overflow.push_front(msg_back);
                        break;
                    }
                    Err(TrySendError::Disconnected(_)) => {}
                }
            }
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
        }
    }

    let acp2 = Rc::clone(acp);
    idle_add(move || {
        if let Ok(ta) = acp2.fetch_widget::<TextView>(TEXT_AREA_ID) {
            if let Some(buf) = ta.get_buffer() {
                let end = &mut buf.get_end_iter();
                ta.scroll_to_iter(end, 0.0, false, 1.0, 1.0);
            }
        }

        ::glib::source::Continue(false)
    });
}
