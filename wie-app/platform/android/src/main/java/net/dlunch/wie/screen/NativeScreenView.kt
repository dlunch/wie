package net.dlunch.wie.screen

import android.app.Activity
import android.graphics.Color
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import android.widget.LinearLayout
import androidx.annotation.Keep
import kotlin.math.ceil

@Keep
class NativeScreenView(activity: Activity, private val webview: WebView) : SurfaceHolder.Callback {
    private val parent = webview.parent as ViewGroup
    private val position = parent.indexOfChild(webview)
    private val originalLayout = webview.layoutParams
    private val app = SurfaceView(activity)
    private val layout = object : LinearLayout(activity) {
        override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
            val availableHeight = MeasureSpec.getSize(heightMeasureSpec) - paddingTop - paddingBottom
            // Match the web keypad's eight 24px rows, seven 4px gaps, and 14px padding.
            val minimumControlsHeight = ceil(234 * resources.displayMetrics.density).toInt()
            webview.layoutParams.height = if (app.visibility == View.VISIBLE) {
                (availableHeight * 260 / 740).coerceAtLeast(minimumControlsHeight)
            } else {
                availableHeight
            }
            super.onMeasure(widthMeasureSpec, heightMeasureSpec)
        }
    }

    init {
        layout.orientation = LinearLayout.VERTICAL
        layout.fitsSystemWindows = true
        layout.setBackgroundColor(Color.BLACK)
        app.visibility = View.GONE
        app.holder.addCallback(this)
        parent.removeView(webview)
        layout.addView(app, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        layout.addView(webview, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0))
        parent.addView(layout, position, originalLayout)
    }

    fun setPlaying(playing: Boolean) {
        app.visibility = if (playing) View.VISIBLE else View.GONE
        if (playing && app.holder.surface.isValid) {
            nativeSurfaceChanged(app.holder.surface, app.width, app.height)
        }
    }

    fun close() {
        nativeSurfaceDestroyed()
        app.holder.removeCallback(this)
        layout.removeView(webview)
        parent.removeView(layout)
        parent.addView(webview, position, originalLayout)
    }

    override fun surfaceCreated(holder: SurfaceHolder) = Unit

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        nativeSurfaceChanged(holder.surface, width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        nativeSurfaceDestroyed()
    }

    private external fun nativeSurfaceChanged(surface: Surface, width: Int, height: Int)
    private external fun nativeSurfaceDestroyed()
}
