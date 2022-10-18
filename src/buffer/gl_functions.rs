use gl::types::{GLboolean, GLsizei, GLuint, GLenum};

use crate::function_mapping::gl_functions;

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint);
	take [error, buffer_manager]
	{
		buffer_manager.gen_buffers(error, buffer_count, buffers);
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint);
	take [error, buffer_manager]
	{
		buffer_manager.free_buffers(error, buffer_count, buffers);
	}

	fn glIsBuffer(buffer: GLuint) -> GLboolean;
	take [buffer_manager]
	{
		buffer_manager.is_buffer(buffer)
	}

	fn glBindBuffer(target: GLenum, buffer: GLuint);
	take [gl_version, error, buffer_manager]
	{
		buffer_manager.bind_buffer(gl_version, error, target, buffer);
	}
}
