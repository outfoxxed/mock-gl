use gl::types::GLint;

use crate::{test::{test_harness, test_harness_handling}, GlVersion, version::VersionType};
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

#[test]
fn gl_errors() {
	test_harness_handling(
		GlVersion::from_version(VersionType::GL, 2, 1),
		crate::ErrorHandling::DoNotPanic,
		|| unsafe {
			gl::GenBuffers(-1, std::ptr::null_mut());
			assert_eq!(gl::GetError(), gl::INVALID_VALUE);

			gl::DeleteBuffers(-1, std::ptr::null_mut());
			assert_eq!(gl::GetError(), gl::INVALID_VALUE);

			gl::BindBuffer(gl::ARRAY_BUFFER, 1);
			assert_eq!(gl::GetError(), gl::INVALID_VALUE);

			let mut buffer = 0;
			gl::GenBuffers(1, &mut buffer);
			gl::BindBuffer(gl::TEXTURE, buffer);
			assert_eq!(gl::GetError(), gl::INVALID_ENUM);
			gl::DeleteBuffers(1, &mut buffer);
		}
	);
}
