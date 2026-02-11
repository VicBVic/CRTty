pub mod effects;
pub mod gl;

#[doc(hidden)]
pub mod hook;

mod config;
mod pass;

#[doc(hidden)]
pub use libc as __libc;

use std::sync::OnceLock;

/// A post-processing effect applied to every frame.
pub trait Effect: Send + 'static {
    /// GLSL 330 core fragment shader source.
    /// Receives `in vec2 v_uv` and `uniform sampler2D u_input`.
    /// Must write `out vec4 o_color`.
    fn fragment_shader(&self) -> &str;

    /// Called once after shader compilation. Cache uniform locations here.
    fn setup(&mut self, _program: u32) {}

    /// Called each frame before draw. Set your custom uniforms.
    fn set_uniforms(&self, _program: u32, _width: i32, _height: i32, _frame: u64) {}

    /// Per-frame toggle. Default: `true`.
    fn enabled(&self) -> bool {
        true
    }

    /// Env var that must be `"1"` to activate. Default: `"ENABLE_CRTTY"`.
    fn env_var(&self) -> Option<&str> {
        Some("ENABLE_CRTTY")
    }
}

#[doc(hidden)]
pub static __PASS_FN: OnceLock<fn()> = OnceLock::new();

#[doc(hidden)]
pub fn __register_pass_fn(f: fn()) {
    let _ = __PASS_FN.set(f);
}

pub fn run_pass(effect: &mut dyn Effect) {
    pass::run_pass(effect);
}

/// Generate all `LD_PRELOAD` entry points for your effect.
/// Call once at top level: `crtty::main!(MyEffect::new());`
#[macro_export]
macro_rules! main {
    ($effect_init:expr) => {
        static __CRTTY_EFFECT: ::std::sync::OnceLock<
            ::std::sync::Mutex<::std::boxed::Box<dyn $crate::Effect>>,
        > = ::std::sync::OnceLock::new();

        fn __crtty_run_pass() {
            let mtx = __CRTTY_EFFECT
                .get_or_init(|| ::std::sync::Mutex::new(::std::boxed::Box::new($effect_init)));
            if let Ok(mut eff) = mtx.lock() {
                $crate::run_pass(&mut **eff);
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn dlsym(
            handle: *mut $crate::__libc::c_void,
            symbol: *const $crate::__libc::c_char,
        ) -> *mut $crate::__libc::c_void {
            $crate::__register_pass_fn(__crtty_run_pass);
            $crate::hook::intercepted_dlsym(handle, symbol)
        }

        #[no_mangle]
        pub unsafe extern "C" fn eglSwapBuffers(
            dpy: *mut $crate::__libc::c_void,
            surface: *mut $crate::__libc::c_void,
        ) -> $crate::__libc::c_uint {
            $crate::__register_pass_fn(__crtty_run_pass);
            $crate::hook::egl_swap_direct(dpy, surface)
        }

        #[no_mangle]
        pub unsafe extern "C" fn glXSwapBuffers(
            dpy: *mut $crate::__libc::c_void,
            drawable: $crate::__libc::c_ulong,
        ) {
            $crate::__register_pass_fn(__crtty_run_pass);
            $crate::hook::glx_swap_direct(dpy, drawable)
        }

        #[no_mangle]
        pub unsafe extern "C" fn eglGetProcAddress(
            name: *const $crate::__libc::c_char,
        ) -> *mut $crate::__libc::c_void {
            $crate::__register_pass_fn(__crtty_run_pass);
            $crate::hook::egl_get_proc_direct(name)
        }
    };
}
