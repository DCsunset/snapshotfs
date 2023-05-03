# snapshotfs

[![crates.io](https://badgen.net/crates/v/snapshotfs)](https://crates.io/crates/snapshotfs)

A fuse-based read-only filesystem to provide a snapshot view (tar archives) of directories or files without actually creating the archives

Snapshotfs is useful for backup or file transfer without creating duplicate archives.
Currently, only Linux system is supported.

## Installation

Pre-built binaries are available at the GitHub release page.

You can also use cargo to install it:

```sh
cargo install snapshotfs
```

## Usage

To mount source dir to a mount point:

```sh
snapshotfs <SOURCE_DIR> <MOUNT_POINT>
```

The mount point will be a read-only filesystem providing a snapshot view of all entries in the the source directory.
Users should make sure the source directory doesn't change when reading the archives in snapshotfs.
Otherwise, the archives might be corrupted.

Note that the program will run in the foreground.
Add `&` to the end to make it run in the background.

See available options using `snapshotfs --help`.


## License

AGPL-3.0. Copyright notice:

```
snapshotfs
Copyright (C) 2023  DCsunset

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
