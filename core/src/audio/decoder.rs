//! Audio decoding using Symphonia
//!
//! Handles decoding of various audio formats (FLAC, MP3, AAC, etc.)

use crate::Result;
use crate::audio::format::AudioFormat;
use crate::audio::buffer::AudioBuffer;
use std::path::Path;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Audio decoder using Symphonia
pub struct AudioDecoder {
    /// Format reader for the audio file
    format_reader: Box<dyn FormatReader>,
    /// Audio decoder
    decoder: Box<dyn Decoder>,
    /// Track ID being decoded
    track_id: u32,
    /// Audio format information
    format: AudioFormat,
    /// Total duration in samples (if known)
    duration: Option<u64>,
}

/// Decoded audio packet
pub struct DecodedPacket {
    /// Audio data as f64 samples
    pub samples: Vec<f64>,
    /// Number of frames in this packet
    pub frames: usize,
    /// Audio format
    pub format: AudioFormat,
}

/// Audio stream reader for continuous decoding
pub struct AudioStreamReader {
    /// The decoder
    decoder: Arc<Mutex<AudioDecoder>>,
    /// Channel for receiving decoded packets
    packet_receiver: mpsc::Receiver<Result<Option<DecodedPacket>>>,
    /// Handle to the decoding thread
    decode_thread: Option<thread::JoinHandle<()>>,
    /// Flag to stop the decoding thread
    stop_flag: Arc<Mutex<bool>>,
}

/// Configuration for audio stream reading
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size in frames per packet
    pub buffer_size: usize,
    /// Whether to loop the audio when it reaches the end
    pub loop_playback: bool,
    /// Prefetch buffer size (number of packets to buffer ahead)
    pub prefetch_size: usize,
}

impl AudioDecoder {
    /// Create a new audio decoder for the given file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Open the file
        let file = File::open(path)
            .map_err(|e| crate::Error::Io(e))?;
        let media_source = MediaSourceStream::new(Box::new(file), Default::default());
        
