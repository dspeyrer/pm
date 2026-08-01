use std::num::NonZero;
use std::ffi::CStr;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionError(NonZero<i32>);

impl ConnectionError {
	fn from_code(code: i32) -> Option<Self> {
		NonZero::new(code).map(Self)
	}
}

#[cfg(target_os = "macos")]
impl ConnectionError {
	pub const CHANNEL_CLOSED: Self = Self(NonZero::new(-3).unwrap());
	pub const CONNECTION_DROPPED: Self = Self(NonZero::new(-7).unwrap());
}

#[cfg(not(target_os = "macos"))]
impl ConnectionError {
	pub const CHANNEL_CLOSED: Self = Self(NonZero::new(-2).unwrap());
	pub const CONNECTION_DROPPED: Self = Self(NonZero::new(-5).unwrap());
}

pub struct Response {
	ptr: *mut u8,
	len: usize,
	cap: usize,
}

impl std::ops::Deref for Response {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
	}
}

pub struct IpcDylib {
	send_fn: extern "C" fn(*const u8, usize, *mut *mut u8, *mut usize, *mut usize) -> i32,
	free_fn: extern "C" fn(*mut u8, usize, usize),
}

impl IpcDylib {
	pub fn load() -> Option<Self> {
		let mut home = std::env::home_dir()?;

		#[cfg(target_os = "macos")]
		let locations = {
			use std::os::unix::ffi::OsStrExt;

			home.push("Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib\0");

			[
				c"/Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib",
				CStr::from_bytes_with_nul(home.as_os_str().as_bytes()).unwrap(),
			]
		};

		#[cfg(target_os = "linux")]
		let locations = [
			c"/usr/bin/1password/libop_sdk_ipc_client.so",
			c"/opt/1Password/libop_sdk_ipc_client.so",
			c"/snap/bin/1password/libop_sdk_ipc_client.so",
		];

		// TODO
		#[cfg(target_os = "windows")]
		let locations = [
 			path.Join(home, "AppData\\Local\\1Password\\op_sdk_ipc_client.dll"),
 			c"C:\\Program Files\\1Password\\app\\8\\op_sdk_ipc_client.dll",
 			c"C:\\Program Files (x86)\\1Password\\app\\8\\op_sdk_ipc_client.dll",
 			path.Join(home, "AppData\\Local\\1Password\\app\\8\\op_sdk_ipc_client.dll"),
  		];

    	locations
     		.into_iter()
       		.flat_map(Self::load_from)
     		.next()
	}

	pub fn load_from(path: &CStr) -> Option<Self> {
		let lib = dl::open(path)?;

		unsafe {
			Some(Self {
				send_fn: dl::sym!(lib, c"op_sdk_ipc_send_message")?,
				free_fn: dl::sym!(lib, c"op_sdk_ipc_free_response")?,
			})
		}
	}

	pub fn send(&self, msg: &[u8]) -> Result<Response, ConnectionError> {
		let mut ptr = std::ptr::null_mut();
		let mut len = 0;
		let mut cap = 0;

		let res = (self.send_fn)(msg.as_ptr(), msg.len(), &mut ptr, &mut len, &mut cap);

		match ConnectionError::from_code(res) {
			None => Ok(Response { ptr, len, cap }),
			Some(err) => Err(err),
		}
	}

	pub fn free(&self, res: Response) {
		(self.free_fn)(res.ptr, res.len, res.cap);
	}
}
