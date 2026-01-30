package com.contextune.plugin.utils

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
        
        try {
            // Strategy 1: Try to load from plugin's lib directory
            val pluginLibPath = getPluginLibPath(libraryName)
            if (pluginLibPath != null && File(pluginLibPath).exists()) {
                System.load(pluginLibPath)
                isLoaded = true
                println("Loaded native library from plugin lib: $pluginLibPath")
                return
            }
            
            // Strategy 2: Try to extract from resources and load
            val resourcePath = "/native/$libraryName"
            val inputStream = NativeLibraryLoader::class.java.getResourceAsStream(resourcePath)
            
            if (inputStream != null) {
                val tempFile = Files.createTempFile("contextune_core", getNativeLibraryExtension())
                tempFile.toFile().deleteOnExit()
                
                FileOutputStream(tempFile.toFile()).use { output ->
                    inputStream.copyTo(output)
                }
                
                System.load(tempFile.toAbsolutePath().toString())
                isLoaded = true
                println("Loaded native library from resources: $resourcePath")
                return
            }
            
            // Strategy 3: Try relative path (for development)
            val relativePath = getNativeLibraryPath(libraryName)
            val relativeFile = File(relativePath)
            if (relativeFile.exists()) {
                System.load(relativeFile.absolutePath)
                isLoaded = true
                println("Loaded native library from relative path: ${relativeFile.absolutePath}")
                return
            }
            
            throw RuntimeException("Native library not found: $libraryName. Tried plugin lib, resources, and relative path.")
            
        } catch (e: Exception) {
            throw RuntimeException("Failed to load native library: $libraryName", e)
        }
    }
    
    /**
     * Get the plugin lib path for the native library
     */
    private fun getPluginLibPath(libraryName: String): String? {
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
                    
                    val osName = System.getProperty("os.name").lowercase()
                    val osArch = System.getProperty("os.arch").lowercase()
                    
                    println("DEBUG: OS Name: $osName")
                    println("DEBUG: OS Arch: $osArch")
                    
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
                        else -> return null
                    }
                    
                    println("DEBUG: Platform directory: $platformDir")
                    
                    // The native library is in lib/platformDir/libraryName
                    val nativeLibFile = File(libDir, "$platformDir/$libraryName")
                    println("DEBUG: Native lib path: ${nativeLibFile.absolutePath}")
                    println("DEBUG: Native lib exists: ${nativeLibFile.exists()}")
                    
                    if (nativeLibFile.exists()) {
                        return nativeLibFile.absolutePath
                    }
                }
            }
        } catch (e: Exception) {
            println("DEBUG: Exception in getPluginLibPath: ${e.message}")
            e.printStackTrace()
        }
        
        return null
    }
    
    /**
     * Get the native library name for the current platform
     */
    private fun getNativeLibraryName(): String {
        val osName = System.getProperty("os.name").lowercase()
        val osArch = System.getProperty("os.arch").lowercase()
        
        return when {
            osName.contains("windows") -> "contextune_core.dll"
            osName.contains("mac") || osName.contains("darwin") -> {
                "libcontextune_core.dylib"
            }
            osName.contains("linux") -> "libcontextune_core.so"
            else -> throw UnsupportedOperationException("Unsupported platform: $osName")
        }
    }
    
    /**
     * Get the full path to the native library (for development/fallback)
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
