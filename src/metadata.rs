use std::{ffi::OsString, time::SystemTime, fs, os::unix::prelude::MetadataExt, path::Path, io};

use fuser::{FileAttr, FileType};
use libc::{S_IXUSR, S_IXGRP, S_IXOTH, S_IFMT};
use crate::block_io::{Block, load_blocks, size_of_blocks};

// InodeInfo corresponds to top level dirs
pub struct InodeInfo {
	/// Include all its parent till source dir
	pub path: OsString,
	/// Blocks for reading and fast seeking in tar file
	/// Each file has a header block and a content block
	pub blocks: Option<Vec<Block>>,
	pub attr: FileAttr
}

impl InodeInfo {
	pub fn new(source_dir: impl AsRef<Path>, path: OsString) -> io::Result<Self> {
		let (blocks, attr) = InodeInfo::get_metadata(source_dir, &path)?;
		Ok(Self {
			path: path,
			blocks: blocks,
			attr: attr
		})
	}

	pub fn update_info(&mut self, source_dir: impl AsRef<Path>) -> io::Result<()> {
		let (blocks, attr) = InodeInfo::get_metadata(source_dir, &self.path)?;
		self.attr = attr;
		self.blocks = blocks;
		Ok(())
	}

	fn get_metadata(source_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> io::Result<(Option<Vec<Block>>, FileAttr)> {
		let src_metadata = fs::metadata(&path)?;
		let mut attr = derive_attr(&src_metadata,	false);
		let blocks = if src_metadata.is_dir() {
			// calculate size and blocks
			let b = load_blocks(source_dir, &path)?;
			attr.size = size_of_blocks(&b) as u64;
			attr.blocks = (attr.size + attr.blksize as u64 - 1) / attr.blksize as u64;
			Some(b)
		} else {
			// passthrough for regular files
			None
		};
		Ok((blocks, attr))
	}
}

// For root, inode must be 1, as specified in https://github.com/libfuse/libfuse/blob/master/include/fuse_lowlevel.h (FUSE_ROOT_ID)
pub const ROOT_INODE: u64 = 1;

// Derive ino from existing ino (dir)
pub fn derive_ino(ino: u64) -> u64 {
	// set least significant bit to 0 tto distinguish the root
	ino << 1
}

// Derive attr from metadata of existing directory
pub fn derive_attr(src_metadata: &fs::Metadata, root: bool) -> FileAttr {
	let cur_time = SystemTime::now();
	// permission bits (excluding the format bits)
	let mut perm = src_metadata.mode() & !S_IFMT;
	if src_metadata.is_dir() {
		// remove executable bit
		perm &= !(S_IXUSR | S_IXGRP | S_IXOTH);
	}

	FileAttr {
		ino: if root { ROOT_INODE } else { derive_ino(src_metadata.ino()) },
		size: src_metadata.size(),
		blocks: src_metadata.blocks(),
		// Convert unix timestamp to SystemTime
		atime: cur_time,
		mtime: src_metadata.modified().unwrap_or(cur_time),
		ctime: cur_time,
		crtime: cur_time, // macOS only
		kind: if root { FileType::Directory } else { FileType::RegularFile },
		perm: perm as u16,
		nlink: 0,
		uid: src_metadata.uid(),
		gid: src_metadata.gid(),
		rdev: src_metadata.rdev() as u32,
		blksize:  src_metadata.blksize() as u32,
		flags: 0 // macOS only
	}
}
