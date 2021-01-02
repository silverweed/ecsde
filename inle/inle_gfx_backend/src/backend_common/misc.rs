#[cfg(debug_assertions)]
#[inline]
#[track_caller]
pub fn check_gl_err() {
    unsafe {
        let err = gl::GetError();
        match err {
            gl::NO_ERROR => {}
            gl::INVALID_ENUM => panic!("GL_INVALID_ENUM"),
            gl::INVALID_OPERATION => panic!("GL_INVALID_OPERATION"),
            gl::INVALID_VALUE => panic!("GL_INVALID_VALUE"),
            _ => panic!("Other GL error: {}", err),
        }
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn check_gl_err() {}
