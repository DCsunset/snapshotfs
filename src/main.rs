mod singlefs;

use singlefs::SingleFS;
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
		SingleFS::new(args.source_dir),
		args.mount_point,
		&options
	).unwrap();
}
