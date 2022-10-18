use super::{ext::ARB_buffer_storage, GlVersion};
use crate::{function_mapping::gl_functions, test::test_harness};

gl_functions! {
	fn req_extension();
	require gl 4 . 6;
	require ext ARB_buffer_storage;
	{}

	fn req_version();
	require gl 3 . 2;
	{}
}

#[test]
#[should_panic]
fn missing_extension() {
	test_harness(GlVersion::clear(), || unsafe {
		req_extension();
	})
}

#[test]
#[should_panic]
fn old_gl_version() {
	test_harness(GlVersion::clear(), || unsafe {
		req_version();
	})
}

#[test]
fn extension_present() {
	test_harness(GlVersion::from_extensions(&[&ARB_buffer_storage]), || unsafe {
		req_extension();
	})
}

#[test]
fn gl_version_met() {
	test_harness(GlVersion::from_version(super::VersionType::GL, 4, 6), || unsafe {
		req_version();
		req_extension();
	})
}

#[test]
#[should_panic]
fn es_version_met_gl() {
	test_harness(GlVersion::from_version(super::VersionType::ES, 4, 6), || unsafe {
		req_version();
	})
}
