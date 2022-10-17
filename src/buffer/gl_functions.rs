use gl::types::{GLsizei, GLuint, GLboolean};

use crate::function_mapping::gl_functions;

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint) [error, buffer_manager] {
		buffer_manager.gen_buffers(error, buffer_count, buffers);
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint) [error, buffer_manager] {
		buffer_manager.free_buffers(error, buffer_count, buffers);
	}

	fn glIsBuffer(buffer: GLuint) [buffer_manager] -> GLboolean {
		buffer_manager.is_buffer(buffer)
	}
}
