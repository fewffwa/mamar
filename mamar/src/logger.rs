use log::*;

pub static LOGGER: MyLogger = MyLogger;

pub struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let is_mamar = record.file().map(|s| s.contains("mamar")).unwrap_or_default();
        if !is_mamar {
            return;
        }

        let show_file = record.file().is_some()
            && record.line().is_some()
            && (cfg!(debug_assertions) || record.level() != Level::Info);

        if show_file {
            println!(
                "{}:{} {}",
                record.file().unwrap(),
                record.line().unwrap(),
                record.args()
            );
        } else {
            println!("{}", record.args());
        }
    }
    fn flush(&self) {}
}
