package com.opopencam.camera

import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.dp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

/**
 * Hosts the Compose camera shell.
 *
 * The Activity only manages the window and permissions; all camera work is in
 * the Rust `opopencam` native library.
 */
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            window.attributes = window.attributes.apply {
                layoutInDisplayCutoutMode =
                    WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
            }
        }
        hideSystemBars()
        setContent {
            MaterialTheme(
                colorScheme = darkColorScheme(
                    primary = Color(0xFFFF6A00),
                    background = Color.Black,
                    surface = Color(0xFF121212),
                    onPrimary = Color.White,
                    onBackground = Color.White,
                    onSurface = Color.White,
                )
            ) {
                // One portrait canvas rotated with the display, exactly like the
                // stock camera: controls keep their place relative to the phone,
                // the preview is never re-oriented or restarted, and only glyphs
                // are counter-rotated (handled inside the screen).
                val displayRotation = LocalView.current.display?.rotation ?: 0
                val canvasRotation = when (displayRotation) {
                    0 -> 0f
                    1 -> -90f
                    2 -> 180f
                    else -> 90f
                }
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Box(
                        Modifier
                            // The canvas is exactly the portrait screen size and
                            // must ignore the landscape window constraints.
                            .requiredSize(360.dp, 792.dp)
                            .graphicsLayer { rotationZ = canvasRotation }
                    ) {
                        CompositionLocalProvider(
                            LocalGlyphRotation provides -canvasRotation
                        ) {
                            CameraScreen()
                        }
                    }
                }
            }
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) hideSystemBars()
    }

    private fun hideSystemBars() {
        WindowInsetsControllerCompat(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            hide(WindowInsetsCompat.Type.systemBars())
        }
    }
}
