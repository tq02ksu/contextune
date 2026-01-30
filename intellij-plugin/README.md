# Contexture Music Player - IntelliJ Platform Plugin

An intelligent HiFi music player plugin for IntelliJ Platform IDEs (IntelliJ IDEA, PyCharm, WebStorm, etc.) that combines high-quality audio playback with AI-powered music discovery and contextual information.

## Features

- **HiFi-Grade Audio Playback**: Bit-perfect audio output using a high-performance Rust audio core
- **Multiple Format Support**: FLAC, WAV, MP3, AAC, ALAC, OGG, and more
- **CUE Sheet Support**: Play albums stored as single audio files with CUE sheets
- **Local Music Library**: Scan and manage your local music collection
- **Playlist Management**: Create and manage playlists
- **Keyboard Shortcuts**: Control playback without leaving your code
- **Minimal IDE Impact**: Efficient resource usage, won't slow down your IDE

## Technology Stack

- **Language**: Kotlin
- **Build Tool**: Gradle with Kotlin DSL
- **IDE Platform**: IntelliJ Platform 2023.2+
- **Audio Core**: Rust (native library via JNA)
- **FFI**: JNA (Java Native Access) for Rust interop

## Building the Plugin

### Prerequisites

- JDK 17 or later
- Gradle 8.5+ (or use the included Gradle wrapper)
- Rust toolchain (for building the native audio core)

### Build Steps

1. Build the Rust audio core:
   ```bash
   cd ../core
   cargo build --release
   ```

2. Copy the native library to the plugin libs directory:
   ```bash
   # Linux
   mkdir -p libs/linux-x64
   cp ../target/release/libmusic_player_core.so libs/linux-x64/
   
   # macOS (Intel)
   mkdir -p libs/macos-x64
   cp ../target/release/libmusic_player_core.dylib libs/macos-x64/
   
   # macOS (Apple Silicon)
   mkdir -p libs/macos-aarch64
   cp ../target/release/libmusic_player_core.dylib libs/macos-aarch64/
   
   # Windows
   mkdir -p libs/windows-x64
   cp ../target/release/music_player_core.dll libs/windows-x64/
   ```

3. Build the plugin:
   ```bash
   ./gradlew buildPlugin
   ```

The plugin ZIP file will be created in `build/distributions/`.

## Development

### Running the Plugin in Development Mode

```bash
./gradlew runIde
```

This will start a new IntelliJ IDEA instance with the plugin installed.

### Running Tests

```bash
./gradlew test
```

### Plugin Verification

```bash
./gradlew runPluginVerifier
```

### Cleaning Build Artifacts

```bash
./gradlew clean
```

## Installation

1. Download the plugin ZIP from the releases page
2. In IntelliJ IDEA, go to `Settings` → `Plugins` → `⚙️` → `Install Plugin from Disk...`
3. Select the downloaded ZIP file
4. Restart IntelliJ IDEA

## Usage

### Opening the Music Player

- Go to `View` → `Tool Windows` → `Contexture Music Player`
- Or use the tool window button on the right side of the IDE

### Keyboard Shortcuts

- `Ctrl+Alt+P` - Play/Pause
- `Ctrl+Alt+S` - Stop
- `Ctrl+Alt+N` - Next Track
- `Ctrl+Alt+B` - Previous Track
- `Ctrl+Alt+↑` - Volume Up
- `Ctrl+Alt+↓` - Volume Down
- `Ctrl+Alt+M` - Mute/Unmute

## Project Structure

```
intellij-plugin/
├── build.gradle.kts                 # Gradle build configuration
├── settings.gradle.kts              # Gradle settings
├── gradle.properties                # Plugin properties
├── gradlew                          # Gradle wrapper (Unix)
├── gradlew.bat                      # Gradle wrapper (Windows)
│
├── src/
│   ├── main/
│   │   ├── kotlin/
│   │   │   └── com/contexture/plugin/
│   │   │       ├── MusicPlayerPlugin.kt
│   │   │       ├── actions/         # IDE actions
│   │   │       ├── audio/           # JNA wrapper for Rust audio engine
│   │   │       ├── services/        # Plugin services
│   │   │       ├── settings/        # Settings UI
│   │   │       ├── ui/              # User interface
│   │   │       └── utils/           # Utilities
│   │   └── resources/
│   │       ├── META-INF/
│   │       │   └── plugin.xml       # Plugin manifest
│   │       └── icons/               # UI icons
│   └── test/
│       └── kotlin/                  # Tests
│
└── libs/                            # Native libraries
    ├── linux-x64/
    ├── macos-x64/
    ├── macos-aarch64/
    └── windows-x64/
```

## Reference

This plugin is based on the [IntelliJ Platform Plugin Template](https://github.com/JetBrains/intellij-platform-plugin-template) by JetBrains.

## License

See the main project LICENSE file.

## Contributing

See the main project CONTRIBUTING.md file.
