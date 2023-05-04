use std::{io::Write, fs::Metadata, iter};

use crate::block_io::Reader;

/// Extended attributes of pax (UTF-8)
pub struct PaxAttr(Vec<u8>);

impl PaxAttr {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn add(&mut self, key: &str, value: &str) {
		// The length of the length field
		let mut length_len: usize = 1;
		let rest_len = key.len() + value.len() + 3;
		while rest_len + length_len >= 10usize.pow(length_len as u32) {
			// +1 when the length field can't represent the total length
			length_len += 1;
		}
		let len = length_len + rest_len;
		writeln!(&mut self.0, "{len} {key}={value}").unwrap();
	}
}

// tar POSIX.1-2001/pax format (see https://man.archlinux.org/man/tar.5.en)
pub enum PaxHeader {
	Ustar(tar::Header),
	/// with extension
	Extended((tar::Header, PaxAttr, tar::Header))
}

impl PaxHeader {
	pub fn new(path: &str, metadata: &Metadata) -> std::io::Result<Self> {
		if path.len() >= 100 {
			// use pax extension for long path
			let mut h0 = tar::Header::new_ustar();
			let mut attr = PaxAttr::new();
			let mut h1 = tar::Header::new_ustar();

			attr.add("path", path);
			h0.set_entry_type(tar::EntryType::XHeader);
			h0.set_size(attr.bytes().len() as u64);
			h0.set_cksum();
			h1.set_metadata(metadata);
			h1.set_cksum();

			Ok(Self::Extended((h0, attr, h1)))
		}
		else {
			let mut h = tar::Header::new_ustar();
			h.set_metadata(metadata);
			// Strip source_dir to avoid including the full path
			h.set_path(path)?;
			h.set_cksum();
			Ok(Self::Ustar(h))
		}
	}

	pub fn to_readers(self) -> Box<dyn Iterator<Item = HeaderReader>> {
		match self {
			Self::Ustar(h) => Box::new(iter::once(HeaderReader::new(h))),
			Self::Extended((h0, attr, h1)) => Box::new(
				iter::once(HeaderReader::new(h0))
					.chain(iter::once(HeaderReader::new(attr)))
					.chain(iter::once(HeaderReader::new(h1)))
			)
		}
	}
}

pub trait AsBytes {
	fn bytes(&self) -> &[u8];
}

impl AsBytes for PaxAttr {
	fn bytes(&self) -> &[u8] { &self.0 }
}
impl AsBytes for tar::Header {
	fn bytes(&self) -> &[u8] { self.as_bytes() }
}

pub struct HeaderReader {
	header: Box<dyn AsBytes>
}

impl HeaderReader {
	// use 'static lifetime to take ownership
	pub fn new(h: impl AsBytes + 'static) -> Self {
		Self { header: Box::new(h) }
	}
}

impl Reader for HeaderReader {
	fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
		let size = buf.len();
		let off = offset as usize;
		buf.copy_from_slice(&self.header.bytes()[off..off+size]);
		Ok(size)
	}

	// header size is always 512
	fn size(&self) -> usize { self.header.bytes().len() }
}

