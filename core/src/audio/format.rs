//! Audio format detection and validation
//!
//! Detects audio file formats and validates file integrity

use serde::{Deserialize, Serialize};

/// Audio sample format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat {
    /// 8-bit unsigned integer
    U8,
    /// 8-bit signed integer
    I8,
    /// 16-bit unsigned integer
    U16,
    /// 16-bit signed integer
    I16,
    /// 24-bit signed integer (stored in i32)
    I24,
    /// 32-bit signed integer
    I32,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
}

impl SampleFormat {
    /// Get the size in bytes of this sample format
    pub fn size_bytes(&self) -> usize {
        match self {
            SampleFormat::U8 => 1,
            SampleFormat::I8 => 1,
            SampleFormat::U16 => 2,
            SampleFormat::I16 => 2,
            SampleFormat::I24 => 3,
            SampleFormat::I32 => 4,
            SampleFormat::F32 => 4,
            SampleFormat::F64 => 8,
        }
    }

    /// Check if this is a floating point format
    pub fn is_float(&self) -> bool {
        matches!(self, SampleFormat::F32 | SampleFormat::F64)
    }

    /// Check if this is an integer format
    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }
}

/// Audio format specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
    /// Sample format
    pub sample_format: SampleFormat,
    /// Channel layout (optional)
    pub channel_layout: Option<ChannelLayout>,
}

impl AudioFormat {
    /// Create a new audio format
    pub fn new(sample_rate: u32, channels: u16, sample_format: SampleFormat) -> Self {
        Self {
            sample_rate,
            channels,
            sample_format,
            channel_layout: ChannelLayout::from_channel_count(channels),
        }
    }

    /// Get the frame size in bytes (all channels for one sample)
    pub fn frame_size(&self) -> usize {
        self.channels as usize * self.sample_format.size_bytes()
    }

    /// Get the byte rate (bytes per second)
    pub fn byte_rate(&self) -> u64 {
        self.sample_rate as u64 * self.frame_size() as u64
    }

    /// Check if this format is compatible with another format
    pub fn is_compatible_with(&self, other: &AudioFormat) -> bool {
        self.sample_rate == other.sample_rate && self.channels == other.channels
    }

    /// Check if this is a high-resolution format (>= 48kHz or > 16-bit)
    pub fn is_high_resolution(&self) -> bool {
        self.sample_rate >= 48000 || !matches!(self.sample_format, SampleFormat::I16)
    }

    /// Convert to CPAL stream config
    pub fn to_cpal_config(&self) -> cpal::StreamConfig {
        cpal::StreamConfig {
            channels: self.channels,
            sample_rate: self.sample_rate,
            buffer_size: cpal::BufferSize::Default,
        }
    }

    /// Create from CPAL stream config
    pub fn from_cpal_config(config: &cpal::StreamConfig, sample_format: SampleFormat) -> Self {
        Self {
            sample_rate: config.sample_rate,
            channels: config.channels,
            sample_format,
            channel_layout: ChannelLayout::from_channel_count(config.channels),
        }
    }
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self::new(44100, 2, SampleFormat::F32)
    }
}

/// Channel layout specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelLayout {
    /// Mono (1 channel)
    Mono,
    /// Stereo (2 channels: Left, Right)
    Stereo,
    /// 2.1 (3 channels: Left, Right, LFE)
    Surround21,
    /// 5.1 (6 channels: Left, Right, Center, LFE, Left Surround, Right Surround)
    Surround51,
    /// 7.1 (8 channels: 5.1 + Left Back, Right Back)
    Surround71,
    /// Custom channel layout
    Custom(Vec<Channel>),
}

impl ChannelLayout {
    /// Get the number of channels in this layout
    pub fn channel_count(&self) -> u16 {
        match self {
            ChannelLayout::Mono => 1,
            ChannelLayout::Stereo => 2,
            ChannelLayout::Surround21 => 3,
            ChannelLayout::Surround51 => 6,
            ChannelLayout::Surround71 => 8,
            ChannelLayout::Custom(channels) => channels.len() as u16,
        }
    }

    /// Create a channel layout from channel count
    pub fn from_channel_count(count: u16) -> Option<Self> {
        match count {
            1 => Some(ChannelLayout::Mono),
            2 => Some(ChannelLayout::Stereo),
            3 => Some(ChannelLayout::Surround21),
            6 => Some(ChannelLayout::Surround51),
            8 => Some(ChannelLayout::Surround71),
            _ => None, // Custom layouts need explicit specification
        }
    }

    /// Get the channels in this layout
    pub fn channels(&self) -> Vec<Channel> {
        match self {
            ChannelLayout::Mono => vec![Channel::Center],
            ChannelLayout::Stereo => vec![Channel::Left, Channel::Right],
            ChannelLayout::Surround21 => vec![Channel::Left, Channel::Right, Channel::LFE],
            ChannelLayout::Surround51 => vec![
                Channel::Left,
                Channel::Right,
                Channel::Center,
                Channel::LFE,
                Channel::LeftSurround,
                Channel::RightSurround,
            ],
            ChannelLayout::Surround71 => vec![
                Channel::Left,
                Channel::Right,
                Channel::Center,
                Channel::LFE,
                Channel::LeftSurround,
                Channel::RightSurround,
                Channel::LeftBack,
                Channel::RightBack,
            ],
            ChannelLayout::Custom(channels) => channels.clone(),
        }
    }
}

