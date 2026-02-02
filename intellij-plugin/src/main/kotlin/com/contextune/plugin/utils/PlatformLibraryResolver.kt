package com.contextune.plugin.utils

/**
 * Encapsulates platform-specific logic for library naming and path resolution.
 * 
 * This object handles the detection of the current platform (OS and architecture)
 * and provides the correct library file names, extensions, and directory paths
 * for each supported platform.
 */
internal object PlatformLibraryResolver {
    
    private val osName = System.getProperty("os.name").lowercase()
    private val osArch = System.getProperty("os.arch").lowercase()
    
    /**
     * Get the platform-specific library file name.
     * 
     * @return The complete library file name including extension (e.g., "libcontextune_core.dylib")
     */
    fun getLibraryFileName(): String {
        return when {
            isWindows() -> "contextune_core.dll"
            isMacOS() -> "libcontextune_core.dylib"
            isLinux() -> "libcontextune_core.so"
            else -> throw UnsupportedOperationException("Unsupported platform: $osName ($osArch)")
        }
    }
    
    /**
     * Get the platform-specific directory name for library storage.
     * 
     * @return The directory name (e.g., "macos-aarch64", "windows-x64", "linux-x64")
     */
    fun getPlatformDirectory(): String {
        return when {
            isWindows() && isX64() -> "windows-x64"
            isMacOS() && isX64() -> "macos-x64"
            isMacOS() && isARM64() -> "macos-aarch64"
            isLinux() && isX64() -> "linux-x64"
            else -> throw UnsupportedOperationException("Unsupported platform combination: $osName ($osArch)")
        }
    }
    
    /**
     * Get the platform-specific library file extension.
     * 
     * @return The file extension without the dot (e.g., "dylib", "dll", "so")
     */
    fun getLibraryExtension(): String {
        return when {
            isWindows() -> "dll"
            isMacOS() -> "dylib"
            isLinux() -> "so"
            else -> throw UnsupportedOperationException("Unsupported platform: $osName ($osArch)")
        }
    }
    
    /**
     * Validate that the current platform is supported.
     * 
     * @return true if the platform is supported, false otherwise
     */
    fun validatePlatform(): Boolean {
        return try {
            getPlatformDirectory()
            getLibraryFileName()
            true
        } catch (e: UnsupportedOperationException) {
            false
        }
    }
    
    /**
     * Get comprehensive platform information for debugging and diagnostics.
     * 
     * @return A map containing detailed platform information
     */
    fun getPlatformInfo(): Map<String, String> {
        return try {
            mapOf(
                "os_name" to osName,
                "os_arch" to osArch,
                "platform_directory" to getPlatformDirectory(),
                "library_filename" to getLibraryFileName(),
                "library_extension" to getLibraryExtension(),
                "is_supported" to validatePlatform().toString(),
                "detected_os" to getDetectedOS(),
                "detected_arch" to getDetectedArchitecture()
            )
        } catch (e: Exception) {
            mapOf(
                "os_name" to osName,
                "os_arch" to osArch,
                "error" to e.message.orEmpty(),
                "is_supported" to "false",
                "detected_os" to getDetectedOS(),
                "detected_arch" to getDetectedArchitecture()
            )
        }
    }
    
    /**
     * Check if the current OS is Windows.
     */
    private fun isWindows(): Boolean {
        return osName.contains("windows") || osName.contains("win")
    }
    
    /**
     * Check if the current OS is macOS.
     */
    private fun isMacOS(): Boolean {
        return osName.contains("mac") || osName.contains("darwin")
    }
    
    /**
     * Check if the current OS is Linux.
     */
    private fun isLinux(): Boolean {
        return osName.contains("linux")
    }
    
    /**
     * Check if the current architecture is x64/x86_64.
     */
    private fun isX64(): Boolean {
        return osArch.contains("x86_64") || osArch.contains("amd64") || osArch.contains("x64")
    }
    
    /**
     * Check if the current architecture is ARM64/AArch64.
     */
    private fun isARM64(): Boolean {
        return osArch.contains("aarch64") || osArch.contains("arm64")
    }
    
    /**
     * Get a human-readable description of the detected OS.
     */
    private fun getDetectedOS(): String {
        return when {
            isWindows() -> "Windows"
            isMacOS() -> "macOS"
            isLinux() -> "Linux"
            else -> "Unknown ($osName)"
        }
    }
    
    /**
     * Get a human-readable description of the detected architecture.
     */
    private fun getDetectedArchitecture(): String {
        return when {
            isX64() -> "x64"
            isARM64() -> "ARM64"
            else -> "Unknown ($osArch)"
        }
    }
}