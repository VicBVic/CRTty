//! Render pass engine.

use crate::gl;
use crate::gl::*;
use crate::Effect;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime};

static STATE: Mutex<Option<PassState>> = Mutex::new(None);
static FRAME: AtomicU64 = AtomicU64::new(0);
static RELOAD: Mutex<Option<ReloadState>> = Mutex::new(None);

struct ReloadState {
    path: String,
    last_mtime: SystemTime,
    check_counter: u64,
}

struct PassState {
    program: GLuint,
    vao: GLuint,
    tex: GLuint,
    tex_w: GLsizei,
    tex_h: GLsizei,
    u_input: GLint,
    u_time: GLint,
    u_resolution: GLint,
    start: Instant,
}

const VERT_SRC: &str = r#"#version 330 core
out vec2 v_uv;
void main() {
    float x = float((gl_VertexID & 1) << 2) - 1.0;
    float y = float((gl_VertexID & 2) << 1) - 1.0;
    v_uv = vec2(x, y) * 0.5 + 0.5;
    gl_Position = vec4(x, y, 0.0, 1.0);
}
"#;

pub fn run_pass(effect: &mut dyn Effect) {
    // Check activation env var (cached)
    static ENV_OK: OnceLock<bool> = OnceLock::new();
    let env_ok = *ENV_OK.get_or_init(|| match effect.env_var() {
        Some(var) => std::env::var(var).map(|v| v == "1").unwrap_or(false),
        None => true,
    });
    if !env_ok {
        return;
    }
    if !effect.enabled() {
        return;
    }

    gl::ensure_loaded();

    let mut guard = STATE.lock().unwrap();
    if guard.is_none() {
        // Set up hot-reload if the effect has a file path
        if let Some(path) = effect.shader_path() {
            let mtime = std::fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            eprintln!("[CRTty] hot-reload watching: {path}");
            *RELOAD.lock().unwrap() = Some(ReloadState {
                path: path.to_string(),
                last_mtime: mtime,
                check_counter: 0,
            });
        }
        match unsafe { init_state(effect) } {
            Ok(s) => *guard = Some(s),
            Err(e) => {
                eprintln!("[CRTty] init failed: {}", e);
                return;
            }
        }
    }

    // Hot-reload: check file mtime every ~10 frames
    {
        let mut reload = RELOAD.lock().unwrap();
        if let Some(ref mut rs) = *reload {
            rs.check_counter += 1;
            if rs.check_counter % 10 == 0 {
                let cur_mtime = std::fs::metadata(&rs.path)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                if cur_mtime != rs.last_mtime {
                    rs.last_mtime = cur_mtime;
                    eprintln!("[CRTty] file changed, recompiling...");
                    match std::fs::read_to_string(&rs.path) {
                        Ok(new_src) => match unsafe { compile_program(VERT_SRC, &new_src) } {
                            Ok(new_prog) => {
                                let state = guard.as_mut().unwrap();
                                unsafe {
                                    (glDeleteProgram.unwrap())(state.program);
                                }
                                state.program = new_prog;
                                state.u_input = unsafe {
                                    (glGetUniformLocation.unwrap())(
                                        new_prog,
                                        b"u_input\0".as_ptr() as *const _,
                                    )
                                };
                                state.u_time = unsafe {
                                    (glGetUniformLocation.unwrap())(
                                        new_prog,
                                        b"u_time\0".as_ptr() as *const _,
                                    )
                                };
                                state.u_resolution = unsafe {
                                    (glGetUniformLocation.unwrap())(
                                        new_prog,
                                        b"u_resolution\0".as_ptr() as *const _,
                                    )
                                };
                                effect.setup(new_prog);
                                eprintln!("[CRTty] shader reloaded from {}", rs.path);
                            }
                            Err(e) => eprintln!("[CRTty] reload compile error: {e}"),
                        },
                        Err(e) => eprintln!("[CRTty] failed to read {}: {e}", rs.path),
                    }
                }
            }
        }
    }

    let state = guard.as_mut().unwrap();
    unsafe { do_pass(state, effect) };

    let n = FRAME.fetch_add(1, Ordering::Relaxed);
    if n == 0 || n == 60 || n % 600 == 0 {
        eprintln!("[CRTty] pass OK — frame {}", n);
    }
}

