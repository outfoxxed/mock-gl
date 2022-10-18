use std::{collections::HashMap, slice};

use gl::types::{GLboolean, GLenum, GLsizei, GLuint, GLint};
use enum_map::{enum_map, Enum, EnumMap};

use crate::{debug, error, warning};
use crate::GlVersion;

pub mod gl_functions;

pub struct BufferManager {
	buffer_index: GLuint,
	active_buffers: HashMap<GLuint, Buffer>,
	deleted_buffers: Vec<GLuint>,
	bound_buffers: EnumMap<BufferBinding, GLuint>,
}

macro_rules! buffer_binding {
	($($name:ident($(gl: $gl_major:literal . $gl_minor:literal)? $(, es: $es_major:literal . $es_minor:literal)?);)*) => {
		#[derive(Copy, Clone, Enum)]
		#[allow(non_camel_case_types)]
		pub enum BufferBinding {
			$($name,)*
		}

		impl BufferBinding {
			pub fn from_gl(gl: GLenum) -> Option<Self> {
				match gl {
					$(gl::$name => Some(Self::$name),)*
					_ => None,
				}
			}

			pub fn check_version(&self, version: &GlVersion) {
				match self {
					$(Self::$name if !$crate::version::at_least!(version, $(gl: $gl_major . $gl_minor)? $(, es: $es_major . $es_minor)?) => {
						error!("{}", concat!(
							"GL_",
							stringify!($name),
							" requires",
							$(concat!(" OpenGL ", $gl_major, ".", $gl_minor),)?
							$(concat!(" or OpenGL ES ", $es_major, ".", $es_minor),)?
						));
					},)*
					_ => {},
				}
			}

			pub fn to_gl(&self) -> GLenum {
				match self {
					$(Self::$name => gl::$name,)*
				}
			}

			fn empty_mapping() -> EnumMap<Self, GLuint> {
				enum_map! {
					$(Self::$name => 0,)*
				}
			}

			fn check_bound(gl_version: &GlVersion, target: GLenum, map: &EnumMap<Self, GLuint>) -> Option<GLuint> {
				match target {
					$(::paste::paste!(gl::[<$name _BINDING>]) => {
						Self::$name.check_version(gl_version);
						Some(map[Self::$name])
					},)*
					_ => None,
				}
			}
		}

		impl std::fmt::Display for BufferBinding {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					$(Self::$name => write!(f, concat!("GL_", stringify!($name))),)*
				}
			}
		}
	}
}

buffer_binding! {
	ARRAY_BUFFER(gl: 2 . 1, es: 2 . 0);
	ELEMENT_ARRAY_BUFFER(gl: 2 . 1, es: 2 . 0);
	PIXEL_PACK_BUFFER(gl: 2 . 1, es: 3 . 0);
	PIXEL_UNPACK_BUFFER(gl: 2 . 1, es: 3 . 0);
	COPY_READ_BUFFER(gl: 3 . 1, es: 3 . 0);
	COPY_WRITE_BUFFER(gl: 3 . 1, es: 3 . 0);
	TEXTURE_BUFFER(gl: 3 . 1);
	TRANSFORM_FEEDBACK_BUFFER(gl: 3 . 0, es: 3 . 0);
	UNIFORM_BUFFER(gl: 3 . 1, es: 3 . 0);
	ATOMIC_COUNTER_BUFFER(gl: 4 . 2, es: 3 . 1);
	DISPATCH_INDIRECT_BUFFER(gl: 4 . 3, es: 3 . 1);
	DRAW_INDIRECT_BUFFER(gl: 4 . 0, es: 3 . 1);
	QUERY_BUFFER(gl: 4 . 4);
	SHADER_STORAGE_BUFFER(gl: 4 . 3, es: 3 . 1);
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
			bound_buffers: BufferBinding::empty_mapping(),
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

	pub fn bind_buffer(&mut self, gl_version: &GlVersion, error: &mut GLenum, target: GLenum, buffer_id: GLuint) {
		let target = match BufferBinding::from_gl(target) {
			None => {
				*error = gl::INVALID_ENUM;
				error!("attempted to bind buffer {} to invalid target {}", buffer_id, target);
				return
			},
			Some(target) => {
				target.check_version(gl_version);
				target
			}
		};

		if buffer_id == 0 {
			self.bound_buffers[target] = 0;
			debug!("unbound buffer target {}", target);
		} else if self.active_buffers.contains_key(&buffer_id) {
			self.bound_buffers[target] = buffer_id;
			debug!("bound buffer {} to {}", buffer_id, target);
		} else {
			*error = gl::INVALID_VALUE;
			if self.deleted_buffers.contains(&buffer_id) {
				error!("attempted to bind buffer that has already been freed");
			} else {
				error!("attempted to bind an unallocated buffer");
			}
		}
	}

	pub fn get_int(&self, gl_version: &GlVersion, pname: GLenum) -> Option<GLint> {
		BufferBinding::check_bound(gl_version, pname, &self.bound_buffers).map(|i| i as i32)
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

	use crate::{test::test_harness, GlVersion, version::VersionType};
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

	#[test]
	fn bind_buffer() {
		test_harness(GlVersion::from_version(VersionType::GL, 2, 1), || unsafe {
			let mut get_val: GLint = -1;
			gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut get_val);
			assert_eq!(get_val, 0);
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
			gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut get_val);
			assert_eq!(get_val, buffer as GLint);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut get_val);
			assert_eq!(get_val, 0);
			gl::DeleteBuffers(1, &mut buffer);
		})
	}

	#[test]
	#[should_panic]
	fn bind_invalid_buffer() {
		test_harness(GlVersion::from_version(VersionType::GL, 2, 1), || unsafe {
			gl::BindBuffer(gl::ARRAY_BUFFER, 1);
		})
	}

	#[test]
	#[should_panic]
	fn bind_invalid_target() {
		test_harness(GlVersion::from_version(VersionType::GL, 2, 1), || unsafe {
			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::BindBuffer(gl::TEXTURE0, buffer);
		})
	}
}
