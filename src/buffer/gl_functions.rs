use gl::types::{GLsizei, GLuint};

use crate::function_mapping::gl_functions;

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint) {
		crate::context().buffers.gen_buffers(
			std::slice::from_raw_parts_mut(buffers, buffer_count as usize)
		)
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint) {
		crate::context().buffers.free_buffers(
			std::slice::from_raw_parts_mut(buffers, buffer_count as usize)
		)
	}
}
