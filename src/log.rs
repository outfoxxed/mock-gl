use std::fmt;

use backtrace::{BacktraceFmt, PrintFmt};

macro_rules! error {
	(unrecoverable: $unrecoverable:expr, $fmt:literal $(, $($tt:tt)*)?) => {
		if $unrecoverable
			|| { matches!($crate::meta().error_handling, $crate::ErrorHandling::PanicOnError) } {
			::std::panic!(concat!("mock-gl: ", $fmt), $($($tt)*)?);
		} else {
			$crate::meta().any_errors = true;
			::log::error!(target: "mock-gl", concat!($fmt, "\n{:#?}") $(, $($tt)*)?, $crate::CurrentBacktrace);
		}
	};
	($fmt:literal $(, $($tt:tt)*)?) => {
		error!(unrecoverable: false, $fmt, $($($tt)*)?);
	};
}

macro_rules! debug {
	($($tt:tt)+) => {
		::log::debug!(target: "mock-gl", $($tt)+);
	}
}

pub(crate) use debug;
pub(crate) use error;

pub(crate) struct CurrentBacktrace;

impl fmt::Debug for CurrentBacktrace {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut pp = move |fmt: &mut fmt::Formatter<'_>, path: backtrace::BytesOrWideString<'_>| {
			let path = path.into_path_buf();
			fmt::Display::fmt(&path.display(), fmt)
		};
		let mut f = BacktraceFmt::new(fmt, PrintFmt::Short, &mut pp);

		f.add_context()?;

		let bt = backtrace::Backtrace::new();
		let mut frames = bt.frames().to_vec();

		let first_mgl_frame = frames.iter().enumerate().rev().find_map(|(i, frame)| {
			for symbol in frame.symbols().iter() {
				if let Some(name) = symbol.name() {
					let name = format!("{}", name);
					if name.starts_with("mock_gl") || name.starts_with("<mock_gl") {
						return Some(i)
					}
				}
			}
			None
		});
		if let Some(frame) = first_mgl_frame {
			frames.drain(..=frame);
		}

		let short_bt_frame = frames.iter().enumerate().find_map(|(i, frame)| {
			for symbol in frame.symbols().iter() {
				if let Some(name) = symbol.name() {
					let name = format!("{}", name);
					if name.contains("__rust_begin_short_backtrace") {
						return Some(i)
					}
				}
			}
			None
		});
		if let Some(frame) = short_bt_frame {
			frames.drain(frame..);
		}

		for frame in frames {
			f.frame().backtrace_frame(&frame)?;
		}

		f.finish()?;
		Ok(())
	}
}
