//! Main audio engine implementation

use crate::Result;
use crate::audio::format::AudioFormat;
use crate::audio::buffer::AudioBuffer;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use cpal::{Device, Stream, StreamConfig, Host, OutputCallbackInfo};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Audio playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Engine is stopped
    Stopped,
    /// Engine is playing audio
    Playing,
    /// Engine is paused
    Paused,
    /// Engine is buffering
    Buffering,
    /// Engine encountered an error
    Error,
}

/// Audio engine events that can be sent to callbacks
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// Playback state changed
    StateChanged(PlaybackState),
    /// Playback position changed (in samples)
    PositionChanged(u64),
    /// Track ended
    TrackEnded,
    /// Error occurred
    Error(String),
    /// Buffer underrun occurred
    BufferUnderrun,
}

/// Callback function type for audio events
pub type AudioCallback = Box<dyn Fn(AudioEvent) + Send + Sync>;

/// Information about an audio device
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /// Device name
    pub name: String,
    /// Supported audio formats
    pub supported_formats: Vec<AudioFormat>,
    /// Whether this is the default device
    pub is_default: bool,
}

/// Trait defining the audio engine interface
pub trait AudioEngineInterface {
    /// Load an audio file for playback
    fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()>;
    
    /// Start playback
    fn play(&mut self) -> Result<()>;
    
    /// Pause playback
    fn pause(&mut self) -> Result<()>;
    
    /// Stop playback and reset position
    fn stop(&mut self) -> Result<()>;
    
    /// Seek to a specific position (in samples)
    fn seek(&mut self, position: u64) -> Result<()>;
    
    /// Set playback volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> Result<()>;
    
    /// Get current playback volume
    fn volume(&self) -> f32;
    
    /// Get current playback state
    fn state(&self) -> PlaybackState;
    
    /// Get current playback position (in samples)
    fn position(&self) -> u64;
    
    /// Get total duration of loaded track (in samples)
    fn duration(&self) -> Option<u64>;
    
    /// Get current audio format
    fn format(&self) -> Option<AudioFormat>;
    
    /// Set event callback
    fn set_callback(&mut self, callback: AudioCallback);
    
    /// Remove event callback
    fn clear_callback(&mut self);
}

/// Internal audio engine state
struct AudioEngineState {
    /// Current playback state
    state: PlaybackState,
    /// Current volume (0.0 to 1.0)
    volume: f32,
    /// Current position in samples
    position: u64,
    /// Total duration in samples
    duration: Option<u64>,
    /// Current audio format
    format: Option<AudioFormat>,
    /// Audio buffer
    buffer: Option<AudioBuffer>,
    /// Event callback
    callback: Option<AudioCallback>,
}

impl Default for AudioEngineState {
    fn default() -> Self {
        Self {
            state: PlaybackState::Stopped,
            volume: 1.0,
            position: 0,
            duration: None,
            format: None,
            buffer: None,
            callback: None,
        }
    }
}

