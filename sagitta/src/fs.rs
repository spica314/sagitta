use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use fuser::{FileAttr, FileType, Filesystem, MountOption, ReplyDirectory};
use libc::ENOENT;
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
}

impl Filesystem for SagittaFS {
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

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        info!("lookup(parent={}, name={:?})", parent, name);
        let parent_path = self.ino_to_path.get(&parent).unwrap().clone();
        let mut path = parent_path.clone();
        path.push(name.to_str().unwrap().to_string());

        if parent == 1 {
            let workspaces = self.client.workspace_list().unwrap();
            let workspaces = workspaces.workspaces;
            if workspaces.contains(&path[0]) || path[0] == "trunk" {
                let ino = self.record_ino(&path);
                let perm = if path[0] == "trunk" { 0o555 } else { 0o755 };
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
                reply.entry(&Duration::from_secs(1), &attr, 0);
                return;
            }
        }

        {
            let cow_file_exists = self
                .local_system_workspace_manager
                .check_cow_file(&path[0], &path[1..])
                .unwrap();
            if cow_file_exists {
                let ino = self.record_ino(&path);
                let len = self
                    .local_system_workspace_manager
                    .get_len_of_cow_file(&path[0], &path[1..])
                    .unwrap();
                let attr = FileAttr {
                    ino,
                    size: len,
                    blocks: 0,
                    atime: self.clock.now(),
                    mtime: self.clock.now(),
                    ctime: self.clock.now(),
                    crtime: self.clock.now(),
                    kind: FileType::RegularFile,
                    perm: 0o644,
                    nlink: 1,
                    uid: self.config.uid,
                    gid: self.config.gid,
                    rdev: 0,
                    flags: 0,
                    blksize: 512,
                };
                reply.entry(&Duration::from_secs(1), &attr, 0);
                return;
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
                reply.entry(&Duration::from_secs(1), &attr, 0);
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
        let ino = self.record_ino(&path);

        let attr = match tree.clone() {
            SagittaTreeObject::Dir(dir) => {
                let perm_mask = if path[0] == "trunk" { 0o555 } else { 0o755 };
                let tree_as_dir: SagittaTreeObjectDir = dir;
                FileAttr {
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
                }
            }
            SagittaTreeObject::File(file) => {
                let perm_mask = if path[0] == "trunk" { 0o444 } else { 0o644 };
                FileAttr {
                    ino,
                    size: file.size,
                    blocks: 0,
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
                }
            }
        };
        reply.entry(&Duration::from_secs(1), &attr, 0);
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        info!("getattr(ino={})", ino);

        if ino == 1 {
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
            reply.attr(&Duration::from_secs(1), &attr);
            return;
        }

        let path = self.ino_to_path.get(&ino).unwrap().clone();

        if path.len() == 1 {
            let perm = if path[0] == "trunk" { 0o555 } else { 0o755 };
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
            reply.attr(&Duration::from_secs(1), &attr);
            return;
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

        let attr = match tree.clone() {
            SagittaTreeObject::Dir(dir) => {
                let perm_mask = if path[0] == "trunk" { 0o555 } else { 0o755 };
                let tree_as_dir: SagittaTreeObjectDir = dir;
                FileAttr {
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
                }
            }
            SagittaTreeObject::File(file) => {
                let perm_mask = if path[0] == "trunk" { 0o444 } else { 0o644 };
                FileAttr {
                    ino,
                    size: file.size,
                    blocks: 0,
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
                }
            }
        };
        reply.attr(&Duration::from_secs(1), &attr);
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
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
}

#[derive(Debug)]
pub struct SagittaConfig {
    pub base_url: String,
    pub mountpoint: String,
    pub uid: u32,
    pub gid: u32,
    pub clock: Clock,
    pub local_system_workspace_base_path: PathBuf,
}

pub fn run_fs(config: SagittaConfig) {
    let mountpoint = std::path::Path::new(&config.mountpoint).to_path_buf();
    if !mountpoint.exists() {
        std::fs::create_dir_all(&mountpoint).unwrap();
    }

    let fs = SagittaFS::new(config);
    let options = vec![
        MountOption::RO,
        MountOption::FSName("sagitta".to_string()),
        MountOption::AutoUnmount,
    ];
    fuser::mount2(fs, mountpoint, &options).unwrap();
}
