package com.fibremint.cirrus

import android.app.*
import android.app.PendingIntent.FLAG_IMMUTABLE
import android.app.PendingIntent.FLAG_MUTABLE
import android.content.ContentValues.TAG
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat

class CirrusService : Service() {
  companion object{
//    const val ACTION_STOP = "${BuildConfig.APPLICATION_ID}.stop"
  const val ACTION_STOP = "cirrus_action_stop"
  }

  override fun onBind(intent: Intent?): IBinder? {
    return null
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    Log.i(TAG, "onStartCommand@CirrusService")
    
    if (intent?.action != null && intent.action.equals(
        ACTION_STOP, ignoreCase = true)) {
      Log.i(TAG, "onStartCommand,stopForeground@CirrusService")

      stopForeground(true)
      stopSelf()
    } else {
//    generateForegroundNotification()
      Log.i(TAG, "onStartCommand,startForeground@CirrusService")

      val notification = CirrusNotification.createNotification(this)
//      val notification = intent?.let { CirrusNotification.createNotification(this, it) }
      startForeground(1000, notification)
    }
    
    return START_STICKY
  }

  override fun onCreate() {
    super.onCreate()
    Log.i("onCreate@CirrusService", "")

  }
  
  //Notififcation for ON-going
//  private var iconNotification: Bitmap? = null
//  private var notification: Notification? = null
//  var mNotificationManager: NotificationManager? = null
//  private val mNotificationId = 123
//  
//  private fun generateForegroundNotification() {
//    Log.i(TAG, "generateForegroundNotification@CirrusService")
//    
//    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
//      val intentMainLanding = Intent(this, MainActivity::class.java)
//      val pendingIntent =
//        PendingIntent.getActivity(this, 0, intentMainLanding, FLAG_MUTABLE)
//
//      iconNotification = BitmapFactory.decodeResource(resources, R.mipmap.ic_launcher)
//      if (mNotificationManager == null) {
//        mNotificationManager = this.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
//      }
//      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
//        assert(mNotificationManager != null)
//        mNotificationManager?.createNotificationChannelGroup(
//          NotificationChannelGroup("chats_group", "Chats")
//        )
//        val notificationChannel =
//          NotificationChannel("service_channel", "Service Notifications",
//            NotificationManager.IMPORTANCE_MIN)
//        notificationChannel.enableLights(false)
//        notificationChannel.lockscreenVisibility = Notification.VISIBILITY_SECRET
//        mNotificationManager?.createNotificationChannel(notificationChannel)
//      }
//      val builder = NotificationCompat.Builder(this, "service_channel")
//
//      builder.setContentTitle(StringBuilder(resources.getString(R.string.app_name)).append(" service is running").toString())
//        .setTicker(StringBuilder(resources.getString(R.string.app_name)).append("service is running").toString())
//        .setContentText("Touch to open") //                    , swipe down for more options.
////        .setSmallIcon(R.drawable.ic_alaram)
//        .setPriority(NotificationCompat.PRIORITY_LOW)
//        .setWhen(0)
//        .setOnlyAlertOnce(true)
//        .setContentIntent(pendingIntent)
//        .setOngoing(true)
//      if (iconNotification != null) {
//        builder.setLargeIcon(Bitmap.createScaledBitmap(iconNotification!!, 128, 128, false))
//      }
//      builder.color = resources.getColor(R.color.purple_200)
//      notification = builder.build()
//
//      startForeground(mNotificationId, notification)
//    }

//  }
}