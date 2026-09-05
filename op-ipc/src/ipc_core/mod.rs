mod b64;
mod dylib;

use dylib::{IpcDylib, ConnectionError};

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
	InitClient,
	Invoke,
	ReleaseClient,
}

#[derive(serde::Serialize)]
struct Operation<'a> {
	account_name: &'a str,
	kind: OperationKind,
	#[serde(serialize_with = "b64::serialize_base64")]
	payload: &'a [u8],
}

pub enum IpcError {
	Connection(ConnectionError),
	Parse(serde_json::Error),
	DesktopSessionExpired(Vec<u8>),
	RateLimitExceeded(Vec<u8>),
	Failure(Vec<u8>),
}

impl From<ConnectionError> for IpcError {
	fn from(err: ConnectionError) -> Self {
		Self::Connection(err)
	}
}

impl From<serde_json::Error> for IpcError {
	fn from(err: serde_json::Error) -> Self {
		Self::Parse(err)
	}
}

#[derive(serde::Deserialize)]
struct OperationResult {
	success: bool,
	payload: Vec<u8>,
}

pub struct IpcCore {
	account_name: Box<str>,
	dylib: IpcDylib,
}

impl IpcCore {
	pub fn new(account_name: impl Into<Box<str>>) -> Option<Self> {
		Some(Self {
			account_name: account_name.into(),
			dylib: IpcDylib::load()?,
		})
	}

	pub fn execute_operation(&self, kind: OperationKind, payload: &[u8]) -> Result<Vec<u8>, IpcError> {
		let op_buf = serde_json::to_vec(&Operation {
			account_name: &self.account_name,
			kind,
			payload,
		}).expect("operation does not contain non-string keys");

		let response = self.dylib.send(&op_buf)?;

		let result = serde_json::from_slice::<OperationResult>(&response)?;

		self.dylib.free(response);

		if result.success {
			Ok(result.payload)
		} else {
			Err(IpcError::Failure(result.payload))
		}
	}

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opkind_serialization() {
		assert!(
			serde_json::to_string(&OperationKind::InitClient)
				.is_ok_and(|res| res == "\"init_client\""),
		);
	}
}