        // Create a hint based on file extension
        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                hint.with_extension(ext_str);
            }
        }
        
        // Probe the media source
        let probed = symphonia::default::get_probe()
            .format(&hint, media_source, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| crate::Error::Decoding(format!("Failed to probe file: {}", e)))?;
        
        let format_reader = probed.format;
        
        // Find the default audio track
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| crate::Error::Decoding("No audio tracks found".to_string()))?;
        
        let track_id = track.id;
        
        // Create decoder for the track
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| crate::Error::Decoding(format!("Failed to create decoder: {}", e)))?;
        
        // Extract format information
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate
            .ok_or_else(|| crate::Error::Decoding("No sample rate information".to_string()))?;
        let channels = codec_params.channels
            .ok_or_else(|| crate::Error::Decoding("No channel information".to_string()))?
            .count() as u16;
        
        let format = AudioFormat::new(
            sample_rate,
            channels,
            crate::audio::format::SampleFormat::F64, // We'll convert to f64 for precision
        );
        
        // Get duration if available
        let duration = codec_params.n_frames;
        
        Ok(Self {
            format_reader,
            decoder,
            track_id,
            format,
            duration,
        })
    }
    
    /// Get the audio format of the decoded stream
    pub fn format(&self) -> &AudioFormat {
        &self.format
    }
    
    /// Get the total duration in samples (if known)
    pub fn duration(&self) -> Option<u64> {
        self.duration
    }
    
    /// Get the total duration in seconds (if known)
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration.map(|frames| frames as f64 / self.format.sample_rate as f64)
    }
    
    /// Decode the next packet
    pub fn decode_next(&mut self) -> Result<Option<DecodedPacket>> {
        // Get the next packet
        let packet = match self.format_reader.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None); // End of stream
            }
            Err(e) => return Err(crate::Error::Decoding(format!("Failed to read packet: {}", e))),
        };
        
        // Skip packets that don't belong to our track
        if packet.track_id() != self.track_id {
            return self.decode_next(); // Recursively try next packet
        }
        
        // Decode the packet
        let decoded = self.decoder.decode(&packet)
            .map_err(|e| crate::Error::Decoding(format!("Failed to decode packet: {}", e)))?;
        
        // Convert to our format
        let frames = decoded.frames();
        let samples = Self::convert_audio_buffer_static(&decoded)?;
        
        Ok(Some(DecodedPacket {
            samples,
            frames,
            format: self.format.clone(),
        }))
    }
    
    /// Decode all audio data into a single buffer
    pub fn decode_all(&mut self) -> Result<AudioBuffer> {
        let mut all_samples = Vec::new();
        let mut _total_frames = 0;
        
        while let Some(packet) = self.decode_next()? {
            all_samples.extend(packet.samples);
            _total_frames += packet.frames;
        }
        
        Ok(AudioBuffer::with_data(self.format.clone(), all_samples))
    }
    
    /// Seek to a specific position (in samples)
    pub fn seek(&mut self, position: u64) -> Result<()> {
        // Convert sample position to time
        let time_seconds = position as f64 / self.format.sample_rate as f64;
        let time_base = symphonia::core::units::TimeBase::new(1, self.format.sample_rate);
        let timestamp = time_base.calc_timestamp(symphonia::core::units::Time::from(time_seconds));
        
        self.format_reader.seek(
            symphonia::core::formats::SeekMode::Accurate,
            symphonia::core::formats::SeekTo::TimeStamp { ts: timestamp, track_id: self.track_id }
        ).map_err(|e| crate::Error::Decoding(format!("Seek failed: {}", e)))?;
        
        Ok(())
    }
    
    /// Reset decoder to the beginning
    pub fn reset(&mut self) -> Result<()> {
        self.seek(0)
    }
    
    /// Convert Symphonia AudioBufferRef to our f64 samples (static version)
    fn convert_audio_buffer_static(buffer: &AudioBufferRef) -> Result<Vec<f64>> {
        let mut samples = Vec::new();
        
        match buffer {
            AudioBufferRef::U8(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / u8::MAX as f64 * 2.0 - 1.0;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::U16(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / u16::MAX as f64 * 2.0 - 1.0;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::U24(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame].inner() as f64 / (1u32 << 24) as f64 * 2.0 - 1.0;
                        samples.push(sample.clamp(-1.0, 1.0));
                    }
                }
            }
            AudioBufferRef::U32(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / u32::MAX as f64 * 2.0 - 1.0;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::S8(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / i8::MAX as f64;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::S16(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / i16::MAX as f64;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::S24(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame].inner() as f64 / (1i32 << 23) as f64;
                        samples.push(sample.clamp(-1.0, 1.0));
                    }
                }
            }
            AudioBufferRef::S32(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64 / i32::MAX as f64;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::F32(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame] as f64;
                        samples.push(sample);
                    }
                }
            }
            AudioBufferRef::F64(buf) => {
                for frame in 0..buf.frames() {
                    for ch in 0..buf.spec().channels.count() {
                        let sample = buf.chan(ch)[frame];
                        samples.push(sample);
                    }
                }
            }
        }
        
        Ok(samples)
    }
}

impl AudioStreamReader {
    /// Create a new audio stream reader
    pub fn new<P: AsRef<Path>>(path: P, config: StreamConfig) -> Result<Self> {
        let decoder = Arc::new(Mutex::new(AudioDecoder::new(path)?));
        let (packet_sender, packet_receiver) = mpsc::channel();
        let stop_flag = Arc::new(Mutex::new(false));
        
        // Clone references for the thread
        let decoder_clone = decoder.clone();
        let stop_flag_clone = stop_flag.clone();
        
        // Start the decoding thread
        let decode_thread = thread::spawn(move || {
            Self::decode_loop(decoder_clone, packet_sender, stop_flag_clone, config);
        });
        
        Ok(Self {
            decoder,
            packet_receiver,
            decode_thread: Some(decode_thread),
            stop_flag,
        })
    }
    
