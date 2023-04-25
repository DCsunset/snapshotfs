use fuser::{
	Filesystem,
	FileType,
	Request,
	ReplyDirectory, FileAttr
};
use std::{
	fs::{self, DirEntry}, os::{unix::prelude::DirEntryExt, linux::fs::MetadataExt},
	time::{UNIX_EPOCH, Duration, SystemTime}, ffi::{OsString, OsStr}, io,
	collections::HashMap,
	path::Path,
	string::String
};
use log::{warn, error, debug};
use libc::{EIO, ENOENT};

pub struct SnapshotFS {
	/// Source dir
	source_dir: String,
	/// map inode to actually filename
	// TODO: garbage collect too old items
	inode_map: HashMap<u64, OsString>
}

impl SnapshotFS {
	pub fn new(source_dir: String) -> Self {
		Self {
			source_dir: source_dir,
			inode_map: HashMap::new()
		}
	}

	pub fn read_source_dir(&self) -> io::Result<impl Iterator<Item = DirEntry>> {
		let entries = fs::read_dir(&self.source_dir)?
			.filter(|res| {
				match res {
					Ok(_) => true,
					Err(e) => {
						warn!("Error reading entry: {e}");
						false
					}
				}
			})
			.map(|res| res.unwrap());
		Ok(entries)
	}

	pub fn read_source_file_attr(&self, filename: &OsStr) -> io::Result<FileAttr> {
		Ok(derive_attr(
			fs::metadata(
				Path::new(&self.source_dir)
					.join(filename)
			)?,
			false
		))
	}
}

// For root, inode must be 1, as specified in https://github.com/libfuse/libfuse/blob/master/include/fuse_lowlevel.h (FUSE_ROOT_ID)
const ROOT_INODE: u64 = 1;

// Derive ino from existing ino (dir)
fn derive_ino(ino: u64) -> u64 {
	// set least significant bit to 0 tto distinguish the root
	ino << 1
}

fn derive_attr(metadata: fs::Metadata, root: bool) -> FileAttr {
	FileAttr {
		ino: if root { ROOT_INODE } else { derive_ino(metadata.st_ino()) },
		size: metadata.st_size(),
		blocks: metadata.st_blocks(),
		// Convert unix timestamp to SystemTime
		atime: unix_time(metadata.st_atime()),
		mtime: unix_time(metadata.st_mtime()),
		ctime: unix_time(metadata.st_ctime()),
		crtime: UNIX_EPOCH, // macOS only
		kind: if root { FileType::Directory } else { FileType::RegularFile },
		perm: metadata.st_mode() as u16,
		nlink: metadata.st_nlink() as u32,
		uid: metadata.st_uid(),
		gid: metadata.st_gid(),
		rdev: metadata.st_rdev() as u32,
		blksize:  metadata.st_blksize() as u32,
		flags: 0 // macOS only
	}		
}

const TTL: Duration = Duration::from_secs(1);

fn unix_time(secs: i64) -> SystemTime {
	UNIX_EPOCH + Duration::from_secs(secs as u64)
}

impl Filesystem for SnapshotFS {
	fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: fuser::ReplyEntry) {
		if parent == ROOT_INODE {
			if let Ok(mut entries) = self.read_source_dir() {
				if let Some(e) = entries.find(|e| e.file_name() == name) {
					if let Ok(metadata) = e.metadata() {
						let attr = derive_attr(metadata, false);
						self.inode_map.insert(attr.ino, e.file_name());
						reply.entry(&TTL, &attr, 0);
						return;
					}
				}
			}
		}
		reply.error(ENOENT);
	}

	fn getattr(&mut self, _req: &Request, ino: u64, reply: fuser::ReplyAttr) {
		if ino == ROOT_INODE {
			match fs::metadata(&self.source_dir) {
				Ok(metadata) => {
					reply.attr(&TTL, &derive_attr(metadata, true));
				},
				Err(e) => {
					error!("error reading source dir: {e}");
					reply.error(e.raw_os_error().unwrap_or(EIO));
				}
			}
		} else {
			if let Some(filename) = self.inode_map.get(&ino) {
				if let Ok(attr) = self.read_source_file_attr(filename) {
					reply.attr(&TTL, &attr);
					return;
				}
			}
			
			reply.error(ENOENT);
		}
	}

	fn readdir(
		&mut self,
		_req: &Request,
		ino: u64,
		_fh: u64,
		offset: i64,
		mut reply: ReplyDirectory,
	) {
		if ino != ROOT_INODE {
			reply.error(ENOENT);
			return;
		}

		// special entries
		let mut entries = vec![
			(ROOT_INODE, FileType::Directory, OsString::from(".")),
			(ROOT_INODE, FileType::Directory, OsString::from("..")),
		];
		entries.extend(match self.read_source_dir() {
			Ok(it)=> it.map(|e| (
				derive_ino(e.ino()),
				FileType::RegularFile,
				e.file_name()
			)),
			Err(e) => {
				error!("error reading source dir: {e}");
				reply.error(e.raw_os_error().unwrap_or(EIO));
				return;
			}
		});

		for (i, e) in entries.into_iter().enumerate().skip(offset as usize) {
			self.inode_map.insert(e.0, e.2.clone());
			// offset is used by kernel for future readdir calls (should be next entry)
			if reply.add(e.0, (i+1) as i64, e.1, e.2) {
				// return true when buffer full
				break;
			}
		}

		reply.ok();
	}	
}
