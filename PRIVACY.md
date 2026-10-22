# Privacy

## Permissions

| Permission | Why |
|---|---|
| `CAMERA` | preview and still capture |
| `RECORD_AUDIO` | reserved for video / Live Photo (not used by the still pipeline yet) |
| `READ_MEDIA_IMAGES` | future gallery integration |
| `VIBRATE` | shutter/haptic feedback |

The app does **not** request `INTERNET` and has no analytics, advertising or
telemetry SDK.

## Where photos go

Still images are written to app-specific external storage:

```
/sdcard/Android/data/com.oneplus.camera/files/Pictures/
```

They are not uploaded anywhere. MediaStore/gallery integration is a future
milestone; until then, export files manually or via adb.

## Microphone

The still camera does not record audio. The microphone permission is reserved for
the Live Photo/video milestone; it will be used only while the user records.

## Device identifiers

The app does not collect or transmit device serial numbers, advertising IDs or
account data. The package name `com.oneplus.camera` is used locally for HAL
interoperability; see `TRADEMARKS.md`.
