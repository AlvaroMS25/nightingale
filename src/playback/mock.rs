use songbird::input::{AudioStream, AudioStreamError, Compose};
use symphonia::core::io::MediaSource;
use std::io;

#[derive(Clone, Copy)]
pub struct MockMediaSource;

#[async_trait::async_trait]
impl Compose for MockMediaSource {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        Ok(AudioStream {
            input: Box::new(*self) as Box<_>,
            hint: None
        })
    }

    /// Create a source asynchronously.
    ///
    /// If [`should_create_async`] returns `true`, this method will chosen at runtime.
    ///
    /// [`should_create_async`]: Self::should_create_async
    async fn create_async(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError>
    {
        Err(AudioStreamError::Unsupported)
    }

    fn should_create_async(&self) -> bool {
        false
    }
}

impl MediaSource for MockMediaSource {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        Some(0)
    }
}

impl io::Read for MockMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl io::Seek for MockMediaSource {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        Ok(0)
    }
}
