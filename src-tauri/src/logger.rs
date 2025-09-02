use std::panic::set_hook;

use log::error;

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