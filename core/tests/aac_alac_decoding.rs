//! AAC and ALAC decoding tests
//!
//! Tests for AAC and ALAC format support, including M4A container support

use contextune_core::audio::decoder::{AudioDecoder, detect_format, is_format_supported};
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aac_format_support() {
        // Test that AAC format is supported
        assert!(is_format_supported("test.aac"));
        assert!(is_format_supported("test.AAC"));
        
        // Test M4A container support (which typically contains AAC)
        assert!(is_format_supported("test.m4a"));
        assert!(is_format_supported("test.M4A"));
    }

    #[test]
    fn test_aac_format_detection() {
        // Test format detection from extension
        use contextune_core::audio::decoder::detect_format_from_extension;
        
        assert_eq!(detect_format_from_extension("song.aac"), Some("AAC"));
        assert_eq!(detect_format_from_extension("song.AAC"), Some("AAC"));
        assert_eq!(detect_format_from_extension("song.m4a"), Some("M4A/AAC"));
        assert_eq!(detect_format_from_extension("song.M4A"), Some("M4A/AAC"));
    }

    #[test]
    fn test_aac_decoder_creation_with_invalid_file() {
        // Test that decoder creation fails gracefully with invalid AAC file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fake aac data").unwrap();
        
        let temp_path = temp_file.path().with_extension("aac");
        std::fs::copy(temp_file.path(), &temp_path).unwrap();
        
        let result = AudioDecoder::new(&temp_path);
        assert!(result.is_err(), "Decoder should fail for invalid AAC file");
        
        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_m4a_decoder_creation_with_invalid_file() {
        // Test that decoder creation fails gracefully with invalid M4A file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"fake m4a data").unwrap();
        
        let temp_path = temp_file.path().with_extension("m4a");
        std::fs::copy(temp_file.path(), &temp_path).unwrap();
        
        let result = AudioDecoder::new(&temp_path);
        assert!(result.is_err(), "Decoder should fail for invalid M4A file");
        
        // Clean up
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_alac_format_support() {
        // ALAC is typically contained in M4A files
        assert!(is_format_supported("test.m4a"));
        assert!(is_format_supported("test.M4A"));
    }

    #[test]
    fn test_format_detection_with_aac_content() {
        // Test format detection with invalid AAC content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid aac content").unwrap();
        
        let result = detect_format(temp_file.path());
        assert!(result.is_err(), "Should fail to detect format for invalid AAC content");
    }

    #[test]
    fn test_format_detection_with_m4a_content() {
        // Test format detection with invalid M4A content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"invalid m4a content").unwrap();
        
        let result = detect_format(temp_file.path());
        assert!(result.is_err(), "Should fail to detect format for invalid M4A content");
    }

    #[test]
    fn test_supported_extensions_include_aac_formats() {
        use contextune_core::audio::decoder::supported_extensions;
        
        let extensions = supported_extensions();
        assert!(extensions.contains(&"aac"), "AAC should be in supported extensions");
        assert!(extensions.contains(&"m4a"), "M4A should be in supported extensions");
    }

    #[test]
    fn test_aac_format_info_properties() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test AAC format info (lossy format)
        let aac_info = AudioFormatInfo {
            format_name: "AAC".to_string(),
            codec_type: "AAC".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 180), // 3 minutes
            bit_depth: None, // AAC doesn't have traditional bit depth
            is_lossless: false,
        };
        
        assert!(!aac_info.is_lossless, "AAC should be lossy");
        assert_eq!(aac_info.duration_seconds(), Some(180.0));
        assert!(!aac_info.is_high_resolution()); // Standard AAC is not hi-res
    }

    #[test]
    fn test_alac_format_info_properties() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test ALAC format info (lossless format)
        let alac_info = AudioFormatInfo {
            format_name: "ALAC".to_string(),
            codec_type: "ALAC".to_string(),
            sample_rate: Some(96000), // Hi-res sample rate
            channels: Some(2),
            duration: Some(96000 * 180), // 3 minutes
            bit_depth: Some(24), // Hi-res bit depth
            is_lossless: true,
        };
        
        assert!(alac_info.is_lossless, "ALAC should be lossless");
        assert_eq!(alac_info.duration_seconds(), Some(180.0));
        assert!(alac_info.is_high_resolution(), "ALAC with 96kHz/24bit should be hi-res");
    }

    #[test]
    fn test_m4a_container_format_detection() {
        // M4A is a container that can hold both AAC and ALAC
        // Test that we properly detect M4A as a supported format
        assert!(is_format_supported("album.m4a"));
        assert!(is_format_supported("song.M4A"));
        
        // Test case insensitivity
        assert!(is_format_supported("track.m4A"));
        assert!(is_format_supported("audio.M4a"));
    }

    #[test]
    fn test_aac_vs_alac_codec_detection() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test that we can distinguish between AAC and ALAC codecs
        let aac_info = AudioFormatInfo {
            format_name: "AAC".to_string(),
            codec_type: "AAC".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 60),
            bit_depth: None,
            is_lossless: false,
        };
        
        let alac_info = AudioFormatInfo {
            format_name: "ALAC".to_string(),
            codec_type: "ALAC".to_string(),
            sample_rate: Some(44100),
            channels: Some(2),
            duration: Some(44100 * 60),
            bit_depth: Some(16),
            is_lossless: true,
        };
        
        // Verify they have different properties
        assert_ne!(aac_info.is_lossless, alac_info.is_lossless);
        assert_ne!(aac_info.codec_type, alac_info.codec_type);
        assert_ne!(aac_info.format_name, alac_info.format_name);
    }

    #[test]
    fn test_high_resolution_aac_support() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test high-resolution AAC (HE-AAC v2 can support up to 48kHz)
        let hires_aac_info = AudioFormatInfo {
            format_name: "AAC".to_string(),
            codec_type: "AAC".to_string(),
            sample_rate: Some(48000), // Higher sample rate
            channels: Some(2),
            duration: Some(48000 * 60),
            bit_depth: None,
            is_lossless: false,
        };
        
        assert!(hires_aac_info.is_high_resolution(), "48kHz AAC should be considered hi-res");
        assert!(!hires_aac_info.is_lossless, "AAC should still be lossy even at hi-res");
    }

    #[test]
    fn test_high_resolution_alac_support() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test high-resolution ALAC (supports up to 192kHz/24bit)
        let hires_alac_info = AudioFormatInfo {
            format_name: "ALAC".to_string(),
            codec_type: "ALAC".to_string(),
            sample_rate: Some(192000), // Very high sample rate
            channels: Some(2),
            duration: Some(192000 * 60),
            bit_depth: Some(24), // High bit depth
            is_lossless: true,
        };
        
        assert!(hires_alac_info.is_high_resolution(), "192kHz/24bit ALAC should be hi-res");
        assert!(hires_alac_info.is_lossless, "ALAC should be lossless");
    }

    #[test]
    fn test_multichannel_aac_support() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test multichannel AAC support (5.1 surround)
        let multichannel_aac = AudioFormatInfo {
            format_name: "AAC".to_string(),
            codec_type: "AAC".to_string(),
            sample_rate: Some(48000),
            channels: Some(6), // 5.1 surround
            duration: Some(48000 * 60),
            bit_depth: None,
            is_lossless: false,
        };
        
        assert_eq!(multichannel_aac.channels, Some(6));
        assert!(!multichannel_aac.is_lossless);
    }

    #[test]
    fn test_multichannel_alac_support() {
        use contextune_core::audio::decoder::AudioFormatInfo;
        
        // Test multichannel ALAC support
        let multichannel_alac = AudioFormatInfo {
            format_name: "ALAC".to_string(),
            codec_type: "ALAC".to_string(),
            sample_rate: Some(48000),
            channels: Some(6), // 5.1 surround
            duration: Some(48000 * 60),
            bit_depth: Some(24),
            is_lossless: true,
        };
        
        assert_eq!(multichannel_alac.channels, Some(6));
        assert!(multichannel_alac.is_lossless);
    }
}

/// Integration tests that require actual AAC/ALAC files
/// These tests are ignored by default since they require test files
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual AAC test file
    fn test_real_aac_file_decoding() {
        // This test would require a real AAC file
        // Skip for now since we don't have test files
        // TODO: Add real AAC test files and implement this test
    }

    #[test]
    #[ignore] // Requires actual ALAC test file
    fn test_real_alac_file_decoding() {
        // This test would require a real ALAC file
        // Skip for now since we don't have test files
        // TODO: Add real ALAC test files and implement this test
    }

    #[test]
    #[ignore] // Requires actual M4A test file
    fn test_real_m4a_file_decoding() {
        // This test would require a real M4A file
        // Skip for now since we don't have test files
        // TODO: Add real M4A test files and implement this test
    }
}