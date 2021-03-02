use super::TtsEngine;
use crate::{config::OpenJTalkConfig, error::AppError};
use std::io::{Read, Write};
use tempfile::NamedTempFile;

pub struct OpenJTalk {
    config: OpenJTalkConfig,
}

impl TtsEngine for OpenJTalk {
    type Config = OpenJTalkConfig;
    type Error = AppError;

    fn from_config(config: OpenJTalkConfig) -> Result<OpenJTalk, AppError> {
        Ok(Self { config })
    }

    fn generate(&self, text: &str) -> Result<Vec<u8>, AppError> {
        let mut input_file = NamedTempFile::new()?;
        let mut output_file = NamedTempFile::new()?;

        input_file.write(text.as_bytes())?;

        self.config.execute(input_file.path(), output_file.path())?;

        let mut buffer = Vec::new();
        output_file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }

    fn generate_i16(&self, text: &str) -> Result<Vec<i16>, AppError> {
        let mut input_file = NamedTempFile::new()?;
        let mut output_file = NamedTempFile::new()?;

        input_file.write(text.as_bytes())?;

        self.config.execute(input_file.path(), output_file.path())?;

        let (_header, body) = wav::read(&mut output_file)?;
        if let wav::bit_depth::BitDepth::Sixteen(body) = body {
            Ok(body)
        } else {
            Err(AppError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid data",
            )))
        }
    }
}
