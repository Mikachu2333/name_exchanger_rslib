pub mod process_input {
    pub mod metadata_get {
        use std::{ffi::OsStr, path::PathBuf};

        pub fn if_exist(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            (!path1.exists(), !path2.exists())
        }

        pub fn if_relative(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            (path1.is_relative(), path2.is_relative())
        }

        pub fn if_file(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            (path1.is_file(), path2.is_file())
        }

        pub fn get_info(file_path: &PathBuf) -> (String, String, PathBuf) {
            let get_string_closure = |x: &Option<&OsStr>, y: bool| {
                let mut tmp = String::from(".");
                match x {
                    Some(i) => {
                        if y {
                            tmp.push_str(i.to_str().unwrap());
                            tmp
                        } else {
                            i.to_str().unwrap().to_string()
                        }
                    }
                    None => String::new(),
                }
            };
            let name = get_string_closure(&file_path.file_stem(), false);

            let ext = get_string_closure(&file_path.extension(), true);

            let dir = match &file_path.parent() {
                Some(i) => i.to_path_buf(),
                None => PathBuf::new(),
            };
            (name, ext, dir)
        }

        pub fn if_root(path1: &PathBuf, path2: &PathBuf) -> u8 {
            let path1 = path1.to_string_lossy().to_ascii_lowercase();
            let path2 = path2.to_string_lossy().to_ascii_lowercase();

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
    }

    pub mod change_name {
        use std::fs;
        use std::{path::PathBuf, process::exit};

        use super::metadata_get::{self, if_root};

        fn make_name(dir: PathBuf, mut other_name: String, ext: String) -> (PathBuf, PathBuf) {
            let mut new_name = dir.clone();
            let mut new_pre_name = dir;

            let mut temp_additional_name = String::from("E9EAE3BB7E262210FF2B");
            temp_additional_name.push_str(&ext);
            new_pre_name.push(&temp_additional_name);

            other_name.push_str(&ext);
            new_name.push(&other_name);

            (new_pre_name, new_name)
        }

        fn rename_each(
            path1: PathBuf,
            final_name1: PathBuf,
            path2: PathBuf,
            final_name2: PathBuf,
            tmp_name2: PathBuf,
            relevant: bool,
        ) {
            //1 first
            println!("4\n\nPath1: {:?}\nFinal1: {:?}", path1, final_name1);
            println!(
                "Path2: {:?}\nFinal2: {:?}\nTmp2: {:?}",
                path2, final_name2, tmp_name2
            );
            if relevant {
                let _ = fs::rename(&path1, &final_name1).unwrap_or_else(|_err| {
                    exit(-3);
                });
                let _ = fs::rename(&path2, &final_name2).unwrap_or_else(|_err| {
                    exit(-3);
                });
            } else {
                let _ = fs::rename(&path2, &tmp_name2).unwrap_or_else(|_err| {
                    exit(-3);
                });
                let _ = fs::rename(&path1, &final_name1).unwrap_or_else(|_err| {
                    exit(-3);
                });
                let _ = fs::rename(&tmp_name2, &final_name2);
            }
        }

        #[no_mangle]
        pub extern "C" fn exchange(path1: String, path2: String) {
            let dir_check = |s: String| {
                let s = PathBuf::from(s);
                if s.ends_with("\"") {
                    let s = s.to_str().unwrap().strip_suffix("\"").unwrap();
                    let s = if s.ends_with("\\") {
                        s.strip_suffix("\\").unwrap()
                    } else {
                        s
                    };
                    PathBuf::from(s)
                } else {
                    PathBuf::from(s)
                }
            };
            let path1 = dir_check(path1);
            let path2 = dir_check(path2);
            println!("1{:?}, {:?}\n", path1, path2);
            let (no_exist1, no_exist2) = metadata_get::if_exist(&path1, &path2);
            if no_exist1 || no_exist2 {
                exit(-1);
            }
            let (re_1, re_2) = metadata_get::if_relative(&path1, &path2);
            if re_1 || re_2 {
                exit(-2);
            }

            let (is_file1, is_file2) = metadata_get::if_file(&path1, &path2);
            let (name_1, ext_1, dir_1) = metadata_get::get_info(&path1);
            let (name_2, ext_2, dir_2) = metadata_get::get_info(&path2);

            let (pre_name1, new_name1) = make_name(dir_1, name_2, ext_1);
            let (pre_name2, new_name2) = make_name(dir_2, name_1, ext_2);

            let mode = if_root(&path1, &path2);
            println!("rel mode: {}",&mode);

            if is_file1 && is_file2 {
                rename_each(path1, new_name1, path2, new_name2, pre_name2, false);
            } else if (!is_file1) && (!is_file2) {
                //all dir
                if mode == 1 {
                    //if file1 contains file2
                    println!("mode1");
                    rename_each(path1, new_name1, path2, new_name2, pre_name2, true);
                } else if mode == 2 {
                    //if file1 contains file2
                    println!("mode2");
                    rename_each(path2, new_name2, path1, new_name1, pre_name1, true);
                } else {
                    println!("mode0");
                    //if file2 contains file1 or no contains
                    rename_each(path2, new_name2, path1, new_name1, pre_name1, false);
                }
            } else {
                if is_file1 {
                    if mode == 1 {
                        //file1 rename first
                        println!("mode1");
                        rename_each(path1, new_name1, path2, new_name2, pre_name2, true);
                    } else {
                        println!("mode0/2");
                        rename_each(path1, new_name1, path2, new_name2, pre_name2, false);
                    }
                } else {
                    if mode == 2 {
                        println!("mode2");
                        //file2 rename first
                        rename_each(path2, new_name2, path1, new_name1, pre_name1, true);
                    } else {
                        println!("mode0/1");
                        //file2 rename first
                        rename_each(path2, new_name2, path1, new_name1, pre_name1, false);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //-1 no exist
        //-2 not absolte
        //-3 no permission
        process_input::change_name::exchange(
            String::from(r"D:\aardio\新建 DOCX 文档.dll"),
            String::from(r"d:\aardio\新建有3 文件夹\塞.docx"),
        );
    }
}
