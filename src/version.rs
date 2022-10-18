#[cfg(test)]
mod test;

pub struct GlVersion {
	pub ty: VersionType,
	pub major: u8,
	pub minor: u8,
	pub extensions: Vec<&'static GlExtension>,
}

impl GlVersion {
	pub fn at_least(&self, gl: Option<(u8, u8)>, es: Option<(u8, u8)>) -> bool {
		match (self.ty, (self.major, self.minor), gl, es) {
			(VersionType::GL, (ma, mi), Some((rma, rmi)), _)
				if ma > rma || (ma == rma && mi >= rmi) =>
				true,
			(VersionType::ES, (ma, mi), _, Some((rma, rmi)))
				if ma > rma || (ma == rma && mi >= rmi) =>
				true,
			_ => false,
		}
	}
}

macro_rules! at_least {
	($v:expr, $(gl: $gl_major:literal . $gl_minor:literal)? $(, es: $es_major:literal . $es_minor:literal)?) => {
		$v.at_least(
			$crate::version::at_least!(opt $(($gl_major, $gl_minor))?),
			$crate::version::at_least!(opt $(($es_major, $es_minor))?),
		)
	};
	(opt ) => { None };
	(opt $($expr:tt)+) => { Some($($expr)*) };
}

pub(crate) use at_least;

#[derive(Copy, Clone)]
pub enum VersionType {
	GL,
	ES,
}

pub struct GlExtension {
	pub name: &'static str,
	pub unlock_gl: Option<(u8, u8)>,
	pub unlock_es: Option<(u8, u8)>,
	pub provided_str: &'static str,
}

impl PartialEq for GlExtension {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl GlVersion {
	pub fn new(ty: VersionType, major: u8, minor: u8, extensions: &[&'static GlExtension]) -> Self {
		Self {
			ty,
			major,
			minor,
			extensions: {
				let mut ext = ext::version_extensions(ty, major, minor);
				ext.extend(extensions);
				ext
			},
		}
	}

	pub fn from_extensions(extensions: &[&'static GlExtension]) -> Self {
		Self::new(VersionType::GL, 0, 0, extensions)
	}

	pub fn from_version(ty: VersionType, major: u8, minor: u8) -> Self {
		Self::new(ty, major, minor, &[])
	}

	pub fn clear() -> Self {
		Self::new(VersionType::GL, 0, 0, &[])
	}
}

pub mod ext {
	use super::{GlExtension, VersionType};

	macro_rules! extensions {
		($($name:ident($(gl: $gl_major:literal . $gl_minor:literal)? $(, es: $es_major:literal . $es_minor:literal)?);)*) => {
			$(
				#[allow(unused, non_upper_case_globals)]
				pub const $name: GlExtension = GlExtension {
					name: stringify!($name),
					unlock_gl: extensions!(opt| $(($gl_major, $gl_minor))?),
					unlock_es: extensions!(opt| $(($es_major, $es_minor))?),
					provided_str: concat!(
						stringify!($name)
						$(, concat!(" or OpenGL ", $gl_major, ".", $gl_minor))?
						$(, concat!(" or OpenGL ES ", $es_major, ".", $es_minor))?
					),
				};
			)*

			pub(crate) fn version_extensions(ty: VersionType, major: u8, minor: u8) -> Vec<&'static GlExtension> {
				let mut extensions = Vec::new();
				$(
					match (ty, major, minor) {
						$((VersionType::GL, major, minor) if major > $gl_major || (major == $gl_major && minor >= $gl_minor)
							=> extensions.push(&$name),)?
						$((VersionType::ES, major, minor) if major > $es_major || (major == $es_major && minor >= $es_minor)
							=> extensions.push(&$name),)?
						_ => {},
					}
				)*

				extensions
			}
		};
		(opt| ) => { None };
		(opt| $($expr:tt)+) => { Some($($expr)+) };
	}

	extensions! {
		ARB_buffer_storage(gl: 4 . 6);
	}
}
