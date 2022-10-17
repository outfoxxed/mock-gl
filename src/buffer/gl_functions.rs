use gl::types::{GLsizei, GLuint};

use crate::function_mapping::gl_functions;

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint) [error, buffer_manager] {
		buffer_manager.gen_buffers(error, buffer_count, buffers);
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint) [error, buffer_manager] {
		buffer_manager.free_buffers(error, buffer_count, buffers);
	}
}
