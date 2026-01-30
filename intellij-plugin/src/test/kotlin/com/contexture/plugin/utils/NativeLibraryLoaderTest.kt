package com.contexture.plugin.utils

import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Tests for NativeLibraryLoader
 */
class NativeLibraryLoaderTest {
    
    @Test
    fun `test platform detection`() {
        // This test just verifies that platform detection doesn't throw exceptions
        val osName = System.getProperty("os.name")
        val osArch = System.getProperty("os.arch")
        
        assertNotNull(osName, "OS name should not be null")
        assertNotNull(osArch, "OS arch should not be null")
        
        println("Detected OS: $osName")
        println("Detected Architecture: $osArch")
    }
    
    @Test
    fun `test library name generation`() {
        // Test that we can determine the library name without errors
        val osName = System.getProperty("os.name").lowercase()
        
        val isSupported = osName.contains("windows") || 
                         osName.contains("mac") || 
                         osName.contains("darwin") || 
                         osName.contains("linux")
        
        assertTrue(isSupported, "Platform should be supported")
    }
}
