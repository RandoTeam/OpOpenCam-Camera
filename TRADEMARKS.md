# Trademarks and Interoperability

OpOpenCam is an independent project. It is **not affiliated with, endorsed by or
sponsored by** OnePlus, Oppo, Hasselblad, Google, or the authors of LMC/GCam ports.

Names and marks such as **OnePlus**, **Oppo**, **Hasselblad**, **Google Camera**,
**GCam**, **LMC**, **Snapdragon** and **Android** are trademarks of their respective
owners. They are used here only for identification and interoperability.

## Package identity

The APK uses `com.oneplus.camera` because the OnePlus Chi HAL exposes certain
camera features (physical camera IDs, RAW 50, vendor session parameters) to that
package name. This is an interoperability measure on the user's own device and does
not imply affiliation or endorsement.

## Reverse-engineered formats

Some modules implement publicly documented or reverse-engineered file formats
(Motion Photo XMP, DNG/TIFF, Camera2 metadata) for interoperability. No vendor
binary, LUT, EEPROM or calibration asset is redistributed.

## No warranty / legal compliance

Users are responsible for complying with the laws and terms applicable in their
jurisdiction and to their device. Do not use this project to bypass access controls,
redistribute proprietary firmware/assets, or violate vendor agreements.
