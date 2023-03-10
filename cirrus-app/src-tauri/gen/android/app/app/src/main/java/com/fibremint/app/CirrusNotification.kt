package com.fibremint.app

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.PendingIntent.FLAG_IMMUTABLE
import android.app.PendingIntent.FLAG_MUTABLE
import android.content.ContentValues.TAG
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.content.getSystemService

object CirrusNotification {
  const val CHAN_ID = "cirrus_service_chan"
  
  fun createNotification(
    context: Context
  ): Notification {
    Log.i(TAG, "createNotification@CirrusNotification")
    
    val notificationIntent = Intent(context, MainActivity::class.java)
    notificationIntent.action = "main"
    notificationIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
    
    val pendingIntent = PendingIntent
      .getActivity(context, 0, notificationIntent, FLAG_MUTABLE)
    
    val notification = NotificationCompat.Builder(context, CHAN_ID)
      .setContentTitle("Cirrus")
      .setContentText("Text")
      .setSmallIcon(R.drawable.ic_launcher_foreground)
      .setOngoing(true)
      .setContentIntent(pendingIntent)
      .build()

//    mNotificationManager = this.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      Log.i(TAG, "createNotification,createnotiChan@CirrusNotification")

      val serviceChannel = NotificationChannel(
        CHAN_ID,
        "Cirrus Channel",
        NotificationManager.IMPORTANCE_DEFAULT
      )
      
      val manager = context.getSystemService(NotificationManager::class.java)
      manager?.createNotificationChannel(serviceChannel)
    }
    
    return notification
  }
}