#[allow(unused_imports)]
use std::ffi::{c_char, c_int, c_void};
use std::ptr::NonNull;
use std::path::Path;

pub fn open(path: &Path) -> Option<NonNull<c_void>> {
	let path_os = path.as_os_str();

	#[cfg(windows)]
	let path_enc = {
		use std::os::windows::ffi::OsStrExt;
		path_os.encode_wide().collect::<Vec<_>>()
	};

	#[cfg(unix)]
	let path_enc = {
		use std::os::unix::ffi::OsStrExt;
		path_os.as_bytes()
	};

	assert!(path_enc.last().copied() == Some(0));

	#[cfg(windows)]
	let handle = unsafe { LoadLibraryW(path_enc.as_ptr()) };

	#[cfg(unix)]
	let handle = unsafe { dlopen(path_enc.as_ptr().cast::<c_char>(), RTLD_LAZY) };

	NonNull::new(handle)
}

#[macro_export]
macro_rules! sym {
	($handle:expr, $symbol:literal) => {
		::std::mem::transmute::<_, Option<_>>(
			$crate::sym(
				::std::ptr::NonNull::as_ptr($handle),
				::std::ffi::CStr::as_ptr($symbol)
			)
		)
	}
}

#[cfg(windows)]
unsafe extern "system" {
	fn LoadLibraryW(lpFileName: *const u16) -> *mut c_void;

	#[link_name = "GetProcAddress"]
	#[doc(hidden)]
	pub fn sym(handle: *mut c_void, sym: *const c_char) -> *mut c_void;
}

#[cfg(unix)]
const RTLD_LAZY: c_int = 1;

#[cfg(unix)]
unsafe extern "C" {
	fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;

	#[link_name = "dlsym"]
	#[doc(hidden)]
	pub fn sym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