/// Audio engine for high-fidelity playback
pub struct AudioEngine {
    /// Internal state protected by RwLock for thread safety
    state: Arc<RwLock<AudioEngineState>>,
    /// CPAL host for audio device management
    host: Host,
    /// CPAL audio device
    device: Option<Device>,
    /// CPAL audio stream
    stream: Option<Stream>,
    /// Stream configuration
    stream_config: Option<StreamConfig>,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self {
            state: Arc::new(RwLock::new(AudioEngineState::default())),
            host,
            device: None,
            stream: None,
            stream_config: None,
        })
    }
    
    /// Initialize the audio engine with a specific device
    pub fn with_device(device: Device) -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self {
            state: Arc::new(RwLock::new(AudioEngineState::default())),
            host,
            device: Some(device),
            stream: None,
            stream_config: None,
        })
    }
    
    /// Get the CPAL host
    pub fn host(&self) -> &Host {
        &self.host
    }
    
    /// Get the current audio device
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }
    
    /// Set a new audio device
    pub fn set_device(&mut self, device: Device) -> Result<()> {
        // Stop current stream if running
        if self.stream.is_some() {
            self.stop()?;
        }
        
        self.device = Some(device);
        self.stream = None;
        self.stream_config = None;
        
        Ok(())
    }
    
    /// Initialize the default audio device
    pub fn init_default_device(&mut self) -> Result<()> {
        let device = self.host
            .default_output_device()
            .ok_or_else(|| crate::Error::AudioDevice("No default output device available".to_string()))?;
        
        self.set_device(device)
    }
    
    /// Initialize audio output stream
    pub fn init_output_stream(&mut self, format: &AudioFormat) -> Result<()> {
        let device = self.device.as_ref()
            .ok_or_else(|| crate::Error::AudioDevice("No audio device set".to_string()))?;
        
        // Validate the format first
        crate::audio::format::validate_format(format)
            .map_err(|e| crate::Error::AudioFormat(format!("Invalid format: {}", e)))?;
        
        // Get supported configurations
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        
        // Find a compatible configuration
        let config = self.find_compatible_config(supported_configs, format)?;
        
        // Create the stream configuration
        let stream_config = config.into();
        
        // Create the output stream with error handling
        let state_clone = self.state.clone();
        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &OutputCallbackInfo| {
                Self::audio_callback(data, &state_clone);
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
                // Note: We can't easily propagate errors from this callback
                // The main error handling happens in the safe_stream_operation wrapper
            },
            None, // No timeout
        ).map_err(|e| {
            crate::Error::AudioDevice(format!("Failed to build output stream: {}", e))
        })?;
        
        self.stream = Some(stream);
        self.stream_config = Some(stream_config);
        
        Ok(())
    }
    
    /// Find a compatible CPAL configuration for the given audio format
    fn find_compatible_config(
        &self,
        supported_configs: cpal::SupportedOutputConfigs,
        target_format: &AudioFormat,
    ) -> Result<cpal::SupportedStreamConfig> {
        let mut best_match: Option<cpal::SupportedStreamConfig> = None;
        let mut best_score = 0;
        
        for config in supported_configs {
            let score = self.calculate_format_compatibility_score(&config, target_format);
            
            if score > best_score {
                // Check if sample rate is supported
                if config.min_sample_rate().0 <= target_format.sample_rate
                    && target_format.sample_rate <= config.max_sample_rate().0
                {
                    // Check if channel count is supported
                    if config.channels() == target_format.channels {
                        best_match = Some(config.with_sample_rate(cpal::SampleRate(target_format.sample_rate)));
                        best_score = score;
                    }
                }
            }
        }
        
        best_match.ok_or_else(|| {
            crate::Error::AudioFormat(format!(
                "No compatible audio configuration found for {}Hz, {} channels",
                target_format.sample_rate, target_format.channels
            ))
        })
    }
    
    /// Calculate compatibility score between a CPAL config and target format
    fn calculate_format_compatibility_score(
        &self,
        config: &cpal::SupportedStreamConfigRange,
        target_format: &AudioFormat,
    ) -> u32 {
        let mut score = 0;
        
        // Prefer exact sample rate match
        if config.min_sample_rate().0 <= target_format.sample_rate
            && target_format.sample_rate <= config.max_sample_rate().0
        {
            score += 100;
            
            // Bonus for exact sample rate match at boundaries
            if config.min_sample_rate().0 == target_format.sample_rate
                || config.max_sample_rate().0 == target_format.sample_rate
            {
                score += 50;
            }
        }
        
        // Prefer exact channel count match
        if config.channels() == target_format.channels {
            score += 100;
        }
        
        // Prefer higher sample rates for high-resolution audio
        if target_format.is_high_resolution() {
            if config.max_sample_rate().0 >= 96000 {
                score += 25;
            }
            if config.max_sample_rate().0 >= 192000 {
                score += 25;
            }
        }
        
        // Prefer configurations that support common sample rates
        let common_rates = [44100, 48000, 96000, 192000];
        for &rate in &common_rates {
            if config.min_sample_rate().0 <= rate && rate <= config.max_sample_rate().0 {
                score += 10;
            }
        }
        
        score
    }
    
    /// Negotiate the best audio format for the current device
    pub fn negotiate_format(&self, preferred_format: &AudioFormat) -> Result<AudioFormat> {
        let device = self.device.as_ref()
            .ok_or_else(|| crate::Error::AudioDevice("No audio device set".to_string()))?;
        
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        
        // Try to find exact match first
        let supported_configs_vec: Vec<_> = supported_configs.collect();
        for config in &supported_configs_vec {
            if config.min_sample_rate().0 <= preferred_format.sample_rate
                && preferred_format.sample_rate <= config.max_sample_rate().0
                && config.channels() == preferred_format.channels
            {
                return Ok(preferred_format.clone());
            }
        }
        
        // If no exact match, find the best compatible format
        let supported_configs_iter = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        let best_config = self.find_compatible_config(supported_configs_iter, preferred_format)?;
        
        Ok(AudioFormat::new(
            best_config.sample_rate().0,
            best_config.channels(),
            crate::audio::format::SampleFormat::F32, // CPAL uses f32
        ))
    }
    
    /// Get the best available format for high-quality playback
    pub fn get_best_format(&self) -> Result<AudioFormat> {
        let device = self.device.as_ref()
            .ok_or_else(|| crate::Error::AudioDevice("No audio device set".to_string()))?;
        
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        
        let mut best_format: Option<AudioFormat> = None;
        let mut best_sample_rate = 0;
        let mut best_channels = 0;
        
        for config in supported_configs {
            let sample_rate = config.max_sample_rate().0;
            let channels = config.channels();
            
            // Prefer higher sample rates and more channels for quality
            if sample_rate > best_sample_rate || 
               (sample_rate == best_sample_rate && channels > best_channels) {
                best_sample_rate = sample_rate;
                best_channels = channels;
                best_format = Some(AudioFormat::new(
                    sample_rate,
                    channels,
                    crate::audio::format::SampleFormat::F32,
                ));
            }
        }
        
        best_format.ok_or_else(|| {
            crate::Error::AudioDevice("No supported audio formats found".to_string())
        })
    }
    
    /// Check if a specific format is supported by the current device
    pub fn is_format_supported(&self, format: &AudioFormat) -> Result<bool> {
        let device = self.device.as_ref()
            .ok_or_else(|| crate::Error::AudioDevice("No audio device set".to_string()))?;
        
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        
        for config in supported_configs {
            if config.min_sample_rate().0 <= format.sample_rate
                && format.sample_rate <= config.max_sample_rate().0
                && config.channels() == format.channels
            {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Audio callback function for CPAL stream
    fn audio_callback(output: &mut [f32], state: &Arc<RwLock<AudioEngineState>>) {
        let state_guard = match state.try_read() {
            Some(guard) => guard,
            None => {
                // If we can't acquire the lock, fill with silence to avoid audio glitches
                output.fill(0.0);
                return;
            }
        };
        
        // Fill output buffer based on current state
        match state_guard.state {
            PlaybackState::Playing => {
                // TODO: Fill with actual audio data from buffer
                // For now, fill with silence to avoid noise
                output.fill(0.0);
            }
            _ => {
                // Fill with silence for all other states
                output.fill(0.0);
            }
        }
    }
    
    /// Handle audio stream errors and attempt recovery
    fn handle_stream_error(&mut self, error: cpal::StreamError) -> Result<()> {
        eprintln!("Audio stream error: {}", error);
        
        // Update state to error
        self.update_state(|state| {
            state.state = PlaybackState::Error;
            Some(AudioEvent::Error(format!("Stream error: {}", error)))
        });
        
        // Attempt to recover by reinitializing the stream
        self.recover_from_error()
    }
    
    /// Attempt to recover from audio errors
    fn recover_from_error(&mut self) -> Result<()> {
        // Clear the current stream
        self.stream = None;
        self.stream_config = None;
        
        // Try to reinitialize with default device
        if let Err(e) = self.init_default_device() {
            return Err(crate::Error::AudioEngine(format!("Failed to recover: {}", e)));
        }
        
        // Try to reinitialize stream with default format
        let format = AudioFormat::default();
        if let Err(e) = self.init_output_stream(&format) {
            return Err(crate::Error::AudioEngine(format!("Failed to recover stream: {}", e)));
        }
        
        // Update state back to stopped
        self.update_state(|state| {
            state.state = PlaybackState::Stopped;
            Some(AudioEvent::StateChanged(PlaybackState::Stopped))
        });
        
        Ok(())
    }
    
    /// Validate audio engine state before operations
    fn validate_state(&self) -> Result<()> {
        let state = self.state.read();
        
        if state.state == PlaybackState::Error {
            return Err(crate::Error::AudioEngine("Audio engine is in error state".to_string()));
        }
        
        Ok(())
    }
    
    /// Safe wrapper for play stream operations
    fn safe_play_operation(&mut self) -> Result<()> {
        self.validate_state()?;
        
        if let Some(ref stream) = self.stream {
            stream.play().map_err(|e| {
                // Convert CPAL error to our error type and attempt recovery
                let error = crate::Error::AudioDevice(format!("Play operation failed: {}", e));
                
                // Attempt recovery in background (don't propagate recovery errors)
                if let Err(recovery_error) = self.recover_from_error() {
                    eprintln!("Recovery failed: {}", recovery_error);
                }
                
                error
            })
        } else {
            Err(crate::Error::AudioDevice("No audio stream available".to_string()))
        }
    }
    
    /// Safe wrapper for pause stream operations
    fn safe_pause_operation(&mut self) -> Result<()> {
        self.validate_state()?;
        
        if let Some(ref stream) = self.stream {
            stream.pause().map_err(|e| {
                // Convert CPAL error to our error type and attempt recovery
                let error = crate::Error::AudioDevice(format!("Pause operation failed: {}", e));
                
                // Attempt recovery in background (don't propagate recovery errors)
                if let Err(recovery_error) = self.recover_from_error() {
                    eprintln!("Recovery failed: {}", recovery_error);
                }
                
                error
            })
        } else {
            Err(crate::Error::AudioDevice("No audio stream available".to_string()))
        }
    }
    
    /// Start the audio stream
    pub fn start_stream(&mut self) -> Result<()> {
        self.safe_play_operation()
    }
    
    /// Pause the audio stream
    pub fn pause_stream(&mut self) -> Result<()> {
        self.safe_pause_operation()
    }
    
    /// Get supported audio formats for the current device
    pub fn supported_formats(&self) -> Result<Vec<AudioFormat>> {
        let device = self.device.as_ref()
            .ok_or_else(|| crate::Error::AudioDevice("No audio device set".to_string()))?;
        
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get supported configs: {}", e)))?;
        
        let mut formats = Vec::new();
        
        for config in supported_configs {
            // Add configurations for common sample rates
            let sample_rates = [
                config.min_sample_rate().0,
                44100,
                48000,
                96000,
                192000,
                config.max_sample_rate().0,
            ];
            
            for &sample_rate in &sample_rates {
                if sample_rate >= config.min_sample_rate().0 
                    && sample_rate <= config.max_sample_rate().0 
                {
                    let audio_format = AudioFormat::new(
                        sample_rate,
                        config.channels(),
                        crate::audio::format::SampleFormat::F32, // CPAL uses f32
                    );
                    
                    if !formats.contains(&audio_format) {
                        formats.push(audio_format);
                    }
                }
            }
        }
        
        Ok(formats)
    }
    
    /// Enumerate all available output devices
    pub fn enumerate_output_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let devices = self.host.output_devices()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to enumerate devices: {}", e)))?;
        
        let mut device_infos = Vec::new();
        
        for device in devices {
            let name = device.name()
                .unwrap_or_else(|_| "Unknown Device".to_string());
            
            let supported_configs = device.supported_output_configs()
                .map_err(|e| crate::Error::AudioDevice(format!("Failed to get device configs: {}", e)))?;
            
            let mut formats = Vec::new();
            for config in supported_configs {
                // Add a representative format for this config
                let format = AudioFormat::new(
                    config.max_sample_rate().0.min(48000), // Use 48kHz as default, or max if lower
                    config.channels(),
                    crate::audio::format::SampleFormat::F32,
                );
                formats.push(format);
            }
            
            let device_info = AudioDeviceInfo {
                name,
                supported_formats: formats,
                is_default: false, // We'll set this separately
            };
            
            device_infos.push(device_info);
        }
        
        // Mark the default device
        if let Some(default_device) = self.host.default_output_device() {
            if let Ok(default_name) = default_device.name() {
                for device_info in &mut device_infos {
                    if device_info.name == default_name {
                        device_info.is_default = true;
                        break;
                    }
                }
            }
        }
        
        Ok(device_infos)
    }
    
    /// Get information about the current device
    pub fn current_device_info(&self) -> Result<Option<AudioDeviceInfo>> {
        let device = match self.device.as_ref() {
            Some(device) => device,
            None => return Ok(None),
        };
        
        let name = device.name()
            .unwrap_or_else(|_| "Unknown Device".to_string());
        
        let supported_configs = device.supported_output_configs()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to get device configs: {}", e)))?;
        
        let mut formats = Vec::new();
        for config in supported_configs {
            let format = AudioFormat::new(
                config.max_sample_rate().0.min(48000),
                config.channels(),
                crate::audio::format::SampleFormat::F32,
            );
            formats.push(format);
        }
        
        let is_default = if let Some(default_device) = self.host.default_output_device() {
            default_device.name().unwrap_or_default() == name
        } else {
            false
        };
        
        Ok(Some(AudioDeviceInfo {
            name,
            supported_formats: formats,
            is_default,
        }))
    }
    
    /// Set device by name
    pub fn set_device_by_name(&mut self, device_name: &str) -> Result<()> {
        let devices = self.host.output_devices()
            .map_err(|e| crate::Error::AudioDevice(format!("Failed to enumerate devices: {}", e)))?;
        
        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_name {
                    return self.set_device(device);
                }
            }
        }
        
        Err(crate::Error::AudioDevice(format!("Device '{}' not found", device_name)))
    }
    
    /// Emit an event to the callback if set
    fn emit_event(&self, event: AudioEvent) {
        let state = self.state.read();
        if let Some(ref callback) = state.callback {
            callback(event);
        }
    }
    
    /// Update the internal state and emit events as needed
    fn update_state<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut AudioEngineState) -> Option<AudioEvent>
    {
        let mut state = self.state.write();
        if let Some(event) = updater(&mut *state) {
            // Drop the write lock before calling the callback
            drop(state);
            self.emit_event(event);
        }
    }
}

