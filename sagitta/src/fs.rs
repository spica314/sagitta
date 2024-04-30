use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    path::PathBuf,
    time::SystemTime,
};

use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyCreate, ReplyDirectory, ReplyOpen,
    ReplyWrite, TimeOrNow,
};
use libc::{ENOENT, EOPNOTSUPP, EPERM};
use log::info;
use sagitta_common::clock::Clock;
use sagitta_local_system_workspace::LocalSystemWorkspaceManager;
use sagitta_objects::{SagittaTreeObject, SagittaTreeObjectDir};
use std::time::Duration;

use crate::api_client::SagittaApiClient;

#[derive(Debug)]
pub struct SagittaFS {
    pub config: SagittaConfig,
    pub next_inode: u64,
    pub ino_to_path: HashMap<u64, Vec<String>>,
    pub path_to_ino: HashMap<Vec<String>, u64>,
    pub client: SagittaApiClient,
    pub clock: Clock,
    pub local_system_workspace_manager: LocalSystemWorkspaceManager,
    pub next_fh: u64,
}

impl Filesystem for SagittaFS {
    fn access(&mut self, _req: &fuser::Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        info!("access(ino={}, mask={})", ino, mask);
        reply.ok();
    }

    // fn bmap(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     blocksize: u32,
    //     idx: u64,
    //     reply: fuser::ReplyBmap,
    // ) {
    //     info!("bmap(ino={}, blocksize={}, idx={})", ino, blocksize, idx);
    //     reply.error(ENOSYS);
    // }

    // fn copy_file_range(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino_in: u64,
    //     fh_in: u64,
    //     offset_in: i64,
    //     ino_out: u64,
    //     fh_out: u64,
    //     offset_out: i64,
    //     len: u64,
    //     flags: u32,
    //     reply: ReplyWrite,
    // ) {
    //     info!("copy_file_range(ino_in={}, fh_in={}, offset_in={}, ino_out={}, fh_out={}, offset_out={}, len={}, flags={})", ino_in, fh_in, offset_in, ino_out, fh_out, offset_out, len, flags);
    //     reply.error(ENOSYS);
    // }

    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        info!(
            "create(parent={}, name={:?}, mode={}, umask={}, flags={})",
            parent, name, mode, umask, flags
        );

        let parent_path = self.ino_to_path.get(&parent).unwrap().clone();
        let mut file_path = parent_path.clone();
        file_path.push(name.to_str().unwrap().to_string());

        if file_path[0] == "trunk" {
            reply.error(EPERM);
            return;
        }

        self.local_system_workspace_manager
            .create_cow_file(&file_path[0], &file_path[1..], &[])
            .unwrap();

