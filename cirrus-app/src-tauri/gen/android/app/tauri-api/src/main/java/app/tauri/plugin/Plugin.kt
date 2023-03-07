// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.plugin

import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.webkit.WebView
import androidx.core.app.ActivityCompat
import app.tauri.FsUtils
import app.tauri.Logger
import app.tauri.PermissionHelper
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.PluginMethod
import app.tauri.annotation.TauriPlugin
import org.json.JSONException
import java.util.*

abstract class Plugin(private val activity: Activity) {
  var handle: PluginHandle? = null

  open fun load(webView: WebView) {}

  /**
   * Start activity for result with the provided Intent and resolve calling the provided callback method name.
   *
   * If there is no registered activity callback for the method name passed in, the call will
   * be rejected. Make sure a valid activity result callback method is registered using the
   * [ActivityCallback] annotation.
   *
   * @param invoke the invoke object
   * @param intent the intent used to start an activity
   * @param callbackName the name of the callback to run when the launched activity is finished
   */
  fun startActivityForResult(invoke: Invoke, intent: Intent, callbackName: String) {
    handle!!.startActivityForResult(invoke, intent, callbackName)
  }

  /**
   * Get the plugin log tags.
   * @param subTags
   */
  protected fun getLogTag(vararg subTags: String): String {
    return Logger.tags(*subTags)
  }

  /**
   * Gets a log tag with the plugin's class name as subTag.
   */
  protected fun getLogTag(): String {
    return Logger.tags(this.javaClass.simpleName)
  }

  /**
   * Convert an URI to an URL that can be loaded by the webview.
   */
  fun assetUrl(u: Uri): String {
    var path = FsUtils.getFileUrlForUri(activity, u)
    if (path?.startsWith("file://") == true) {
      path = path.replace("file://", "")
    }
    return "asset://localhost$path"
  }

  /**
   * Exported plugin method for checking the granted status for each permission
   * declared on the plugin. This plugin call responds with a mapping of permissions to
   * the associated granted status.
   */
  @PluginMethod
  @PermissionCallback
  fun checkPermissions(invoke: Invoke) {
    val permissionsResult: Map<String, PermissionState?> = getPermissionStates()
    if (permissionsResult.isEmpty()) {
      // if no permissions are defined on the plugin, resolve undefined
      invoke.resolve()
    } else {
      val permissionsResultJSON = JSObject()
      for ((key, value) in permissionsResult) {
        permissionsResultJSON.put(key, value)
      }
      invoke.resolve(permissionsResultJSON)
    }
  }

  /**
   * Exported plugin method to request all permissions for this plugin.
   * To manually request permissions within a plugin use:
   * [.requestAllPermissions], or
   * [.requestPermissionForAlias], or
   * [.requestPermissionForAliases]
   *
   * @param invoke
   */
  @PluginMethod
  open fun requestPermissions(invoke: Invoke) {
    val annotation = handle?.annotation
    if (annotation != null) {
      // handle permission requests for plugins defined with @TauriPlugin
      var permAliases: Array<String>? = null
      val autoGrantPerms: MutableSet<String> = HashSet()

      // If call was made with a list of specific permission aliases to request, save them
      // to be requested
      val providedPerms: JSArray = invoke.getArray("permissions", JSArray())
      var providedPermsList: List<String?>? = null
      try {
        providedPermsList = providedPerms.toList()
      } catch (ignore: JSONException) {
        // do nothing
      }

      // If call was made without any custom permissions, request all from plugin annotation
      val aliasSet: MutableSet<String> = HashSet()
      if (providedPermsList == null || providedPermsList.isEmpty()) {
        for (perm in annotation.permissions) {
          // If a permission is defined with no permission strings, separate it for auto-granting.
          // Otherwise, the alias is added to the list to be requested.
          if (perm.strings.isEmpty() || perm.strings.size == 1 && perm.strings[0]
              .isEmpty()
          ) {
            if (!perm.alias.isEmpty()) {
              autoGrantPerms.add(perm.alias)
            }
          } else {
            aliasSet.add(perm.alias)
          }
        }
        permAliases = aliasSet.toTypedArray()
      } else {
        for (perm in annotation.permissions) {
          if (providedPermsList.contains(perm.alias)) {
            aliasSet.add(perm.alias)
          }
        }
        if (aliasSet.isEmpty()) {
          invoke.reject("No valid permission alias was requested of this plugin.")
        } else {
          permAliases = aliasSet.toTypedArray()
        }
      }
      if (permAliases != null && permAliases.isNotEmpty()) {
        // request permissions using provided aliases or all defined on the plugin
        requestPermissionForAliases(permAliases, invoke, "checkPermissions")
      } else if (autoGrantPerms.isNotEmpty()) {
        // if the plugin only has auto-grant permissions, return all as GRANTED
        val permissionsResults = JSObject()
        for (perm in autoGrantPerms) {
          permissionsResults.put(perm, PermissionState.GRANTED.toString())
        }
        invoke.resolve(permissionsResults)
      } else {
        // no permissions are defined on the plugin, resolve undefined
        invoke.resolve()
      }
    }
  }

  /**
   * Checks if the given permission alias is correctly declared in AndroidManifest.xml
   * @param alias a permission alias defined on the plugin
   * @return true only if all permissions associated with the given alias are declared in the manifest
   */
  fun isPermissionDeclared(alias: String): Boolean {
    val annotation = handle?.annotation
    if (annotation != null) {
      for (perm in annotation.permissions) {
        if (alias.equals(perm.alias, ignoreCase = true)) {
          var result = true
          for (permString in perm.strings) {
            result = result && PermissionHelper.hasDefinedPermission(activity, permString)
          }
          return result
        }
      }
    }
    Logger.error(
      String.format(
        "isPermissionDeclared: No alias defined for %s " + "or missing @TauriPlugin annotation.",
        alias
      )
    )
    return false
  }

