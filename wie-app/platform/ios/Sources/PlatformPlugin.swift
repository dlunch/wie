import GoogleMobileAds
import Tauri
import UIKit
import WebKit

class PlatformPlugin: Plugin {
    private var navigation: AdvertisingNavigation?

    @objc override func load(webview: WKWebView) {
        let navigation = AdvertisingNavigation(webview: webview)
        self.navigation = navigation
        webview.navigationDelegate = navigation
        webview.uiDelegate = navigation
        MobileAds.shared.register(webview)
    }

    @objc func initializeAdvertising(_ invoke: Invoke) {
        // Tauri loads the plugin on the main thread before dispatching commands.
        invoke.resolve()
    }
}

private class AdvertisingNavigation: NSObject, WKNavigationDelegate, WKUIDelegate {
    private let navigationDelegate: WKNavigationDelegate?
    private let uiDelegate: WKUIDelegate?

    init(webview: WKWebView) {
        navigationDelegate = webview.navigationDelegate
        uiDelegate = webview.uiDelegate
        super.init()
    }

    override func responds(to selector: Selector!) -> Bool {
        super.responds(to: selector)
            || navigationDelegate?.responds(to: selector) == true
            || uiDelegate?.responds(to: selector) == true
    }

    override func forwardingTarget(for selector: Selector!) -> Any? {
        if navigationDelegate?.responds(to: selector) == true { return navigationDelegate }
        if uiDelegate?.responds(to: selector) == true { return uiDelegate }
        return super.forwardingTarget(for: selector)
    }

    func webView(
        _ webView: WKWebView,
        decidePolicyFor action: WKNavigationAction,
        decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
    ) {
        if action.targetFrame?.isMainFrame == true,
           let currentHost = webView.url?.host,
           let url = action.request.url,
           url.host != currentHost {
            openExternal(url)
            decisionHandler(.cancel)
        } else if navigationDelegate?.webView?(webView, decidePolicyFor: action, decisionHandler: decisionHandler) == nil {
            decisionHandler(.allow)
        }
    }

    func webView(
        _ webView: WKWebView,
        createWebViewWith configuration: WKWebViewConfiguration,
        for action: WKNavigationAction,
        windowFeatures: WKWindowFeatures
    ) -> WKWebView? {
        if action.targetFrame == nil, let url = action.request.url {
            openExternal(url)
            return nil
        }
        return uiDelegate?.webView?(
            webView, createWebViewWith: configuration, for: action, windowFeatures: windowFeatures
        )
    }

    private func openExternal(_ url: URL) {
        if url.scheme == "https" || url.scheme == "http" || url.scheme == "itms-apps" {
            UIApplication.shared.open(url)
        }
    }
}

@_cdecl("init_plugin_wie_platform")
func initPlugin() -> Plugin {
    PlatformPlugin()
}