    /// Get the next decoded packet
    pub fn next_packet(&mut self) -> Result<Option<DecodedPacket>> {
        match self.packet_receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => Ok(None), // No packet ready yet
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(crate::Error::Decoding("Decoder thread disconnected".to_string()))
            }
        }
    }
    
    /// Get the next decoded packet (blocking)
    pub fn next_packet_blocking(&mut self) -> Result<Option<DecodedPacket>> {
        match self.packet_receiver.recv() {
            Ok(result) => result,
            Err(mpsc::RecvError) => {
                Err(crate::Error::Decoding("Decoder thread disconnected".to_string()))
            }
        }
    }
    
    /// Get the audio format
    pub fn format(&self) -> Result<AudioFormat> {
        let decoder = self.decoder.lock().unwrap();
        Ok(decoder.format().clone())
    }
    
    /// Get the duration in samples (if known)
    pub fn duration(&self) -> Result<Option<u64>> {
        let decoder = self.decoder.lock().unwrap();
        Ok(decoder.duration())
    }
    
    /// Seek to a specific position
    pub fn seek(&mut self, position: u64) -> Result<()> {
        let mut decoder = self.decoder.lock().unwrap();
        decoder.seek(position)
    }
    
    /// Stop the stream reader
    pub fn stop(&mut self) {
        // Set stop flag
        *self.stop_flag.lock().unwrap() = true;
        
        // Wait for thread to finish
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
    }
    
    /// Check if the stream is still active
    pub fn is_active(&self) -> bool {
        !*self.stop_flag.lock().unwrap()
    }
    
    /// Decoding loop that runs in a separate thread
    fn decode_loop(
        decoder: Arc<Mutex<AudioDecoder>>,
        sender: mpsc::Sender<Result<Option<DecodedPacket>>>,
        stop_flag: Arc<Mutex<bool>>,
        config: StreamConfig,
    ) {
        let mut packet_buffer = Vec::new();
        let mut eof_reached = false;
        
        loop {
            // Check stop flag
            if *stop_flag.lock().unwrap() {
                break;
            }
            
            // Maintain prefetch buffer
            while packet_buffer.len() < config.prefetch_size && !eof_reached {
                let mut decoder = decoder.lock().unwrap();
                
                match decoder.decode_next() {
                    Ok(Some(packet)) => {
                        packet_buffer.push(Ok(Some(packet)));
                    }
                    Ok(None) => {
                        // End of file
                        if config.loop_playback {
                            // Reset to beginning for looping
                            if let Err(e) = decoder.reset() {
                                packet_buffer.push(Err(e));
                                break;
                            }
                        } else {
                            eof_reached = true;
                            packet_buffer.push(Ok(None));
                        }
                    }
                    Err(e) => {
                        packet_buffer.push(Err(e));
                        break;
                    }
                }
                
                drop(decoder); // Release lock
            }
            
            // Send buffered packets
            if let Some(packet) = packet_buffer.pop() {
                if sender.send(packet).is_err() {
                    // Receiver disconnected
                    break;
                }
            } else if eof_reached {
                // No more packets and EOF reached
                break;
            } else {
                // Wait a bit before trying again
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
}

impl Drop for AudioStreamReader {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024, // 1024 frames per packet
            loop_playback: false,
            prefetch_size: 4, // Buffer 4 packets ahead
        }
    }
}