        let attr = self.get_file_attr(
            &file_path[..file_path.len() - 1],
            &file_path[file_path.len() - 1],
        );
        let attr = attr.unwrap();
        reply.created(&Duration::from_secs(0), &attr, 0, 0, 0);
    }

    // fn destroy(&mut self) {
    //     info!("destroy()");
    // }

    // fn fallocate(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     offset: i64,
    //     length: i64,
    //     mode: i32,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!(
    //         "fallocate(ino={}, fh={}, offset={}, length={}, mode={})",
    //         ino, fh, offset, length, mode
    //     );
    //     reply.error(ENOSYS);
    // }

    fn flush(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        info!("flush(ino={}, fh={}, lock_owner={})", ino, fh, lock_owner);
        reply.ok();
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        info!("forget(ino={}, nlookup={})", ino, nlookup);
        let path = self.ino_to_path.get(&ino).unwrap().clone();
        self.local_system_workspace_manager
            .delete_cow_file(&path[0], &path[1..])
            .unwrap();
    }

    // fn fsync(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     datasync: bool,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!("fsync(ino={}, fh={}, datasync={})", ino, fh, datasync);
    //     reply.ok();
    // }

    // fn fsyncdir(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     datasync: bool,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!("fsyncdir(ino={}, fh={}, datasync={})", ino, fh, datasync);
    //     reply.error(ENOSYS);
    // }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        // std::thread::sleep(Duration::from_secs(1));
        info!("getattr(ino={})", ino);

        if ino == 1 {
            let attr = self.get_root_file_attr();
            reply.attr(&Duration::from_secs(0), &attr);
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();

        let attr = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if let Some(attr) = attr {
            reply.attr(&Duration::from_secs(0), &attr);
        } else {
            reply.error(ENOENT);
        }
    }

    // fn getlk(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     lock_owner: u64,
    //     start: u64,
    //     end: u64,
    //     typ: i32,
    //     pid: u32,
    //     reply: fuser::ReplyLock,
    // ) {
    //     info!(
    //         "getlk(ino={}, fh={}, lock_owner={}, start={}, end={}, typ={}, pid={})",
    //         ino, fh, lock_owner, start, end, typ, pid
    //     );
    //     reply.error(ENOSYS);
    // }

    fn getxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        info!("getxattr(ino={}, name={:?}, size={})", ino, name, size);
        reply.error(EOPNOTSUPP);
    }

    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        info!("init()");
        Ok(())
    }

    // fn ioctl(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     flags: u32,
    //     cmd: u32,
    //     in_data: &[u8],
    //     out_size: u32,
    //     reply: fuser::ReplyIoctl,
    // ) {
    //     info!(
    //         "ioctl(ino={}, fh={}, flags={}, cmd={}, in_data={:?}, out_size={})",
    //         ino, fh, flags, cmd, in_data, out_size
    //     );
    //     reply.error(ENOSYS);
    // }

    // fn link(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     newparent: u64,
    //     newname: &OsStr,
    //     reply: fuser::ReplyEntry,
    // ) {
    //     info!(
    //         "link(ino={}, newparent={}, newname={:?})",
    //         ino, newparent, newname
    //     );
    //     reply.error(ENOSYS);
    // }

    fn listxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        info!("listxattr(ino={}, size={})", ino, size);
        reply.error(EOPNOTSUPP);
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        // std::thread::sleep(Duration::from_secs(1));
        info!("lookup(parent={}, name={:?})", parent, name);
        let parent_path = self.ino_to_path.get(&parent).unwrap().clone();
        let mut path = parent_path.clone();
        path.push(name.to_str().unwrap().to_string());

        let attr = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if let Some(attr) = attr {
            reply.entry(&Duration::from_secs(0), &attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    // fn lseek(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     offset: i64,
    //     whence: i32,
    //     reply: ReplyLseek,
    // ) {
    //     info!(
    //         "lseek(ino={}, fh={}, offset={}, whence={})",
    //         ino, fh, offset, whence
    //     );
    //     reply.error(ENOSYS);
    // }

    // fn mkdir(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     parent: u64,
    //     name: &OsStr,
    //     mode: u32,
    //     umask: u32,
    //     reply: fuser::ReplyEntry,
    // ) {
    //     info!(
    //         "mkdir(parent={}, name={:?}, mode={}, umask={})",
    //         parent, name, mode, umask
    //     );
    //     reply.error(ENOSYS);
    // }

    // fn mknod(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     parent: u64,
    //     name: &OsStr,
    //     mode: u32,
    //     umask: u32,
    //     rdev: u32,
    //     reply: fuser::ReplyEntry,
    // ) {
    //     info!(
    //         "mknod(parent={}, name={:?}, mode={}, umask={}, rdev={})",
    //         parent, name, mode, umask, rdev
    //     );
    //     reply.error(ENOSYS);
    // }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        self.debug_sleep();
        info!("open(ino={}, flags={})", ino, flags);

        reply.opened(self.next_fh, 0);
        self.next_fh += 1;
    }

    fn opendir(&mut self, _req: &fuser::Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
        info!("opendir(ino={}, flags={})", _ino, _flags);
        reply.opened(self.next_fh, 0);
        self.next_fh += 1;
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        info!("read(ino={}, offset={}, size={})", ino, offset, size);
        let path = self.ino_to_path.get(&ino).unwrap().clone();

        {
            let cow_file_exists = self
                .local_system_workspace_manager
                .check_cow_file(&path[0], &path[1..])
                .unwrap();
            if cow_file_exists {
                let data = self
                    .local_system_workspace_manager
                    .read_cow_file(&path[0], &path[1..], offset, size)
                    .unwrap();
                reply.data(&data);
                return;
            }
        }

        let Some(root_dir) = self.get_workspace_root(&path[0]) else {
            reply.error(ENOENT);
            return;
        };

        let tree = match self.follow_path(&path[1..], root_dir.clone()) {
            Some(tree) => tree,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let data = match tree.clone() {
            SagittaTreeObject::File(file) => {
                let blob = self.client.blob_read(&file.blob_id).unwrap();
                blob.blob
            }
            _ => panic!(),
        };
        let begin = offset as usize;
        let end = std::cmp::min(data.len(), (offset + size as i64) as usize);
        reply.data(&data[begin..end]);
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        info!("readdir(ino={}, offset={})", ino, offset);
        if ino == 1 {
            let mut entries = vec![];
            entries.push((ino, FileType::Directory, ".".to_string()));
            entries.push((ino, FileType::Directory, "..".to_string()));

            let trunk = self.record_ino(&vec!["trunk".to_string()]);
            entries.push((trunk, FileType::Directory, "trunk".to_string()));

            let mut entry_offset = 0;
            for entry in entries.into_iter() {
                if entry_offset >= offset
                    && reply.add(entry.0, entry_offset + 1, entry.1, entry.2.as_str())
                {
                    reply.ok();
                    return;
                }
                entry_offset += 1;
            }

            let workspaces = self.client.workspace_list().unwrap();
            for workspace in workspaces.workspaces.into_iter() {
                let ino = self.record_ino(&vec![workspace.clone()]);
                if entry_offset >= offset
                    && reply.add(
                        ino,
                        entry_offset + 1,
                        FileType::Directory,
                        workspace.as_str(),
                    )
                {
                    reply.ok();
                    return;
                }
                entry_offset += 1;
            }

            reply.ok();
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();
        assert!(!path.is_empty());

        let Some(root_dir) = self.get_workspace_root(&path[0]) else {
            info!("readdir: {:?}, root not found", path);
            reply.error(ENOENT);
            return;
        };

        let parent = if path.is_empty() {
            1
        } else {
            self.record_ino(&path[..path.len() - 1].to_vec())
        };

        let mut entries = vec![];
        entries.push((ino, FileType::Directory, ".".to_string()));
        entries.push((parent, FileType::Directory, "..".to_string()));

        if ino == 1 {
            let trunk = self.record_ino(&vec!["trunk".to_string()]);
            entries.push((trunk, FileType::Directory, "trunk".to_string()));
        } else if let Some(tree) = self.follow_path(&path[1..], root_dir.clone()) {
            let tree_as_dir: SagittaTreeObjectDir = match tree.clone() {
                SagittaTreeObject::Dir(dir) => dir,
                _ => panic!(),
            };
            for item in &tree_as_dir.items {
                let tree = self.client.blob_read_as_tree_object(&item.1).unwrap();
                match tree {
                    SagittaTreeObject::Dir(_dir) => {
                        let mut path = path.clone();
                        path.push(item.0.clone());
                        let ino_child = self.record_ino(&path);
                        entries.push((ino_child, FileType::Directory, item.0.clone()));
                    }
                    SagittaTreeObject::File(_file) => {
                        let mut path = path.clone();
                        path.push(item.0.clone());
                        let ino_child = self.record_ino(&path);
                        entries.push((ino_child, FileType::RegularFile, item.0.clone()));
                    }
                }
            }
        }

        let mut visited = HashSet::new();
        if path[0] != "trunk" {
            let local_entries = self
                .local_system_workspace_manager
                .read_cow_dir(&path[0], &path[1..]);
            if let Ok(local_entries) = local_entries {
                for entry in local_entries {
                    if visited.contains(&entry.name) {
                        continue;
                    }
                    visited.insert(entry.name.clone());

                    let mut path = path.clone();
                    path.push(entry.name.clone());
                    let ino_child = self.record_ino(&path);
                    entries.push((ino_child, FileType::RegularFile, entry.name.clone()));
                }
            }
        }

        for (i, entry) in entries.iter().enumerate().skip(offset as usize) {
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2.as_str()) {
                break;
            }
        }
        reply.ok();
    }

    // fn readdirplus(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     offset: i64,
    //     reply: fuser::ReplyDirectoryPlus,
    // ) {
    //     info!("readdirplus(ino={}, fh={}, offset={})", ino, fh, offset);
    //     reply.error(ENOSYS);
    // }

    // fn readlink(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
    //     info!("readlink(ino={})", ino);
    //     reply.error(ENOSYS);
    // }

    fn release(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        // std::thread::sleep(Duration::from_secs(1));
        info!(
            "release(ino={}, fh={}, flags={}, lock_owner={:?}, flush={})",
            ino, fh, flags, lock_owner, flush
        );
        reply.ok();
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        info!("releasedir(ino={}, fh={}, flags={})", _ino, _fh, _flags);
        reply.ok();
    }

    // fn removexattr(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     name: &OsStr,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!("removexattr(ino={}, name={:?})", ino, name);
    //     reply.error(ENOSYS);
    // }

    // fn rename(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     parent: u64,
    //     name: &OsStr,
    //     newparent: u64,
    //     newname: &OsStr,
    //     flags: u32,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!(
    //         "rename(parent={}, name={:?}, newparent={}, newname={:?}, flags={})",
    //         parent, name, newparent, newname, flags
    //     );
    //     reply.error(ENOSYS);
    // }

    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        info!("rmdir(parent={}, name={:?})", parent, name);
        reply.ok();
    }

    fn setattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        // std::thread::sleep(Duration::from_secs(1));
        info!("setattr(ino={}, mode={:?}, uid={:?}, gid={:?}, size={:?}, atime={:?}, mtime={:?}, ctime={:?}, fh={:?}, crtime={:?}, chgtime={:?}, bkuptime={:?}, flags={:?})", ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime, flags);

        if ino == 1 {
            let attr = self.get_root_file_attr();
            reply.attr(&Duration::from_secs(0), &attr);
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();

        // truncate
        if size == Some(0) {
            self.local_system_workspace_manager
                .create_cow_file(&path[0], &path[1..], &[])
                .unwrap();
        }

        let attr = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if let Some(attr) = attr {
            reply.attr(&Duration::from_secs(0), &attr);
        } else {
            reply.error(ENOENT);
        }
    }

    // fn setlk(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     fh: u64,
    //     lock_owner: u64,
    //     start: u64,
    //     end: u64,
    //     typ: i32,
    //     pid: u32,
    //     sleep: bool,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!(
    //         "setlk(ino={}, fh={}, lock_owner={}, start={}, end={}, typ={}, pid={}, sleep={})",
    //         ino, fh, lock_owner, start, end, typ, pid, sleep
    //     );
    //     reply.error(ENOSYS);
    // }

    // fn setxattr(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     ino: u64,
    //     name: &OsStr,
    //     _value: &[u8],
    //     flags: i32,
    //     position: u32,
    //     reply: fuser::ReplyEmpty,
    // ) {
    //     info!(
    //         "setxattr(ino={}, name={:?}, flags={}, position={})",
    //         ino, name, flags, position
    //     );
    //     reply.error(ENOSYS);
    // }

    // fn statfs(&mut self, _req: &fuser::Request<'_>, _ino: u64, reply: fuser::ReplyStatfs) {
    //     info!("statfs(ino={})", _ino);
    //     unimplemented!()
    // }

    // fn symlink(
    //     &mut self,
    //     _req: &fuser::Request<'_>,
    //     parent: u64,
    //     link_name: &OsStr,
    //     target: &std::path::Path,
    //     reply: fuser::ReplyEntry,
    // ) {
    //     info!(
    //         "symlink(parent={}, link_name={:?}, target={:?})",
    //         parent, link_name, target
    //     );
    //     reply.error(ENOSYS);
    // }

    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        info!("unlink(parent={}, name={:?})", parent, name);
        reply.ok();
    }

    fn write(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        info!(
            "write(ino={}, fh={}, offset={}, write_flags={}, flags={}, lock_owner={:?})",
            ino, fh, offset, write_flags, flags, lock_owner
        );
        info!("data: {:?}", data);

        let path = self.ino_to_path.get(&ino).unwrap().clone();
        self.local_system_workspace_manager
            .write_cow_file(&path[0], &path[1..], offset, data)
            .unwrap();

        reply.written(data.len() as u32);
    }
}

