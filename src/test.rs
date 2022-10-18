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
