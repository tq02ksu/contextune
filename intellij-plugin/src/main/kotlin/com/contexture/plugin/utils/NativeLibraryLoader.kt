package com.contexture.plugin.utils

import java.io.File
import java.io.FileOutputStream
import java.nio.file.Files

/**
 * Handles loading of the native Rust library
 */
object NativeLibraryLoader {
    
    private var isLoaded = false
    
    /**
     * Load the native library for the current platform
     */
    @Synchronized
    fun loadNativeLibrary() {
        if (isLoaded) {
            return
        }
        
        val libraryName = getNativeLibraryName()
        val libraryPath = getNativeLibraryPath(libraryName)
        
        try {
            // Try to load from plugin libs directory first
            val libFile = File(libraryPath)
            if (libFile.exists()) {
                System.load(libFile.absolutePath)
                isLoaded = true
                println("Loaded native library from: ${libFile.absolutePath}")
                return
            }
            
            // Try to extract from resources and load
            val resourcePath = "/native/$libraryName"
            val inputStream = NativeLibraryLoader::class.java.getResourceAsStream(resourcePath)
            
            if (inputStream != null) {
                val tempFile = Files.createTempFile("music_player_core", getNativeLibraryExtension())
                tempFile.toFile().deleteOnExit()
                
                FileOutputStream(tempFile.toFile()).use { output ->
                    inputStream.copyTo(output)
                }
                
                System.load(tempFile.toAbsolutePath().toString())
                isLoaded = true
                println("Loaded native library from resources: $resourcePath")
                return
            }
            
            // Last resort: try system library path
            System.loadLibrary("music_player_core")
            isLoaded = true
            println("Loaded native library from system path")
            
        } catch (e: Exception) {
            throw RuntimeException("Failed to load native library: $libraryName", e)
        }
    }
    
    /**
     * Get the native library name for the current platform
     */
    private fun getNativeLibraryName(): String {
        val osName = System.getProperty("os.name").lowercase()
        val osArch = System.getProperty("os.arch").lowercase()
        
        return when {
            osName.contains("windows") -> "music_player_core.dll"
            osName.contains("mac") || osName.contains("darwin") -> {
                if (osArch.contains("aarch64") || osArch.contains("arm")) {
                    "libmusic_player_core_aarch64.dylib"
                } else {
                    "libmusic_player_core.dylib"
                }
            }
            osName.contains("linux") -> "libmusic_player_core.so"
            else -> throw UnsupportedOperationException("Unsupported platform: $osName")
        }
    }
    
    /**
     * Get the full path to the native library
     */
    private fun getNativeLibraryPath(libraryName: String): String {
        val osName = System.getProperty("os.name").lowercase()
        val osArch = System.getProperty("os.arch").lowercase()
        
        val platformDir = when {
            osName.contains("windows") -> "windows-x64"
            osName.contains("mac") || osName.contains("darwin") -> {
                if (osArch.contains("aarch64") || osArch.contains("arm")) {
                    "macos-aarch64"
                } else {
                    "macos-x64"
                }
            }
            osName.contains("linux") -> "linux-x64"
            else -> throw UnsupportedOperationException("Unsupported platform: $osName")
        }
        
        return "libs/$platformDir/$libraryName"
    }
    
    /**
     * Get the native library file extension
     */
    private fun getNativeLibraryExtension(): String {
        val osName = System.getProperty("os.name").lowercase()
        return when {
            osName.contains("windows") -> ".dll"
            osName.contains("mac") || osName.contains("darwin") -> ".dylib"
            osName.contains("linux") -> ".so"
            else -> ""
        }
    }
}
