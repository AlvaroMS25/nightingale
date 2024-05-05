use std::{fs, io};
use std::fmt::Arguments;
use std::io::{IoSlice, Read, Stdout, stdout, StdoutLock, Write};
use std::path::Path;

pub struct TracingWriter {
    inner: Box<dyn Write + Send>
}

impl TracingWriter {
    pub fn stdout() -> Self {
        Self {
            inner: Box::new(StdOutWriter::new())
        }
    }

    pub fn file(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            inner: Box::new(FileWriter::new(path)?)
        })
    }
}

impl Write for TracingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.inner.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
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
