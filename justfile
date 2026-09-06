#!/usr/bin/env just --justfile

target := "aarch64-linux-android"
api := "35"
host := if os() == "macos" { "darwin-x86_64" } else { "linux-x86_64" }
cc := env("ANDROID_NDK") / "toolchains/llvm/prebuilt" / host / "bin" / (target + api + "-clang")

check:
    cargo fmt --check
    cargo clippy --all-targets --target {{target}} --config target.{{target}}.linker='"{{cc}}"' -- -D warnings

build:
    cargo build --release --target {{target}} --config target.{{target}}.linker='"{{cc}}"'

package: build
    mkdir -p target/module/lib
    cp module/module.prop module/zynx.toml module/customize.sh target/module/
    cp target/{{target}}/release/liblibrepods_l2c_hook.so target/module/lib/librepods-l2c-hook.so
    rm -f target/librepods-l2c-hook.zip
    cd target/module && zip ../librepods-l2c-hook.zip module.prop zynx.toml customize.sh lib/librepods-l2c-hook.so
    unzip -t target/librepods-l2c-hook.zip
