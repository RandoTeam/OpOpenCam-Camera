package com.opopencam.camera

import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer

/**
 * Counter-rotation for readable glyphs inside the rotated portrait canvas.
 *
 * The canvas itself rotates with the display, so controls keep their place
 * relative to the phone. Text and icons must not rotate with it: each of them
 * is turned back in place, around its own centre, without changing its position.
 */
val LocalGlyphRotation = compositionLocalOf { 0f }

/** Turns this glyph back in place so it stays readable in any rotation. */
@Composable
fun Modifier.upright(): Modifier {
    val degrees = LocalGlyphRotation.current
    return graphicsLayer { rotationZ = degrees }
}
