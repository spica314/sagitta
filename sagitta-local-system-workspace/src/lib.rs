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
        std::fs::write(cow_path, data).map_err(Error::IOError)?;
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
        let entries = std::fs::read_dir(cow_path).map_err(Error::IOError)?;
        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(Error::IOError)?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap().to_string();
            let item = ReadCowDirItem { name: file_name };
            result.push(item);
        }
        Ok(result)
    }

    pub fn read_cow_file(
        &self,
        workspace_id: &str,
        path: &[String],
        offset: i64,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        let workspace_path = self.base_path.join(workspace_id);
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
            .open(cow_path)
            .map_err(Error::IOError)?;
        file.seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(Error::IOError)?;
        file.write_all(data).map_err(Error::IOError)?;
        Ok(())
    }

    pub fn delete_cow_file(&self, workspace_id: &str, path: &[String]) -> Result<(), Error> {
        let workspace_path = self.base_path.join(workspace_id);
        let mut cow_path = workspace_path.join("cow");
        for p in path {
            cow_path = cow_path.join(p);
        }
        if cow_path.exists() {
            std::fs::remove_file(cow_path).map_err(Error::IOError)?;
        }
        Ok(())
    }
}
