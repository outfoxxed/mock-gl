use gl::types::{GLsizei, GLuint};

use crate::{context, error, function_mapping::gl_functions};

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint) {
		if buffer_count < 0 {
			context().error = gl::INVALID_VALUE;
			error!("glGenBuffers called with invalid buffer count {}", buffer_count);
		} else {
			context().buffers.gen_buffers(
				std::slice::from_raw_parts_mut(buffers, buffer_count as usize)
			);
		}
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint) {
		if buffer_count < 0 {
			context().error = gl::INVALID_VALUE;
			error!("glDeleteBuffers called with invalid buffer count {}", buffer_count);
		} else {
			context().buffers.free_buffers(
				std::slice::from_raw_parts_mut(buffers, buffer_count as usize)
			);
		}
	}
}
