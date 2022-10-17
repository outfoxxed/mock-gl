use std::collections::HashMap;

use gl::types::GLuint;

use crate::{debug, error};

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
			buffer_index: 0,
			active_buffers: HashMap::new(),
			deleted_buffers: Vec::new(),
		}
	}

	pub fn gen_buffers(&mut self, buffers: &mut [GLuint]) {
		for buffer_id in buffers.iter_mut() {
			*buffer_id = self.buffer_index;
			self.buffer_index += 1;

			self.active_buffers.insert(*buffer_id, Buffer::new(*buffer_id));
		}
		debug!("created {} buffer(s) {:?}", buffers.len(), buffers);
	}

	pub fn free_buffers(&mut self, buffers: &mut [GLuint]) {
		for buffer_id in buffers.iter() {
			if self.deleted_buffers.contains(buffer_id) {
				error!("double freed buffer {}", buffer_id);
			} else if self.active_buffers.remove(buffer_id).is_none() {
				error!("attempted to free unallocated buffer {}", buffer_id);
			} else {
				// the above branch conditional will remove the buffer from `self.active_buffers`
				self.deleted_buffers.push(*buffer_id);
				debug!("freed {} buffer(s) {:?}", buffers.len(), buffers);
			}
		}
	}
}

impl Buffer {
	fn new(id: GLuint) -> Self {
		Self { _id: id }
	}
}

impl Drop for BufferManager {
	fn drop(&mut self) {
		if !self.active_buffers.is_empty() {
			error!(
				"mock-gl context was dropped with dangling buffers {:?}",
				self.active_buffers.keys().collect::<Vec<_>>()
			);
		}
	}
}

#[cfg(test)]
mod test {
	use crate::test::test_harness;
	#[test]
	fn create_destroy() {
		test_harness(|| unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn dangling() {
		test_harness(|| unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn double_free() {
		test_harness(|| unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
			gl::DeleteBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn invalid_free() {
		test_harness(|| unsafe {
			let buffer = 42;
			gl::DeleteBuffers(1, &buffer);
		})
	}

	#[test]
	#[should_panic]
	fn gen_negative() {
		test_harness(|| unsafe {
			gl::GenBuffers(-1, std::ptr::null_mut());
		})
	}

	#[test]
	#[should_panic]
	fn free_negative() {
		test_harness(|| unsafe {
			gl::DeleteBuffers(-1, std::ptr::null_mut());
		})
	}
}
