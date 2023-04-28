use fuser::{
	Filesystem,
	FileType,
	Request,
	ReplyDirectory
};
use std::{
	fs,
	time::Duration, ffi::{OsString, OsStr}, io,
	collections::HashMap,
	path::Path,
	string::String
};
use log::{error, debug, warn};
use libc::{EIO, ENOENT};
use crate::{metadata::{InodeInfo, derive_attr, ROOT_INODE}, block_io::read_from_blocks};
use crate::utils;

// TTL for fuse reply
// TODO: add an option to change it
const TTL: Duration = Duration::from_secs(1);

pub struct SnapshotFS {
	/// Source dir
	source_dir: String,
	/// map inode to actually filename
	// TODO: garbage collect too old items
	inode_map: HashMap<u64, InodeInfo>,
	// Map file name to inode
	file_map: HashMap<OsString, u64>
}

impl SnapshotFS {
	pub fn new(source_dir: String) -> Self {
		Self {
			source_dir: source_dir,
			inode_map: HashMap::new(),
			file_map: HashMap::new()
		}
	}

	// Add file to file_map and inode_map if it doesn't exist
	pub fn add_file(&mut self, name: &OsStr) -> io::Result<&InodeInfo> {
		match self.file_map.get(name) {
			Some(ino) => Ok(self.inode_map.get(ino).unwrap()),
			None => {
				let path = Path::new(&self.source_dir).join(name).as_os_str().to_os_string();
				match InodeInfo::new(path) {
					Ok(info) => {
						let ino = info.attr.ino;
						debug!("adding file {:?}: ino {}", name, ino);
						self.file_map.insert(name.to_os_string(), ino);
						self.inode_map.insert(ino, info);
						Ok(self.inode_map.get(&ino).unwrap())
					}
					Err(err) => {
						warn!("error create inode info: {}", err);
						Err(err)
					}
				}
			}
		}
	}
}


impl Filesystem for SnapshotFS {
	fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: fuser::ReplyEntry) {
		if parent == ROOT_INODE {
			match self.file_map.get(name) {
				Some(ino) => {
					let info = self.inode_map.get_mut(ino).unwrap();
					match info.update_info() {
						Ok(_) => {
							reply.entry(&TTL, &info.attr, 0);
							return;
						}
						Err(err) => {
							warn!("error updating inode info: {}", err);
							self.inode_map.remove(ino);
							if let Some(e) = err.raw_os_error() {
								reply.error(e);
								return;
							}
						}
					};
				},
				None => {
					if utils::read_dir(&self.source_dir, 1).find(|e| e.file_name() == name).is_some() {
						match self.add_file(name) {
							Ok(info) => {
								reply.entry(&TTL, &info.attr, 0);
								return;		
							},
							Err(err) => {
								if let Some(e) = err.raw_os_error() {
									reply.error(e);
									return;
								}
							}
						};
					}
				}
			};
		}
		reply.error(ENOENT);
	}

	fn getattr(&mut self, _req: &Request, ino: u64, reply: fuser::ReplyAttr) {
		if ino == ROOT_INODE {
			match fs::metadata(&self.source_dir) {
				Ok(metadata) => {
					reply.attr(&TTL, &derive_attr(&metadata, true));
				},
				Err(e) => {
					error!("error reading source dir: {e}");
					reply.error(e.raw_os_error().unwrap_or(EIO));
				}
			}
		} else {
			if let Some(info) = self.inode_map.get_mut(&ino) {
				match info.update_info() {
					Ok(_) => {
						reply.attr(&TTL, &info.attr);
						return;
					},
					Err(err) => {
						warn!("error updating inode info: {}", err);
					}
				}
			}
			reply.error(ENOENT);
		}
	}

	fn readdir(
		&mut self,
		_req: &Request,
		ino: u64,
		_fh: u64,  // use inode only as we returned a dummy fh for opendir (by default 0)
		offset: i64,
		mut reply: ReplyDirectory,
	) {
		if ino != ROOT_INODE {
			reply.error(ENOENT);
			return;
		}
		assert!(offset >= 0);

		// special entries
		let mut entries = vec![
			(ROOT_INODE, FileType::Directory, OsString::from(".")),
			(ROOT_INODE, FileType::Directory, OsString::from("..")),
		];
		entries.extend(utils::read_dir(&self.source_dir.clone(), 1).filter_map(|e| {
			match self.add_file(e.file_name()) {
				Ok(info) => {
					Some((
						info.attr.ino,
						FileType::RegularFile,
						e.file_name().to_os_string()
					))
				},
				Err(_) => None
			}
		}));

		for (i, e) in entries.into_iter().enumerate().skip(offset as usize) {
			// offset is used by kernel for future readdir calls (should be next entry)
			if reply.add(e.0, (i+1) as i64, e.1, e.2) {
				// return true when buffer full
				break;
			}
		}

		reply.ok();
	}	

	fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: fuser::ReplyOpen) {
		match self.inode_map.get_mut(&ino) {
			Some(info) => {
				match info.update_info() {
					Ok(_) => reply.opened(0, 0),
					Err(err) => {
						warn!("error opening file {:?}: {}", info.path, err);
						if let Some(e) = err.raw_os_error() {
							reply.error(e);
							return;
						}
					}
				}
				// Return dummy fh and flags as we only use ino in read
			},
			None => reply.error(ENOENT)
		};
	}

	fn read(
		&mut self,
		_req: &Request,
		ino: u64,
		_fh: u64,
		offset: i64,
		size: u32,
		_flags: i32,
		_lock_owner: Option<u64>,
		reply: fuser::ReplyData,
	) {
		assert!(offset >= 0);
		match self.inode_map.get(&ino) {
			Some(info) => {
				// Headers should be available already
				if info.blocks.is_none() {
					error!("files not available: {:?}", info.path);
					reply.error(EIO);
					return;
				}
				
				// Use block abstraction to simplify iteration
				match read_from_blocks(info.blocks.as_ref().unwrap(), offset as u64, size as usize) {
					Ok(data) => {
						reply.data(&data);
					},
					Err(err) => {
						error!("error reading file {:?}: {}", info.path, err);
						reply.error(EIO);
					}
				}
			},
			None => reply.error(ENOENT)
		};
	}
}
