use std::{ffi::c_void, ptr};

use gl::types::{GLenum, GLint};

use crate::{buffer::gl_functions::*, MockContextRef};

macro_rules! mapping {
	($($($name:literal)|* => $func:expr;)*) => {
		impl MockContextRef {
			/// Function supplying addresses of mocked OpenGL functions
			pub fn get_proc_address(&self, func: &str) -> *const c_void {
				match func {
					$($($name)|* => $func as *const c_void,)*
					// gl::load_with attempts to load every function
					_ => ptr::null(),
				}
			}
		}
	}
}

// good luck
macro_rules! gl_functions {
	{$(
		fn $name:ident($($param:ident: $ty:ty),*$(,)?) $(-> $return:ty)?;
		$(require gl $gl_major:literal . $gl_minor:literal;)?
		$(require es $es_major:literal . $es_minor:literal;)?
		$(require ext $req:ident;)?
		$(take [$($take:ident),*])?
		$block:block
	)*} => {
		$(
			#[allow(non_snake_case)]
			pub(crate) unsafe extern "system" fn $name ($($param: $ty),*) $(-> $return)? {
				::log::trace!(
					target: "mock-gl",
					concat!(
						"{}(",
						gl_functions!(print $($param),*),
						")",
					),
					stringify!($name),
					$(stringify!($param), $param),*
				);

				#[allow(unused)]
				let context = &mut *$crate::context();

				let gl_version_met = $crate::version::at_least!(
					context.gl_version,
					$(gl: $gl_major . $gl_minor)?
					$(, es: $es_major . $es_minor)?
				);

				if !gl_version_met {
					if false {
					} $(else if context.gl_version.extensions.contains(&&$crate::version::ext::$req) {
					} else if true {
						$crate::error!(
							"{} requires {}",
							stringify!($name),
							$crate::version::ext::$req.provided_str
						);
					})? else {
						$crate::error!(
							"{}",
							concat!(
								stringify!($name),
								" requires ",
								$(concat!("OpenGL ", $gl_major, ".", $gl_minor),)?
								$(concat!(" or OpenGL ES ", $es_major, ".", $es_minor),)?
							)
						);
					}
				}

				let $crate::MockContextData { $($($take),*,)? .. } = context;

				$block
			}
		)*
	};
	(print ) => { "" };
	(print $param:ident) => { "{}: {:?}" };
	(print $param:ident, $($rest:tt),*) => {
		concat!("{}: {:?}, ", gl_functions!(print $($rest),*))
	};
}

pub(crate) use gl_functions;

mapping! {
	"glGetError" => glGetError;
	"glGetIntegerv" => glGetIntegerv;
	"glGenBuffers" | "glGenBuffersARB" => glGenBuffers;
	"glDeleteBuffers" | "glDeleteBuffersARB" => glDeleteBuffers;
	"glIsBuffer" | "glIsBufferARB" => glIsBuffer;
	"glBindBuffer" | "glBindBufferARB" => glBindBuffer;
	"glBufferData" | "glBufferDataARB" => glBufferData;
	"glNamedBufferData" => glNamedBufferData;
}

gl_functions! {
	fn glGetError() -> GLenum;
	require gl 2 . 1;
	require es 2 . 0;
	take [error]
	{
		let e = *error;
		*error = gl::NO_ERROR;

		e
	}

	fn glGetIntegerv(pname: GLenum, params: *mut GLint);
	require gl 2 . 1;
	require es 2 . 0;
	take [gl_version, error, buffer_manager]
	{
		let int = buffer_manager.get_int(gl_version, pname);
		if let Some(int) = int {
			*params = int;
		} else {
			*error = gl::INVALID_ENUM;
			crate::error!("mock-gl does not support glGet target {}", pname);
		}
	}
}
