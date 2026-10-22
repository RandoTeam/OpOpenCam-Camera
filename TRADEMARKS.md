# Trademarks and Interoperability

OpOpenCam is an independent project. It is **not affiliated with, endorsed by or
sponsored by** OnePlus, Oppo, Hasselblad, Google, or the authors of LMC/GCam ports.

Names and marks such as **OnePlus**, **Oppo**, **Hasselblad**, **Google Camera**,
**GCam**, **LMC**, **OPCameraPro**, **Snapdragon** and **Android** are trademarks of
their respective owners. They are used here only for identification and
interoperability.

This project does not include Hasselblad/GR/LUMO/XPAN watermarks, branded filters or
vendor LUTs. Original looks and user-supplied LUTs are used instead.

## Package identity

The APK uses `com.oneplus.camera` because the OnePlus Chi HAL exposes certain
camera features (physical camera IDs, RAW 50, vendor session parameters) to that
package name. This is an interoperability measure on the user's own device and does
not imply affiliation or endorsement.

## Reverse-engineered formats

Some modules implement publicly documented or reverse-engineered file formats
(Motion Photo XMP, DNG/TIFF, Camera2 metadata) for interoperability. No vendor
binary, LUT, EEPROM or calibration asset is redistributed.

The GPL-3.0 `com.tlsu.opluscamerapro` source was reviewed for feature discovery
only. No module code, resources, LUTs or vendor-hook implementation is copied into
this MIT repository; see [docs/DECISIONS/0007](docs/DECISIONS/0007-clean-room-feature-parity.md).

## No warranty / legal compliance

Users are responsible for complying with the laws and terms applicable in their
jurisdiction and to their device. Do not use this project to bypass access controls,
redistribute proprietary firmware/assets, or violate vendor agreements.
