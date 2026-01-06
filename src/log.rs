use std::{fs, path::Path, time::SystemTime};

pub fn setup_logger() -> Result<(), fern::InitError> {
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        if let Err(e) = fs::create_dir_all(&logs_dir) {
            eprintln!("Failed to create logs directory");
        }
    }
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs/output.log")?)
        .apply()?;
    Ok(())
}
