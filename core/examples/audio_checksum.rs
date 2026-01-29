//! Audio checksum calculation tool
//!
//! Command-line utility to calculate checksums of audio files.
//!
//! Usage:
//!   cargo run --example audio_checksum -- <audio_file> [algorithm]
//!
//! Algorithms: simple, crc32, md5, sha256 (default: simple)

use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    let file_path = &args[1];
    let algorithm = if args.len() >= 3 {
        parse_algorithm(&args[2])?
    } else {
        contexture_core::audio::checksum::ChecksumAlgorithm::Simple
    };

    println!("Audio Checksum Calculator");
    println!("=========================");
    println!();
    println!("File: {}", file_path);
    println!("Algorithm: {:?}", algorithm);
    println!();

    // Read audio file
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("Error: File not found: {}", file_path);
        return Err("File not found".into());
    }

    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    println!("Audio Format:");
    println!("  Sample Rate: {} Hz", spec.sample_rate);
    println!("  Channels: {}", spec.channels);
    println!("  Bit Depth: {} bits", spec.bits_per_sample);
    println!("  Sample Format: {:?}", spec.sample_format);
    println!();

    // Read samples
    let samples: Vec<i16> = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?;

    println!("Audio Data:");
    println!("  Total Samples: {}", samples.len());
    println!(
        "  Duration: {:.2} seconds",
        samples.len() as f64 / (spec.sample_rate as f64 * spec.channels as f64)
    );
    println!();

    // Calculate checksum
    let checksum = contexture_core::audio::checksum::calculate_checksum(&samples, algorithm);

    println!("Checksum:");
    println!("  Algorithm: {:?}", checksum.algorithm);
    println!("  Value: {}", checksum.value);
    println!("  Samples: {}", checksum.sample_count);
    println!();

    // Calculate audio statistics
    let stats = contexture_core::audio::checksum::calculate_audio_stats(&samples);

    println!("Audio Statistics:");
    println!("  RMS: {:.6}", stats.rms);
    println!(
        "  Peak: {} ({:.2}%)",
        stats.peak,
        stats.peak as f64 / i16::MAX as f64 * 100.0
    );
    println!("  Min: {}", stats.min);
    println!("  Max: {}", stats.max);
    println!("  Average: {:.2}", stats.average);
    println!();

    // Calculate all checksums for comparison
    if algorithm == contexture_core::audio::checksum::ChecksumAlgorithm::Simple {
        println!("All Checksums:");

        let algorithms = [
            contexture_core::audio::checksum::ChecksumAlgorithm::Simple,
            contexture_core::audio::checksum::ChecksumAlgorithm::Crc32,
            contexture_core::audio::checksum::ChecksumAlgorithm::Md5,
            contexture_core::audio::checksum::ChecksumAlgorithm::Sha256,
        ];

        for alg in algorithms {
            let cs = contexture_core::audio::checksum::calculate_checksum(&samples, alg);
            println!("  {:?}: {}", alg, cs.value);
        }
    }

    Ok(())
}

fn parse_algorithm(
    s: &str,
) -> Result<contexture_core::audio::checksum::ChecksumAlgorithm, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "simple" => Ok(contexture_core::audio::checksum::ChecksumAlgorithm::Simple),
        "crc32" => Ok(contexture_core::audio::checksum::ChecksumAlgorithm::Crc32),
        "md5" => Ok(contexture_core::audio::checksum::ChecksumAlgorithm::Md5),
        "sha256" => Ok(contexture_core::audio::checksum::ChecksumAlgorithm::Sha256),
        _ => Err(format!("Unknown algorithm: {}", s).into()),
    }
}

fn print_usage(program: &str) {
    println!("Audio Checksum Calculator");
    println!();
    println!("Usage:");
    println!("  {} <audio_file> [algorithm]", program);
    println!();
    println!("Algorithms:");
    println!("  simple  - Fast hash-based checksum (default)");
    println!("  crc32   - CRC32 checksum");
    println!("  md5     - MD5 hash");
    println!("  sha256  - SHA256 hash");
    println!();
    println!("Examples:");
    println!("  {} test.wav", program);
    println!("  {} test.wav sha256", program);
}
