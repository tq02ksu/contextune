package com.contextune.plugin.utils

import java.io.File

/**
 * Handles JNA-compatible configuration for the native Rust library.
 * 
 * This class configures the JNA library path so that Native.load() can
 * successfully locate and bind to the native library. It replaces the
 * previous JNI-based System.load() approach with JNA-compatible configuration.
 */
object NativeLibraryLoader {
    
    private var isConfigured = false
    private var loadException: Throwable? = null
    private val errorMessages = mutableListOf<String>()
    
    /**
     * Configure JNA library path for the native library.
     * This method sets up the jna.library.path system property so that
     * Native.load() can find the library.
     */
    @Synchronized
    fun configureJNALibraryPath(): Boolean {
        if (isConfigured) {
            return true
        }

        // Clear any previous error state
        errorMessages.clear()
        loadException = null

        // Strategy 1: Try to configure from plugin's lib directory
        if (tryPluginLibStrategy()) {
            isConfigured = true
            println("DEBUG: Successfully configured JNA library path from plugin lib")
            return true
        }

        // Strategy 2: Try to extract from resources and configure
        if (tryResourceExtractionStrategy()) {
            isConfigured = true
            println("DEBUG: Successfully configured JNA library path from resource extraction")
            return true
        }

        // Strategy 3: Try relative path (for development)
        if (tryDevelopmentPathStrategy()) {
            isConfigured = true
            println("DEBUG: Successfully configured JNA library path from development path")
            return true
        }

        // All strategies failed
        val exception = RuntimeException(
            "Failed to configure JNA library path for native library. " +
            "Tried plugin lib, resource extraction, and development path.\n" +
            "Details:\n${errorMessages.joinToString("\n")}\n" +
            "Platform info: ${PlatformLibraryResolver.getPlatformInfo()}"
        )
        loadException = exception
        throw exception
    }
    
    /**
     * Try to configure JNA library path from plugin's lib directory.
     * 
     * @return true if configuration succeeded, false otherwise
     */
    private fun tryPluginLibStrategy(): Boolean {
        return try {
            val pluginLibDirectory = getPluginLibDirectory()
            if (pluginLibDirectory != null) {
                val libraryFile = File(pluginLibDirectory, PlatformLibraryResolver.getLibraryFileName())
                if (libraryFile.exists() && libraryFile.canRead()) {
                    JNAConfigurationManager.setLibraryPath(pluginLibDirectory)
                    println("DEBUG: Configured JNA library path from plugin lib: $pluginLibDirectory")
                    return true
                } else {
                    errorMessages.add("Library file not found in plugin lib directory: ${libraryFile.absolutePath}")
                    return false
                }
            } else {
                errorMessages.add("Plugin lib directory not found")
                return false
            }
            false
        } catch (e: Exception) {
            errorMessages.add("Failed to configure from plugin lib: ${e.message}")
            false
        }
    }
    
    /**
     * Try to configure JNA library path from resource extraction.
     * 
     * @return true if configuration succeeded, false otherwise
     */
    private fun tryResourceExtractionStrategy(): Boolean {
        return try {
            val platformDir = PlatformLibraryResolver.getPlatformDirectory()
            val libraryFileName = PlatformLibraryResolver.getLibraryFileName()
            val resourcePath = "/native/$platformDir/$libraryFileName"
            
            val tempFile = TemporaryLibraryManager.extractLibraryToTemp(resourcePath)
            if (tempFile != null && tempFile.exists()) {
                val tempDirectory = tempFile.parentFile.absolutePath
                JNAConfigurationManager.setLibraryPath(tempDirectory)
                println("DEBUG: Configured JNA library path from resource extraction: $tempDirectory")
                return true
            } else {
                errorMessages.add("Failed to extract library from resources: $resourcePath")
                return false
            }
            false
        } catch (e: Exception) {
            errorMessages.add("Failed to configure from resource extraction: ${e.message}")
            false
        }
    }
    
    /**
     * Try to configure JNA library path from development paths.
     *
     * @return true if configuration succeeded, false otherwise
     */
    private fun tryDevelopmentPathStrategy(): Boolean {
        return try {
            val platformDir = PlatformLibraryResolver.getPlatformDirectory()
            val libraryFileName = PlatformLibraryResolver.getLibraryFileName()
            val developmentPath = "libs/$platformDir"
            
            val developmentDirectory = File(developmentPath)
            if (developmentDirectory.exists() && developmentDirectory.isDirectory) {
                val libraryFile = File(developmentDirectory, libraryFileName)
                if (libraryFile.exists() && libraryFile.canRead()) {
                    JNAConfigurationManager.setLibraryPath(developmentDirectory.absolutePath)
                    println("DEBUG: Configured JNA library path from development path: ${developmentDirectory.absolutePath}")
                    return true
                } else {
                    errorMessages.add("Library file not found in development directory: ${libraryFile.absolutePath}")
                    return false
                }
            } else {
                errorMessages.add("Development directory not found or not accessible: ${developmentDirectory.absolutePath}")
                return false
            }
        } catch (e: Exception) {
            errorMessages.add("Failed to configure from development path: ${e.message}")
            return false
        }
    }
    
