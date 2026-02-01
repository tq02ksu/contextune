package com.contextune.plugin.utils

import java.io.File

/**
 * Manages JNA-specific configuration and system property management.
 * 
 * This class handles the configuration of JNA library paths to ensure that
 * Native.load() can successfully locate native libraries. It provides methods
 * to set, append, and validate the jna.library.path system property.
 */
internal object JNAConfigurationManager {
    
    private const val JNA_LIBRARY_PATH_PROPERTY = "jna.library.path"
    private val PATH_SEPARATOR = System.getProperty("path.separator")
    
    /**
     * Set the JNA library path to the specified directory.
     * This replaces any existing jna.library.path configuration.
     * 
     * @param directoryPath The directory path containing native libraries
     * @throws IllegalArgumentException if the directory path is invalid
     */
    fun setLibraryPath(directoryPath: String) {
        require(directoryPath.isNotBlank()) { "Directory path cannot be blank" }
        require(validateLibraryPath(directoryPath)) { "Invalid library path: $directoryPath" }
        
        System.setProperty(JNA_LIBRARY_PATH_PROPERTY, directoryPath)
        println("DEBUG: Set jna.library.path to: $directoryPath")
    }
    
    /**
     * Append a directory to the existing JNA library path.
     * If no existing path is set, this behaves like setLibraryPath().
     * 
     * @param directoryPath The directory path to append
     * @throws IllegalArgumentException if the directory path is invalid
     */
    fun appendToLibraryPath(directoryPath: String) {
        require(directoryPath.isNotBlank()) { "Directory path cannot be blank" }
        require(validateLibraryPath(directoryPath)) { "Invalid library path: $directoryPath" }
        
        val currentPath = getCurrentLibraryPath()
        val newPath = if (currentPath.isNullOrBlank()) {
            directoryPath
        } else {
            "$currentPath$PATH_SEPARATOR$directoryPath"
        }
        
        System.setProperty(JNA_LIBRARY_PATH_PROPERTY, newPath)
        println("DEBUG: Appended to jna.library.path, now: $newPath")
    }
    
    /**
     * Get the current JNA library path configuration.
     * 
     * @return The current jna.library.path value, or null if not set
     */
    fun getCurrentLibraryPath(): String? {
        return System.getProperty(JNA_LIBRARY_PATH_PROPERTY)
    }
    
    /**
     * Validate that a directory path is suitable for use as a JNA library path.
     * 
     * @param directoryPath The directory path to validate
     * @return true if the path is valid, false otherwise
     */
    fun validateLibraryPath(directoryPath: String): Boolean {
        if (directoryPath.isBlank()) {
            return false
        }
        
        return try {
            val directory = File(directoryPath)
            directory.exists() && directory.isDirectory && directory.canRead()
        } catch (e: Exception) {
            println("DEBUG: Library path validation failed for '$directoryPath': ${e.message}")
            false
        }
    }
    
    /**
     * Clear the JNA library path configuration.
     * This removes the jna.library.path system property.
     */
    fun clearLibraryPath() {
        System.clearProperty(JNA_LIBRARY_PATH_PROPERTY)
        println("DEBUG: Cleared jna.library.path")
    }
    
    /**
     * Check if the JNA library path is currently configured.
     * 
     * @return true if jna.library.path is set and not empty, false otherwise
     */
    fun isLibraryPathConfigured(): Boolean {
        val path = getCurrentLibraryPath()
        return !path.isNullOrBlank()
    }
    
    /**
     * Get all directories in the current JNA library path.
     * 
     * @return List of directory paths, empty if no path is configured
     */
    fun getLibraryPathDirectories(): List<String> {
        val path = getCurrentLibraryPath()
        return if (path.isNullOrBlank()) {
            emptyList()
        } else {
            path.split(PATH_SEPARATOR).filter { it.isNotBlank() }
        }
    }
}