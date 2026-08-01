use std::ffi::{CStr, c_char, c_int, c_void};
use std::ptr::NonNull;

pub fn open(path: &CStr) -> Option<NonNull<c_void>> {
	#[cfg(windows)]
	let handle = {
		use std::os::windows::ffi::OsStrExt;
		let wpath = path.as_os_str().encode_wide().collect::<Vec<_>>();
		unsafe { LoadLibraryW(wpath.as_ptr()) }
	};

	#[cfg(unix)]
	let handle = unsafe { dlopen(path.as_ptr(), RTLD_LAZY) };

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
