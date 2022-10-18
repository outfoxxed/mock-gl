//! Crate for mocking an OpenGL context

use std::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
	sync::{Mutex, MutexGuard},
	thread::{self, ThreadId},
};

pub mod buffer;
pub mod function_mapping;
pub mod log;
pub mod version;

use gl::types::GLenum;
use version::GlVersion;

use self::log::*;

static INSTANCE: Mutex<Option<MockContextData>> = Mutex::new(None);
static META: Mutex<Option<MockContextMetadata>> = Mutex::new(None);

pub enum ErrorHandling {
	/// Panic on bad behavior instead of
	/// logging - note that some behavior cannot be logged
	/// and will always panic
	///
	/// * `warn` - panic on warnings
	PanicEarly {
		warn: bool,
	},
	/// Panic if any errors occured
	/// during the context's lifetime on finalize
	PanicOnFinalize,
	DoNotPanic,
}

pub fn new(version: GlVersion, error_handling: ErrorHandling) -> MockContextRef {
	if INSTANCE.lock().unwrap_or_else(|p| p.into_inner()).is_some() {
		panic!("Only once MockContext can exist at a time");
	}

	*META.lock().unwrap() = Some(MockContextMetadata {
		thread: thread::current().id(),
		any_errors: false,
		error_handling,
	});

	*INSTANCE.lock().unwrap_or_else(|p| p.into_inner()) = Some(MockContextData {
		gl_version: version,
		error: gl::NO_ERROR,
		buffer_manager: buffer::BufferManager::new(),
	});

	MockContextRef(PhantomData)
}

/// Mock OpenGL context
struct MockContextMetadata {
	thread: ThreadId,
	error_handling: ErrorHandling,
	any_errors: bool,
}

struct MockContextData {
	gl_version: GlVersion,
	error: GLenum,
	buffer_manager: buffer::BufferManager,
}

pub struct MockContextRef(PhantomData<()>);

impl MockContextRef {
	/// Log dangling references
	pub fn finalize(self) {
		let MockContextData {
			gl_version: _,
			error: _,
			buffer_manager,
		} = INSTANCE.lock().unwrap_or_else(|p| p.into_inner()).take().unwrap();
		buffer_manager.finalize();

		let should_panic = {
			let m = meta();
			matches!(m.error_handling, ErrorHandling::PanicOnFinalize) && m.any_errors
		};

		*META.lock().unwrap() = None;

		if should_panic {
			panic!("mock-gl: errors occured in context");
		}
	}
}

struct MockDataRef<'a>(MutexGuard<'a, Option<MockContextData>>);
struct MockMetaRef<'a>(MutexGuard<'a, Option<MockContextMetadata>>);

fn context<'a>() -> MockDataRef<'a> {
	MockDataRef(INSTANCE.lock().unwrap_or_else(|p| p.into_inner()))
}

fn meta<'a>() -> MockMetaRef<'a> {
	MockMetaRef(META.lock().unwrap())
}

impl Deref for MockDataRef<'_> {
	type Target = MockContextData;

	fn deref(&self) -> &Self::Target {
		let context = self.0.as_ref().unwrap();

		if thread::current().id() != meta().thread {
			panic!("mock-gl: context accessed off-thread");
		}

		context
	}
}

impl DerefMut for MockDataRef<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		let context = self.0.as_mut().unwrap();

		if thread::current().id() != meta().thread {
			panic!("mock-gl: context accessed off-thread");
		}

		context
	}
}

impl Deref for MockMetaRef<'_> {
	type Target = MockContextMetadata;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref().unwrap()
	}
}

impl DerefMut for MockMetaRef<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.as_mut().unwrap()
	}
}

impl Drop for MockContextRef {
	fn drop(&mut self) {
		// finalize should be called if there was no panic,
		// but in case of a panic, this should clean it up
		if META.lock().unwrap().is_some() {
			::log::warn!(
				target: "mock-gl",
				"MockContextRef dropped without calling finalize - dangling references will be ignored"
			);
		}

		*META.lock().unwrap() = None;
		*INSTANCE.lock().unwrap_or_else(|p| p.into_inner()) = None;
	}
}

#[cfg(test)]
mod test {
	use std::sync::Mutex;

	use crate::{ErrorHandling, GlVersion};

	// tests can't run in parallel  when they depend on a global variable
	static TEST_LOCK: Mutex<()> = Mutex::new(());

	pub fn test_lock(f: impl FnOnce()) {
		let _lock = TEST_LOCK.lock();
		f();
	}

	pub fn init_logger() {
		static INIT: Mutex<bool> = Mutex::new(false);
		let mut lock = INIT.lock().unwrap();
		if !*lock {
			let _ =
				env_logger::builder().filter_level(log::LevelFilter::Trace).is_test(true).init();
			*lock = true;
		}
	}

	fn mk_context() -> crate::MockContextRef {
		crate::new(GlVersion::clear(), ErrorHandling::PanicEarly { warn: true })
	}

	pub fn test_harness(version: GlVersion, f: impl FnOnce()) {
		test_lock(|| {
			init_logger();

			let context = crate::new(version, ErrorHandling::PanicEarly { warn: true });

			gl::load_with(|s| context.get_proc_address(s));

			f();

			context.finalize();
		});
	}

	#[test]
	#[should_panic]
	fn max_one_context() {
		init_logger();
		test_lock(|| {
			let _ctx1 = mk_context();
			let _ctx2 = mk_context();
		})
	}

	#[test]
	fn multiple_contexts() {
		init_logger();
		test_lock(|| {
			let ctx1 = mk_context();
			ctx1.finalize();
			let ctx2 = mk_context();
			ctx2.finalize();
		})
	}

	#[test]
	fn panic_on_finalize() {
		init_logger();
		test_lock(|| {
			let mut instant_panic = false;
			let instant_panic_ptr = &mut instant_panic as *mut bool;
			let late_panic = std::panic::catch_unwind(|| {
				let ctx = crate::new(GlVersion::clear(), ErrorHandling::PanicOnFinalize);
				let panic = std::panic::catch_unwind(|| crate::error!("this should not panic"));
				unsafe { *instant_panic_ptr = panic.is_err() };
				ctx.finalize();
			})
			.is_err();
			assert_eq!(instant_panic, false, "paniced on error");
			assert_eq!(late_panic, true, "did not panic with logged error");
		})
	}
}