/// Individual audio channel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    /// Left channel
    Left,
    /// Right channel
    Right,
    /// Center channel
    Center,
    /// Low Frequency Effects (subwoofer)
    LFE,
    /// Left surround channel
    LeftSurround,
    /// Right surround channel
    RightSurround,
    /// Left back channel
    LeftBack,
    /// Right back channel
    RightBack,
    /// Custom channel
    Custom(String),
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Left => write!(f, "L"),
            Channel::Right => write!(f, "R"),
            Channel::Center => write!(f, "C"),
            Channel::LFE => write!(f, "LFE"),
            Channel::LeftSurround => write!(f, "LS"),
            Channel::RightSurround => write!(f, "RS"),
            Channel::LeftBack => write!(f, "LB"),
            Channel::RightBack => write!(f, "RB"),
            Channel::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Audio format validation errors
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// Invalid sample rate provided
    #[error("Invalid sample rate: {0} Hz")]
    InvalidSampleRate(u32),

    /// Invalid channel count provided
    #[error("Invalid channel count: {0}")]
    InvalidChannelCount(u16),

    /// Unsupported sample format for this operation
    #[error("Unsupported sample format: {0:?}")]
    UnsupportedSampleFormat(SampleFormat),

    /// Audio formats are not compatible
    #[error("Incompatible formats")]
    IncompatibleFormats,
}

/// Validate an audio format
pub fn validate_format(format: &AudioFormat) -> Result<(), FormatError> {
    // Check sample rate
    if format.sample_rate == 0 || format.sample_rate > 192_000 {
        return Err(FormatError::InvalidSampleRate(format.sample_rate));
    }

    // Check channel count
    if format.channels == 0 || format.channels > 32 {
        return Err(FormatError::InvalidChannelCount(format.channels));
    }

    // All sample formats are currently supported
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_format_properties() {
        assert_eq!(SampleFormat::I16.size_bytes(), 2);
        assert_eq!(SampleFormat::I24.size_bytes(), 3);
        assert_eq!(SampleFormat::I32.size_bytes(), 4);
        assert_eq!(SampleFormat::F32.size_bytes(), 4);
        assert_eq!(SampleFormat::F64.size_bytes(), 8);

        assert!(!SampleFormat::I16.is_float());
        assert!(SampleFormat::F32.is_float());
        assert!(SampleFormat::I16.is_integer());
        assert!(!SampleFormat::F32.is_integer());
    }

    #[test]
    fn test_audio_format_creation() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        assert_eq!(format.sample_rate, 44100);
        assert_eq!(format.channels, 2);
        assert_eq!(format.sample_format, SampleFormat::F32);
    }

    #[test]
    fn test_audio_format_calculations() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        assert_eq!(format.frame_size(), 8); // 2 channels * 4 bytes
        assert_eq!(format.byte_rate(), 44100 * 8);
    }

    #[test]
    fn test_format_compatibility() {
        let format1 = AudioFormat::new(44100, 2, SampleFormat::F32);
        let format2 = AudioFormat::new(44100, 2, SampleFormat::I16);
        let format3 = AudioFormat::new(48000, 2, SampleFormat::F32);

        assert!(format1.is_compatible_with(&format2));
        assert!(!format1.is_compatible_with(&format3));
    }

    #[test]
    fn test_high_resolution_detection() {
        let cd_quality = AudioFormat::new(44100, 2, SampleFormat::I16);
        let high_res_rate = AudioFormat::new(96000, 2, SampleFormat::I16);
        let high_res_depth = AudioFormat::new(44100, 2, SampleFormat::I24);

        assert!(!cd_quality.is_high_resolution());
        assert!(high_res_rate.is_high_resolution());
        assert!(high_res_depth.is_high_resolution());
    }

    #[test]
    fn test_channel_layout() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
        assert_eq!(ChannelLayout::Surround51.channel_count(), 6);

        assert_eq!(
            ChannelLayout::from_channel_count(1),
            Some(ChannelLayout::Mono)
        );
        assert_eq!(
            ChannelLayout::from_channel_count(2),
            Some(ChannelLayout::Stereo)
        );
        assert_eq!(ChannelLayout::from_channel_count(5), None);
    }

    #[test]
    fn test_format_validation() {
        let valid_format = AudioFormat::new(44100, 2, SampleFormat::F32);
        assert!(validate_format(&valid_format).is_ok());

        let invalid_rate = AudioFormat::new(0, 2, SampleFormat::F32);
        assert!(validate_format(&invalid_rate).is_err());

        let invalid_channels = AudioFormat::new(44100, 0, SampleFormat::F32);
        assert!(validate_format(&invalid_channels).is_err());
    }

    #[test]
    fn test_cpal_conversion() {
        let format = AudioFormat::new(44100, 2, SampleFormat::F32);
        let cpal_config = format.to_cpal_config();

        assert_eq!(cpal_config.channels, 2);
        assert_eq!(cpal_config.sample_rate, 44100);

        let converted_back = AudioFormat::from_cpal_config(&cpal_config, SampleFormat::F32);
        assert_eq!(converted_back.sample_rate, format.sample_rate);
        assert_eq!(converted_back.channels, format.channels);
    }
}
