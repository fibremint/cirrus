# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

-keep class com.fibremint.app.TauriActivity {
  getAppClass(...);
  getVersion();
}

-keep class com.fibremint.app.RustWebView {
  public <init>(...);
  loadUrlMainThread(...);
}

-keep class com.fibremint.app.Ipc {
  public <init>(...);
  @android.webkit.JavascriptInterface public <methods>;
}

-keep class com.fibremint.app.RustWebChromeClient,com.fibremint.app.RustWebViewClient {
  public <init>(...);
}

-keep class com.fibremint.app.MainActivity {
  public getPluginManager();
}
