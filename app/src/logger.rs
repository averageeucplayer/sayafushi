use std::{panic::set_hook, path::Path};
use flexi_logger::{
    Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, Naming, WriteMode,
};
use log::*;

pub fn setup_logger(log_dir: &Path) {

    let logger = Logger::try_with_str("info, tao=off")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .suppress_timestamp()
                .basename("loa_logs")
                .directory(log_dir),
        )
        .use_utc()
        .write_mode(WriteMode::BufferAndFlush)
        .append()
        .format(default_format_with_time)
        .rotate(
            Criterion::Size(5_000_000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(2),
        );

    #[cfg(debug_assertions)]
    {
        let _ = logger.duplicate_to_stdout(Duplicate::All);
    }
}

fn default_format_with_time(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] {} [{}] {}",
        now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}

pub fn setup_panic_hook() {
    set_hook(Box::new(|info| {
        error!("Panicked: {:?}", info);
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<Any>"
        };

        let location = info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".into());

        error!(
            target: "panic",
            "Thread '{}' panicked at '{}', {}",
            thread_name,
            msg,
            location
        );

        log::logger().flush();
    }));
}