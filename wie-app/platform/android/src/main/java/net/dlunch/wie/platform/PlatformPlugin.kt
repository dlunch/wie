package net.dlunch.wie.platform

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.os.Message
import android.util.Log
import android.view.View
import android.webkit.ConsoleMessage
import android.webkit.CookieManager
import android.webkit.GeolocationPermissions
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.google.android.libraries.ads.mobile.sdk.MobileAds
import com.google.android.libraries.ads.mobile.sdk.initialization.InitializationConfig
import java.util.concurrent.CompletableFuture

@TauriPlugin
class PlatformPlugin(private val activity: Activity) : Plugin(activity) {
    private val advertisingReady = CompletableFuture<Void>()

    override fun load(webView: WebView) {
        webView.webViewClient = AdvertisingNavigationClient(webView.webViewClient, ::openExternal)
        webView.webChromeClient = AdvertisingChromeClient(webView.webChromeClient!!, ::openExternal)
        webView.settings.setSupportMultipleWindows(true)
        CookieManager.getInstance().setAcceptThirdPartyCookies(webView, true)
        webView.settings.mediaPlaybackRequiresUserGesture = false
        Thread {
            try {
                MobileAds.initialize(
                    activity,
                    InitializationConfig.Builder(InitializationConfig.WEBVIEW_APIS_FOR_ADS_APPLICATION_ID).build(),
                ) {
                    activity.runOnUiThread {
                        try {
                            MobileAds.registerWebView(webView)
                            advertisingReady.complete(null)
                        } catch (error: Exception) {
                            advertisingReady.completeExceptionally(error)
                        }
                    }
                }
            } catch (error: Exception) {
                advertisingReady.completeExceptionally(error)
            }
        }.start()
    }

    @Command
    fun initializeAdvertising(invoke: Invoke) {
        advertisingReady.whenComplete { _, error ->
            if (error == null) {
                invoke.resolve()
            } else {
                invoke.reject("Advertising initialization failed: ${error.message}")
            }
        }
    }

    private fun openExternal(uri: Uri) {
        if (uri.scheme != "https" && uri.scheme != "http" && uri.scheme != "market") return
        try {
            activity.startActivity(Intent(Intent.ACTION_VIEW, uri).addCategory(Intent.CATEGORY_BROWSABLE))
        } catch (error: ActivityNotFoundException) {
            Log.w("WiePlatform", "No browser is available to open $uri", error)
        }
    }
}

private class AdvertisingNavigationClient(
    private val delegate: WebViewClient,
    private val openExternal: (Uri) -> Unit,
) : WebViewClient() {
    override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
        val currentHost = view.url?.let(Uri::parse)?.host
        if (request.isForMainFrame && currentHost != null && request.url.host != currentHost) {
            openExternal(request.url)
            return true
        }
        return delegate.shouldOverrideUrlLoading(view, request)
    }

    override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? =
        delegate.shouldInterceptRequest(view, request)

    override fun onPageStarted(view: WebView, url: String, favicon: Bitmap?) = delegate.onPageStarted(view, url, favicon)

    override fun onPageFinished(view: WebView, url: String) = delegate.onPageFinished(view, url)

    override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) =
        delegate.onReceivedError(view, request, error)
}

private class AdvertisingChromeClient(
    private val delegate: WebChromeClient,
    private val openExternal: (Uri) -> Unit,
) : WebChromeClient() {
    override fun onCreateWindow(view: WebView, isDialog: Boolean, isUserGesture: Boolean, resultMsg: Message): Boolean {
        if (!isUserGesture) return false
        // A popup gets no Tauri scripts or bridge and is destroyed as soon as its destination is known.
        val popup = WebView(view.context)
        popup.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
                if (request.url.toString() == "about:blank") return false
                openExternal(request.url)
                view.destroy()
                return true
            }
        }
        (resultMsg.obj as WebView.WebViewTransport).webView = popup
        resultMsg.sendToTarget()
        return true
    }

    override fun onShowCustomView(view: View, callback: CustomViewCallback) = delegate.onShowCustomView(view, callback)

    override fun onHideCustomView() = delegate.onHideCustomView()

    override fun onPermissionRequest(request: PermissionRequest) = delegate.onPermissionRequest(request)

    override fun getDefaultVideoPoster(): Bitmap? = delegate.defaultVideoPoster

    override fun onJsAlert(view: WebView, url: String, message: String, result: JsResult): Boolean =
        delegate.onJsAlert(view, url, message, result)

    override fun onJsConfirm(view: WebView, url: String, message: String, result: JsResult): Boolean =
        delegate.onJsConfirm(view, url, message, result)

    override fun onJsPrompt(view: WebView, url: String, message: String, defaultValue: String, result: JsPromptResult): Boolean =
        delegate.onJsPrompt(view, url, message, defaultValue, result)

    override fun onGeolocationPermissionsShowPrompt(origin: String, callback: GeolocationPermissions.Callback) =
        delegate.onGeolocationPermissionsShowPrompt(origin, callback)

    override fun onShowFileChooser(view: WebView, callback: ValueCallback<Array<Uri?>?>, params: FileChooserParams): Boolean =
        delegate.onShowFileChooser(view, callback, params)

    override fun onConsoleMessage(message: ConsoleMessage): Boolean = delegate.onConsoleMessage(message)

    override fun onReceivedTitle(view: WebView, title: String) = delegate.onReceivedTitle(view, title)
}