impl SagittaFS {
    pub fn new(config: SagittaConfig) -> Self {
        let mut ino_to_path = HashMap::new();

        // root
        ino_to_path.insert(1, vec![]);
        let mut path_to_ino = HashMap::new();
        path_to_ino.insert(vec![], 1);

        // trunk
        ino_to_path.insert(2, vec!["trunk".to_string()]);
        path_to_ino.insert(vec!["trunk".to_string()], 2);

        let base_url = config.base_url.clone();
        let clock = config.clock.clone();
        let local_system_workspace_base_path = config.local_system_workspace_base_path.clone();
        Self {
            config,
            next_inode: 3,
            ino_to_path,
            path_to_ino,
            client: SagittaApiClient::new(base_url),
            clock,
            local_system_workspace_manager: LocalSystemWorkspaceManager::new(
                local_system_workspace_base_path,
            ),
            next_fh: 1,
        }
    }

    pub fn record_ino(&mut self, path: &Vec<String>) -> u64 {
        if let Some(ino) = self.path_to_ino.get(path) {
            return *ino;
        }
        let ino = self.next_inode;
        self.next_inode += 1;
        self.ino_to_path.insert(ino, path.clone());
        self.path_to_ino.insert(path.clone(), ino);
        info!("record_ino: {} = {:?}", ino, path);
        ino
    }

