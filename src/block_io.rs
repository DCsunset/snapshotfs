use std::{os::unix::prelude::{FileExt, MetadataExt}, fs::{File, self}, ffi::OsString, io, path::Path};
use crate::utils;

pub trait Reader {
	fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
	// total size available
	fn size(&self) -> usize;
}

pub struct FileReader {
	pub path: OsString,
	pub size: usize
}

impl Reader for FileReader {
	fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
		let f = File::open(&self.path)?; f.read_at(buf, offset)
	}

	fn size(&self) -> usize { self.size }
}
pub struct HeaderReader {
	pub header: tar::Header
}

impl Reader for HeaderReader {
	fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
		let size = buf.len();
		let off = offset as usize;
		buf.copy_from_slice(&self.header.as_bytes()[off..off+size]);
		Ok(size)
	}

	// header size is always 512
	fn size(&self) -> usize { 512 }
}

/// Padding to make size of file content a multple of 512 bytes
pub struct PaddingReader {
	pub size: usize	
}

impl Reader for PaddingReader {
	fn read_at(&self, buf: &mut [u8], _offset: u64) -> io::Result<usize> {
		// Do nothing as buf is expanded with 0
		Ok(buf.len())
	}

	fn size(&self) -> usize { self.size }
}

/// Data blocks in tar (header or file content)
pub struct Block {
	pub reader: Box<dyn Reader>,
	// Offset from the first beginning of the directory archive
	pub offset: u64,
}

// Use block abstraction to simplify iteration through header and content
pub fn read_from_blocks(blocks: &Vec<Block>, offset: u64, size: usize) -> io::Result<Vec<u8>> {
	let mut data = Vec::new();

	// binary search for offset
	let idx = match blocks.binary_search_by_key(&offset, |b| b.offset) {
		Ok(i) => i,
		Err(i) => {
			// no blocks
			if i == 0 {
				return Ok(data);
			} else {
				i - 1	
			}
		}
	};

	// Offset in a block (0 for blocks except the first)
	assert!(blocks[idx].offset <= offset);
	let mut off = offset - blocks[idx].offset;
	// Offset out ot bound
	if off >= blocks[idx].reader.size() as u64 {
		return Ok(data);
	}

	// remaining size to read
	let mut remaining = size;
	for b in &blocks[idx..] {
		let size = b.reader.size() - off as usize;
		let cur_len = data.len();
		let size_to_read = std::cmp::min(size, remaining);
		data.resize(cur_len + size_to_read as usize, 0);
		let ret = b.reader.read_at(&mut data[cur_len..], off)?;
		if ret != size_to_read {
			return Err(io::Error::new(
				io::ErrorKind::UnexpectedEof,
				"file size has changed"
			));
		}

		remaining -= size_to_read;
		if remaining == 0 {
			break;
		}
		// Offset is 0 for subsequent blocks
		off = 0;
	}
	Ok(data)
}


// read from corresponding source dir to compute the blocks
pub fn load_blocks(source_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> io::Result<Vec<Block>> {
	let mut blocks: Vec<Block> = Vec::new();
	// current offset from beginning of the tar file
	let mut offset = 0;
	for e in utils::read_dir(path, 0, usize::MAX) {
		let m = fs::metadata(e.path())?;
		let mut h = tar::Header::new_gnu();
		h.set_metadata(&m);
		h.set_path(e.path().strip_prefix(&source_dir).unwrap())?;
		h.set_cksum();

		// header block
		blocks.push(Block {
			reader: Box::new(HeaderReader { header: h }),
			offset: offset
		});
		offset += 512;  // header size

		// file content block (for directory or symlink, no content is added)
		if m.is_file() {
			let size = m.size();
			blocks.push(Block {
				reader: Box::new(FileReader {
					path: e.path().as_os_str().to_os_string(),
					size: size as usize
				}),
				offset: offset
			});
			offset += size;

			if size % 512 != 0 {
				// Add padding to make offset a multiple of 512 bytes
				let padding = 512 - size % 512;
				if padding > 0 {
					blocks.push(Block {
						reader: Box::new(PaddingReader {
							size: padding as usize
						}),
						offset: offset
					});
					offset += padding;
				}
			}
		}
	}
	// End of archive (at least two consecutive zero-filled 512-byte blocks)
	blocks.push(Block {
		reader: Box::new(PaddingReader {
			size: 1024
		}),
		offset: offset
	});
	
	Ok(blocks)
}

pub fn size_of_blocks(blocks: &Vec<Block>) -> usize {
	match blocks.last() {
		Some(b) => (b.offset as usize) + b.reader.size(),
		None => 0
	}
}
