/**
 * Copyright (C) 2023 DCsunset
 * See full notice in README.md in this project
 */

mod snapshot_fs;
mod utils;
mod metadata;
mod block_io;
mod pax;

use std::{path::PathBuf, fs};

use daemonize::Daemonize;
use snapshot_fs::SnapshotFS;
use clap::Parser;
use fuser::{self, MountOption};
use anyhow::{Result, anyhow};

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
	allow_root: bool,

	/// Timeout for metadata and cache in seconds
	#[arg(short, long, default_value_t = 1)]
	timeout: u64,

	/// Unmount automatically when program exists.
	/// (need --allow-root or --allow-other; auto set one if not specified)
	#[arg(short, long)]
	auto_unmount: bool,

  /// Run in foreground
  #[arg(long)]
  foreground: bool,

  /// Redirect stdout to file (only when in background)
  #[arg(long)]
  stdout: Option<PathBuf>,

  /// Redirect stderr to file (only when in background)
  #[arg(long)]
  stderr: Option<PathBuf>
}

fn main() -> Result<()> {
	env_logger::init();
	let args = Args::parse();
	// Convert to absolute path
	let source_dir = fs::canonicalize(&args.source_dir)?;
	let mut options = vec![
		MountOption::RO,
		MountOption::FSName(source_dir.to_string_lossy().to_string()),
		MountOption::Subtype("snapshotfs".to_string()),
	];
	if args.allow_other {
		options.push(MountOption::AllowOther);
	}
	if args.allow_root {
		options.push(MountOption::AllowRoot);
	}
	if args.auto_unmount {
		options.push(MountOption::AutoUnmount);
	}

  let mount_fs = || {
	  fuser::mount2(
		  SnapshotFS::new(source_dir, args.timeout),
		  args.mount_point,
		  &options
	  )
  };

  if args.foreground {
    mount_fs()?;
  } else {
    let mut daemon = Daemonize::new().working_directory(".");
    if let Some(stdout) = args.stdout {
      daemon = daemon.stdout(std::fs::File::create(stdout)?);
    }
    if let Some(stderr) = args.stderr {
      daemon = daemon.stderr(std::fs::File::create(stderr)?);
    }

    match daemon.start() {
      Ok(_) => mount_fs()?,
      Err(e) => return Err(anyhow!("error creating daemon: {}", e))
    };
  }

  Ok(())
}