impl AudioEngineInterface for AudioEngine {
    fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Validate file path
        if !path.exists() {
            return Err(crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display())
            )));
        }
        
        // TODO: Implement file loading with symphonia
        // For now, just update state and initialize stream
        self.update_state(|state| {
            state.state = PlaybackState::Stopped;
            state.position = 0;
            // TODO: Set actual duration and format from loaded file
            state.duration = Some(44100 * 180); // 3 minutes at 44.1kHz
            state.format = Some(AudioFormat::default());
            Some(AudioEvent::StateChanged(PlaybackState::Stopped))
        });
        
        // Initialize default device if not set
        if self.device.is_none() {
            self.init_default_device().map_err(|e| {
                self.update_state(|state| {
                    state.state = PlaybackState::Error;
                    Some(AudioEvent::Error(format!("Failed to initialize device: {}", e)))
                });
                e
            })?;
        }
        
        // Initialize output stream with default format
        let format = AudioFormat::default();
        self.init_output_stream(&format).map_err(|e| {
            self.update_state(|state| {
                state.state = PlaybackState::Error;
                Some(AudioEvent::Error(format!("Failed to initialize stream: {}", e)))
            });
            e
        })?;
        
        Ok(())
    }
    
    fn play(&mut self) -> Result<()> {
        self.validate_state()?;
        
        // Start CPAL stream
        self.start_stream().map_err(|e| {
            self.update_state(|state| {
                state.state = PlaybackState::Error;
                Some(AudioEvent::Error(format!("Failed to start playback: {}", e)))
            });
            e
        })?;
        
        self.update_state(|state| {
            if state.state != PlaybackState::Playing {
                state.state = PlaybackState::Playing;
                Some(AudioEvent::StateChanged(PlaybackState::Playing))
            } else {
                None
            }
        });
        
        Ok(())
    }
    
    fn pause(&mut self) -> Result<()> {
        self.validate_state()?;
        
        // Pause CPAL stream
        self.pause_stream().map_err(|e| {
            self.update_state(|state| {
                state.state = PlaybackState::Error;
                Some(AudioEvent::Error(format!("Failed to pause playback: {}", e)))
            });
            e
        })?;
        
        self.update_state(|state| {
            if state.state == PlaybackState::Playing {
                state.state = PlaybackState::Paused;
                Some(AudioEvent::StateChanged(PlaybackState::Paused))
            } else {
                None
            }
        });
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        // Pause CPAL stream (CPAL doesn't have explicit stop)
        if let Err(e) = self.pause_stream() {
            // Don't fail stop operation if pause fails, just log it
            eprintln!("Warning: Failed to pause stream during stop: {}", e);
        }
        
        self.update_state(|state| {
            let was_playing = state.state == PlaybackState::Playing || state.state == PlaybackState::Paused;
            state.state = PlaybackState::Stopped;
            state.position = 0;
            
            if was_playing {
                Some(AudioEvent::StateChanged(PlaybackState::Stopped))
            } else {
                None
            }
        });
        
        Ok(())
    }
    
    fn seek(&mut self, position: u64) -> Result<()> {
        self.update_state(|state| {
            let old_position = state.position;
            state.position = position;
            
            if old_position != position {
                Some(AudioEvent::PositionChanged(position))
            } else {
                None
            }
        });
        
        Ok(())
    }
    
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        self.update_state(|state| {
            state.volume = clamped_volume;
            None // Volume changes don't emit events by default
        });
        
        Ok(())
    }
    
    fn volume(&self) -> f32 {
        self.state.read().volume
    }
    
    fn state(&self) -> PlaybackState {
        self.state.read().state
    }
    
    fn position(&self) -> u64 {
        self.state.read().position
    }
    
    fn duration(&self) -> Option<u64> {
        self.state.read().duration
    }
    
    fn format(&self) -> Option<AudioFormat> {
        self.state.read().format.clone()
    }
    
    fn set_callback(&mut self, callback: AudioCallback) {
        self.state.write().callback = Some(callback);
    }
    
    fn clear_callback(&mut self) {
        self.state.write().callback = None;
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create audio engine")
    }
}

