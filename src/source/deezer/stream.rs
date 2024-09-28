use std::any::Any;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::io::ErrorKind::WouldBlock;
use std::ops::{Deref, DerefMut};
use async_trait::async_trait;
use blowfish::Blowfish;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::cipher::block_padding::NoPadding;
use cbc::Decryptor;
use songbird::input::core::io::MediaSource;
use songbird::input::{AsyncAdapterStream, AudioStream, AudioStreamError, AuxMetadata, Compose, HttpRequest};
use tracing::error;
use crate::source::deezer::SECRET_IV;

pub struct DeezerHttpStream {
    pub inner: HttpRequest,
    pub key: String,
}

#[inline]
fn into_audio_stream(b: Box<dyn MediaSource>) -> AsyncAdapterStream {
    // SAFETY: We know the objects passed to this function are always AsyncAdapterStream
    *unsafe { Box::from_raw(Box::into_raw(b) as *mut _) }
}

#[async_trait]
impl Compose for DeezerHttpStream {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        self.inner.create()
    }

    async fn create_async(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        let inner = self.inner.create_async().await?;
        let stream = into_audio_stream(inner.input);
        Ok(AudioStream {
            input: Box::new(DeezerMediaSource {
                inner: stream,
                read_buf: Buf::new(),
                out_buf: Vec::with_capacity(2048),
                decryptor: Decryptor::new_from_slices(self.key.as_bytes(), &SECRET_IV).unwrap(),
                current_chunk: 0,
            }) as Box<dyn MediaSource>,
            hint: inner.hint
        })
    }

    fn should_create_async(&self) -> bool {
        self.inner.should_create_async()
    }

    async fn aux_metadata(&mut self) -> Result<AuxMetadata, AudioStreamError> {
        self.inner.aux_metadata().await
    }
}

pub struct DeezerMediaSource {
    inner: AsyncAdapterStream,
    read_buf: Buf<2048>,
    out_buf: Vec<u8>,
    current_chunk: usize,
    decryptor: Decryptor<Blowfish>
}

impl DeezerMediaSource {
    fn decrypt_block(&mut self) {
        let dec = self.decryptor
            .clone()
            .decrypt_padded_mut::<NoPadding>(self.read_buf.inner_mut());

        match dec {
            Ok(d) => self.out_buf.extend(d),
            Err(e) => error!("Failed to decrypt block, error: {e}")
        }
    }
}

impl Read for DeezerMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while !self.read_buf.is_full() && self.read_buf.read_from(&mut self.inner)? > 0 {}

        if self.current_chunk % 3 > 0 || self.read_buf.len() < 2048 {
            self.out_buf.extend(&*self.read_buf);
        } else {
            self.decrypt_block();
        }

        self.read_buf.reset();

        self.current_chunk += 1;

        let end = std::cmp::min(buf.len(), self.out_buf.len());
        let drain = self.out_buf.drain(0..end);
        let drain_len = drain.len();
        buf[0..drain_len].copy_from_slice(drain.as_ref());

        //tracing::info!("Read {} bytes", drain_len);
        Ok(drain_len)
    }
}

impl Seek for DeezerMediaSource {
    fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(p)
    }
}

impl MediaSource for DeezerMediaSource {
    fn is_seekable(&self) -> bool {
        self.inner.is_seekable()
    }

    fn byte_len(&self) -> Option<u64> {
        self.inner.byte_len()
    }
}

pub struct Buf<const SIZE: usize> {
    inner: [u8; SIZE],
    len: usize
}

impl<const SIZE: usize> Buf<SIZE> {
    pub fn new() -> Self {
        Self {
            inner: [0; SIZE],
            len: 0
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn remaining_mut(&mut self) -> &mut [u8] {
        &mut self.inner[self.len..]
    }

    pub fn read_from<R: Read>(&mut self, reader: &mut R) -> std::io::Result<usize> {
        let read_bytes = reader.read(self.remaining_mut())?;
        self.len += read_bytes;

        Ok(read_bytes)
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == SIZE
    }

    #[inline]
    pub fn reset(&mut self) {
        self.len = 0;
    }

    pub fn inner(&self) -> &[u8] {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl<const SIZE: usize> Deref for Buf<SIZE> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.inner[..self.len]
    }
}

impl<const SIZE: usize> DerefMut for Buf<SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner[..self.len]
    }
}

