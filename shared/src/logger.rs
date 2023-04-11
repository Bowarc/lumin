pub fn init(log_file_opt: Option<&str>) {
    let mut builder = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "╭[{} {} {}:{}]\n╰❯{}",
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                record.file().unwrap_or("Unknown file"),
                record
                    .line()
                    .map(|l| l.to_string())
                    .unwrap_or("?".to_string()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());
    if let Some(log_file) = log_file_opt {
        builder = builder.chain(fern::log_file(log_file).unwrap());
    }
    builder.apply().unwrap();

    log_panics::Config::new()
        .backtrace_mode(log_panics::BacktraceMode::Resolved)
        .install_panic_hook()
}