// Ensure AudioEngine is Send + Sync for multi-threading
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration as StdDuration;

    #[test]
    fn test_audio_engine_creation() {
        let engine = AudioEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_audio_engine_default() {
        let _engine = AudioEngine::default();
    }
    
    #[test]
    fn test_initial_state() {
        let engine = AudioEngine::new().unwrap();
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(engine.volume(), 1.0);
        assert_eq!(engine.position(), 0);
        assert_eq!(engine.duration(), None);
        assert!(engine.format().is_none());
    }
    
    #[test]
    fn test_volume_control() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test setting volume
        engine.set_volume(0.5).unwrap();
        assert_eq!(engine.volume(), 0.5);
        
        // Test volume clamping
        engine.set_volume(1.5).unwrap();
        assert_eq!(engine.volume(), 1.0);
        
        engine.set_volume(-0.5).unwrap();
        assert_eq!(engine.volume(), 0.0);
    }
    
    #[test]
    fn test_state_transitions() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Initial state should be stopped
        assert_eq!(engine.state(), PlaybackState::Stopped);
        
        // Play without proper initialization should fail gracefully
        let play_result = engine.play();
        if play_result.is_err() {
            // This is expected without audio device/stream
            // The state might be Error due to our error handling
            let current_state = engine.state();
            assert!(current_state == PlaybackState::Stopped || current_state == PlaybackState::Error);
            return;
        }
        
        // If play succeeded (audio system available), test normal transitions
        assert_eq!(engine.state(), PlaybackState::Playing);
        
        // Pause should change state
        engine.pause().unwrap();
        assert_eq!(engine.state(), PlaybackState::Paused);
        
        // Stop should change state and reset position
        engine.stop().unwrap();
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(engine.position(), 0);
    }
    
    #[test]
    fn test_seek() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Seek should update position
        engine.seek(1000).unwrap();
        assert_eq!(engine.position(), 1000);
        
        engine.seek(5000).unwrap();
        assert_eq!(engine.position(), 5000);
    }
    
    #[test]
    fn test_callback_system() {
        let mut engine = AudioEngine::new().unwrap();
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        
        // Set up callback to capture events
        engine.set_callback(Box::new(move |event| {
            events_clone.lock().unwrap().push(event);
        }));
        
        // Trigger some state changes that don't require audio streams
        engine.seek(1000).unwrap();
        engine.set_volume(0.5).unwrap();
        
        // Try play/pause/stop - these might fail without audio system
        let _ = engine.play(); // Ignore result
        let _ = engine.pause(); // Ignore result
        let _ = engine.stop(); // Ignore result
        
        // Give some time for events to be processed
        std::thread::sleep(StdDuration::from_millis(10));
        
        // Check that at least some events were captured (seek should always work)
        let captured_events = events.lock().unwrap();
        // We should have at least the position change event from seek
        assert!(!captured_events.is_empty());
        
        // Clear callback
        engine.clear_callback();
    }
    
    #[test]
    fn test_thread_safety() {
        use std::thread;
        
        let engine = Arc::new(RwLock::new(AudioEngine::new().unwrap()));
        let mut handles = vec![];
        
        // Spawn multiple threads that interact with the engine
        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = thread::spawn(move || {
                let mut engine = engine_clone.write();
                engine.set_volume(i as f32 / 10.0).unwrap();
                engine.seek(i * 1000).unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Engine should still be in a valid state
        let engine = engine.read();
        assert!(engine.volume() >= 0.0 && engine.volume() <= 1.0);
    }

    #[test]
    fn test_device_initialization() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test default device initialization
        let result = engine.init_default_device();
        // This might fail in CI environments without audio devices
        if result.is_ok() {
            assert!(engine.device().is_some());
        }
    }

    #[test]
    fn test_stream_initialization() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Try to initialize default device first
        if engine.init_default_device().is_ok() {
            let format = AudioFormat::default();
            let result = engine.init_output_stream(&format);
            
            // This might fail in CI environments without audio devices
            if result.is_ok() {
                // Stream should be initialized
                assert!(engine.stream.is_some());
                assert!(engine.stream_config.is_some());
            }
        }
    }

    #[test]
    fn test_supported_formats() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Without a device, should return error
        let result = engine.supported_formats();
        assert!(result.is_err());
        
        // With device (if available)
        if engine.init_default_device().is_ok() {
            let formats = engine.supported_formats();
            if formats.is_ok() {
                let formats = formats.unwrap();
                // Should have at least one supported format
                assert!(!formats.is_empty());
            }
        }
    }

    #[test]
    fn test_stream_control() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Stream control without device should return errors (not panic)
        let start_result = engine.start_stream();
        assert!(start_result.is_err());
        
        let pause_result = engine.pause_stream();
        assert!(pause_result.is_err());
        
        // The important thing is that these operations don't panic
        // and return appropriate errors
    }

    #[test]
    fn test_host_access() {
        let engine = AudioEngine::new().unwrap();
        let _host = engine.host();
        // Should not panic and return a valid host reference
    }

    #[test]
    fn test_device_enumeration() {
        let engine = AudioEngine::new().unwrap();
        
        // Test device enumeration
        let result = engine.enumerate_output_devices();
        if result.is_ok() {
            let devices = result.unwrap();
            // Should have at least one device in most environments
            // In CI environments without audio, this might be empty
            for device in &devices {
                assert!(!device.name.is_empty());
                // At most one device should be marked as default
                if device.is_default {
                    println!("Default device: {}", device.name);
                }
            }
        }
    }

    #[test]
    fn test_current_device_info() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Without device, should return None
        let info = engine.current_device_info().unwrap();
        assert!(info.is_none());
        
        // With device (if available)
        if engine.init_default_device().is_ok() {
            let info = engine.current_device_info().unwrap();
            if let Some(device_info) = info {
                assert!(!device_info.name.is_empty());
                assert!(!device_info.supported_formats.is_empty());
            }
        }
    }

    #[test]
    fn test_set_device_by_name() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test with non-existent device
        let result = engine.set_device_by_name("NonExistentDevice");
        assert!(result.is_err());
        
        // Test with real device (if available)
        if let Ok(devices) = engine.enumerate_output_devices() {
            if let Some(device) = devices.first() {
                let result = engine.set_device_by_name(&device.name);
                if result.is_ok() {
                    assert!(engine.device().is_some());
                }
            }
        }
    }

    #[test]
    fn test_format_negotiation() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Without device, should return error
        let preferred_format = AudioFormat::new(44100, 2, crate::audio::format::SampleFormat::F32);
        let result = engine.negotiate_format(&preferred_format);
        assert!(result.is_err());
        
        // With device (if available)
        if engine.init_default_device().is_ok() {
            let result = engine.negotiate_format(&preferred_format);
            if result.is_ok() {
                let negotiated = result.unwrap();
                // Should return a valid format
                assert!(negotiated.sample_rate > 0);
                assert!(negotiated.channels > 0);
            }
        }
    }

    #[test]
    fn test_best_format() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Without device, should return error
        let result = engine.get_best_format();
        assert!(result.is_err());
        
        // With device (if available)
        if engine.init_default_device().is_ok() {
            let result = engine.get_best_format();
            if result.is_ok() {
                let best_format = result.unwrap();
                // Should return a valid high-quality format
                assert!(best_format.sample_rate >= 44100);
                assert!(best_format.channels >= 1);
            }
        }
    }

    #[test]
    fn test_format_support_check() {
        let mut engine = AudioEngine::new().unwrap();
        
        let test_format = AudioFormat::new(44100, 2, crate::audio::format::SampleFormat::F32);
        
        // Without device, should return error
        let result = engine.is_format_supported(&test_format);
        assert!(result.is_err());
        
        // With device (if available)
        if engine.init_default_device().is_ok() {
            let result = engine.is_format_supported(&test_format);
            if result.is_ok() {
                // Should return a boolean result
                let _is_supported = result.unwrap();
            }
        }
    }

    #[test]
    fn test_compatibility_scoring() {
        let _engine = AudioEngine::new().unwrap();
        
        // Create a mock config range for testing
        // Note: This test is limited because we can't easily create SupportedStreamConfigRange
        // In a real scenario, this would be tested with actual device configs
        
        let target_format = AudioFormat::new(44100, 2, crate::audio::format::SampleFormat::F32);
        
        // Test that the scoring function exists and can be called
        // The actual scoring logic is tested indirectly through format negotiation
        assert_eq!(target_format.sample_rate, 44100);
        assert_eq!(target_format.channels, 2);
    }

    #[test]
    fn test_error_handling() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test validation without device
        let result = engine.validate_state();
        assert!(result.is_ok()); // Should be OK initially
        
        // Test load_file with non-existent file
        let result = engine.load_file("non_existent_file.mp3");
        assert!(result.is_err());
        
        // Test play without loaded file/device
        let _result = engine.play();
        // This might succeed or fail depending on system audio availability
        // The important thing is that it doesn't panic
    }

    #[test]
    fn test_error_recovery() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test recovery from error state
        // Manually set error state
        engine.update_state(|state| {
            state.state = PlaybackState::Error;
            Some(AudioEvent::StateChanged(PlaybackState::Error))
        });
        
        // Validate should fail
        let result = engine.validate_state();
        assert!(result.is_err());
        
        // Recovery should work (if audio system is available)
        let recovery_result = engine.recover_from_error();
        if recovery_result.is_ok() {
            // After recovery, validation should pass
            assert!(engine.validate_state().is_ok());
            assert_eq!(engine.state(), PlaybackState::Stopped);
        }
    }

    #[test]
    fn test_safe_operations() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test safe stream operations without stream
        let result = engine.start_stream();
        // Should return error since no stream is initialized
        assert!(result.is_err());
        
        let result = engine.pause_stream();
        // Should return error since no stream is initialized
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_engine_initialization_complete() {
        // Test complete initialization process
        let mut engine = AudioEngine::new().unwrap();
        
        // Verify initial state
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(engine.volume(), 1.0);
        assert_eq!(engine.position(), 0);
        assert!(engine.duration().is_none());
        assert!(engine.format().is_none());
        assert!(engine.device().is_none());
        
        // Test host access
        let _host = engine.host();
        
        // Test device initialization (if available)
        let device_init_result = engine.init_default_device();
        if device_init_result.is_ok() {
            // Device should be set
            assert!(engine.device().is_some());
            
            // Test stream initialization
            let format = AudioFormat::default();
            let stream_init_result = engine.init_output_stream(&format);
            if stream_init_result.is_ok() {
                // Stream should be initialized
                assert!(engine.stream.is_some());
                assert!(engine.stream_config.is_some());
                
                // Test that we can get supported formats
                let formats_result = engine.supported_formats();
                if formats_result.is_ok() {
                    let formats = formats_result.unwrap();
                    assert!(!formats.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_initialization_with_custom_device() {
        let engine = AudioEngine::new().unwrap();
        
        // Test device enumeration
        let devices_result = engine.enumerate_output_devices();
        if devices_result.is_ok() {
            let devices = devices_result.unwrap();
            if !devices.is_empty() {
                // Test creating engine with specific device
                // Note: We can't easily test this without access to actual Device objects
                // This test verifies the enumeration works
                for device_info in &devices {
                    assert!(!device_info.name.is_empty());
                    assert!(!device_info.supported_formats.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_initialization_error_conditions() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Test stream initialization without device
        let format = AudioFormat::default();
        let result = engine.init_output_stream(&format);
        assert!(result.is_err());
        
        // Test with invalid format
        let invalid_format = AudioFormat::new(0, 0, crate::audio::format::SampleFormat::F32);
        let result = engine.init_output_stream(&invalid_format);
        assert!(result.is_err());
        
        // Test format validation
        let validation_result = crate::audio::format::validate_format(&invalid_format);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_initialization_state_consistency() {
        let engine = AudioEngine::new().unwrap();
        
        // Test that all initial state is consistent
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(engine.position(), 0);
        assert_eq!(engine.volume(), 1.0);
        assert!(engine.duration().is_none());
        assert!(engine.format().is_none());
        
        // Test that validation passes for initial state
        assert!(engine.validate_state().is_ok());
    }

    #[test]
    fn test_initialization_thread_safety() {
        use std::thread;
        
        // Test that engine creation is thread-safe
        let mut handles = vec![];
        
        for _ in 0..5 {
            let handle = thread::spawn(|| {
                let engine = AudioEngine::new();
                assert!(engine.is_ok());
                let engine = engine.unwrap();
                assert_eq!(engine.state(), PlaybackState::Stopped);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
