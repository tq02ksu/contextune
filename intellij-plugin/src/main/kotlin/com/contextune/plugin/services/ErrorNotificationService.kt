package com.contextune.plugin.services

import com.intellij.notification.Notification
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.notification.Notifications
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project

/**
 * Service for displaying error notifications to users
 */
@Service(Service.Level.APP)
class ErrorNotificationService {
    
    private val logger = Logger.getInstance(ErrorNotificationService::class.java)
    
    companion object {
        private const val NOTIFICATION_GROUP_ID = "Contextune Music Player"
    }
    
    /**
     * Show an error notification
     */
    fun showError(title: String, message: String, project: Project? = null) {
        logger.error("$title: $message")
        
        val notification = createNotification(
            title,
            message,
            NotificationType.ERROR
        )
        
        Notifications.Bus.notify(notification, project)
    }
    
    /**
     * Show a warning notification
     */
    fun showWarning(title: String, message: String, project: Project? = null) {
        logger.warn("$title: $message")
        
        val notification = createNotification(
            title,
            message,
            NotificationType.WARNING
        )
        
        Notifications.Bus.notify(notification, project)
    }
    
    /**
     * Show an info notification
     */
    fun showInfo(title: String, message: String, project: Project? = null) {
        logger.info("$title: $message")
        
        val notification = createNotification(
            title,
            message,
            NotificationType.INFORMATION
        )
        
        Notifications.Bus.notify(notification, project)
    }
    
    /**
     * Log an error without showing notification
     */
    fun logError(message: String, throwable: Throwable? = null) {
        if (throwable != null) {
            logger.error(message, throwable)
        } else {
            logger.error(message)
        }
    }
    
    /**
     * Log a warning without showing notification
     */
    fun logWarning(message: String) {
        logger.warn(message)
    }
    
    /**
     * Create a notification
     */
    private fun createNotification(
        title: String,
        content: String,
        type: NotificationType
    ): Notification {
        return NotificationGroupManager.getInstance()
            .getNotificationGroup(NOTIFICATION_GROUP_ID)
            .createNotification(title, content, type)
    }
}