    /**
     * Get the plugin lib directory containing native libraries.
     *
     * @return The plugin lib directory path, or null if not found
     */
    private fun getPluginLibDirectory(): String? {
        try {
            // Get the class location
            val pluginClass = NativeLibraryLoader::class.java
            val protectionDomain = pluginClass.protectionDomain
            val codeSource = protectionDomain?.codeSource
            val location = codeSource?.location
            
            println("DEBUG: Class location URL: $location")
            
            if (location != null) {
                val jarPath = File(location.toURI().path)
                println("DEBUG: JAR path: ${jarPath.absolutePath}")
                println("DEBUG: JAR exists: ${jarPath.exists()}")
                println("DEBUG: JAR name: ${jarPath.name}")
                
                // If we're in a JAR, the lib directory is the parent directory
                // Structure: contextune-music-player/lib/contextune-music-player-0.1.0.jar
                //            contextune-music-player/lib/macos-aarch64/libcontextune_core.dylib
                if (jarPath.name.endsWith(".jar")) {
                    val libDir = jarPath.parentFile  // This is the lib/ directory
                    println("DEBUG: Lib directory: ${libDir.absolutePath}")
                    println("DEBUG: Lib directory exists: ${libDir.exists()}")
                    
                    // List all files in lib directory
                    if (libDir.exists()) {
                        println("DEBUG: Files in lib directory:")
                        libDir.listFiles()?.forEach { file ->
                            println("DEBUG:   - ${file.name} (isDir: ${file.isDirectory})")
                            if (file.isDirectory) {
                                file.listFiles()?.forEach { subFile ->
                                    println("DEBUG:     - ${subFile.name}")
                                }
                            }
                        }
                    }
                    
                    val platformDir = PlatformLibraryResolver.getPlatformDirectory()
                    println("DEBUG: Platform directory: $platformDir")
                    
                    // The native library directory is lib/platformDir/
                    val nativeLibDir = File(libDir, platformDir)
                    println("DEBUG: Native lib directory: ${nativeLibDir.absolutePath}")
                    println("DEBUG: Native lib directory exists: ${nativeLibDir.exists()}")
                    
                    if (nativeLibDir.exists() && nativeLibDir.isDirectory) {
                        return nativeLibDir.absolutePath
                    }
                }
            }
        } catch (e: Exception) {
            println("DEBUG: Exception in getPluginLibDirectory: ${e.message}")
            e.printStackTrace()
        }

        return null
    }
    
    /**
     * Check if the JNA library path has been configured.
     * 
     * @return true if configured, false otherwise
     */
    fun isConfigured(): Boolean = isConfigured
    
    /**
     * Get the exception that occurred during configuration, if any.
     * 
     * @return The configuration exception, or null if no error occurred
     */
    fun getConfigurationException(): Throwable? = loadException
    
    /**
     * Get detailed error messages from configuration attempts.
     * 
     * @return List of error messages from each strategy attempt
     */
    fun getErrorMessages(): List<String> = errorMessages.toList()
    
    /**
     * Reset the configuration state.
     * This allows reconfiguration if needed (e.g., during plugin reload).
     */
    @Synchronized
    fun resetConfiguration() {
        isConfigured = false
        loadException = null
        errorMessages.clear()
        println("DEBUG: Reset NativeLibraryLoader configuration state")
    }
    
    /**
     * Get diagnostic information about the current configuration state.
     * 
     * @return A map containing diagnostic information
     */
    fun getDiagnosticInfo(): Map<String, Any> {
        return mapOf(
            "is_configured" to isConfigured,
            "has_error" to (loadException != null),
            "error_message" to (loadException?.message ?: "none"),
            "error_count" to errorMessages.size,
            "error_messages" to errorMessages.toList(),
            "platform_info" to PlatformLibraryResolver.getPlatformInfo(),
            "jna_library_path" to (JNAConfigurationManager.getCurrentLibraryPath() ?: "not set"),
            "jna_path_configured" to JNAConfigurationManager.isLibraryPathConfigured(),
            "temp_files_info" to TemporaryLibraryManager.getTemporaryFileInfo()
        )
    }
}
