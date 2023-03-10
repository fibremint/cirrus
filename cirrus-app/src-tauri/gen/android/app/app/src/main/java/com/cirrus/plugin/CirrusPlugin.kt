package com.cirrus.plugin

import com.plugin.sample.Example


import android.app.Activity
import android.util.Log
import app.tauri.annotation.PluginMethod
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import com.fibremint.app.MainActivity

@TauriPlugin
class CirrusPlugin(private val activity: Activity): Plugin(activity) {
//  private val implementation = Example()
  private var statusIsPlaying = false;

  @PluginMethod
  fun setPlayerStatus(invoke: Invoke) {
//    val value = invoke.getString("value") ?: ""
    val isPlaying = invoke.getBoolean("is_playing")
    Log.i("Got params@setPlayerStatus, content", isPlaying.toString())

    if (isPlaying != null) {
      statusIsPlaying = isPlaying
    }

    (activity as? MainActivity)?.updateService(statusIsPlaying)
    
    val ret = JSObject()
    ret.put("is_playing", isPlaying)
    invoke.resolve(ret)
  }
}
