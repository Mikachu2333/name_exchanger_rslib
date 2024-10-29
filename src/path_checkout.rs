use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct MetadataCollection {
    pub name: String,
    pub ext: String,
    pub parent_dir: PathBuf,
}
impl Default for MetadataCollection {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            ext: "".to_owned(),
            parent_dir: PathBuf::new(),
        }
    }
}

#[derive(Debug)]
pub struct GetPathInfo {
    pub path1: PathBuf,
    pub path2: PathBuf,
}
/// 所有路径相关的操作
impl GetPathInfo {
    /// 校验路径是否存在；如果是相对路径，尝试转化为绝对路径
    pub fn if_no_exist(&mut self, exe_path: &Path) -> (bool, bool) {
        if self.path1.exists() && self.path1.is_relative() {
            self.path1 = exe_path.join(self.path1.clone());
        }
        if self.path2.exists() && self.path2.is_relative() {
            self.path2 = exe_path.join(self.path2.clone());
        }
        (!&self.path1.exists(), !&self.path2.exists())
    }

    ///输入的文件类型是否为文件夹
    pub fn if_file(&self) -> (bool, bool) {
        (self.path1.is_file(), self.path2.is_file())
    }

    ///检测是否存在包含关系（父子目录问题）
    pub fn if_root(&self) -> u8 {
        //下面必须统一取小写或大写，因为rust的“contains()”大小写敏感
        let path1 = self.path1.to_string_lossy().to_ascii_lowercase();
        let path2 = self.path2.to_string_lossy().to_ascii_lowercase();

        if path1.contains(&path2) {
            //path1 should be renamed first
            1
        } else if path2.contains(&path1) {
            //path2 should be renamed first
            2
        } else {
            //no-influence
            0
        }
    }

    ///获取文件名称（无后缀）、后缀、所在文件夹（父文件夹）
    fn get_info(file_path: &Path) -> MetadataCollection {
        let get_string_closure = |x: &Option<&OsStr>, y: bool| {
            let mut tmp = String::from(".");
            match x {
                Some(i) => {
                    if y {
                        //是否在计算后缀，如果不是，去掉一开始的“.”
                        tmp.push_str(i.to_str().unwrap());
                        tmp
                    } else {
                        i.to_str().unwrap().to_string()
                    }
                }
                /*
                取不到就无视
                因前面已经核验完毕，所以此处如果出现Err则是特殊文件命名所致，不影响后面所有操作。
                e.g. "C:\\.cargo\\.config"，该文件取不到后缀，该文件夹也取不到后缀
                */
                None => String::new(),
            }
        };

        let name = get_string_closure(&file_path.file_stem(), false);
        let ext = get_string_closure(&file_path.extension(), true);
        let dir = match &file_path.parent() {
            Some(i) => i.to_path_buf(),
            None => PathBuf::new(),
        };

        MetadataCollection {
            name,
            ext,
            parent_dir: dir,
        }
    }

    pub fn metadata_collect(&self) -> (MetadataCollection, MetadataCollection) {
        let metadata1 = GetPathInfo::get_info(&self.path1);
        let metadata2 = GetPathInfo::get_info(&self.path2);
        (metadata1, metadata2)
    }
}
