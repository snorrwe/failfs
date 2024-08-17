use std::{
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use clap::Parser;
use fuser::{FileAttr, FileType, Filesystem, MountOption};
use libc::{ENOENT, EOWNERDEAD};

const DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const REPORTED_FILE_SIZE: u64 = 1024 * 1024;
const MIN_CHUNKS: u64 = 4;

const FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: REPORTED_FILE_SIZE,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

const TTL: Duration = Duration::from_secs(1); // 1 second

#[derive(clap_derive::Parser)]
struct Args {
    /// Where to mount the filesystem
    #[arg(default_value = "/mnt/failfs")]
    mount_point: PathBuf,
    #[arg(long, short, default_value = "test.txt")]
    filename: String,
}

struct FailFs<'a> {
    name: &'a str,
}

impl Filesystem for FailFs<'_> {
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = [
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, self.name),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        if ino == 2 {
            let chunk_size = (REPORTED_FILE_SIZE / MIN_CHUNKS).min(size as u64);
            let progress = (offset + chunk_size as i64) as f64 / REPORTED_FILE_SIZE as f64;
            if progress < 0.72 {
                println!("Sending chunk. Offset={offset} Size={chunk_size} Progress={progress}");
                // repeat the template string for chunk_size bytes, so the read is
                // incomplete
                const TEMPLATE: &str = "hello world";
                let data = std::iter::repeat(TEMPLATE)
                    .flat_map(|t| t.as_bytes())
                    .copied()
                    .take(chunk_size as usize)
                    .collect::<Vec<_>>();
                reply.data(data.as_slice());
            } else {
                println!("Progress={progress}. Reporting error");
                // if not the first chunk, then raise an error
                reply.error(EOWNERDEAD);
            }
        } else {
            reply.error(ENOENT);
        }
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        if parent == 1 && name.to_str() == Some(self.name) {
            reply.entry(&TTL, &FILE_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &DIR_ATTR),
            2 => reply.attr(&TTL, &FILE_ATTR),
            _ => reply.error(ENOENT),
        }
    }
}

fn main() {
    let args = Args::parse();
    println!("Mounting FailFS at: {}", args.mount_point.display());

    let options = vec![MountOption::RO, MountOption::FSName("failfs".to_string())];

    std::fs::create_dir_all(&args.mount_point).expect("Failed to create the mount directory");

    fuser::mount2(
        FailFs {
            name: &args.filename,
        },
        args.mount_point,
        &options,
    )
    .unwrap();
}
