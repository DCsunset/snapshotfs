mod snapshot_fs;
mod utils;
mod metadata;
mod block_io;

use std::ffi::OsString;

use snapshot_fs::SnapshotFS;
use clap::Parser;
use fuser::{self, MountOption};

#[derive(Parser)]
#[command(version)]
struct Args {
	/// The source directory to make snapshot
	source_dir: OsString,
	/// Mount point to mount snapshotfs
	mount_point: OsString,

	/// Allow other users to access the mounted fs
	#[arg(long)]
	allow_other: bool,

	/// Allow root user to access the mounted fs
	#[arg(long)]
	allow_root: bool
}

fn main() {
	env_logger::init();
	let args = Args::parse();
	// TODO: add metadata for fs (size, remaining space, etc.)
	let mut options = vec![
		MountOption::RO,
		MountOption::FSName("snapshotfs".to_string())
	];
	if args.allow_other {
		options.push(MountOption::AllowOther);
	}
	if args.allow_root {
		options.push(MountOption::AllowRoot);
	}

	// TODO: support background mount
	fuser::mount2(
		SnapshotFS::new(args.source_dir),
		args.mount_point,
		&options
	).unwrap();
}
