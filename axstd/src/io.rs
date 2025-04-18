/// Constructs a new handle to the standard output of the current process.
use core::fmt::{Error, Write};
use spinlock::SpinRaw;

#[derive(Debug)]
pub enum IoError {
    BadState = 1,
}

pub type Result<T = ()> = core::result::Result<T, IoError>;

struct StdoutRaw;

impl Write for StdoutRaw {
    fn write_str(&mut self, s: &str) -> core::result::Result<(), Error> {
        axhal::console::write_bytes(s.as_bytes());
        Ok(())
    }
}

static STDOUT: SpinRaw<StdoutRaw> = SpinRaw::new(StdoutRaw);

pub fn __print_impl(args: core::fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}
