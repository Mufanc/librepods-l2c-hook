# LibrePods L2CAP Hook

An Android ARM64 hook for [LibrePods](https://github.com/librepods-org/librepods). It hooks
`l2c_fcr_chk_chan_modes`, preserves the original function's side effects, and forces a
successful result so LibrePods can establish its L2CAP channel when Android rejects the
negotiated FCR channel modes.

## Deployment

This hook requires [Zynx](https://github.com/Mufanc/zynx) with LiteLoader enabled. LiteLoader
loads files from `/data/adb/zynx/liteloader/` using the
`<package_name>@<library_name>.so` naming convention.

Place the library in `/data/adb/zynx/liteloader/` as
`com.android.bluetooth@librepods-l2c-hook.so`. For devices whose Bluetooth package is
`com.google.android.bluetooth`, add a symlink named
`com.google.android.bluetooth@librepods-l2c-hook.so` pointing to that file. This keeps only
one physical copy of the library:

```text
/data/adb/zynx/liteloader/
├── com.android.bluetooth@librepods-l2c-hook.so
└── com.google.android.bluetooth@librepods-l2c-hook.so
    -> com.android.bluetooth@librepods-l2c-hook.so
```

Restart the Bluetooth process or reboot after deployment.
