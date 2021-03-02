use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::AppError;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Config {
    pub openjtalk: OpenJTalkConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenJTalkConfig {
    pub dictionary: PathBuf,
    pub hts_path: PathBuf,
    pub sampling: Option<i64>,
    pub frame_period: Option<i64>,
    pub all_pass: Option<f64>,
    pub postfilter_coef: f64,
    pub speed_rate: f64,
    pub additional_half_tone: f64,
    pub unvoiced_threshold: f64,
    pub spectrum_weight: f64,
    pub spectrum_f0: f64,
}

impl Default for OpenJTalkConfig {
    fn default() -> OpenJTalkConfig {
        OpenJTalkConfig {
            dictionary: PathBuf::new(),
            hts_path: PathBuf::new(),
            sampling: None,
            frame_period: None,
            all_pass: None,
            postfilter_coef: 0.0,
            speed_rate: 1.0,
            additional_half_tone: 0.0,
            unvoiced_threshold: 0.5,
            spectrum_weight: 1.0,
            spectrum_f0: 1.0,
        }
    }
}

impl Config {
    pub fn from_config() -> Result<Config, AppError> {
        let config_file = {
            if let Ok(path) = env::var("TTS_API_CONFIG_PATH") {
                PathBuf::from(path)
            } else {
                dirs::config_dir().expect("Failed to find config directory")
            }
        };
        let config_file = config_file.join("ttsapi_config.toml");

        let config = fs::read_to_string(&config_file)
            .map_err(|e| AppError::FileNotFound(config_file.clone(), e))?;

        let config: Config = toml::from_str(&config)
            .map_err(|e| AppError::ConfigDeserializationError(config_file, e))?;

        Ok(config)
    }
}

impl OpenJTalkConfig {
    pub fn execute<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<(), AppError> {
        let output = Command::new("open_jtalk")
            .arg("-x")
            .arg(&self.dictionary)
            .arg("-m")
            .arg(&self.hts_path)
            .arg("-a")
            .arg(format!("{}", self.all_pass.unwrap_or_default()))
            .arg("-b")
            .arg(format!("{}", self.postfilter_coef))
            .arg("-r")
            .arg(format!("{}", self.speed_rate))
            .arg("-fm")
            .arg(format!("{}", self.additional_half_tone))
            .arg("-u")
            .arg(format!("{}", self.unvoiced_threshold))
            .arg("-jm")
            .arg(format!("{}", self.spectrum_weight))
            .arg("-jf")
            .arg(format!("{}", self.spectrum_f0))
            .arg("-ow")
            .arg(output_path.as_ref())
            .arg(input_path.as_ref())
            .output()
            .map_err(|e| AppError::CommandSpawnError(e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(AppError::CommandError(
                String::from_utf8_lossy(&output.stdout).into(),
                String::from_utf8_lossy(&output.stderr).into(),
                output.status.code(),
            ))
        }
    }
}
