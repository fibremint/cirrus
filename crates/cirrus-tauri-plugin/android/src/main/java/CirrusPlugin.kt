package com.fibremint.cirrus

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

import android.content.Intent
import android.util.Log
import android.widget.Toast
import com.fibremint.cirrus.CirrusService.Companion.ACTION_STOP


@TauriPlugin
class CirrusPlugin(private val activity: Activity): Plugin(activity) {
  // private var statusIsPlaying = false;

  private fun updateService(isPlaying: Boolean) {
    // Log.i("foo@MainActivity", "")
    
    if (isPlaying) {
      Log.i("start service", "")
      
      val startServiceIntent = Intent(activity, CirrusService::class.java)
      activity.startService(startServiceIntent)
      
      Toast.makeText(activity, "Service start", Toast.LENGTH_SHORT).show()
    } else {
      Log.i("stop service", "")

      val intentStop = Intent(activity, CirrusService::class.java)
      intentStop.action = ACTION_STOP
      activity.startService(intentStop)

      Toast.makeText(activity, "Service stop", Toast.LENGTH_SHORT).show()
    }
  }

  @Command
  fun setPlayerStatus(invoke: Invoke) {
    val isPlaying = invoke.getBoolean("is_playing")
    Log.i("Got params@setPlayerStatus, content", isPlaying.toString())
    
    if (isPlaying != null) {
      updateService(isPlaying)
    }
    
    val ret = JSObject()
    ret.put("is_playing", isPlaying)
    invoke.resolve(ret)
  }
}