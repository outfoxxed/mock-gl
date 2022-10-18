use std::{collections::HashMap, slice};

use gl::types::{GLboolean, GLenum, GLsizei, GLuint};

use crate::{debug, error, warning};

pub mod gl_functions;

pub struct BufferManager {
	buffer_index: GLuint,
	active_buffers: HashMap<GLuint, Buffer>,
	deleted_buffers: Vec<GLuint>,
}

pub struct Buffer {
	_id: GLuint,
}

impl BufferManager {
	pub fn new() -> Self {
		Self {
			buffer_index: 1,
			active_buffers: HashMap::new(),
			deleted_buffers: Vec::new(),
		}
	}

	pub fn gen_buffers(&mut self, error: &mut GLenum, buffer_count: GLsizei, buffers: *mut GLuint) {
		if buffer_count < 0 {
			*error = gl::INVALID_VALUE;
			error!("glGenBuffers called with invalid buffer count {}", buffer_count);
		} else {
			let buffers = unsafe { slice::from_raw_parts_mut(buffers, buffer_count as usize) };

			for buffer_id in buffers.iter_mut() {
				*buffer_id = self.buffer_index;
				self.buffer_index += 1;

				self.active_buffers.insert(*buffer_id, Buffer::new(*buffer_id));
			}

			debug!("created {} buffer(s) {:?}", buffers.len(), buffers);
		}
	}

	pub fn free_buffers(
		&mut self,
		error: &mut GLenum,
		buffer_count: GLsizei,
		buffers: *mut GLuint,
	) {
		if buffer_count < 0 {
			*error = gl::INVALID_VALUE;
			error!("glDeleteBuffers called with invalid buffer count {}", buffer_count);
		} else {
			let buffers = unsafe { slice::from_raw_parts_mut(buffers, buffer_count as usize) };

			for buffer_id in buffers.iter() {
				if self.deleted_buffers.contains(buffer_id) {
					warning!("double freed buffer {}", buffer_id);
				} else if self.active_buffers.remove(buffer_id).is_none() {
					warning!("attempted to free unallocated buffer {}", buffer_id);
				} else {
					// the above branch conditional will remove the buffer from `self.active_buffers`
					self.deleted_buffers.push(*buffer_id);
					debug!("freed {} buffer(s) {:?}", buffers.len(), buffers);
				}
			}
		}
	}

	pub fn is_buffer(&mut self, buffer: GLuint) -> GLboolean {
		self.active_buffers.contains_key(&buffer) as u8
	}

	pub fn finalize(self) {
		if !self.active_buffers.is_empty() {
			error!(
				"mock-gl context was dropped with dangling buffers {:?}",
				self.active_buffers.keys().collect::<Vec<_>>()
			);
		}
	}
}

impl Buffer {
	fn new(id: GLuint) -> Self {
		Self { _id: id }
	}
}

#[cfg(test)]
mod test {
	use gl::types::GLint;

	use crate::{test::test_harness, GlVersion};
	#[test]
	fn create_destroy() {
		test_harness(GlVersion::clear(), || unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn dangling() {
		test_harness(GlVersion::clear(), || unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn double_free() {
		test_harness(GlVersion::clear(), || unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn invalid_free() {
		test_harness(GlVersion::clear(), || unsafe {
			let buffer = 42;
			gl::DeleteBuffers(1, &buffer);
		})
	}

	#[test]
	#[should_panic]
	fn gen_negative() {
		test_harness(GlVersion::clear(), || unsafe {
			gl::GenBuffers(-1, std::ptr::null_mut());
		})
	}

	#[test]
	#[should_panic]
	fn free_negative() {
		test_harness(GlVersion::clear(), || unsafe {
			gl::DeleteBuffers(-1, std::ptr::null_mut());
		})
	}

	#[test]
	fn is_buffer() {
		test_harness(GlVersion::clear(), || unsafe {
			let mut buffer = 0;
			assert_eq!(gl::IsBuffer(buffer), gl::FALSE);
			gl::GenBuffers(1, &mut buffer);
			assert_eq!(gl::IsBuffer(buffer), gl::TRUE);
			gl::DeleteBuffers(1, &mut buffer);
			assert_eq!(gl::IsBuffer(buffer), gl::FALSE);
		})
	}
}
