package com.devptp

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.os.IBinder
import androidx.core.app.NotificationCompat

class DevptpService : Service() {
    private var handle: Long = 0L

    override fun onCreate() {
        super.onCreate()
        val channel = NotificationChannel(CHANNEL, "devptp tunnel", NotificationManager.IMPORTANCE_LOW)
        getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
        startForeground(NOTIFICATION_ID, notification("devptp is running"))
        handle = MobileNative.start()
    }

    override fun onDestroy() {
        if (handle != 0L) MobileNative.shutdown(handle)
        handle = 0L
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun notification(text: String): Notification =
        NotificationCompat.Builder(this, CHANNEL)
            .setContentTitle("devptp")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.stat_sys_upload)
            .setOngoing(true)
            .build()

    companion object {
        private const val CHANNEL = "devptp"
        private const val NOTIFICATION_ID = 1001
    }
}
