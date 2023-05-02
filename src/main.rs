/**
 * Copyright (C) 2023 DCsunset
 * See full notice in README.md in this project
 */

mod snapshot_fs;
mod utils;
mod metadata;
mod block_io;

use std::{path::PathBuf, fs};

use snapshot_fs::SnapshotFS;
use clap::Parser;
use fuser::{self, MountOption};

#[derive(Parser)]
#[command(version)]
struct Args {
	/// The source directory to make snapshot
	source_dir: PathBuf,
	/// Mount point to mount snapshotfs
	mount_point: PathBuf,

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
	// Convert to absolute path
	let source_dir = fs::canonicalize(&args.source_dir).unwrap();
	let mut options = vec![
		MountOption::RO,
		MountOption::FSName(source_dir.to_string_lossy().to_string()),
		MountOption::Subtype("snapshotfs".to_string())
	];
	if args.allow_other {
		options.push(MountOption::AllowOther);
	}
	if args.allow_root {
		options.push(MountOption::AllowRoot);
	}

	// TODO: support background mount
	fuser::mount2(
		SnapshotFS::new(source_dir),
		args.mount_point,
		&options
	).unwrap();
}
