use std::{collections::HashMap, ffi::c_void, slice};

use enum_map::{enum_map, Enum, EnumMap};
use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid};

use crate::{debug, error, warning, GlVersion};

pub mod gl_functions;

#[cfg(test)]
mod test;

pub struct BufferManager {
	buffer_index: GLuint,
	active_buffers: HashMap<GLuint, Option<Buffer>>,
	deleted_buffers: Vec<GLuint>,
	bound_buffers: EnumMap<BufferBinding, GLuint>,
}

macro_rules! gl_enum {
	($ename:ident { $($name:ident($(gl: $gl_major:literal . $gl_minor:literal)? $(, es: $es_major:literal . $es_minor:literal)?);)* }) => {
		#[derive(Copy, Clone, Enum)]
		#[allow(non_camel_case_types)]
		pub enum $ename {
			$($name,)*
		}

		impl $ename {
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
		}

		impl std::fmt::Display for $ename {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					$(Self::$name => write!(f, concat!("GL_", stringify!($name))),)*
				}
			}
		}
	}
}

macro_rules! buffer_binding {
	($($name:ident($(gl: $gl_major:literal . $gl_minor:literal)? $(, es: $es_major:literal . $es_minor:literal)?);)*) => {
		gl_enum! {
			BufferBinding {
				$($name($(gl: $gl_major . $gl_minor)? $(, es: $es_major . $es_minor)?);)*
			}
		}

		impl BufferBinding {
			fn empty_map() -> EnumMap<Self, GLuint> {
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

gl_enum! {
	BufferUsage {
		STREAM_DRAW(gl: 2 . 1, es: 2 . 0);
		STREAM_READ(gl: 2 . 1, es: 3 . 0);
		STREAM_COPY(gl: 2 . 1, es: 3 . 0);
		STATIC_DRAW(gl: 2 . 1, es: 2 . 0);
		STATIC_READ(gl: 2 . 1, es: 3 . 0);
		STATIC_COPY(gl: 2 . 1, es: 3 . 0);
		DYNAMIC_DRAW(gl: 2 . 1, es: 2 . 0);
		DYNAMIC_READ(gl: 2 . 1, es: 3 . 0);
		DYNAMIC_COPY(gl: 2 . 1, es: 3 . 0);
	}
}

pub struct Buffer {
	usage: BufferUsage,
	memory: Vec<u8>,
}

impl BufferManager {
	pub fn new() -> Self {
		Self {
			buffer_index: 1,
			active_buffers: HashMap::new(),
			deleted_buffers: Vec::new(),
			bound_buffers: BufferBinding::empty_map(),
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

				self.active_buffers.insert(*buffer_id, None);
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
					for (_, binding) in self.bound_buffers.iter_mut() {
						if *binding == *buffer_id {
							*binding = 0;
							break
						}
					}
					debug!("freed {} buffer(s) {:?}", buffers.len(), buffers);
				}
			}
		}
	}

	pub fn is_buffer(&mut self, buffer: GLuint) -> GLboolean {
		self.active_buffers.contains_key(&buffer) as u8
	}

	pub fn bind_buffer(
		&mut self,
		gl_version: &GlVersion,
		error: &mut GLenum,
		target: GLenum,
		buffer_id: GLuint,
	) {
		let target = match BufferBinding::from_gl(target) {
			None => {
				*error = gl::INVALID_ENUM;
				error!("attempted to bind buffer {} to invalid target {}", buffer_id, target);
				return
			},
			Some(target) => {
				target.check_version(gl_version);
				target
			},
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

	fn buffer_data(
		&mut self,
		gl_version: &GlVersion,
		error: &mut GLenum,
		buffer_id: GLuint,
		size: GLsizeiptr,
		data: *const GLvoid,
		usage: GLenum,
	) {
		let usage = match BufferUsage::from_gl(usage) {
			None => {
				*error = gl::INVALID_ENUM;
				error!("called glBufferData on buffer {} with invalid usage {}", buffer_id, usage);
				return
			},
			Some(usage) => {
				usage.check_version(gl_version);
				usage
			},
		};

		if size < 0 {
			*error = gl::INVALID_VALUE;
			error!("attempted to allocate buffer {} with a negative size", buffer_id);
			return
		}

		// safe to convert size to usize due to the above check
		let (memory, memstr) = {
			// FIXME: allocation can panic
			if data == std::ptr::null() {
				(vec![0u8; size as usize], format!("new {size} byte array"))
			} else {
				let slice = unsafe { slice::from_raw_parts(data as *const u8, size as usize) };

				(Vec::from(slice), format!("{size} bytes from {data:?}"))
			}
		};

		// the buffer is assumed to exist due to being checked
		// in callers of `buffer_data`
		*self.active_buffers.get_mut(&buffer_id).unwrap() = Some(Buffer { usage, memory });

		debug!("allocated buffer {} with {} as {}", buffer_id, memstr, usage);
	}

	pub fn buffer_data_target(
		&mut self,
		gl_version: &GlVersion,
		error: &mut GLenum,
		target: GLenum,
		size: GLsizeiptr,
		data: *const GLvoid,
		usage: GLenum,
	) {
		let target = match BufferBinding::from_gl(target) {
			None => {
				*error = gl::INVALID_ENUM;
				error!("attempted to allocate a buffer for invalid target {}", target);
				return
			},
			Some(target) => {
				target.check_version(gl_version);
				target
			},
		};

		let buffer_id = match self.bound_buffers[target] {
			0 => {
				*error = gl::INVALID_OPERATION;
				error!("attempted to allocate a buffer for unbound target {}", target);
				return
			},
			x => x,
		};

		self.buffer_data(gl_version, error, buffer_id, size, data, usage);
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
