package com.contextune.plugin.utils

import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.ConcurrentHashMap

/**
 * Manages extraction and cleanup of temporary library files.
 * 
 * This object handles extracting native libraries from JAR resources to temporary
 * locations, managing cleanup on JVM exit, and reusing existing temporary files
 * when possible to avoid redundant extractions.
 */
internal object TemporaryLibraryManager {
    
    private val extractedFiles = ConcurrentHashMap<String, File>()
    private val tempDirectories = ConcurrentHashMap<String, File>()
    
    /**
     * Extract a library from resources to a temporary location.
     * 
     * This method extracts the specified resource to a temporary directory,
     * schedules cleanup on JVM exit, and reuses existing files when possible.
     * 
     * @param resourcePath The path to the resource within the JAR (e.g., "/native/macos-aarch64/libcontextune_core.dylib")
     * @return The extracted temporary file, or null if extraction failed
     */
    fun extractLibraryToTemp(resourcePath: String): File? {
        return try {
            // Check if we already extracted this resource
            extractedFiles[resourcePath]?.let { existingFile ->
                if (existingFile.exists() && existingFile.canRead()) {
                    println("DEBUG: Reusing existing temporary library: ${existingFile.absolutePath}")
                    return existingFile
                } else {
                    // Remove stale reference
                    extractedFiles.remove(resourcePath)
                }
            }
            
            // Get the resource as an input stream
            val inputStream = TemporaryLibraryManager::class.java.getResourceAsStream(resourcePath)
            if (inputStream == null) {
                println("DEBUG: Resource not found: $resourcePath")
                return null
            }
            
            // Create temporary directory if needed
            val tempDir = getOrCreateTempDirectory()
            if (tempDir == null) {
                println("DEBUG: Failed to create temporary directory")
                return null
            }
            
            // Extract filename from resource path
            val fileName = resourcePath.substringAfterLast('/')
            if (fileName.isEmpty()) {
                println("DEBUG: Invalid resource path, no filename: $resourcePath")
                return null
            }
            
            // Create temporary file
            val tempFile = File(tempDir, fileName)
            
            // Extract the resource to temporary file
            inputStream.use { input ->
                FileOutputStream(tempFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            // Verify extraction
            if (!tempFile.exists() || tempFile.length() == 0L) {
                println("DEBUG: Extraction failed or resulted in empty file: ${tempFile.absolutePath}")
                tempFile.delete()
                return null
            }
            
            // Schedule cleanup and cache the file
            scheduleCleanup(tempFile)
            extractedFiles[resourcePath] = tempFile
            
            println("DEBUG: Successfully extracted library to: ${tempFile.absolutePath} (${tempFile.length()} bytes)")
            return tempFile
            
        } catch (e: Exception) {
            println("DEBUG: Exception during library extraction: ${e.message}")
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Create a temporary directory for library extraction.
     * 
     * @return A temporary directory, or null if creation failed
     */
    fun createTempDirectory(): File {
        return try {
            val tempDir = Files.createTempDirectory("contextune-native-libs").toFile()
            tempDir.deleteOnExit()
            println("DEBUG: Created temporary directory: ${tempDir.absolutePath}")
            tempDir
        } catch (e: Exception) {
            println("DEBUG: Failed to create temporary directory: ${e.message}")
            throw RuntimeException("Failed to create temporary directory for native library extraction", e)
        }
    }
    
    /**
     * Schedule cleanup of a temporary file on JVM exit.
     * 
     * @param tempFile The temporary file to clean up
     */
    fun scheduleCleanup(tempFile: File) {
        try {
            tempFile.deleteOnExit()
            
            // Also schedule cleanup of parent directory if it's empty
            val parentDir = tempFile.parentFile
            if (parentDir != null && parentDir != tempFile) {
                Runtime.getRuntime().addShutdownHook(Thread {
                    try {
                        // Only delete parent if it's empty (our temp directory)
                        if (parentDir.exists() && parentDir.isDirectory) {
                            val files = parentDir.listFiles()
                            if (files == null || files.isEmpty()) {
                                parentDir.delete()
                                println("DEBUG: Cleaned up empty temporary directory: ${parentDir.absolutePath}")
                            }
                        }
                    } catch (e: Exception) {
                        // Ignore cleanup errors during shutdown
                        println("DEBUG: Error during shutdown cleanup: ${e.message}")
                    }
                })
            }
            
            println("DEBUG: Scheduled cleanup for: ${tempFile.absolutePath}")
        } catch (e: Exception) {
            println("DEBUG: Failed to schedule cleanup for ${tempFile.absolutePath}: ${e.message}")
        }
    }
    
    /**
     * Manually clean up temporary files.
     * 
     * This method attempts to delete all tracked temporary files and directories.
     * It's useful for explicit cleanup when the plugin is unloaded.
     */
    fun cleanupTempFiles() {
        try {
            var cleanedCount = 0
            var errorCount = 0
            
            // Clean up extracted files
            extractedFiles.values.forEach { file ->
                try {
                    if (file.exists() && file.delete()) {
                        cleanedCount++
                        println("DEBUG: Cleaned up temporary file: ${file.absolutePath}")
                    }
                } catch (e: Exception) {
                    errorCount++
                    println("DEBUG: Failed to clean up temporary file ${file.absolutePath}: ${e.message}")
                }
            }
            
            // Clean up temporary directories
            tempDirectories.values.forEach { dir ->
                try {
                    if (dir.exists() && dir.isDirectory) {
                        val files = dir.listFiles()
                        if (files == null || files.isEmpty()) {
                            if (dir.delete()) {
                                cleanedCount++
                                println("DEBUG: Cleaned up temporary directory: ${dir.absolutePath}")
                            }
                        }
                    }
                } catch (e: Exception) {
                    errorCount++
                    println("DEBUG: Failed to clean up temporary directory ${dir.absolutePath}: ${e.message}")
                }
            }
            
            // Clear tracking maps
            extractedFiles.clear()
            tempDirectories.clear()
            
            println("DEBUG: Cleanup completed. Cleaned: $cleanedCount, Errors: $errorCount")
            
        } catch (e: Exception) {
            println("DEBUG: Exception during manual cleanup: ${e.message}")
            e.printStackTrace()
        }
    }
    
    /**
     * Get information about temporary files for diagnostics.
     * 
     * @return A map containing information about temporary files and directories
     */
    fun getTemporaryFileInfo(): Map<String, Any> {
        return try {
            val extractedInfo = extractedFiles.mapValues { (_, file) ->
                mapOf(
                    "path" to file.absolutePath,
                    "exists" to file.exists(),
                    "size" to if (file.exists()) file.length() else 0L,
                    "readable" to file.canRead()
                )
            }
            
            val tempDirInfo = tempDirectories.mapValues { (_, dir) ->
                mapOf(
                    "path" to dir.absolutePath,
                    "exists" to dir.exists(),
                    "is_directory" to dir.isDirectory,
                    "file_count" to (dir.listFiles()?.size ?: 0)
                )
            }
            
            mapOf(
                "extracted_files_count" to extractedFiles.size,
                "temp_directories_count" to tempDirectories.size,
                "extracted_files" to extractedInfo,
                "temp_directories" to tempDirInfo
            )
        } catch (e: Exception) {
            mapOf(
                "error" to e.message.orEmpty(),
                "extracted_files_count" to extractedFiles.size,
                "temp_directories_count" to tempDirectories.size
            )
        }
    }
    
    /**
     * Get or create the shared temporary directory for library extraction.
     * 
     * @return The temporary directory, or null if creation failed
     */
    private fun getOrCreateTempDirectory(): File? {
        val key = "contextune-libs"
        
        return tempDirectories[key]?.let { existingDir ->
            if (existingDir.exists() && existingDir.isDirectory) {
                existingDir
            } else {
                tempDirectories.remove(key)
                null
            }
        } ?: try {
            val newDir = createTempDirectory()
            tempDirectories[key] = newDir
            newDir
        } catch (e: Exception) {
            println("DEBUG: Failed to create temporary directory: ${e.message}")
            null
        }
    }
}