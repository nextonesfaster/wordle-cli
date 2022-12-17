use std::io::Write;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

/// A simple [`Result`](std::result::Result) type used in this application.
pub type Result<T> = std::result::Result<T, Error>;

/// A simple generic [`Error`](std::error::Error) type used throughtout this application.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Exits the application with an error message and code.
pub fn exit(err: Error, code: i32) -> ! {
    let error = || -> Result<()> {
        let bufwtr = BufferWriter::stderr(ColorChoice::Auto);
        let mut buffer = bufwtr.buffer();
        buffer.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;

        write!(&mut buffer, "error")?;
        buffer.reset()?;
        writeln!(&mut buffer, ": {}", err)?;
        bufwtr.print(&buffer)?;

        Ok(())
    };

    if let Err(e) = error() {
        eprintln!("error: {}", e);
    }

    std::process::exit(code);
}
