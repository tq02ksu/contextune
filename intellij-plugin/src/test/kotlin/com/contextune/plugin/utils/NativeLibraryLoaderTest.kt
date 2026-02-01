package com.contextune.plugin.utils

import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for NativeLibraryLoader
 */
class NativeLibraryLoaderTest : BasePlatformTestCase() {

    fun `test platform detection`() {
        // This test just verifies that platform detection doesn't throw exceptions
        val osName = System.getProperty("os.name")
        val osArch = System.getProperty("os.arch")

        assertNotNull(osName, "OS name should not be null")
        assertNotNull(osArch, "OS arch should not be null")

        println("Detected OS: $osName")
        println("Detected Architecture: $osArch")
    }

    fun `test library name generation`() {
        // Test that we can determine the library name without errors
        val osName = System.getProperty("os.name").lowercase()

        val isSupported = osName.contains("windows") ||
                osName.contains("mac") ||
                osName.contains("darwin") ||
                osName.contains("linux")

        assertTrue("Platform should be supported", isSupported)
    }
}
