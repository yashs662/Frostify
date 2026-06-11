//! Optional runtime hot-patching via `subsecond` (Dioxus' framework-agnostic
//! hotpatch engine). Feature-gated behind `hotreload`; every entry point
//! degrades to a no-op shim when the feature is off, so call sites stay
//! free of `#[cfg]`.
//!
//! Run it with the Dioxus CLI (matching the pinned crate version):
//!
//! ```text
//! cargo install dioxus-cli@0.7.9
//! dx serve --hotpatch --features hotreload
//! ```
//!
//! Edit any `Component::view` body (or anything it calls) and `dx` thin-links
//! a patch + ships it over the devserver socket; the running app re-runs the
//! patched build with no restart and no state loss.
//!
//! Windows note: launch `dx serve` from a **Visual Studio developer shell**.
//! The thin-link step links against the MSVC system import libs
//! (`kernel32.lib`, `ws2_32.lib`, …) which are only on `PATH`/`LIB` inside
//! that shell — a bare PowerShell prompt fails with linker errors.

use std::sync::Arc;

use frostify_gfx::WakeHandle;

/// Re-entry point for hot-patched code. With `hotreload` on, routes the
/// closure through subsecond's jump table so an applied patch re-runs the
/// new function bodies reachable from here (the `Component::view` builders);
/// off, it calls straight through with zero overhead.
#[cfg(feature = "hotreload")]
pub fn call<O>(f: impl FnMut() -> O) -> O {
    subsecond::call(f)
}

#[cfg(not(feature = "hotreload"))]
#[inline]
pub fn call<O>(mut f: impl FnMut() -> O) -> O {
    f()
}

/// Set true by the patch handler (patch thread), drained on the UI thread by
/// [`take_patched`]. The handler can't touch the engine's `Rc<Cell<bool>>`
/// rebuild token (it's `!Send` and UI-thread-owned), so it signals through
/// this `Send` flag + the cross-thread wake instead.
#[cfg(feature = "hotreload")]
static PATCHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Connect to the `dx serve` devserver and arm a rebuild on every applied
/// patch. No-op when the feature is off.
///
/// `connect_subsecond` spawns the background thread that receives + applies
/// patches; our registered handler runs after each apply — on that thread —
/// so it only does `Send` work: latch [`PATCHED`] and wake the (possibly
/// `Wait`-parked) event loop. The actual `rebuild_scene` happens on the UI
/// thread, driven by [`take_patched`] in the per-frame tick.
pub fn connect(wake: Arc<WakeHandle>) {
    #[cfg(feature = "hotreload")]
    {
        dioxus_devtools::connect_subsecond();
        subsecond::register_handler(Arc::new(move || {
            PATCHED.store(true, std::sync::atomic::Ordering::Release);
            wake.wake();
        }));
        log::info!("hotreload: connected to dx devserver (subsecond)");
    }
    #[cfg(not(feature = "hotreload"))]
    let _ = wake;
}

/// UI-thread poll: did a patch land since the last check? Clears on read.
/// On `true`, the caller requests a scene rebuild so the patched `view`
/// bodies run. Always `false` when the feature is off.
#[inline]
pub fn take_patched() -> bool {
    #[cfg(feature = "hotreload")]
    {
        PATCHED.swap(false, std::sync::atomic::Ordering::AcqRel)
    }
    #[cfg(not(feature = "hotreload"))]
    {
        false
    }
}
