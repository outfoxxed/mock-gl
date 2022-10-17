//! Crate for mocking an OpenGL context

pub mod function_mapping;

/// Mock OpenGL context
pub struct MockContext {}

impl MockContext {
	/// Make sure the `MockContext` is dropped, as
	/// missing free errors are located in drop
	pub fn new() -> Self {
		Self {}
	}
}

impl Drop for MockContext {
	fn drop(&mut self) {}
}
