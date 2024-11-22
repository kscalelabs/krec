use crate::proto::{KRecFrame, KRecHeader};
use bytes::BytesMut;
use color_eyre::Result;
use prost::Message;
use std::fs::File;
use std::io::{Read, Write};
use tracing::{debug, info, instrument};

#[derive(Debug, Clone)]
pub struct KRec {
    pub header: KRecHeader,
    pub frames: Vec<KRecFrame>,
}

impl KRec {
    #[instrument]
    pub fn new(header: KRecHeader) -> Self {
        info!("Creating new KRec instance");
        Self {
            header,
            frames: Vec::new(),
        }
    }

    #[instrument(skip(frame))]
    pub fn add_frame(&mut self, frame: KRecFrame) {
        debug!("Adding frame to KRec");
        self.frames.push(frame);
    }

    #[instrument]
    pub fn save(&self, path: &str) -> Result<()> {
        info!("Saving KRec to file: {}", path);
        let mut file = File::create(path)?;

        // Write header
        let mut header_bytes = BytesMut::new();
        self.header.encode(&mut header_bytes)?;
        file.write_all(&header_bytes)?;
        debug!("Wrote header ({} bytes)", header_bytes.len());

        // Write frames
        for (i, frame) in self.frames.iter().enumerate() {
            let mut frame_bytes = BytesMut::new();
            frame.encode(&mut frame_bytes)?;
            file.write_all(&frame_bytes)?;
            debug!("Wrote frame {} ({} bytes)", i, frame_bytes.len());
        }

        info!("Successfully saved KRec with {} frames", self.frames.len());
        Ok(())
    }

    #[instrument]
    pub fn load(path: &str) -> Result<Self> {
        info!("Loading KRec from file: {}", path);
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let header = KRecHeader::decode(&buffer[..])?;
        let mut frames = Vec::new();

        let mut pos = header.encoded_len();
        debug!("Read header ({} bytes)", pos);

        while pos < buffer.len() {
            let frame = KRecFrame::decode(&buffer[pos..])?;
            let frame_len = frame.encoded_len();
            pos += frame_len;
            frames.push(frame);
            debug!("Read frame {} ({} bytes)", frames.len(), frame_len);
        }

        info!("Successfully loaded KRec with {} frames", frames.len());
        Ok(Self { header, frames })
    }
}
