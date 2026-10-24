package com.opopencam.camera

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Environment
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

private val GalleryAccent = Color(0xFFFF6A00)

/** A file-backed gallery so captures are visible before/without MediaStore. */
data class GalleryItem(val file: File) {
    val isVideo: Boolean get() = file.extension.equals("mp4", true)
    val isRaw: Boolean get() = file.extension.equals("dng", true)
}

/** Result of a DNG render: the app-specific JPEG and the refreshed gallery. */
private data class RenderOutcome(
    val file: File,
    val published: Boolean,
    val items: List<GalleryItem>,
)

fun listGalleryItems(context: Context): List<GalleryItem> {
    val pictures = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
    val movies = context.getExternalFilesDir(Environment.DIRECTORY_MOVIES)
    return sequenceOf(pictures, movies)
        .filterNotNull()
        .flatMap { dir ->
            dir.walkTopDown()
                .filter { it.isFile }
                .filter { it.extension.lowercase() in setOf("jpg", "jpeg", "dng", "mp4") }
        }
        .sortedByDescending { it.lastModified() }
        .map { GalleryItem(it) }
        .toList()
}

@Composable
fun GalleryOverlay(context: Context, onClose: () -> Unit) {
    var items by remember { mutableStateOf<List<GalleryItem>>(emptyList()) }
    var selected by remember { mutableStateOf<GalleryItem?>(null) }
    var renderInProgress by remember { mutableStateOf(false) }
    var renderMessage by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()

    fun dismissDetail() {
        // Keep the detail open while the CPU render is running so the result
        // cannot be lost behind a dismissed overlay.
        if (renderInProgress) return
        selected = null
        renderMessage = null
    }

    fun renderJpeg(item: GalleryItem) {
        if (renderInProgress) return
        renderInProgress = true
        renderMessage = null
        scope.launch {
            try {
                val outcome = withContext(Dispatchers.IO) {
                    val dir = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
                        ?: context.filesDir
                    dir.mkdirs()
                    val output = File(dir, "IMG_${System.currentTimeMillis()}.jpg")
                    // Same neutral raw_render pipeline as the desktop CLI. It is
                    // CPU-heavy (5-15 s for 50 MP) and must stay off the main
                    // thread; it also never takes the camera engine lock.
                    val rendered = runCatching {
                        NativeBridge.nativeRenderDngToJpeg(
                            item.file.absolutePath,
                            output.absolutePath,
                        )
                    }.getOrDefault(false)
                    if (!rendered) {
                        // Do not leave a truncated JPEG for the gallery to list.
                        output.delete()
                        null
                    } else {
                        val uri = MediaStoreSaver.publishImage(context, output)
                        RenderOutcome(output, uri != null, listGalleryItems(context))
                    }
                }
                if (outcome != null) items = outcome.items
                val message = when {
                    outcome == null -> "Render failed \u2014 see logcat"
                    outcome.published -> "Saved ${outcome.file.name} \u00b7 Gallery"
                    else -> "Saved ${outcome.file.name} \u00b7 not indexed"
                }
                renderMessage = message
                Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
            } catch (e: CancellationException) {
                throw e
            } catch (t: Throwable) {
                val message = "Render failed: ${t.message ?: t.javaClass.simpleName}"
                renderMessage = message
                Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
            } finally {
                renderInProgress = false
            }
        }
    }

    BackHandler {
        if (selected != null) {
            dismissDetail()
        } else {
            onClose()
        }
    }
    LaunchedEffect(Unit) {
        items = withContext(Dispatchers.IO) { listGalleryItems(context) }
    }
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF070707))
    ) {
        Column(Modifier.fillMaxSize().padding(top = 48.dp, start = 12.dp, end = 12.dp)) {
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("OpOpenCam", color = Color.White, fontWeight = FontWeight.SemiBold)
                Text(
                    "Close",
                    color = Color.White.copy(alpha = 0.75f),
                    modifier = Modifier.clickable { onClose() }.padding(8.dp),
                )
            }
            Spacer(Modifier.height(10.dp))
            if (items.isEmpty()) {
                Text(
                    "No photos or videos yet",
                    color = Color.White.copy(alpha = 0.55f),
                    modifier = Modifier.fillMaxWidth().padding(top = 80.dp),
                    textAlign = TextAlign.Center,
                )
            } else {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(3),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    items(items, key = { it.file.absolutePath }) { item ->
                        GalleryTile(context, item) {
                            if (item.isVideo) {
                                Toast.makeText(context, item.file.name, Toast.LENGTH_SHORT).show()
                            } else {
                                renderMessage = null
                                selected = item
                            }
                        }
                    }
                }
            }
        }
        selected?.let { item ->
            Box(
                Modifier.fillMaxSize().background(Color.Black).clickable { dismissDetail() },
                contentAlignment = Alignment.Center,
            ) {
                val bitmap = remember(item.file.absolutePath) {
                    if (item.isRaw) null else BitmapFactory.decodeFile(item.file.absolutePath)
                }
                if (bitmap != null) {
                    Image(
                        bitmap = bitmap.asImageBitmap(),
                        contentDescription = item.file.name,
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(8.dp)
                            .padding(bottom = if (item.isRaw) 104.dp else 0.dp),
                        contentScale = ContentScale.Fit,
                    )
                } else {
                    Text(
                        if (item.isRaw) "RAW DNG · ${item.file.name}" else item.file.name,
                        color = Color.White,
                        modifier = Modifier.padding(bottom = if (item.isRaw) 104.dp else 0.dp),
                    )
                }

                if (item.isRaw) {
                    Column(
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .fillMaxWidth()
                            .background(
                                Brush.verticalGradient(
                                    listOf(Color.Transparent, Color.Black.copy(alpha = 0.92f))
                                )
                            )
                            .navigationBarsPadding()
                            .padding(horizontal = 20.dp, vertical = 18.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Row(
                            modifier = Modifier
                                .clip(RoundedCornerShape(24.dp))
                                .background(
                                    if (renderInProgress) {
                                        Color(0xFF1A1A1A)
                                    } else {
                                        GalleryAccent.copy(alpha = 0.16f)
                                    }
                                )
                                .border(
                                    width = 1.dp,
                                    color = GalleryAccent.copy(
                                        alpha = if (renderInProgress) 0.30f else 0.65f
                                    ),
                                    shape = RoundedCornerShape(24.dp),
                                )
                                .clickable(enabled = !renderInProgress) { renderJpeg(item) }
                                .padding(horizontal = 24.dp, vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            if (renderInProgress) {
                                CircularProgressIndicator(
                                    color = GalleryAccent,
                                    strokeWidth = 2.dp,
                                    modifier = Modifier.size(16.dp),
                                )
                                Spacer(Modifier.width(10.dp))
                            }
                            Text(
                                text = if (renderInProgress) "Rendering JPEG…" else "Render JPEG",
                                color = if (renderInProgress) {
                                    Color.White.copy(alpha = 0.65f)
                                } else {
                                    Color.White
                                },
                                fontWeight = FontWeight.Medium,
                                fontSize = 14.sp,
                            )
                        }
                        Spacer(Modifier.height(9.dp))
                        Text(
                            text = renderMessage
                                ?: "Neutral render · 50 MP may take 5–15 s",
                            color = if (renderMessage?.startsWith("Render failed") == true) {
                                Color(0xFFFF5252)
                            } else {
                                Color.White.copy(alpha = 0.50f)
                            },
                            fontSize = 10.sp,
                            textAlign = TextAlign.Center,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun GalleryTile(context: Context, item: GalleryItem, onClick: () -> Unit) {
    var bitmap by remember(item.file.absolutePath) { mutableStateOf<Bitmap?>(null) }
    LaunchedEffect(item.file.absolutePath) {
        if (!item.isRaw && !item.isVideo) {
            bitmap = withContext(Dispatchers.IO) {
                BitmapFactory.decodeFile(
                    item.file.absolutePath,
                    BitmapFactory.Options().apply { inSampleSize = 4 },
                )
            }
        }
    }
    Box(
        modifier = Modifier
            .size(112.dp)
            .clip(RoundedCornerShape(4.dp))
            .background(Color(0xFF1A1A1A))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        val bmp = bitmap
        if (bmp != null) {
            Image(
                bitmap = bmp.asImageBitmap(),
                contentDescription = item.file.name,
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
            )
        } else {
            Text(
                when {
                    item.isRaw -> "DNG"
                    item.isVideo -> "VIDEO"
                    else -> "IMG"
                },
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 11.sp,
            )
        }
    }
}

/** Small latest-capture tile for the capture deck. */
@Composable
fun LatestMediaThumbnail(context: Context, item: GalleryItem?, onClick: () -> Unit) {
    var bitmap by remember(item?.file?.absolutePath) { mutableStateOf<Bitmap?>(null) }
    LaunchedEffect(item?.file?.absolutePath) {
        val file = item?.file
        if (file != null && !item.isRaw && !item.isVideo) {
            bitmap = withContext(Dispatchers.IO) {
                BitmapFactory.decodeFile(
                    file.absolutePath,
                    BitmapFactory.Options().apply { inSampleSize = 8 },
                )
            }
        }
    }
    Box(
        modifier = Modifier
            .size(56.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(Color(0xFF1A1A1A))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        val bmp = bitmap
        if (bmp != null) {
            Image(
                bitmap = bmp.asImageBitmap(),
                contentDescription = "Latest capture",
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
            )
        } else {
            Text(
                when {
                    item == null -> "·"
                    item.isRaw -> "DNG"
                    item.isVideo -> "VID"
                    else -> "IMG"
                },
                color = Color.White.copy(alpha = 0.7f),
                fontSize = 10.sp,
            )
        }
    }
}
