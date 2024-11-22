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
        // Write the length of the header first
        let header_len = header_bytes.len() as u32;
        debug!("Writing header length: {} bytes", header_len);
        file.write_all(&header_len.to_le_bytes())?;
        file.write_all(&header_bytes)?;
        debug!("Wrote header ({} bytes)", header_bytes.len());

        // Write frames
        for (i, frame) in self.frames.iter().enumerate() {
            let mut frame_bytes = BytesMut::new();
            frame.encode(&mut frame_bytes)?;
            let frame_len = frame_bytes.len() as u32;
            debug!("Writing frame {} length: {} bytes", i, frame_len);
            file.write_all(&frame_len.to_le_bytes())?;
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

        debug!("Read file of {} bytes", buffer.len());
        let mut pos = 0;

        // Read header length and decode header
        if buffer.len() < 4 {
            return Err(color_eyre::eyre::eyre!(
                "File too short: {} bytes",
                buffer.len()
            ));
        }
        let header_len = u32::from_le_bytes(buffer[..4].try_into().unwrap()) as usize;
        pos += 4;

        debug!("Header length prefix: {} bytes", header_len);
        if pos + header_len > buffer.len() {
            return Err(color_eyre::eyre::eyre!(
                "Incomplete header data: need {} bytes, have {} bytes",
                header_len,
                buffer.len() - pos
            ));
        }
        let header = KRecHeader::decode(&buffer[pos..pos + header_len])?;
        pos += header_len;
        debug!(
            "Read header ({} bytes), position now at {}",
            header_len, pos
        );

        let mut frames = Vec::new();

        while pos + 4 <= buffer.len() {
            let frame_len = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            debug!("Frame length prefix: {} bytes", frame_len);

            if pos + frame_len > buffer.len() {
                return Err(color_eyre::eyre::eyre!(
                    "Incomplete frame data: at position {}, need {} bytes, have {} bytes remaining",
                    pos,
                    frame_len,
                    buffer.len() - pos
                ));
            }

            let frame = KRecFrame::decode(&buffer[pos..pos + frame_len])?;
            pos += frame_len;
            frames.push(frame);
            debug!(
                "Read frame {} ({} bytes), position now at {}",
                frames.len(),
                frame_len,
                pos
            );
        }

        if pos != buffer.len() {
            return Err(color_eyre::eyre::eyre!(
                "Trailing data: {} bytes remaining after position {}",
                buffer.len() - pos,
                pos
            ));
        }

        info!("Successfully loaded KRec with {} frames", frames.len());
        Ok(Self { header, frames })
    }
}