unsafe fn init_state(effect: &mut dyn Effect) -> Result<PassState, String> {
    let program = compile_program(VERT_SRC, effect.fragment_shader())?;

    effect.setup(program);

    let mut vao: GLuint = 0;
    (glGenVertexArrays.unwrap())(1, &mut vao);

    let mut tex: GLuint = 0;
    (glGenTextures.unwrap())(1, &mut tex);
    (glBindTexture.unwrap())(GL_TEXTURE_2D, tex);
    (glTexParameteri.unwrap())(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    (glTexParameteri.unwrap())(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    (glTexParameteri.unwrap())(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    (glTexParameteri.unwrap())(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    (glBindTexture.unwrap())(GL_TEXTURE_2D, 0);

    let u_input = (glGetUniformLocation.unwrap())(program, b"u_input\0".as_ptr() as *const _);
    let u_time = (glGetUniformLocation.unwrap())(program, b"u_time\0".as_ptr() as *const _);
    let u_resolution =
        (glGetUniformLocation.unwrap())(program, b"u_resolution\0".as_ptr() as *const _);

    eprintln!(
        "[CRTty] Initialized \u{2014} program={}, vao={}, tex={}",
        program, vao, tex,
    );

    Ok(PassState {
        program,
        vao,
        tex,
        tex_w: 0,
        tex_h: 0,
        u_input,
        u_time,
        u_resolution,
        start: Instant::now(),
    })
}

unsafe fn do_pass(state: &mut PassState, effect: &dyn Effect) {
    // 1. Query viewport
    let mut vp = [0i32; 4];
    (glGetIntegerv.unwrap())(GL_VIEWPORT, vp.as_mut_ptr());
    let (w, h) = (vp[2], vp[3]);
    if w <= 0 || h <= 0 {
        return;
    }

    let saved = SavedState::save();

    // 3. Ensure capture texture matches viewport
    (glActiveTexture.unwrap())(GL_TEXTURE0);
    (glBindTexture.unwrap())(GL_TEXTURE_2D, state.tex);

    if state.tex_w != w || state.tex_h != h {
        (glTexImage2D.unwrap())(
            GL_TEXTURE_2D,
            0,
            GL_RGBA8 as GLint,
            w,
            h,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            std::ptr::null(),
        );
        state.tex_w = w;
        state.tex_h = h;
        eprintln!("[CRTty] Capture texture resized to {}x{}", w, h);
    }

    // 4. Copy framebuffer -> texture
    (glBindFramebuffer.unwrap())(GL_READ_FRAMEBUFFER, 0);
    (glCopyTexSubImage2D.unwrap())(GL_TEXTURE_2D, 0, 0, 0, vp[0], vp[1], w, h);

    // 5. Draw fullscreen triangle
    (glBindFramebuffer.unwrap())(GL_DRAW_FRAMEBUFFER, 0);
    (glViewport.unwrap())(vp[0], vp[1], w, h);
    (glDisable.unwrap())(GL_DEPTH_TEST);
    (glDisable.unwrap())(GL_STENCIL_TEST);
    (glDisable.unwrap())(GL_SCISSOR_TEST);
    (glDisable.unwrap())(GL_CULL_FACE);
    (glDisable.unwrap())(GL_BLEND);

    (glUseProgram.unwrap())(state.program);
    (glUniform1i.unwrap())(state.u_input, 0);

    if state.u_time >= 0 {
        (glUniform1f.unwrap())(state.u_time, state.start.elapsed().as_secs_f32());
    }
    if state.u_resolution >= 0 {
        (glUniform2f.unwrap())(state.u_resolution, w as f32, h as f32);
    }

    let frame = FRAME.load(Ordering::Relaxed);
    effect.set_uniforms(state.program, w, h, frame);

    (glBindVertexArray.unwrap())(state.vao);
    (glDrawArrays.unwrap())(GL_TRIANGLES, 0, 3);

    // 6. Restore
    saved.restore();
}

struct SavedState {
    program: GLint,
    active_tex: GLint,
    tex_2d: GLint,
    fbo_read: GLint,
    fbo_draw: GLint,
    vao: GLint,
    viewport: [GLint; 4],
    blend: bool,
    depth: bool,
    scissor: bool,
    cull: bool,
    stencil: bool,
}

impl SavedState {
    unsafe fn save() -> Self {
        let mut s = SavedState {
            program: 0,
            active_tex: 0,
            tex_2d: 0,
            fbo_read: 0,
            fbo_draw: 0,
            vao: 0,
            viewport: [0; 4],
            blend: false,
            depth: false,
            scissor: false,
            cull: false,
            stencil: false,
        };
        (glGetIntegerv.unwrap())(GL_CURRENT_PROGRAM, &mut s.program);
        (glGetIntegerv.unwrap())(GL_ACTIVE_TEXTURE, &mut s.active_tex);
        (glGetIntegerv.unwrap())(GL_TEXTURE_BINDING_2D, &mut s.tex_2d);
        (glGetIntegerv.unwrap())(GL_READ_FRAMEBUFFER_BINDING, &mut s.fbo_read);
        (glGetIntegerv.unwrap())(GL_DRAW_FRAMEBUFFER_BINDING, &mut s.fbo_draw);
        (glGetIntegerv.unwrap())(GL_VERTEX_ARRAY_BINDING, &mut s.vao);
        (glGetIntegerv.unwrap())(GL_VIEWPORT, s.viewport.as_mut_ptr());
        s.blend = (glIsEnabled.unwrap())(GL_BLEND) != GL_FALSE;
        s.depth = (glIsEnabled.unwrap())(GL_DEPTH_TEST) != GL_FALSE;
        s.scissor = (glIsEnabled.unwrap())(GL_SCISSOR_TEST) != GL_FALSE;
        s.cull = (glIsEnabled.unwrap())(GL_CULL_FACE) != GL_FALSE;
        s.stencil = (glIsEnabled.unwrap())(GL_STENCIL_TEST) != GL_FALSE;
        s
    }

    unsafe fn restore(&self) {
        (glUseProgram.unwrap())(self.program as GLuint);
        (glActiveTexture.unwrap())(self.active_tex as GLenum);
        (glBindTexture.unwrap())(GL_TEXTURE_2D, self.tex_2d as GLuint);
        (glBindFramebuffer.unwrap())(GL_READ_FRAMEBUFFER, self.fbo_read as GLuint);
        (glBindFramebuffer.unwrap())(GL_DRAW_FRAMEBUFFER, self.fbo_draw as GLuint);
        (glBindVertexArray.unwrap())(self.vao as GLuint);
        (glViewport.unwrap())(
            self.viewport[0],
            self.viewport[1],
            self.viewport[2],
            self.viewport[3],
        );
        fn toggle(flag: GLenum, on: bool) {
            unsafe {
                if on {
                    (glEnable.unwrap())(flag)
                } else {
                    (glDisable.unwrap())(flag)
                }
            }
        }
        toggle(GL_BLEND, self.blend);
        toggle(GL_DEPTH_TEST, self.depth);
        toggle(GL_SCISSOR_TEST, self.scissor);
        toggle(GL_CULL_FACE, self.cull);
        toggle(GL_STENCIL_TEST, self.stencil);
    }
}

unsafe fn compile_shader(kind: GLenum, source: &str) -> Result<GLuint, String> {
    let create = glCreateShader.get().ok_or("glCreateShader not loaded")?;
    let shader = create(kind);
    if shader == 0 {
        return Err("glCreateShader returned 0".into());
    }
    let src_ptr = source.as_ptr() as *const GLchar;
    let src_len = source.len() as GLint;
    (glShaderSource.unwrap())(shader, 1, &src_ptr, &src_len);
    (glCompileShader.unwrap())(shader);

    let mut status: GLint = 0;
    (glGetShaderiv.unwrap())(shader, GL_COMPILE_STATUS, &mut status);
    if status == 0 {
        let mut len: GLint = 0;
        (glGetShaderiv.unwrap())(shader, GL_INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        (glGetShaderInfoLog.unwrap())(
            shader,
            len,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
        );
        let msg = String::from_utf8_lossy(&buf);
        (glDeleteShader.unwrap())(shader);
        return Err(format!("shader compile error: {}", msg));
    }
    Ok(shader)
}

unsafe fn compile_program(vert_src: &str, frag_src: &str) -> Result<GLuint, String> {
    let vs = compile_shader(GL_VERTEX_SHADER, vert_src)?;
    let fs = match compile_shader(GL_FRAGMENT_SHADER, frag_src) {
        Ok(fs) => fs,
        Err(e) => {
            (glDeleteShader.unwrap())(vs);
            return Err(e);
        }
    };

    let program = (glCreateProgram.unwrap())();
    if program == 0 {
        (glDeleteShader.unwrap())(vs);
        (glDeleteShader.unwrap())(fs);
        return Err("glCreateProgram returned 0".into());
    }
    (glAttachShader.unwrap())(program, vs);
    (glAttachShader.unwrap())(program, fs);
    (glLinkProgram.unwrap())(program);

    (glDeleteShader.unwrap())(vs);
    (glDeleteShader.unwrap())(fs);

    let mut status: GLint = 0;
    (glGetProgramiv.unwrap())(program, GL_LINK_STATUS, &mut status);
    if status == 0 {
        let mut len: GLint = 0;
        (glGetProgramiv.unwrap())(program, GL_INFO_LOG_LENGTH, &mut len);
        let mut buf = vec![0u8; len as usize];
        (glGetProgramInfoLog.unwrap())(
            program,
            len,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut _,
        );
        let msg = String::from_utf8_lossy(&buf);
        (glDeleteProgram.unwrap())(program);
        return Err(format!("program link error: {}", msg));
    }

    Ok(program)
}
