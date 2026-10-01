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

@Keep
class NativeScreenView(activity: Activity, private val webview: WebView) : SurfaceHolder.Callback {
    private val parent = webview.parent as ViewGroup
    private val position = parent.indexOfChild(webview)
    private val originalLayout = webview.layoutParams
    private val layout = LinearLayout(activity)
    private val game = SurfaceView(activity)

    init {
        layout.orientation = LinearLayout.VERTICAL
        layout.fitsSystemWindows = true
        layout.setBackgroundColor(Color.BLACK)
        game.visibility = View.GONE
        game.holder.addCallback(this)
        parent.removeView(webview)
        layout.addView(game, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 480f))
        layout.addView(webview, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 260f))
        parent.addView(layout, position, originalLayout)
    }

    fun setPlaying(playing: Boolean) {
        game.visibility = if (playing) View.VISIBLE else View.GONE
        if (playing && game.holder.surface.isValid) {
            nativeSurfaceChanged(game.holder.surface, game.width, game.height)
        }
    }

    fun close() {
        nativeSurfaceDestroyed()
        game.holder.removeCallback(this)
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
