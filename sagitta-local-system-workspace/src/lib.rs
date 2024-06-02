use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
    time::SystemTime,
};

// file hierarchy
// root
// - trunk
//   - head
//   - objects
//     - 01
//       - 23
//         - 012345...
//     - 03
// - workspace1
//   - head
//   - objects
//     - 01
//       - 23
//         - 012345...
//     - 03
//   - cow
//     - dir1
//       - file1
//     - dir2
//       - file2
// - workspace2

#[derive(Debug, Clone)]
pub struct LocalSystemWorkspaceManager {
    base_path: PathBuf,
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
}

#[derive(Debug, Clone)]
pub struct ReadCowDirItem {
    pub name: String,
}

impl LocalSystemWorkspaceManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path: base_path.clone(),
        }
    }

    pub fn create_cow_file(
        &self,
        workspace_id: &str,
        path: &[String],
        data: &[u8],
    ) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        std::fs::create_dir_all(cow_path.parent().unwrap()).map_err(Error::IOError)?;
        std::fs::write(&cow_path, data).map_err(Error::IOError)?;
        {
            let mut cow_path = cow_path.clone();
            cow_path.pop();
            cow_path.push(format!(".sagitta.delete.{}", path.last().unwrap()));
            if cow_path.exists() {
                std::fs::remove_file(&cow_path).map_err(Error::IOError)?;
            }
        }
        Ok(())
    }

    pub fn create_cow_dir(&self, workspace_id: &str, path: &[String]) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        std::fs::create_dir_all(&cow_path).map_err(Error::IOError)?;
        {
            let mut cow_path = cow_path.clone();
            cow_path.pop();
            cow_path.push(format!(".sagitta.delete.{}", path.last().unwrap()));
            if cow_path.exists() {
                std::fs::remove_file(&cow_path).map_err(Error::IOError)?;
            }
        }
        Ok(())
    }

    pub fn check_cow_file(&self, workspace_id: &str, path: &[String]) -> Result<bool, Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        Ok(cow_path.exists() && cow_path.is_file())
    }

    pub fn get_len_ctime_and_mtime_of_cow_file(
        &self,
        workspace_id: &str,
        path: &[String],
    ) -> Result<(u64, SystemTime, SystemTime), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        let metadata = std::fs::metadata(cow_path).map_err(Error::IOError)?;
        let len = metadata.len();
        let ctime = metadata.created().unwrap();
        let mtime = metadata.modified().unwrap();
        Ok((len, ctime, mtime))
    }

    pub fn check_cow_dir(&self, workspace_id: &str, path: &[String]) -> Result<bool, Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        Ok(cow_path.exists() && cow_path.is_dir())
    }

    pub fn read_cow_dir(
        &self,
        workspace_id: &str,
        path: &[String],
    ) -> Result<Vec<ReadCowDirItem>, Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        let entries = std::fs::read_dir(&cow_path).map_err(Error::IOError)?;
        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(Error::IOError)?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap().to_string();
            if file_name.starts_with(".sagitta.delete.") {
                continue;
            } else {
                let delete_path = cow_path.join(format!(".sagitta.delete.{}", file_name));
                if delete_path.exists() {
                    continue;
                }
            }
            let item = ReadCowDirItem { name: file_name };
            result.push(item);
        }
        Ok(result)
    }

    pub fn read_cow_file(
        &self,
        workspace_name: &str,
        path: &[String],
        offset: i64,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        let workspace_path = self.base_path.join(workspace_name);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(cow_path)
            .map_err(Error::IOError)?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(Error::IOError)?;
        let mut data = vec![0; size as usize];
        let a = file.read(&mut data).map_err(Error::IOError)?;
        data.truncate(a);
        Ok(data)
    }

    pub fn write_cow_file(
        &self,
        workspace_id: &str,
        path: &[String],
        offset: i64,
        data: &[u8],
    ) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(&cow_path)
            .map_err(Error::IOError)?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(Error::IOError)?;
        file.write_all(data).map_err(Error::IOError)?;
        {
            let mut cow_path = cow_path.clone();
            cow_path.pop();
            cow_path.push(format!(".sagitta.delete.{}", path.last().unwrap()));
            if cow_path.exists() {
                std::fs::remove_file(&cow_path).map_err(Error::IOError)?;
            }
        }
        Ok(())
    }

    pub fn delete_cow_file(&self, workspace_id: &str, path: &[String]) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        if cow_path.exists() {
            std::fs::remove_file(&cow_path).map_err(Error::IOError)?;
        }
        {
            let mut cow_path = cow_path.clone();
            cow_path.pop();
            cow_path.push(format!(".sagitta.delete.{}", path.last().unwrap()));
            std::fs::write(&cow_path, "").map_err(Error::IOError)?;
        }
        Ok(())
    }

    pub fn delete_cow_dir(&self, workspace_id: &str, path: &[String]) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        {
            let mut cow_path = cow_path.clone();
            cow_path.pop();
            cow_path.push(format!(".sagitta.delete.{}", path.last().unwrap()));
            std::fs::write(&cow_path, "").map_err(Error::IOError)?;
        }
        Ok(())
    }

    pub fn list_cow_files(&self, workspace_id: &str) -> Result<Vec<Vec<String>>, Error> {
        let mut res = vec![];
        let workspace_path = self.base_path.join(workspace_id);
        let cow_path = workspace_path.join("cow");
        Self::list_cow_files_sub(cow_path, &mut [], &mut res).unwrap();
        Ok(res)
    }

    fn list_cow_files_sub(
        file_path: PathBuf,
        base_path: &mut [String],
        res: &mut Vec<Vec<String>>,
    ) -> Result<(), Error> {
        let entries = std::fs::read_dir(file_path).map_err(Error::IOError)?;
        for entry in entries {
            let entry = entry.map_err(Error::IOError)?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap().to_string();
            let mut base_path = base_path.to_vec();
            base_path.push(file_name);
            if entry.path().is_dir() {
                Self::list_cow_files_sub(entry.path(), &mut base_path, res)?;
            } else {
                res.push(base_path.clone());
            }
        }
        Ok(())
    }

    pub fn archive_cow_dir(
        &self,
        workspace_id: &str,
        paths: &Vec<Vec<String>>,
    ) -> Result<(), Error> {
        let now = SystemTime::now();
        let workspace_path = self.base_path.join(workspace_id);
        let cow_path = workspace_path.join("cow");
        let archive_path = workspace_path.join(format!(
            "cow-{}",
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        for path in paths {
            let mut cow_path = cow_path.clone();
            for p in path {
                cow_path = cow_path.join(p);
            }
            let mut archive_path = archive_path.clone();
            for p in path {
                archive_path = archive_path.join(p);
            }
            std::fs::create_dir_all(archive_path.parent().unwrap()).map_err(Error::IOError)?;
            std::fs::rename(cow_path, archive_path).map_err(Error::IOError)?;
        }

        Ok(())
    }

    pub fn rename_cow_file(
        &self,
        old_workspace_id: &str,
        old_path: &[String],
        new_workspace_id: &str,
        new_path: &[String],
    ) -> Result<(), Error> {
        let old_workspace_path = self.base_path.join(old_workspace_id);
        let mut old_cow_path = old_workspace_path.join("cow");
        for p in old_path {
            old_cow_path = old_cow_path.join(p);
        }
        let new_workspace_path = self.base_path.join(new_workspace_id);
        let mut new_cow_path = new_workspace_path.join("cow");
        for p in new_path {
            new_cow_path = new_cow_path.join(p);
        }
        std::fs::create_dir_all(new_cow_path.parent().unwrap()).map_err(Error::IOError)?;
        std::fs::rename(&old_cow_path, &new_cow_path).map_err(Error::IOError)?;
        Ok(())
    }
}
