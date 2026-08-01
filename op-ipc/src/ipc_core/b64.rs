const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD: u8 = '=' as u8;

#[repr(transparent)]
struct DisplayBase64<'a>(&'a [u8]);

impl std::fmt::Display for DisplayBase64<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for chunk in self.0.chunks(3) {
			let mut val = (chunk[0] as u32) << 16;

			if chunk.len() != 1 {
				val |= (chunk[1] as u32) << 8;

				if chunk.len() == 3 {
					val |= chunk[2] as u32;
				}
			}

            let mut out = [
                ALPHABET[(val >> 18       ) as usize],
                ALPHABET[(val >> 12 & 0x3F) as usize],
                ALPHABET[(val >> 6  & 0x3F) as usize],
                ALPHABET[(val       & 0x3F) as usize],
            ];

            if chunk.len() != 3 {
                out[3] = PAD;

	            if chunk.len() == 1 {
	                out[2] = PAD;
	            }
            }

			f.write_str(unsafe { std::str::from_utf8_unchecked(&out) })?;
		}

		Ok(())
	}
}

pub fn serialize_base64<S: serde::Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
	serializer.collect_str(&DisplayBase64(bytes))
}