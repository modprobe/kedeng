use slog::{o, Drain, Filter, Level, Logger};
use slog_async::OverflowStrategy;
use slog_term::{FullFormat, TermDecorator};
use std::sync::OnceLock;

static LOGGER: OnceLock<Logger> = OnceLock::new();
pub fn get_logger() -> &'static Logger {
    LOGGER.get().unwrap()
}

pub fn init_logger(verbose: bool) {
    LOGGER.set(build_logger(verbose)).unwrap();
}

fn build_logger(verbose: bool) -> Logger {
    let decorator = TermDecorator::new().build();

    let drain = FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    let drain = Filter::new(drain, move |record| {
        if !verbose {
            record.level().is_at_least(Level::Info)
        } else {
            true
        }
    })
    .fuse();

    Logger::root(drain, o!())
}