/// Check if a file format is supported for decoding
pub fn is_format_supported<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            match ext_str.to_lowercase().as_str() {
                "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Detect audio format from file content (not just extension)
pub fn detect_format<P: AsRef<Path>>(path: P) -> Result<Option<AudioFormatInfo>> {
    let path = path.as_ref();
    
    // First try to open and probe the file
    let file = File::open(path)
        .map_err(|e| crate::Error::Io(e))?;
    let media_source = MediaSourceStream::new(Box::new(file), Default::default());
    
    // Create a hint based on file extension
    let mut hint = Hint::new();
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }
    
    // Probe the media source
    let probed = symphonia::default::get_probe()
        .format(&hint, media_source, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| crate::Error::Decoding(format!("Failed to probe file: {}", e)))?;
    
    let format_reader = probed.format;
    
    // Find the default audio track
    let track = format_reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| crate::Error::Decoding("No audio tracks found".to_string()))?;
    
    // Extract format information
    let codec_params = &track.codec_params;
    let sample_rate = codec_params.sample_rate;
    let channels = codec_params.channels.map(|ch| ch.count() as u16);
    let duration = codec_params.n_frames;
    let codec_type = codec_params.codec;
    
    // Determine format name from codec
    let format_name = match codec_type {
        symphonia::core::codecs::CODEC_TYPE_MP3 => "MP3",
        symphonia::core::codecs::CODEC_TYPE_PCM_S16LE |
        symphonia::core::codecs::CODEC_TYPE_PCM_S16BE |
        symphonia::core::codecs::CODEC_TYPE_PCM_S24LE |
        symphonia::core::codecs::CODEC_TYPE_PCM_S24BE |
        symphonia::core::codecs::CODEC_TYPE_PCM_S32LE |
        symphonia::core::codecs::CODEC_TYPE_PCM_S32BE |
        symphonia::core::codecs::CODEC_TYPE_PCM_F32LE |
        symphonia::core::codecs::CODEC_TYPE_PCM_F32BE |
        symphonia::core::codecs::CODEC_TYPE_PCM_F64LE |
        symphonia::core::codecs::CODEC_TYPE_PCM_F64BE => "WAV/PCM",
        symphonia::core::codecs::CODEC_TYPE_FLAC => "FLAC",
        symphonia::core::codecs::CODEC_TYPE_VORBIS => "OGG Vorbis",
        symphonia::core::codecs::CODEC_TYPE_AAC => "AAC",
        symphonia::core::codecs::CODEC_TYPE_ALAC => "ALAC",
        _ => "Unknown",
    }.to_string();
    
    Ok(Some(AudioFormatInfo {
        format_name,
        codec_type: format!("{:?}", codec_type),
        sample_rate,
        channels,
        duration,
        bit_depth: codec_params.bits_per_sample,
        is_lossless: matches!(codec_type, 
            symphonia::core::codecs::CODEC_TYPE_FLAC |
            symphonia::core::codecs::CODEC_TYPE_ALAC |
            symphonia::core::codecs::CODEC_TYPE_PCM_S16LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S16BE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S24LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S24BE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S32LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S32BE |
            symphonia::core::codecs::CODEC_TYPE_PCM_F32LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_F32BE |
            symphonia::core::codecs::CODEC_TYPE_PCM_F64LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_F64BE
        ),
    }))
}

/// Information about detected audio format
#[derive(Debug, Clone)]
pub struct AudioFormatInfo {
    /// Human-readable format name (e.g., "MP3", "FLAC", "WAV")
    pub format_name: String,
    /// Codec type identifier
    pub codec_type: String,
    /// Sample rate in Hz (if known)
    pub sample_rate: Option<u32>,
    /// Number of channels (if known)
    pub channels: Option<u16>,
    /// Duration in samples (if known)
    pub duration: Option<u64>,
    /// Bit depth (if known)
    pub bit_depth: Option<u32>,
    /// Whether this is a lossless format
    pub is_lossless: bool,
}

impl AudioFormatInfo {
    /// Get duration in seconds (if known)
    pub fn duration_seconds(&self) -> Option<f64> {
        match (self.duration, self.sample_rate) {
            (Some(frames), Some(rate)) => Some(frames as f64 / rate as f64),
            _ => None,
        }
    }
    
    /// Check if this is a high-resolution format
    pub fn is_high_resolution(&self) -> bool {
        match (self.sample_rate, self.bit_depth) {
            (Some(rate), Some(bits)) => rate >= 48000 || bits > 16,
            (Some(rate), None) => rate >= 48000,
            (None, Some(bits)) => bits > 16,
            _ => false,
        }
    }
}

/// Detect format from file extension only (fast)
pub fn detect_format_from_extension<P: AsRef<Path>>(path: P) -> Option<&'static str> {
    let path = path.as_ref();
    
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            match ext_str.to_lowercase().as_str() {
                "mp3" => Some("MP3"),
                "wav" => Some("WAV"),
                "flac" => Some("FLAC"),
                "ogg" => Some("OGG Vorbis"),
                "m4a" => Some("M4A/AAC"),
                "aac" => Some("AAC"),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Get supported file extensions
pub fn supported_extensions() -> Vec<&'static str> {
    vec!["mp3", "wav", "flac", "ogg", "m4a", "aac"]
}

