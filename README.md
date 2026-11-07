# LibrePods L2CAP Hook

An Android ARM64 hook for [LibrePods](https://github.com/librepods-org/librepods). It hooks
`l2c_fcr_chk_chan_modes`, preserves the original function's side effects, and forces a
successful result so LibrePods can establish its L2CAP channel when Android rejects the
negotiated FCR channel modes.

## Deployment

Requires an ARM64 device and Zynx with manifest version 2 support. Set `ANDROID_NDK`
to your installed NDK directory, then build the module:

```sh
just package
adb push target/librepods-l2c-hook.zip /sdcard/Download/
```

Install the ZIP through KernelSU Manager alongside the Zynx module, then reboot.
The manifest targets both supported Bluetooth package names; no symlinks are needed.
Remove any previous LiteLoader copy before rebooting to avoid loading the hook twice.