    pub fn follow_path(
        &self,
        path: &[String],
        tree: SagittaTreeObject,
    ) -> Option<SagittaTreeObject> {
        let mut tree = tree;
        for part in path {
            let tree_as_dir: SagittaTreeObjectDir = match tree.clone() {
                SagittaTreeObject::Dir(dir) => dir,
                _ => panic!(),
            };
            let mut found = false;
            for item in &tree_as_dir.items {
                let child = self.client.blob_read_as_tree_object(&item.1).unwrap();
                if item.0 == *part {
                    tree = child;
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        }
        Some(tree)
    }

    pub fn get_workspace_root(&self, workspace: &str) -> Option<SagittaTreeObject> {
        if workspace == "trunk" {
            let root_commit_id = self.client.trunk_get_head().unwrap().id;
            let root_commit = self
                .client
                .blob_read_as_commit_object(&root_commit_id)
                .unwrap();
            let root_dir = self
                .client
                .blob_read_as_tree_object(&root_commit.tree_id)
                .unwrap();
            Some(root_dir)
        } else {
            let workspaces = self.client.workspace_list().unwrap();
            if !workspaces.workspaces.contains(&workspace.to_string()) {
                return None;
            }

            let root_commit_id = self.client.workspace_get_head(workspace).unwrap().id;
            let root_commit = self
                .client
                .blob_read_as_commit_object(&root_commit_id)
                .unwrap();
            let root_dir = self
                .client
                .blob_read_as_tree_object(&root_commit.tree_id)
                .unwrap();
            Some(root_dir)
        }
    }

    pub fn debug_sleep(&self) {
        if let Some(duration) = self.config.debug_sleep_duration {
            std::thread::sleep(duration);
        }
    }

    pub fn get_root_file_attr(&self) -> FileAttr {
        FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: self.clock.now(),
            mtime: self.clock.now(),
            ctime: self.clock.now(),
            crtime: self.clock.now(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: self.config.uid,
            gid: self.config.gid,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }

    pub fn get_file_attr(&mut self, parent: &[String], file_name: &str) -> Option<FileAttr> {
        if parent.is_empty() {
            let mut path = parent.to_vec();
            path.push(file_name.to_string());
            let ino = self.record_ino(&path);

            let workspaces = self.client.workspace_list().unwrap();
            let workspaces = workspaces.workspaces;
            if workspaces.contains(&path[0]) || path[0] == "trunk" {
                let perm = if file_name == "trunk" { 0o555 } else { 0o755 };
                let attr = FileAttr {
                    ino,
                    size: 0,
                    blocks: 0,
                    atime: self.clock.now(),
                    mtime: self.clock.now(),
                    ctime: self.clock.now(),
                    crtime: self.clock.now(),
                    kind: FileType::Directory,
                    perm,
                    nlink: 2,
                    uid: self.config.uid,
                    gid: self.config.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 512,
                };
                return Some(attr);
            } else {
                return None;
            }
        }
        assert!(!parent.is_empty());

        let mut path = parent.to_vec();
        path.push(file_name.to_string());

        let cow_file_exists = self
            .local_system_workspace_manager
            .check_cow_file(&path[0], &path[1..])
            .unwrap();
        if cow_file_exists {
            let ino = self.record_ino(&path);
            let (len, mut ctime, mut mtime) = self
                .local_system_workspace_manager
                .get_len_ctime_and_mtime_of_cow_file(&path[0], &path[1..])
                .unwrap();
            if self.clock.is_fixed() {
                ctime = self.clock.now();
                mtime = self.clock.now();
            }
            let attr = FileAttr {
                ino,
                size: len,
                blocks: (len + 511) / 512,
                atime: self.clock.now(),
                mtime,
                ctime,
                crtime: ctime,
                kind: FileType::RegularFile,
                perm: 0o644,
                nlink: 1,
                uid: self.config.uid,
                gid: self.config.gid,
                rdev: 0,
                flags: 0,
                blksize: 512,
            };
            return Some(attr);
        }

        let cow_dir_exists = self
            .local_system_workspace_manager
            .check_cow_dir(&path[0], &path[1..])
            .unwrap();
        if cow_dir_exists {
            let ino = self.record_ino(&path);
            let attr = FileAttr {
                ino,
                size: 0,
                blocks: 0,
                atime: self.clock.now(),
                mtime: self.clock.now(),
                ctime: self.clock.now(),
                crtime: self.clock.now(),
                kind: FileType::Directory,
                perm: 0o755,
                nlink: 2,
                uid: self.config.uid,
                gid: self.config.gid,
                rdev: 0,
                flags: 0,
                blksize: 512,
            };
            return Some(attr);
        }

        let root_dir = self.get_workspace_root(&path[0])?;

        let tree = match self.follow_path(&path[1..], root_dir.clone()) {
            Some(tree) => tree,
            None => {
                return None;
            }
        };

        let ino = self.record_ino(&path);
        match tree.clone() {
            SagittaTreeObject::Dir(dir) => {
                let perm_mask = if path[0] == "trunk" { 0o555 } else { 0o755 };
                let tree_as_dir: SagittaTreeObjectDir = dir;
                Some(FileAttr {
                    ino,
                    size: 0,
                    blocks: 0,
                    atime: tree_as_dir.ctime,
                    mtime: tree_as_dir.mtime,
                    ctime: tree_as_dir.ctime,
                    crtime: tree_as_dir.ctime,
                    kind: FileType::Directory,
                    perm: tree_as_dir.perm & perm_mask,
                    nlink: 2,
                    uid: self.config.uid,
                    gid: self.config.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 512,
                })
            }
            SagittaTreeObject::File(file) => {
                let perm_mask = if path[0] == "trunk" { 0o444 } else { 0o644 };
                Some(FileAttr {
                    ino,
                    size: file.size,
                    blocks: (file.size + 511) / 512,
                    atime: file.ctime,
                    mtime: file.mtime,
                    ctime: file.ctime,
                    crtime: file.ctime,
                    kind: FileType::RegularFile,
                    perm: file.perm & perm_mask,
                    nlink: 1,
                    uid: self.config.uid,
                    gid: self.config.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 512,
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct SagittaConfig {
    pub base_url: String,
    pub mountpoint: String,
    pub uid: u32,
    pub gid: u32,
    pub clock: Clock,
    pub local_system_workspace_base_path: PathBuf,
    pub debug_sleep_duration: Option<Duration>,
}

pub fn run_fs(config: SagittaConfig) {
    let mountpoint = std::path::Path::new(&config.mountpoint).to_path_buf();
    if !mountpoint.exists() {
        std::fs::create_dir_all(&mountpoint).unwrap();
    }

    let fs = SagittaFS::new(config);
    let options = vec![
        MountOption::RW,
        MountOption::FSName("sagitta".to_string()),
        MountOption::AutoUnmount,
    ];
    fuser::mount2(fs, mountpoint, &options).unwrap();
}
