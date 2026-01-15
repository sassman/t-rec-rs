use std::{env, fs::File, io, time::SystemTime};

/// A lazy file writer that only creates the file on first write.
/// This avoids creating empty log files when nothing is logged.
struct LazyLogFile {
    path: &'static str,
    file: Option<File>,
}

impl LazyLogFile {
    fn new(path: &'static str) -> Self {
        Self { path, file: None }
    }
}

impl io::Write for LazyLogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.file.is_none() {
            self.file = Some(File::create(self.path)?);
        }
        self.file.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            file.flush()
        } else {
            Ok(())
        }
    }
}

/// Initialize logging to a file (./t-rec-recording.log).
///
/// Default level is INFO, can be overridden with RUST_LOG env var.
/// Logging to a file avoids interfering with the terminal output.
/// The log file is only created if something is actually logged.
pub fn init_logging() {
    use env_logger::{Builder, Target};
    use std::io::Write as _;

    let mut builder = Builder::new();

    // Set default filter to INFO, allow RUST_LOG to override
    builder.filter_level(log::LevelFilter::Info);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }

    // Format with timestamp and level
    builder.format(|buf, record| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let millis = now.subsec_millis();
        writeln!(
            buf,
            "[{}.{:03} {} {}] {}",
            secs,
            millis,
            record.level(),
            record.target(),
            record.args()
        )
    });

    // Use lazy file writer - only creates file on first log write
    builder.target(Target::Pipe(Box::new(LazyLogFile::new(
        "t-rec-recording.log",
    ))));

    builder.init();
}