  private fun permissionActivityResult(
    invoke: Invoke,
    permissionStrings: Array<String>,
    callbackName: String
  ) {
    handle!!.requestPermissions(invoke, permissionStrings, callbackName)
  }

  /**
   * Request all of the specified permissions in the TauriPlugin annotation (if any)
   *
   * If there is no registered permission callback for the Invoke passed in, the call will
   * be rejected. Make sure a valid permission callback method is registered using the
   * [PermissionCallback] annotation.
   *
   * @param invoke
   * @param callbackName the name of the callback to run when the permission request is complete
   */
  protected fun requestAllPermissions(
    invoke: Invoke,
    callbackName: String
  ) {
    val annotation = handle!!.annotation
    if (annotation != null) {
      val perms: HashSet<String> = HashSet()
      for (perm in annotation.permissions) {
        perms.addAll(perm.strings)
      }
      permissionActivityResult(invoke, perms.toArray(arrayOfNulls<String>(0)), callbackName)
    }
  }

  /**
   * Request permissions using an alias defined on the plugin.
   *
   * If there is no registered permission callback for the Invoke passed in, the call will
   * be rejected. Make sure a valid permission callback method is registered using the
   * [PermissionCallback] annotation.
   *
   * @param alias an alias defined on the plugin
   * @param invoke  the invoke involved in originating the request
   * @param callbackName the name of the callback to run when the permission request is complete
   */
  protected fun requestPermissionForAlias(
    alias: String,
    call: Invoke,
    callbackName: String
  ) {
    requestPermissionForAliases(arrayOf(alias), call, callbackName)
  }

  /**
   * Request permissions using aliases defined on the plugin.
   *
   * If there is no registered permission callback for the Invoke passed in, the call will
   * be rejected. Make sure a valid permission callback method is registered using the
   * [PermissionCallback] annotation.
   *
   * @param aliases a set of aliases defined on the plugin
   * @param invoke    the invoke involved in originating the request
   * @param callbackName the name of the callback to run when the permission request is complete
   */
  fun requestPermissionForAliases(
    aliases: Array<String>,
    invoke: Invoke,
    callbackName: String
  ) {
    if (aliases.isEmpty()) {
      Logger.error("No permission alias was provided")
      return
    }
    val permissions = getPermissionStringsForAliases(aliases)
    if (permissions.isNotEmpty()) {
      permissionActivityResult(invoke, permissions, callbackName)
    }
  }

  /**
   * Gets the Android permission strings defined on the [TauriPlugin] annotation with
   * the provided aliases.
   *
   * @param aliases aliases for permissions defined on the plugin
   * @return Android permission strings associated with the provided aliases, if exists
   */
  private fun getPermissionStringsForAliases(aliases: Array<String>): Array<String> {
    val annotation = handle?.annotation
    val perms: HashSet<String> = HashSet()
    if (annotation != null) {
      for (perm in annotation.permissions) {
        if (aliases.contains(perm.alias)) {
          perms.addAll(perm.strings)
        }
      }
    }
    return perms.toArray(arrayOfNulls(0))
  }

  /**
   * Get the permission state for the provided permission alias.
   *
   * @param alias the permission alias to get
   * @return the state of the provided permission alias or null
   */
  fun getPermissionState(alias: String): PermissionState? {
    return getPermissionStates()[alias]
  }

  /**
   * Helper to check all permissions defined on a plugin and see the state of each.
   *
   * @return A mapping of permission aliases to the associated granted status.
   */
  open fun getPermissionStates(): Map<String, PermissionState> {
    val permissionsResults: MutableMap<String, PermissionState> = HashMap()
    val annotation = handle?.annotation
    if (annotation != null) {
      for (perm in annotation.permissions) {
        // If a permission is defined with no permission constants, return GRANTED for it.
        // Otherwise, get its true state.
        if (perm.strings.isEmpty() || perm.strings.size == 1 && perm.strings[0]
            .isEmpty()
        ) {
          val key = perm.alias
          if (key.isNotEmpty()) {
            val existingResult = permissionsResults[key]

            // auto set permission state to GRANTED if the alias is empty.
            if (existingResult == null) {
              permissionsResults[key] = PermissionState.GRANTED
            }
          }
        } else {
          for (permString in perm.strings) {
            val key = perm.alias.ifEmpty { permString }
            var permissionStatus: PermissionState
            if (ActivityCompat.checkSelfPermission(
                activity,
                permString
              ) == PackageManager.PERMISSION_GRANTED
            ) {
              permissionStatus = PermissionState.GRANTED
            } else {
              permissionStatus = PermissionState.PROMPT

              // Check if there is a cached permission state for the "Never ask again" state
              val prefs =
                activity.getSharedPreferences("PluginPermStates", Activity.MODE_PRIVATE)
              val state = prefs.getString(permString, null)
              if (state != null) {
                permissionStatus = PermissionState.byState(state)
              }
            }
            val existingResult = permissionsResults[key]

            // multiple permissions with the same alias must all be true, otherwise all false.
            if (existingResult == null || existingResult === PermissionState.GRANTED) {
              permissionsResults[key] = permissionStatus
            }
          }
        }
      }
    }

    return permissionsResults
  }

}
