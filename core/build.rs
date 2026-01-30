// Build script for Contextune Music Player Plugin
// Handles FFI exports, platform-specific configuration, and build optimizations

use std::env;
use std::path::PathBuf;

fn main() {
    // Get build information
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=exports.map");

    // Configure platform-specific settings
    configure_platform_specific(&target);

    // Configure FFI exports
    configure_ffi_exports(&target, &profile);

    // Configure audio libraries
    configure_audio_libraries(&target);

    // Generate build information
    generate_build_info(&out_dir);

    // Configure optimization flags
    configure_optimization(&profile);
}

/// Configure platform-specific build settings
fn configure_platform_specific(target: &str) {
    if target.contains("windows") {
        configure_windows();
    } else if target.contains("apple") {
        configure_macos();
    } else if target.contains("linux") {
        configure_linux();
    }
}

/// Configure Windows-specific settings
fn configure_windows() {
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=ksuser");

    // WASAPI support
    println!("cargo:rustc-link-lib=mmdevapi");
    println!("cargo:rustc-link-lib=audioclient");

    // Export symbols for DLL
    if env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap_or_default()
        .contains("crt-static")
    {
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_create");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_destroy");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_play");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_pause");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_stop");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_seek");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_set_volume");
        println!("cargo:rustc-link-arg=/EXPORT:audio_engine_load_file");
    }
}

/// Configure macOS-specific settings
fn configure_macos() {
    println!("cargo:rustc-link-lib=framework=CoreAudio");
    println!("cargo:rustc-link-lib=framework=AudioUnit");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");

    // Only export symbols in release mode when FFI is enabled
    if env::var("PROFILE").unwrap_or_default() == "release" {
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_create");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_destroy");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_play");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_pause");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_stop");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_seek");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_set_volume");
        println!("cargo:rustc-link-arg=-Wl,-exported_symbol,_audio_engine_load_file");
    }
}

/// Configure Linux-specific settings
fn configure_linux() {
    // ALSA support
    println!("cargo:rustc-link-lib=asound");

    // PulseAudio support (optional)
    if pkg_config::Config::new().probe("libpulse").is_ok() {
        println!("cargo:rustc-cfg=feature=\"pulseaudio\"");
    }

    // JACK support (optional)
    if pkg_config::Config::new().probe("jack").is_ok() {
        println!("cargo:rustc-cfg=feature=\"jack\"");
    }

    // Export symbols for shared library
    println!("cargo:rustc-link-arg=-Wl,--export-dynamic");

    // Use absolute path for exports.map
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let exports_map_path = PathBuf::from(manifest_dir).join("exports.map");
    println!(
        "cargo:rustc-link-arg=-Wl,--version-script={}",
        exports_map_path.display()
    );
}

/// Configure FFI exports based on target and profile
fn configure_ffi_exports(target: &str, profile: &str) {
    // Enable FFI exports for release builds
    if profile == "release" {
        println!("cargo:rustc-cfg=feature=\"ffi\"");
    }

    // Platform-specific FFI configuration
    if target.contains("windows") {
        // Windows DLL exports
        println!("cargo:rustc-cfg=windows_dll");
    } else if target.contains("apple") {
        // macOS dylib exports
        println!("cargo:rustc-cfg=macos_dylib");
    } else {
        // Linux shared library exports
        println!("cargo:rustc-cfg=linux_so");
    }

    // JNI configuration for Java integration
    configure_jni(target);
}

/// Configure JNI (Java Native Interface) settings
fn configure_jni(target: &str) {
    // Try to find Java installation
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let java_home = PathBuf::from(java_home);

        if target.contains("windows") {
            println!("cargo:rustc-link-search={}/lib", java_home.display());
            println!("cargo:rustc-link-lib=jvm");
        } else {
            // macOS and Linux use the same path
            println!("cargo:rustc-link-search={}/lib/server", java_home.display());
            println!("cargo:rustc-link-lib=jvm");
        }

        println!("cargo:rustc-cfg=feature=\"jni\"");
    } else {
        println!("cargo:warning=JAVA_HOME not set, JNI support disabled");
    }
}

/// Configure audio library dependencies
fn configure_audio_libraries(target: &str) {
    // Platform-specific audio library configuration
    if target.contains("windows") {
        // Windows audio libraries are linked above
    } else if target.contains("apple") {
        // macOS audio frameworks are linked above
    } else if target.contains("linux") {
        // Linux audio libraries
        if pkg_config::Config::new().probe("alsa").is_err() {
            println!("cargo:warning=ALSA development libraries not found");
            println!("cargo:warning=Install libasound2-dev on Ubuntu/Debian");
        }
    }
}

/// Generate build information for runtime use
fn generate_build_info(out_dir: &std::path::Path) {
    let build_info_path = out_dir.join("build_info.rs");

    let build_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let git_hash = get_git_hash().unwrap_or_else(|| "unknown".to_string());
    let rust_version = env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    let build_info = format!(
        r#"
// Auto-generated build information
pub const BUILD_TIME: &str = "{}";
pub const GIT_HASH: &str = "{}";
pub const RUST_VERSION: &str = "{}";
pub const TARGET: &str = "{}";
pub const PROFILE: &str = "{}";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
"#,
        build_time, git_hash, rust_version, target, profile
    );

    std::fs::write(build_info_path, build_info).expect("Failed to write build info");
}

/// Get Git commit hash
fn get_git_hash() -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Configure optimization flags based on profile
fn configure_optimization(profile: &str) {
    match profile {
        "release" => {
            // Release optimizations
            println!("cargo:rustc-cfg=optimized");

            // Link-time optimization
            println!("cargo:rustc-link-arg=-flto");

            // Strip debug symbols (platform-specific)
            let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
            if target_os == "linux" {
                // Linux uses GNU ld
                println!("cargo:rustc-link-arg=-Wl,--strip-debug");
            } else if target_os == "macos" {
                // macOS uses Apple's ld which has different syntax
                println!("cargo:rustc-link-arg=-Wl,-dead_strip");
            }
            // Windows doesn't need this flag
        }
        "debug" => {
            // Debug configuration
            println!("cargo:rustc-cfg=debug_build");
        }
        _ => {}
    }
}
