use std::{ffi::c_void, ptr};

use crate::MockContext;

impl MockContext {
	/// Function supplying addresses of mocked OpenGL functions
	pub fn get_proc_address(&self, func: &str) -> *const c_void {
		match func {
			// gl::load_with attempts to load every function
			_ => ptr::null(),
		}
	}
}
