//! Android performance-hint prototype.
//!
//! The primary lever is the Android Dynamic Performance Framework (ADPF),
//! exposed through the NDK `APerformanceHint*` C API on API 33+. The symbols
//! are loaded dynamically so the app still runs on older Android versions.
//!
//! `Window.setSustainedPerformanceMode` and the Game Mode / power-save state
//! are Java APIs and are queried through JNI. The app cannot *set* the game
//! mode; that requires a system permission and is controlled by the user's
//! Game Dashboard / OEM game mode.

use std::{
    sync::{Mutex, OnceLock},
    time::Duration,
};

#[cfg(target_os = "android")]
use log::{info, warn};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[repr(C)]
struct PerformanceHintManager {
    _private: [u8; 0],
}

#[repr(C)]
struct PerformanceHintSession {
    _private: [u8; 0],
}

type GetManager = unsafe extern "C" fn() -> *mut PerformanceHintManager;
type CreateSession = unsafe extern "C" fn(
    *mut PerformanceHintManager,
    *const i32,
    usize,
    i64,
) -> *mut PerformanceHintSession;
type GetPreferredUpdateRate = unsafe extern "C" fn(*mut PerformanceHintManager) -> i64;
type ReportActualWorkDuration = unsafe extern "C" fn(*mut PerformanceHintSession, i64);
type CloseSession = unsafe extern "C" fn(*mut PerformanceHintSession);

struct Adpf {
    _library: libloading::Library,
    manager: usize,
    session: usize,
    report: ReportActualWorkDuration,
    close: CloseSession,
    target_duration_ns: i64,
}

// The opaque ADPF handles are only used while holding the session mutex.
unsafe impl Send for Adpf {}

impl Adpf {
    fn new() -> Option<Self> {
        // SAFETY: loading libandroid.so and resolving the documented ADPF
        // symbols. Missing symbols (older Android) disable the prototype.
        unsafe {
            let library = libloading::Library::new("libandroid.so").ok()?;
            let get_manager = *library
                .get::<GetManager>(b"APerformanceHint_getManager\0")
                .ok()?;
            let create_session = *library
                .get::<CreateSession>(b"APerformanceHint_createSession\0")
                .ok()?;
            let get_preferred = *library
                .get::<GetPreferredUpdateRate>(b"APerformanceHint_getPreferredUpdateRateNanos\0")
                .ok()?;
            let report = *library
                .get::<ReportActualWorkDuration>(b"APerformanceHint_reportActualWorkDuration\0")
                .ok()?;
            let close = *library
                .get::<CloseSession>(b"APerformanceHint_closeSession\0")
                .ok()?;

            let manager = get_manager();
            if manager.is_null() {
                return None;
            }

            let preferred = get_preferred(manager);
            let target_duration_ns = if preferred > 0 { preferred } else { 16_666_667 };

            #[cfg(target_os = "android")]
            let tid = libc::gettid();
            #[cfg(not(target_os = "android"))]
            let tid = std::process::id() as i32;

            let session = create_session(manager, &tid, 1, target_duration_ns);
            if session.is_null() {
                return None;
            }

            Some(Self {
                _library: library,
                manager: manager as usize,
                session: session as usize,
                report,
                close,
                target_duration_ns,
            })
        }
    }

    fn report(&self, duration: Duration) {
        let nanos = duration.as_nanos().min(i64::MAX as u128) as i64;
        if nanos <= 0 {
            return;
        }

        // SAFETY: the session and function pointer were validated in `new`.
        unsafe {
            (self.report)(self.session as *mut PerformanceHintSession, nanos);
        }
    }

    fn target_duration_ns(&self) -> i64 {
        self.target_duration_ns
    }
}

impl Drop for Adpf {
    fn drop(&mut self) {
        // SAFETY: the session was created by ADPF and is closed once.
        unsafe {
            (self.close)(self.session as *mut PerformanceHintSession);
        }
        let _ = self.manager;
    }
}

static ADPF: OnceLock<Mutex<Option<Adpf>>> = OnceLock::new();

fn adpf_slot() -> &'static Mutex<Option<Adpf>> {
    ADPF.get_or_init(|| Mutex::new(None))
}

fn init_adpf() {
    let Ok(mut slot) = adpf_slot().lock() else {
        return;
    };

    if slot.is_some() {
        return;
    }

    if let Some(adpf) = Adpf::new() {
        #[cfg(target_os = "android")]
        info!(
            "android performance: ADPF session created (target {:.2} ms)",
            adpf.target_duration_ns() as f64 / 1_000_000.0
        );
        *slot = Some(adpf);
    } else {
        #[cfg(target_os = "android")]
        info!("android performance: ADPF unavailable");
    }
}

/// Reports the CPU work duration for the completed frame to ADPF.
///
/// A no-op when ADPF is unavailable or when running on another platform.
pub fn report_frame(duration: Duration) {
    let Ok(slot) = adpf_slot().lock() else {
        return;
    };

    if let Some(adpf) = slot.as_ref() {
        adpf.report(duration);
    }
}

#[cfg(target_os = "android")]
#[derive(Debug, Clone, Copy)]
struct AndroidState {
    sustained_performance: bool,
    power_save: bool,
    game_mode: i32,
}

/// Initializes the Android performance prototype.
///
/// Call this from `android_main` before the `AndroidApp` is moved into the
/// event loop. It is idempotent.
#[cfg(target_os = "android")]
pub fn init(app: &AndroidApp) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        match apply_jni_hints(app) {
            Ok(state) => info!(
                "android performance: sustained_performance={} power_save={} game_mode={}",
                state.sustained_performance, state.power_save, state.game_mode
            ),
            Err(error) => warn!("android performance: JNI state query failed: {error}"),
        }

        init_adpf();
    });
}

#[cfg(target_os = "android")]
fn apply_jni_hints(app: &AndroidApp) -> Result<AndroidState, String> {
    use jni::{JValue, JavaVM, jni_sig, jni_str, objects::JObject};

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let activity_raw = app.activity_as_ptr() as jni::sys::jobject;

    vm.attach_current_thread(|env| -> jni::errors::Result<AndroidState> {
        // SAFETY: `activity_raw` is a global reference owned by AndroidApp and
        // remains valid for the duration of this call.
        let activity = unsafe { env.as_cast_raw::<JObject>(&activity_raw)? };

        let window = env
            .call_method(
                &activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;

        let sustained_performance = env
            .call_method(
                &window,
                jni_str!("setSustainedPerformanceMode"),
                jni_sig!("(Z)V"),
                &[JValue::Bool(1)],
            )
            .is_ok();

        let power_name = env.new_string("power")?;
        let power = env
            .call_method(
                &activity,
                jni_str!("getSystemService"),
                jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                &[JValue::Object(&power_name)],
            )?
            .l()?;

        let power_save = env
            .call_method(&power, jni_str!("isPowerSaveMode"), jni_sig!("()Z"), &[])?
            .z()?;

        let game_name = env.new_string("game")?;
        let game = env
            .call_method(
                &activity,
                jni_str!("getSystemService"),
                jni_sig!("(Ljava/lang/String;)Ljava/lang/Object;"),
                &[JValue::Object(&game_name)],
            )?
            .l()?;

        let game_mode = if game.is_null() {
            -1
        } else {
            env.call_method(&game, jni_str!("getGameMode"), jni_sig!("()I"), &[])?
                .i()?
        };

        Ok(AndroidState {
            sustained_performance,
            power_save,
            game_mode,
        })
    })
    .map_err(|error| error.to_string())
}
