mod snapshot_fs;
mod utils;
mod metadata;
mod block_io;

use snapshot_fs::SnapshotFS;
use clap::Parser;
use fuser::{self, MountOption};

#[derive(Parser)]
#[command(version)]
struct Args {
	source_dir: String,
	mount_point: String
}

fn main() {
	env_logger::init();
	let args = Args::parse();
	let options = vec![MountOption::RO, MountOption::FSName("singlefs".to_string())];
	fuser::mount2(
		SnapshotFS::new(args.source_dir),
		args.mount_point,
		&options
	).unwrap();
}
