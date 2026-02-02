package com.contextune.plugin.services

import com.intellij.testFramework.fixtures.BasePlatformTestCase

/**
 * Tests for ErrorNotificationService
 */
class ErrorNotificationServiceTest : BasePlatformTestCase() {
    
    fun `test service creation`() {
        val service = ErrorNotificationService()
        assertNotNull(service)
    }
    
    fun `test log error without exception`() {
        val service = ErrorNotificationService()
        
        // Should not throw
        service.logError("Test error message")
    }
    
    fun `test log error with exception`() {
        val service = ErrorNotificationService()
        val exception = RuntimeException("Test exception")
        
        // Should not throw
        service.logError("Test error with exception", exception)
    }
    
    fun `test log warning`() {
        val service = ErrorNotificationService()
        
        // Should not throw
        service.logWarning("Test warning message")
    }
}
