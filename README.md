# FailFS

Create a readonly-filesystem with a single file that fails when you try to read the contents.

## Install from source

Dependencies:

- [Rust](https://rustup.rs/)
- [fuse3](https://github.com/libfuse/libfuse)
- On MacOS try using [MacFUSE](https://osxfuse.github.io/) (untested)

```sh
cargo install --path .
```

## Usage

```sh
❯ failfs -h
Usage: failfs [OPTIONS] [MOUNT_POINT]

Arguments:
  [MOUNT_POINT]  Where to mount the filesystem [default: /mnt/failsfs]

Options:
  -f, --filename <FILENAME>  [default: test.txt]
  -h, --help                 Print help
```

Example:

```sh
failfs ./test-dir -f testfile.svg
```

Unmounting the directory:

```sh
umount ./test-dir
```
