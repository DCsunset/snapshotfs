use std::{path::Path};
use log::{warn};

use walkdir::{WalkDir, DirEntry};

pub fn read_dir(dir: impl AsRef<Path>, max_depth: usize) -> impl Iterator<Item = DirEntry> {
	// Ignore files that can't be read
	WalkDir::new(dir)
		.min_depth(1)  // skip current dir
		.max_depth(max_depth)
		.into_iter()
		.filter_map(|res| {
			// only log errors without panicking
			match res {
				Ok(e) => Some(e),
				Err(err) => {
					warn!("error reading entry: {}", err);
					None
				}
			}
		})
}
