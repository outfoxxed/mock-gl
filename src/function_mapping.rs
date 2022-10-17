use std::{ffi::c_void, ptr};

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

macro_rules! gl_functions {
	{$(fn $name:ident($($param:ident: $ty:ty),*$(,)?) $( -> $return:ty)? $block:block)*} => {
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
					$(stringify!($param), $param),*,
				);

				$block
			}
		)*
	};
	(print $param:ident) => { "{}: {:?}" };
	(print $param:ident, $($rest:tt),*) => {
		concat!("{}: {:?}, ", gl_functions!(print $($rest),*))
	};
}

pub(crate) use gl_functions;

mapping! {
	"glGenBuffers" | "glGenBuffersARB" => glGenBuffers;
	"glDeleteBuffers" | "glDeleteBuffersARB" => glDeleteBuffers;
}
