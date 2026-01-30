package com.contexture.plugin.services

import org.junit.jupiter.api.Test
import kotlin.test.assertNotNull

/**
 * Tests for ErrorNotificationService
 */
class ErrorNotificationServiceTest {
    
    @Test
    fun `test service creation`() {
        val service = ErrorNotificationService()
        assertNotNull(service)
    }
    
    @Test
    fun `test log error without exception`() {
        val service = ErrorNotificationService()
        
        // Should not throw
        service.logError("Test error message")
    }
    
    @Test
    fun `test log error with exception`() {
        val service = ErrorNotificationService()
        val exception = RuntimeException("Test exception")
        
        // Should not throw
        service.logError("Test error with exception", exception)
    }
    
    @Test
    fun `test log warning`() {
        val service = ErrorNotificationService()
        
        // Should not throw
        service.logWarning("Test warning message")
    }
}
