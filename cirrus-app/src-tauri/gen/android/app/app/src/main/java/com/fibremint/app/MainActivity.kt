package com.fibremint.app

import android.os.Build
import android.os.Bundle
import android.util.Log
import android.webkit.WebView
import androidx.annotation.RequiresApi

import app.tauri.plugin.PluginManager
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

import com.plugin.sample.ExamplePlugin

class MainActivity : TauriActivity() {
    private var manager: PluginManager? = null
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        manager = PluginManager(this)
    }

    fun getPluginManager(): PluginManager? {
        return manager
    }
}