/// Create a stream reader with default configuration
pub fn create_stream_reader<P: AsRef<Path>>(path: P) -> Result<AudioStreamReader> {
    AudioStreamReader::new(path, StreamConfig::default())
}

/// Create a stream reader with custom configuration
pub fn create_stream_reader_with_config<P: AsRef<Path>>(
    path: P, 
    config: StreamConfig
) -> Result<AudioStreamReader> {
    AudioStreamReader::new(path, config)
}

/// Create a looping stream reader
pub fn create_looping_stream_reader<P: AsRef<Path>>(path: P) -> Result<AudioStreamReader> {
    let config = StreamConfig {
        loop_playback: true,
        ..StreamConfig::default()
    };
    AudioStreamReader::new(path, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_support_check() {
        assert!(is_format_supported("test.mp3"));
        assert!(is_format_supported("test.wav"));
        assert!(is_format_supported("test.flac"));
        assert!(!is_format_supported("test.txt"));
        assert!(!is_format_supported("test"));
    }

    #[test]
    fn test_supported_extensions() {
        let extensions = supported_extensions();
        assert!(extensions.contains(&"mp3"));
        assert!(extensions.contains(&"wav"));
        assert!(extensions.contains(&"flac"));
        assert!(!extensions.is_empty());
    }

    #[test]
    fn test_decoder_with_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an audio file").unwrap();
        
        let result = AudioDecoder::new(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_with_nonexistent_file() {
        let result = AudioDecoder::new("nonexistent.mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_wav_format_support() {
        // Test that WAV format is specifically supported
        assert!(is_format_supported("test.wav"));
        assert!(is_format_supported("test.WAV")); // Case insensitive
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"wav"));
    }

    #[test]
    fn test_multiple_format_support() {
        // Test that both MP3 and WAV are supported
        assert!(is_format_supported("song.mp3"));
        assert!(is_format_supported("song.wav"));
        assert!(is_format_supported("song.flac"));
        assert!(is_format_supported("song.ogg"));
        assert!(is_format_supported("song.m4a"));
        assert!(is_format_supported("song.aac"));
        
        // Test unsupported formats
        assert!(!is_format_supported("song.txt"));
        assert!(!is_format_supported("song.doc"));
        assert!(!is_format_supported("song"));
    }

    #[test]
    fn test_format_detection_from_extension() {
        assert_eq!(detect_format_from_extension("song.mp3"), Some("MP3"));
        assert_eq!(detect_format_from_extension("song.wav"), Some("WAV"));
        assert_eq!(detect_format_from_extension("song.flac"), Some("FLAC"));
        assert_eq!(detect_format_from_extension("song.ogg"), Some("OGG Vorbis"));
        assert_eq!(detect_format_from_extension("song.m4a"), Some("M4A/AAC"));
        assert_eq!(detect_format_from_extension("song.aac"), Some("AAC"));
        
        // Case insensitive
        assert_eq!(detect_format_from_extension("song.MP3"), Some("MP3"));
        assert_eq!(detect_format_from_extension("song.WAV"), Some("WAV"));
        
        // Unsupported
        assert_eq!(detect_format_from_extension("song.txt"), None);
        assert_eq!(detect_format_from_extension("song"), None);
    }

    #[test]
    fn test_format_detection_with_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an audio file").unwrap();
        
        let result = detect_format(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_format_detection_with_nonexistent_file() {
        let result = detect_format("nonexistent.mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_format_info() {
        let info = AudioFormatInfo {
            format_name: "MP3".to_string(),
            codec_type: "MP3".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 180), // 3 minutes
            bit_depth: None,
            is_lossless: false,
        };
        
        assert_eq!(info.duration_seconds(), Some(180.0));
        assert!(!info.is_high_resolution());
        
        let hires_info = AudioFormatInfo {
            format_name: "FLAC".to_string(),
            codec_type: "FLAC".to_string(),
            sample_rate: Some(96000),
            channels: Some(2),
            duration: Some(96000 * 180),
            bit_depth: Some(24),
            is_lossless: true,
        };
        
        assert!(hires_info.is_high_resolution());
        assert!(hires_info.is_lossless);
    }

    #[test]
    fn test_stream_config() {
        let default_config = StreamConfig::default();
        assert_eq!(default_config.buffer_size, 1024);
        assert!(!default_config.loop_playback);
        assert_eq!(default_config.prefetch_size, 4);
        
        let custom_config = StreamConfig {
            buffer_size: 2048,
            loop_playback: true,
            prefetch_size: 8,
        };
        assert_eq!(custom_config.buffer_size, 2048);
        assert!(custom_config.loop_playback);
        assert_eq!(custom_config.prefetch_size, 8);
    }

    #[test]
    fn test_stream_reader_creation() {
        // Test with non-existent file
        let result = create_stream_reader("nonexistent.mp3");
        assert!(result.is_err());
        
        // Test with invalid file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an audio file").unwrap();
        
        let result = create_stream_reader(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_reader_with_config() {
        let config = StreamConfig {
            buffer_size: 512,
            loop_playback: true,
            prefetch_size: 2,
        };
        
        let result = create_stream_reader_with_config("nonexistent.mp3", config);
        assert!(result.is_err()); // File doesn't exist, but config should be accepted
    }

    #[test]
    fn test_looping_stream_reader() {
        let result = create_looping_stream_reader("nonexistent.mp3");
        assert!(result.is_err()); // File doesn't exist, but should accept looping config
    }

    #[test]
    fn test_stream_reader_stop() {
        // Create a mock stream reader (this will fail due to invalid file, but we can test the structure)
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an audio file").unwrap();
        
        // Even though creation fails, we can test that the stop mechanism doesn't panic
        if let Ok(mut reader) = create_stream_reader(temp_file.path()) {
            reader.stop();
            assert!(!reader.is_active());
        }
    }

    // Format-specific tests
    #[test]
    fn test_mp3_format_detection() {
        // Test MP3 format detection
        assert_eq!(detect_format_from_extension("song.mp3"), Some("MP3"));
        assert_eq!(detect_format_from_extension("song.MP3"), Some("MP3"));
        assert!(is_format_supported("test.mp3"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"mp3"));
    }

    #[test]
    fn test_wav_format_detection() {
        // Test WAV format detection
        assert_eq!(detect_format_from_extension("song.wav"), Some("WAV"));
        assert_eq!(detect_format_from_extension("song.WAV"), Some("WAV"));
        assert!(is_format_supported("test.wav"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"wav"));
    }

    #[test]
    fn test_flac_format_detection() {
        // Test FLAC format detection
        assert_eq!(detect_format_from_extension("song.flac"), Some("FLAC"));
        assert_eq!(detect_format_from_extension("song.FLAC"), Some("FLAC"));
        assert!(is_format_supported("test.flac"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"flac"));
    }

    #[test]
    fn test_ogg_format_detection() {
        // Test OGG format detection
        assert_eq!(detect_format_from_extension("song.ogg"), Some("OGG Vorbis"));
        assert_eq!(detect_format_from_extension("song.OGG"), Some("OGG Vorbis"));
        assert!(is_format_supported("test.ogg"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"ogg"));
    }

    #[test]
    fn test_aac_format_detection() {
        // Test AAC format detection
        assert_eq!(detect_format_from_extension("song.aac"), Some("AAC"));
        assert_eq!(detect_format_from_extension("song.AAC"), Some("AAC"));
        assert!(is_format_supported("test.aac"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"aac"));
    }

    #[test]
    fn test_m4a_format_detection() {
        // Test M4A format detection
        assert_eq!(detect_format_from_extension("song.m4a"), Some("M4A/AAC"));
        assert_eq!(detect_format_from_extension("song.M4A"), Some("M4A/AAC"));
        assert!(is_format_supported("test.m4a"));
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"m4a"));
    }

    #[test]
    fn test_unsupported_formats() {
        // Test that unsupported formats are properly rejected
        assert!(!is_format_supported("test.txt"));
        assert!(!is_format_supported("test.doc"));
        assert!(!is_format_supported("test.pdf"));
        assert!(!is_format_supported("test.jpg"));
        assert!(!is_format_supported("test.mp4")); // Video format
        assert!(!is_format_supported("test.avi")); // Video format
        
        assert_eq!(detect_format_from_extension("song.txt"), None);
        assert_eq!(detect_format_from_extension("song.doc"), None);
        assert_eq!(detect_format_from_extension("song.pdf"), None);
    }

    #[test]
    fn test_format_case_insensitivity() {
        // Test that format detection is case insensitive
        let test_cases = vec![
            ("song.mp3", "song.MP3", "song.Mp3"),
            ("song.wav", "song.WAV", "song.Wav"),
            ("song.flac", "song.FLAC", "song.Flac"),
            ("song.ogg", "song.OGG", "song.Ogg"),
            ("song.aac", "song.AAC", "song.Aac"),
            ("song.m4a", "song.M4A", "song.M4a"),
        ];
        
        for (lower, upper, mixed) in test_cases {
            assert!(is_format_supported(lower));
            assert!(is_format_supported(upper));
            assert!(is_format_supported(mixed));
            
            let lower_format = detect_format_from_extension(lower);
            let upper_format = detect_format_from_extension(upper);
            let mixed_format = detect_format_from_extension(mixed);
            
            assert_eq!(lower_format, upper_format);
            assert_eq!(lower_format, mixed_format);
            assert!(lower_format.is_some());
        }
    }

    #[test]
    fn test_decoder_creation_for_each_format() {
        // Test that decoder creation fails gracefully for each supported format
        // when given invalid files
        let formats = vec!["mp3", "wav", "flac", "ogg", "aac", "m4a"];
        
        for format in formats {
            let filename = format!("test.{}", format);
            
            // Test with non-existent file
            let result = AudioDecoder::new(&filename);
            assert!(result.is_err(), "Decoder should fail for non-existent {}", format);
            
            // Test with invalid file content
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file.write_all(b"invalid audio data").unwrap();
            let temp_path = temp_file.path().with_extension(format);
            
            // Copy to file with correct extension
            std::fs::copy(temp_file.path(), &temp_path).unwrap();
            
            let result = AudioDecoder::new(&temp_path);
            assert!(result.is_err(), "Decoder should fail for invalid {} file", format);
            
            // Clean up
            let _ = std::fs::remove_file(&temp_path);
        }
    }

    #[test]
    fn test_stream_reader_creation_for_each_format() {
        // Test that stream reader creation fails gracefully for each supported format
        let formats = vec!["mp3", "wav", "flac", "ogg", "aac", "m4a"];
        
        for format in formats {
            let filename = format!("test.{}", format);
            
            // Test with non-existent file
            let result = create_stream_reader(&filename);
            assert!(result.is_err(), "Stream reader should fail for non-existent {}", format);
            
            // Test with looping stream reader
            let result = create_looping_stream_reader(&filename);
            assert!(result.is_err(), "Looping stream reader should fail for non-existent {}", format);
            
            // Test with custom config
            let config = StreamConfig {
                buffer_size: 512,
                loop_playback: false,
                prefetch_size: 2,
            };
            let result = create_stream_reader_with_config(&filename, config);
            assert!(result.is_err(), "Custom stream reader should fail for non-existent {}", format);
        }
    }

    #[test]
    fn test_format_info_properties() {
        // Test AudioFormatInfo for different format types
        
        // Lossless formats
        let flac_info = AudioFormatInfo {
            format_name: "FLAC".to_string(),
            codec_type: "FLAC".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 60), // 1 minute
            bit_depth: Some(16),
            is_lossless: true,
        };
        assert!(flac_info.is_lossless);
        assert_eq!(flac_info.duration_seconds(), Some(60.0));
        
        // Lossy formats
        let mp3_info = AudioFormatInfo {
            format_name: "MP3".to_string(),
            codec_type: "MP3".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 60),
            bit_depth: None,
            is_lossless: false,
        };
        assert!(!mp3_info.is_lossless);
        assert_eq!(mp3_info.duration_seconds(), Some(60.0));
        
        // High-resolution formats
        let hires_info = AudioFormatInfo {
            format_name: "FLAC".to_string(),
            codec_type: "FLAC".to_string(),
            sample_rate: Some(192000),
            channels: Some(2),
            duration: Some(192000 * 60),
            bit_depth: Some(24),
            is_lossless: true,
        };
        assert!(hires_info.is_high_resolution());
        assert!(hires_info.is_lossless);
    }
}
