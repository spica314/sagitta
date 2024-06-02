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
use sagitta_remote_api_schema::v2::{
    get_attr::{V2GetAttrRequest, V2GetAttrResponse},
    get_file_blob_id::{V2GetFileBlobIdRequest, V2GetFileBlobIdResponse},
    get_workspace_id_from_name::{
        V2GetWorkspaceIdFromNameRequest, V2GetWorkspaceIdFromNameResponse,
    },
    get_workspaces::{V2GetWorkspacesRequest, V2GetWorkspacesResponse},
    read_blob::{V2ReadBlobRequest, V2ReadBlobResponse},
    read_dir::{V2ReadDirRequest, V2ReadDirResponse},
};
use std::time::Duration;

use sagitta_remote_api_client::SagittaApiClient;

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
    pub workspace_name_to_id: HashMap<String, String>,
    pub lookup_count: HashMap<u64, i64>,
}

impl Filesystem for SagittaFS {
    fn access(&mut self, _req: &fuser::Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        self.debug_sleep();
        info!("access(ino={}, mask={})", ino, mask);

        if ino == 1 {
            reply.ok();
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();
        let attr = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if attr.is_some() {
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
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
        self.debug_sleep();
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

        let workspace_id = self.get_workspace_id_from_name(&file_path[0]).unwrap();
        self.local_system_workspace_manager
            .create_cow_file(&workspace_id, &file_path[1..], &[])
            .unwrap();

        let attr = self.get_file_attr(
            &file_path[..file_path.len() - 1],
            &file_path[file_path.len() - 1],
        );
        let attr = attr.unwrap();
        *self.lookup_count.entry(attr.ino).or_insert(0) += 1;
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
        self.debug_sleep();
        info!("flush(ino={}, fh={}, lock_owner={})", ino, fh, lock_owner);
        reply.ok();
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        self.debug_sleep();
        info!("forget(ino={}, nlookup={})", ino, nlookup);
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
        self.debug_sleep();
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
        self.debug_sleep();
        info!("getxattr(ino={}, name={:?}, size={})", ino, name, size);
        reply.error(EOPNOTSUPP);
    }

    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        self.debug_sleep();
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
        self.debug_sleep();
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
        self.debug_sleep();
        info!("lookup(parent={}, name={:?})", parent, name);
        let parent_path = self.ino_to_path.get(&parent).unwrap().clone();
        let mut path = parent_path.clone();
        path.push(name.to_str().unwrap().to_string());

        let attr = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if let Some(attr) = attr {
            *self.lookup_count.entry(attr.ino).or_insert(0) += 1;
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

    fn mkdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        self.debug_sleep();
        info!(
            "mkdir(parent={}, name={:?}, mode={}, umask={})",
            parent, name, mode, umask
        );

        let parent_path = self.ino_to_path.get(&parent).unwrap().clone();
        let mut file_path = parent_path.clone();
        file_path.push(name.to_str().unwrap().to_string());

        if file_path[0] == "trunk" {
            reply.error(EPERM);
            return;
        }

        let workspace_id = self.get_workspace_id_from_name(&file_path[0]).unwrap();

        self.local_system_workspace_manager
            .create_cow_dir(&workspace_id, &file_path[1..])
            .unwrap();

        let attr = self.get_file_attr(
            &file_path[..file_path.len() - 1],
            &file_path[file_path.len() - 1],
        );
        let attr = attr.unwrap();
        *self.lookup_count.entry(attr.ino).or_insert(0) += 1;
        reply.entry(&Duration::from_secs(0), &attr, 0);
    }

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
        self.debug_sleep();
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
        self.debug_sleep();
        info!("read(ino={}, offset={}, size={})", ino, offset, size);
        let path = self.ino_to_path.get(&ino).unwrap().clone();

        if path[0] != "trunk" {
            let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
            let cow_file_exists = self
                .local_system_workspace_manager
                .check_cow_file(&workspace_id, &path[1..])
                .unwrap();
            if cow_file_exists {
                let data = self
                    .local_system_workspace_manager
                    .read_cow_file(&workspace_id, &path[1..], offset, size)
                    .unwrap();
                reply.data(&data);
                return;
            }
        }

        let v2_get_file_blob_id_request = V2GetFileBlobIdRequest {
            workspace_id: if path[0] == "trunk" {
                None
            } else {
                Some(path[0].clone())
            },
            path: path[1..].to_vec(),
        };
        let v2_get_file_blob_id_response = self
            .client
            .v2_get_file_blob_id(v2_get_file_blob_id_request)
            .unwrap();

        match v2_get_file_blob_id_response {
            V2GetFileBlobIdResponse::Found { blob_id } => {
                let data = self
                    .client
                    .v2_read_blob_request(V2ReadBlobRequest {
                        blob_id: blob_id.clone(),
                    })
                    .unwrap();
                match data {
                    V2ReadBlobResponse::Direct { blob } => {
                        let begin = offset as usize;
                        let end = std::cmp::min(blob.len(), (offset + size as i64) as usize);
                        reply.data(&blob[begin..end]);
                    }
                    V2ReadBlobResponse::NotFound => {
                        reply.error(ENOENT);
                    }
                }
            }
            V2GetFileBlobIdResponse::NotFound => {
                reply.error(ENOENT);
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        self.debug_sleep();
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

            let workspaces = self
                .client
                .v2_get_workspaces(V2GetWorkspacesRequest {})
                .unwrap();
            match workspaces {
                V2GetWorkspacesResponse::Ok { items } => {
                    for workspace in items {
                        let ino = self.record_ino(&vec![workspace.name.clone()]);
                        if entry_offset >= offset
                            && reply.add(
                                ino,
                                entry_offset + 1,
                                FileType::Directory,
                                workspace.name.as_str(),
                            )
                        {
                            reply.ok();
                            return;
                        }
                        entry_offset += 1;
                    }
                }
                V2GetWorkspacesResponse::Err => {}
            }

            reply.ok();
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();
        assert!(!path.is_empty());

        let v2_read_dir_request = V2ReadDirRequest {
            workspace_id: if path[0] == "trunk" {
                None
            } else {
                let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
                Some(workspace_id)
            },
            path: path[1..].to_vec(),
            include_deleted: false,
        };
        let a = self.client.v2_read_dir(v2_read_dir_request).unwrap();

        let parent = if path.is_empty() {
            1
        } else {
            self.record_ino(&path[..path.len() - 1].to_vec())
        };

        let mut entries = vec![];
        entries.push((ino, FileType::Directory, ".".to_string()));
        entries.push((parent, FileType::Directory, "..".to_string()));

        let mut not_found_flag = false;

        match a {
            V2ReadDirResponse::Found { items } => {
                for item in items {
                    let mut path = path.clone();
                    path.push(item.name.clone());
                    let ino_child = self.record_ino(&path);
                    if item.is_dir {
                        entries.push((ino_child, FileType::Directory, item.name.clone()));
                    } else {
                        entries.push((ino_child, FileType::RegularFile, item.name.clone()));
                    }
                }
            }
            V2ReadDirResponse::NotFound => {
                not_found_flag = true;
            }
        }

        let mut visited = HashSet::new();
        if path[0] != "trunk" {
            let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
            let local_entries = self
                .local_system_workspace_manager
                .read_cow_dir(&workspace_id, &path[1..]);
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
                    not_found_flag = false;
                }
            }
        }

        if not_found_flag {
            reply.error(ENOENT);
            return;
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
        self.debug_sleep();
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
        self.debug_sleep();
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

    fn rename(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        self.debug_sleep();
        info!(
            "rename(parent={}, name={:?}, newparent={}, newname={:?}, flags={})",
            parent, name, newparent, newname, flags
        );

        let mut old_path = self.ino_to_path.get(&parent).unwrap().clone();
        old_path.push(name.to_str().unwrap().to_string());
        let old_path = old_path;

        let mut new_path = self.ino_to_path.get(&newparent).unwrap().clone();
        new_path.push(newname.to_str().unwrap().to_string());
        let new_path = new_path;

        if old_path[0] == "trunk" || new_path[0] == "trunk" {
            reply.error(EPERM);
            return;
        }

        let old_workspace_id = self.get_workspace_id_from_name(&old_path[0]).unwrap();
        let new_workspace_id = self.get_workspace_id_from_name(&new_path[0]).unwrap();

        let res = self.local_system_workspace_manager.rename_cow_file(
            &old_workspace_id,
            &old_path[1..],
            &new_workspace_id,
            &new_path[1..],
        );

        self.ino_change_path(&old_path, &new_path);

        if res.is_ok() {
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }

    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        self.debug_sleep();
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
        self.debug_sleep();
        info!("setattr(ino={}, mode={:?}, uid={:?}, gid={:?}, size={:?}, atime={:?}, mtime={:?}, ctime={:?}, fh={:?}, crtime={:?}, chgtime={:?}, bkuptime={:?}, flags={:?})", ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime, flags);

        if ino == 1 {
            let attr = self.get_root_file_attr();
            reply.attr(&Duration::from_secs(0), &attr);
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();

        // truncate
        if size == Some(0) {
            let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
            self.local_system_workspace_manager
                .create_cow_file(&workspace_id, &path[1..], &[])
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
        self.debug_sleep();
        info!("unlink(parent={}, name={:?})", parent, name);
        let mut path = self.ino_to_path.get(&parent).unwrap().clone();
        path.push(name.to_str().unwrap().to_string());

        let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();

        let a = self.get_file_attr(&path[..path.len() - 1], &path[path.len() - 1]);
        if a.is_none() {
            reply.error(ENOENT);
            return;
        }
        let a = a.unwrap();
        if a.kind == FileType::Directory {
            self.local_system_workspace_manager
                .delete_cow_dir(&workspace_id, &path[1..])
                .unwrap();
            return;
        } else if a.kind == FileType::RegularFile {
            self.local_system_workspace_manager
                .delete_cow_file(&workspace_id, &path[1..])
                .unwrap();
        }

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
        self.debug_sleep();
        info!(
            "write(ino={}, fh={}, offset={}, write_flags={}, flags={}, lock_owner={:?})",
            ino, fh, offset, write_flags, flags, lock_owner
        );
        // info!("data: {:?}", data);

        let path = self.ino_to_path.get(&ino).unwrap().clone();
        let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
        self.local_system_workspace_manager
            .write_cow_file(&workspace_id, &path[1..], offset, data)
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
            workspace_name_to_id: HashMap::new(),
            lookup_count: HashMap::new(),
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

    pub fn ino_change_path(&mut self, old_path: &Vec<String>, new_path: &[String]) {
        let ino = self.path_to_ino.remove(old_path).unwrap();
        self.ino_to_path.insert(ino, new_path.to_owned());
        self.path_to_ino.insert(new_path.to_owned(), ino);
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

    pub fn get_workspace_id_from_name(&mut self, workspace_name: &str) -> Option<String> {
        if let Some(workspace_id) = self.workspace_name_to_id.get(workspace_name) {
            return Some(workspace_id.clone());
        }

        let workspace_id_res = self
            .client
            .v2_get_workspace_id_from_name(V2GetWorkspaceIdFromNameRequest {
                workspace_name: workspace_name.to_string(),
            })
            .unwrap();
        match workspace_id_res {
            V2GetWorkspaceIdFromNameResponse::Found { workspace_id } => {
                self.workspace_name_to_id
                    .insert(workspace_name.to_string(), workspace_id.clone());
                Some(workspace_id)
            }
            V2GetWorkspaceIdFromNameResponse::NotFound => None,
        }
    }

    pub fn get_file_attr(&mut self, parent: &[String], file_name: &str) -> Option<FileAttr> {
        if parent.is_empty() {
            let mut path = parent.to_vec();
            path.push(file_name.to_string());
            let ino = self.record_ino(&path);

            let workspaces = self
                .client
                .v2_get_workspaces(V2GetWorkspacesRequest {})
                .unwrap();
            match workspaces {
                V2GetWorkspacesResponse::Ok { items } => {
                    // record name to id map
                    for item in &items {
                        self.workspace_name_to_id
                            .insert(item.name.clone(), item.id.clone());
                    }

                    // return attr
                    if items.iter().any(|item| item.name == path[0]) || &path[0] == "trunk" {
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
                V2GetWorkspacesResponse::Err => {
                    return None;
                }
            }
        }
        assert!(!parent.is_empty());

        let mut path = parent.to_vec();
        path.push(file_name.to_string());

        if path[0] != "trunk" {
            let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
            let cow_file_exists = self
                .local_system_workspace_manager
                .check_cow_file(&workspace_id, &path[1..])
                .unwrap();
            if cow_file_exists {
                let ino = self.record_ino(&path);
                let (len, mut ctime, mut mtime) = self
                    .local_system_workspace_manager
                    .get_len_ctime_and_mtime_of_cow_file(&workspace_id, &path[1..])
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
                .check_cow_dir(&workspace_id, &path[1..])
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
        }

        let workspace_id = if path[0] == "trunk" {
            None
        } else {
            let workspace_id = self.get_workspace_id_from_name(&path[0]).unwrap();
            Some(workspace_id)
        };
        let attr = self
            .client
            .v2_get_attr(V2GetAttrRequest {
                workspace_id,
                path: path[1..].to_vec(),
            })
            .unwrap();

        let ino = self.record_ino(&path);
        let attr = match attr {
            V2GetAttrResponse::Found {
                is_dir,
                size,
                modified_at,
            } => {
                let perm = if path[0] == "trunk" && is_dir {
                    0o555
                } else if path[0] == "trunk" && !is_dir {
                    0o444
                } else if path[0] != "trunk" && is_dir {
                    0o755
                } else {
                    0o644
                };
                let nlink = if is_dir { 2 } else { 1 };
                let kind = if is_dir {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                };
                FileAttr {
                    ino,
                    size,
                    blocks: (size + 511) / 512,
                    atime: modified_at,
                    mtime: modified_at,
                    ctime: modified_at,
                    crtime: modified_at,
                    kind,
                    perm,
                    nlink,
                    uid: self.config.uid,
                    gid: self.config.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 512,
                }
            }
            V2GetAttrResponse::NotFound => {
                return None;
            }
        };
        Some(attr)
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
