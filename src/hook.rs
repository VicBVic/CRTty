//! dlsym / eglGetProcAddress / swap interception.

use std::ffi::CStr;
use std::sync::OnceLock;

type EGLDisplay = *mut libc::c_void;
type EGLSurface = *mut libc::c_void;
type EGLBoolean = libc::c_uint;
type XDisplay = *mut libc::c_void;
type GLXDrawable = libc::c_ulong;

const EGL_TRUE: EGLBoolean = 1;

extern "C" {
    fn dlvsym(
        handle: *mut libc::c_void,
        symbol: *const libc::c_char,
        version: *const libc::c_char,
    ) -> *mut libc::c_void;
}

type DlsymFn = unsafe extern "C" fn(*mut libc::c_void, *const libc::c_char) -> *mut libc::c_void;
type EglSwapBuffersFn = unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean;
type GlxSwapBuffersFn = unsafe extern "C" fn(XDisplay, GLXDrawable);
type EglGetProcAddressFn = unsafe extern "C" fn(*const libc::c_char) -> *mut libc::c_void;

static REAL_DLSYM: OnceLock<DlsymFn> = OnceLock::new();
static REAL_EGL_SWAP: OnceLock<EglSwapBuffersFn> = OnceLock::new();
static REAL_GLX_SWAP: OnceLock<GlxSwapBuffersFn> = OnceLock::new();
static REAL_EGL_GET_PROC: OnceLock<EglGetProcAddressFn> = OnceLock::new();

fn ensure_real_dlsym() {
    if REAL_DLSYM.get().is_some() {
        return;
    }
    unsafe {
        let fp = dlvsym(
            libc::RTLD_NEXT,
            b"dlsym\0".as_ptr() as *const _,
            b"GLIBC_2.34\0".as_ptr() as *const _,
        );
        if !fp.is_null() {
            let _ = REAL_DLSYM.set(std::mem::transmute(fp));
            return;
        }
        let fp2 = dlvsym(
            libc::RTLD_NEXT,
            b"dlsym\0".as_ptr() as *const _,
            b"GLIBC_2.2.5\0".as_ptr() as *const _,
        );
        if !fp2.is_null() {
            let _ = REAL_DLSYM.set(std::mem::transmute(fp2));
            return;
        }
    }
    eprintln!("[CRTty] FATAL: could not resolve real dlsym via dlvsym");
}

pub(crate) unsafe fn real_dlsym(
    handle: *mut libc::c_void,
    symbol: *const libc::c_char,
) -> *mut libc::c_void {
    ensure_real_dlsym();
    match REAL_DLSYM.get() {
        Some(f) => f(handle, symbol),
        None => std::ptr::null_mut(),
    }
}

pub(crate) unsafe fn get_real_gl_proc(name: *const libc::c_char) -> *mut libc::c_void {
    match REAL_EGL_GET_PROC.get() {
        Some(f) => f(name),
        None => {
            let glx_name = b"glXGetProcAddress\0";
            let glx_fn = real_dlsym(libc::RTLD_DEFAULT, glx_name.as_ptr() as *const _);
            if !glx_fn.is_null() {
                let glx_get: unsafe extern "C" fn(*const libc::c_char) -> *mut libc::c_void =
                    std::mem::transmute(glx_fn);
                glx_get(name)
            } else {
                std::ptr::null_mut()
            }
        }
    }
}

fn run_pass() {
    if let Some(f) = crate::__PASS_FN.get() {
        f();
    }
}

unsafe extern "C" fn crtty_egl_swap(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean {
    run_pass();
    match REAL_EGL_SWAP.get() {
        Some(f) => f(dpy, surface),
        None => {
            eprintln!("[CRTty] ERROR: real eglSwapBuffers not resolved");
            EGL_TRUE
        }
    }
}

unsafe extern "C" fn crtty_glx_swap(dpy: XDisplay, drawable: GLXDrawable) {
    run_pass();
    if let Some(f) = REAL_GLX_SWAP.get() {
        f(dpy, drawable);
    } else {
        eprintln!("[CRTty] ERROR: real glXSwapBuffers not resolved");
    }
}

unsafe extern "C" fn crtty_egl_get_proc_address(
    name: *const libc::c_char,
) -> *mut libc::c_void {
    if !name.is_null() {
        let sym = CStr::from_ptr(name);
        if sym.to_bytes() == b"eglSwapBuffers" {
            if let Some(real_gpa) = REAL_EGL_GET_PROC.get() {
                let real = real_gpa(name);
                if !real.is_null() {
                    let _ = REAL_EGL_SWAP.set(std::mem::transmute(real));
                }
            }
            return crtty_egl_swap as *mut libc::c_void;
        }
    }
    match REAL_EGL_GET_PROC.get() {
        Some(f) => f(name),
        None => std::ptr::null_mut(),
    }
}

pub unsafe fn intercepted_dlsym(
    handle: *mut libc::c_void,
    symbol: *const libc::c_char,
) -> *mut libc::c_void {
    ensure_real_dlsym();

    if !symbol.is_null() {
        let name = CStr::from_ptr(symbol);
        match name.to_bytes() {
            b"eglSwapBuffers" => {
                let real = real_dlsym(handle, symbol);
                if !real.is_null() {
                    let _ = REAL_EGL_SWAP.set(std::mem::transmute(real));
                    eprintln!("[CRTty] dlsym intercepted eglSwapBuffers — real={:p}", real);
                }
                return crtty_egl_swap as *mut libc::c_void;
            }
            b"glXSwapBuffers" => {
                let real = real_dlsym(handle, symbol);
                if !real.is_null() {
                    let _ = REAL_GLX_SWAP.set(std::mem::transmute(real));
                    eprintln!("[CRTty] dlsym intercepted glXSwapBuffers — real={:p}", real);
                }
                return crtty_glx_swap as *mut libc::c_void;
            }
            b"eglGetProcAddress" => {
                let real = real_dlsym(handle, symbol);
                if !real.is_null() {
                    let _ = REAL_EGL_GET_PROC.set(std::mem::transmute(real));
                    eprintln!("[CRTty] dlsym intercepted eglGetProcAddress — real={:p}", real);
                }
                return crtty_egl_get_proc_address as *mut libc::c_void;
            }
            _ => {}
        }
    }
    real_dlsym(handle, symbol)
}

pub unsafe fn egl_swap_direct(dpy: *mut libc::c_void, surface: *mut libc::c_void) -> u32 {
    crtty_egl_swap(dpy, surface)
}

pub unsafe fn glx_swap_direct(dpy: *mut libc::c_void, drawable: libc::c_ulong) {
    crtty_glx_swap(dpy, drawable)
}

pub unsafe fn egl_get_proc_direct(name: *const libc::c_char) -> *mut libc::c_void {
    crtty_egl_get_proc_address(name)
}
