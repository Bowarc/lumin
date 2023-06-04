pub fn init(log_file_opt: Option<&str>) {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors = ColoredLevelConfig::new().debug(Color::Magenta);

    let mut builder = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level_text = || -> String {
                #[cfg(not(debug_assertions))]
                return record.level().to_string();
                #[cfg(debug_assertions)]
                return colors.color(record.level()).to_string();
            };

            out.finish(format_args!(
                "╭[{time} {level} {file_path}:{line_nbr}]\n╰❯{message}",
                time = chrono::Local::now().format("%H:%M:%S"),
                level = level_text(),
                file_path = record.file().unwrap_or("Unknown file"),
                line_nbr = record
                    .line()
                    .map(|l| l.to_string())
                    .unwrap_or("?".to_string()),
                message = message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("html5ever", log::LevelFilter::Warn)
        .level_for("selectors", log::LevelFilter::Warn)
        .chain(std::io::stdout());
    #[cfg(not(debug_assertions))]
    if let Some(log_file) = log_file_opt {
        builder = builder.chain(fern::log_file(log_file).unwrap());
    }
    builder.apply().unwrap();

    log_panics::Config::new()
        .backtrace_mode(log_panics::BacktraceMode::Resolved)
        .install_panic_hook()
}
