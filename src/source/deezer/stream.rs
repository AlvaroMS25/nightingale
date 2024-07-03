use std::io::{ErrorKind, Read, Seek, SeekFrom};
use async_trait::async_trait;
use blowfish::Blowfish;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::cipher::block_padding::NoPadding;
use cbc::Decryptor;
use songbird::input::core::io::MediaSource;
use songbird::input::{AudioStream, AudioStreamError, AuxMetadata, Compose, HttpRequest};
use tracing::error;
use crate::source::deezer::SECRET_IV;

pub struct DeezerHttpStream {
    pub inner: HttpRequest,
    pub key: String,
}

#[async_trait]
impl Compose for DeezerHttpStream {
    fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        self.inner.create()
    }

    async fn create_async(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
        let inner = self.inner.create_async().await?;
        Ok(AudioStream {
            input: Box::new(DeezerMediaSource {
                inner: inner.input,
                read_buf: [0; 2048],
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
    inner: Box<dyn MediaSource>,
    read_buf: [u8; 2048],
    out_buf: Vec<u8>,
    current_chunk: usize,
    decryptor: Decryptor<Blowfish>
}

impl DeezerMediaSource {
    fn decrypt_block(&mut self) {
        let dec = self.decryptor
            .clone()
            .decrypt_padded_mut::<NoPadding>(&mut self.read_buf);

        match dec {
            Ok(d) => self.out_buf.extend(d),
            Err(e) => error!("Failed to decrypt block, error: {e}")
        }
    }
}

impl Read for DeezerMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read_bytes = self.inner.read(&mut self.read_buf)?;

        if self.current_chunk % 3 > 0 || read_bytes < 2048 {
            self.out_buf.extend(&self.read_buf[0..read_bytes]);
        } else {
            self.decrypt_block();
        }

        self.current_chunk += 1;

        let end = std::cmp::min(buf.len(), self.out_buf.len());
        let drain = self.out_buf.drain(0..end);
        let drain_len = drain.len();
        buf[0..drain_len].copy_from_slice(drain.as_ref());

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
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.inner.byte_len()
    }
}
