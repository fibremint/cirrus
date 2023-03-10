package com.fibremint.app

import android.app.ActivityManager
import android.content.ContentValues.TAG
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.webkit.WebView
import androidx.annotation.RequiresApi
import android.webkit.*
import android.widget.Toast

import app.tauri.plugin.PluginManager
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.cirrus.plugin.CirrusPlugin
//import com.cirrus.plugin.CirrusService

import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL

import com.plugin.sample.ExamplePlugin

class MainActivity : TauriActivity() {
    private var manager: PluginManager? = null
//    private var webView: RustWebView? = null;
//    private var webViewClient: RustWebViewClient? = null;
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        manager = PluginManager(this)
      
//        val ctx = super.getApplicationContext()
//        webView = RustWebView(ctx)
////        webView?.loadUrl("http://192.168.0.101:5173")
//          webView?.loadUrl("https://tauri.localhost")

//      webViewClient = RustWebViewClient(ctx)
    }

    fun getPluginManager(): PluginManager? {
        return manager
    }

  override fun onResume() {
    super.onResume()
    Log.i("Resumed", "")
    Log.i("service running status", isMyServiceRunning(CirrusService::class.java).toString())
    // Log.i("plugins in manager", manager?.plugins.toString())
    
//    val samplePluginHandle = manager?.plugins?.get("sample")
//    samplePluginHandle.in
//    manager?.dispatch

//    Log.e("tauri rust webview url", webView?.url.toString())
//    webView?.loadUrl("javascript:window")
//    webView?.
//    webView?.evaluateJavascript("javascript:window.__TAURI__.tauri.invoke") { value ->
//      Log.e("js eval value", value)
//    }
  }

  override fun onPause() {
    super.onPause()
    Log.i("Paused", "")
  }

  override fun onDestroy() {
    super.onDestroy()
    updateService(false)
    Log.i(TAG, "destroy app")
  }
  
  fun updateService(isPlaying: Boolean) {
    Log.i("foo@MainActivity", "")
    
    if (isPlaying) {
      Log.i("start service @MainActivity", "")
      
      val startServiceIntent = Intent(this@MainActivity, CirrusService::class.java)
      startService(startServiceIntent)
      Toast.makeText(this@MainActivity, "Service start", Toast.LENGTH_SHORT).show()
    } else {
      Log.i("stop service @MainActivity", "")

      val intentStop = Intent(this@MainActivity, CirrusService::class.java)
      intentStop.action = ACTION_STOP
      startService(intentStop)

      Toast.makeText(this@MainActivity, "Service stop", Toast.LENGTH_SHORT).show()
    }
  }
  
  private fun isMyServiceRunning(serviceClass: Class<*>): Boolean {
    try {
      val manager =
        getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
      for (service in manager.getRunningServices(
        Int.MAX_VALUE
      )) {
        if (serviceClass.name == service.service.className) {
          return true
        }
      }
    } catch (e: Exception) {
      return false
    }
    
    return false
  }

  companion object{
    const val ACTION_STOP = "${BuildConfig.APPLICATION_ID}.stop"
  }
}

//fun httpRequestTest(url: String) {
//  val url = URL(url)
//  
//  try {
//    val conn = url.openConnection() as HttpURLConnection
//    conn.requestMethod = "GET"
//
//    BufferedReader(InputStreamReader(conn.inputStream)).use { br ->
//      var line: String?
//      while (br.readLine().also { line = it } != null) {
//        Log.i("request content: ", line.toString())
//      }
//    }
//  } catch (_: Exception) {
//    Log.i("failed to request url", url.toString())
//  }
//}
