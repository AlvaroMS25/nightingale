use std::{fs, io};
use std::io::{IoSlice, Stdout, stdout, Write};
use std::path::Path;

enum WriterSource {
    Stdout(StdOutWriter),
    File(FileWriter)
}
pub struct TracingWriter {
    inner: WriterSource
}

impl TracingWriter {
    pub fn stdout() -> Self {
        Self {
            inner: WriterSource::Stdout(StdOutWriter::new())
        }
    }

    pub fn file(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            inner: WriterSource::File(FileWriter::new(path)?)
        })
    }
}

impl Write for TracingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            WriterSource::Stdout(out) => out.write(buf),
            WriterSource::File(file) => file.write(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        match &mut self.inner {
            WriterSource::Stdout(out) => out.write_vectored(bufs),
            WriterSource::File(file) => file.write_vectored(bufs)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            WriterSource::Stdout(out) => out.flush(),
            WriterSource::File(file) => file.flush()
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match &mut self.inner {
            WriterSource::Stdout(out) => out.write_all(buf),
            WriterSource::File(file) => file.write_all(buf)
        }
    }
}

pub struct StdOutWriter {
    stdout: Stdout,
}

impl StdOutWriter {
    pub fn new() -> Self {
        let out = stdout();

        Self {
            stdout: out
        }
    }
}

impl Write for StdOutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.stdout.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdout.write_all(buf)
    }
}

pub struct FileWriter {
    file: fs::File
}

impl FileWriter {
    pub fn new(filename: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            file: fs::OpenOptions::new().append(true).create(true).open(filename)?
        })
    }
}

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.file.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
