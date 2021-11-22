use crate::rt::services::stdout::stdout_write;
use core::fmt::Write;
use libhedron::mem::PAGE_SIZE;
use log::{
    LevelFilter,
    Log,
    Metadata,
    Record,
};

static LOGGER: UserRustLogger = UserRustLogger;

#[derive(Debug)]
pub struct UserRustLogger;

impl UserRustLogger {
    pub fn init() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::max());
    }
}

impl Log for UserRustLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        const SIZE: usize = 4 * PAGE_SIZE;
        let mut buf = arrayvec::ArrayString::<SIZE>::new();
        write!(&mut buf, "[{:?}] {}", record.level(), record.args()).unwrap();
        stdout_write(buf.as_str());
    }

    fn flush(&self) {}
}
