use android_logger::Config;
use ctor::ctor;
use libc::{c_int, c_void, dl_phdr_info};
use log::{LevelFilter, error, info};
use r3solvr::{BasicResolver, Query, SymbolResolver};
use std::collections::HashSet;
use std::error::Error;
use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{mem, thread};
use wisp::{Wisp, orig_fn};

const TARGET_LIBRARIES: [&str; 2] = ["libbluetooth_jni.so", "libbluetooth_qti.so"];
const TARGET_SYMBOL: &str = "_Z22l2c_fcr_chk_chan_modesP8tL2C_CCB";
const POLL_INTERVAL: Duration = Duration::from_millis(100);

type HookResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Eq, Hash, PartialEq)]
struct LoadedLibrary {
    base: usize,
    path: PathBuf,
}

#[ctor(unsafe)]
fn init() {
    android_logger::init_once(
        Config::default()
            .with_tag("LibrePodsL2cHook")
            .with_max_level(LevelFilter::Info),
    );

    if let Err(error) = thread::Builder::new()
        .name("l2c-fcr-hook".into())
        .spawn(hook_when_loaded)
    {
        error!("failed to start hook worker: {error}");
    }
}

fn hook_when_loaded() {
    let mut attempted = HashSet::new();
    info!("hook worker started; waiting for Bluetooth library");

    loop {
        for library in loaded_target_libraries() {
            if !attempted.insert(library.clone()) {
                continue;
            }

            match install_hook(&library.path, library.base) {
                Ok(address) => {
                    info!(
                        "hooked l2c_fcr_chk_chan_modes in {} at {address:#x}",
                        library.path.display()
                    );
                    return;
                }
                Err(error) => {
                    error!("failed to hook {}: {error}", library.path.display());
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn install_hook(path: &Path, base: usize) -> HookResult<usize> {
    let address = resolve_runtime_address(path, base)?;

    // SAFETY: `address` belongs to a loaded executable image and the proxy uses
    // the same C ABI as the target function.
    let stub = unsafe {
        Wisp::hook_fn(
            address as *const c_void,
            hook_l2c_fcr_chk_chan_modes as *const c_void,
            None,
        )
    }?;

    // The hook is permanent for the lifetime of the Bluetooth process.
    mem::forget(stub);
    Ok(address)
}

#[inline(never)]
extern "C" fn hook_l2c_fcr_chk_chan_modes(channel: *mut c_void) -> u8 {
    let original = orig_fn!();
    // SAFETY: Wisp returns the trampoline for the target function whose ABI is
    // declared here.
    let original: extern "C" fn(*mut c_void) -> u8 = unsafe { mem::transmute(original) };

    let _ = original(channel);
    1
}

fn resolve_runtime_address(path: &Path, base: usize) -> HookResult<usize> {
    let resolver = BasicResolver::from_file(path)?;
    let symbol = resolver.lookup_symbol(Query::new(TARGET_SYMBOL).with_debugdata(true))?;

    base.checked_add(symbol.addr)
        .ok_or_else(|| std::io::Error::other("symbol runtime address overflowed").into())
}

fn is_target_library(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| TARGET_LIBRARIES.contains(&name))
}

fn loaded_target_libraries() -> Vec<LoadedLibrary> {
    let mut libraries = Vec::new();
    // SAFETY: the callback only accesses `libraries` during this call.
    unsafe {
        libc::dl_iterate_phdr(
            Some(collect_target_library),
            (&mut libraries as *mut Vec<LoadedLibrary>).cast(),
        );
    }
    libraries
}

unsafe extern "C" fn collect_target_library(
    info: *mut dl_phdr_info,
    _size: usize,
    data: *mut c_void,
) -> c_int {
    if info.is_null() || data.is_null() {
        return 0;
    }

    // SAFETY: both pointers are checked above and supplied by
    // `dl_iterate_phdr`/`loaded_target_libraries` for this callback invocation.
    let info = unsafe { &*info };
    if info.dlpi_name.is_null() {
        return 0;
    }

    let name = unsafe { CStr::from_ptr(info.dlpi_name) };
    let path = Path::new(OsStr::from_bytes(name.to_bytes()));

    if is_target_library(path) {
        let libraries = unsafe { &mut *data.cast::<Vec<LoadedLibrary>>() };
        libraries.push(LoadedLibrary {
            base: info.dlpi_addr as usize,
            path: path.into(),
        });
    }

    0
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn identifies_target_libraries() {
        assert!(super::is_target_library(Path::new(
            "/apex/com.android.bt/lib64/libbluetooth_jni.so"
        )));
        assert!(super::is_target_library(Path::new(
            "/vendor/lib64/libbluetooth_qti.so"
        )));
        assert!(!super::is_target_library(Path::new(
            "/system/lib64/libbluetooth.so"
        )));
    }
}
