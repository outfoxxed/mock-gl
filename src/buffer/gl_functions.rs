use gl::types::{GLboolean, GLenum, GLsizei, GLsizeiptr, GLuint, GLvoid};

use crate::function_mapping::gl_functions;

gl_functions! {
	fn glGenBuffers(buffer_count: GLsizei, buffers: *mut GLuint);
	require gl 2 . 1;
	require es 2 . 0;
	take [error, buffer_manager]
	{
		buffer_manager.gen_buffers(error, buffer_count, buffers);
	}

	fn glDeleteBuffers(buffer_count: GLsizei, buffers: *mut GLuint);
	require gl 2 . 1;
	require es 2 . 0;
	take [error, buffer_manager]
	{
		buffer_manager.free_buffers(error, buffer_count, buffers);
	}

	fn glIsBuffer(buffer: GLuint) -> GLboolean;
	require gl 2 . 1;
	require es 2 . 0;
	take [buffer_manager]
	{
		buffer_manager.is_buffer(buffer)
	}

	fn glBindBuffer(target: GLenum, buffer: GLuint);
	require gl 2 . 1;
	require es 2 . 0;
	take [gl_version, error, buffer_manager]
	{
		buffer_manager.bind_buffer(gl_version, error, target, buffer);
	}

	fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const GLvoid, usage: GLenum);
	require gl 2 . 1;
	require es 2 . 0;
	take [gl_version, error, buffer_manager]
	{
		buffer_manager.buffer_data_target(
			gl_version,
			error,
			target,
			size,
			data,
			usage,
		);
	}

	fn glNamedBufferData(buffer: GLuint, size: GLsizeiptr, data: *const GLvoid, usage: GLenum);
	require gl 4 . 0;
	take [gl_version, error, buffer_manager]
	{
		buffer_manager.buffer_data_named(
			gl_version,
			error,
			buffer,
			size,
			data,
			usage,
		);
	}
}
