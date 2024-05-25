use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn exchange_inputs(input1: *const c_char, input2: *const c_char) -> *mut c_char {
    let input1 = unsafe { CStr::from_ptr(input1) }
        .to_string_lossy()
        .to_string();
    let input2 = unsafe { CStr::from_ptr(input2) }
        .to_string_lossy()
        .to_string();
    //println!("{} {}", &input1, &input2);
    let result_num = process_input::change_name::exchange(input1, input2);
    CString::new(process_input::show_error::switch_error_msg(result_num))
        .unwrap()
        .into_raw()
}

pub(crate) mod process_input {
    pub(crate) mod metadata_get {
        use std::{ffi::OsStr, path::PathBuf};

        pub fn if_no_exist(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            // 核验用户输入是否存在
            (!path1.exists(), !path2.exists())
        }

        pub fn if_relative(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            //核验用户输入是否为绝对路径
            (path1.is_relative(), path2.is_relative())
        }

        pub fn if_file(path1: &PathBuf, path2: &PathBuf) -> (bool, bool) {
            //输入的文件类型是否为文件夹
            (path1.is_file(), path2.is_file())
        }

        pub fn get_info(file_path: &PathBuf) -> (String, String, PathBuf) {
            //获取文件名称（无后缀）、后缀、所在文件夹（父文件夹）
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
            (name, ext, dir)
        }

        pub fn if_root(path1: &PathBuf, path2: &PathBuf) -> u8 {
            //检测是否存在包含关系（父子目录问题）
            //下面必须统一取小写或大写，因为rust的“contains()”大小写敏感
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

    pub(crate) mod change_name {
        //改名的主体
        use std::path::PathBuf;
        use std::{fs, io};

        use super::metadata_get::{self, if_root};

        fn make_name(dir: PathBuf, mut other_name: String, ext: String) -> (PathBuf, PathBuf) {
            //获取临时文件名与改后文件名
            let mut new_name = dir.clone();
            let mut new_pre_name = dir;

            //任意长字符串用作区分
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
        ) -> u8 {
            //改名具体执行部分
            let get_err_or_ok = |x: io::Result<()>| -> u8 {
                match x {
                    Ok(_) => 0,
                    Err(x) => match x.kind() {
                        io::ErrorKind::PermissionDenied => 3,
                        io::ErrorKind::AlreadyExists => 4,
                        _ => 255,
                    },
                }
            };
            //1 first
            if relevant {
                //如果存在相关性（父子目录或文件），后面的exit(3)是为了核验是否有权限改名
                let rename_1_result = get_err_or_ok(fs::rename(&path1, &final_name1));
                let rename_2_result = get_err_or_ok(fs::rename(&path2, &final_name2));
                if rename_1_result != 0 {
                    println!("FAILED: \n{:?} => {:?}", &path1, &final_name1);
                    rename_1_result
                } else if rename_2_result != 0 {
                    println!("FAILED: \n{:?} => {:?}", &path2, &final_name2);
                    rename_2_result
                } else {
                    println!("SUCCESS: \n{:?} => {:?}", &path1, &final_name1);
                    println!("SUCCESS: \n{:?} => {:?}", &path2, &final_name2);
                    0
                }
            } else {
                //不存在相关性：正常操作
                let rename_1_result = get_err_or_ok(fs::rename(&path2, &tmp_name2));
                let rename_2_result = get_err_or_ok(fs::rename(&path1, &final_name1));
                let rename_3_result = get_err_or_ok(fs::rename(&tmp_name2, &final_name2));
                if rename_1_result != 0 {
                    println!("FAILED: \n{:?} => {:?}", &path2, &tmp_name2);
                    rename_1_result
                } else if rename_2_result != 0 {
                    println!("FAILED: \n{:?} => {:?}", &path1, &final_name1);
                    rename_2_result
                } else if rename_3_result != 0 {
                    println!("FAILED: \n{:?} => {:?}", &tmp_name2, &final_name2);
                    rename_3_result
                } else {
                    println!("SUCCESS: \n{:?} => {:?}", &path2, &tmp_name2);
                    println!("SUCCESS: \n{:?} => {:?}", &path1, &final_name1);
                    println!("SUCCESS: \n{:?} => {:?}", &tmp_name2, &final_name2);
                    0
                }
            }
        }

        pub fn exchange(path1: String, path2: String) -> u8 {
            // 改名逻辑主体
            // 核验用户输入
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

            let (no_exist1, no_exist2) = metadata_get::if_no_exist(&path1, &path2);
            if no_exist1 || no_exist2 {
                return 1;
            }
            let (re_1, re_2) = metadata_get::if_relative(&path1, &path2);
            if re_1 || re_2 {
                return 2;
            }

            let (is_file1, is_file2) = metadata_get::if_file(&path1, &path2);
            let (name_1, ext_1, dir_1) = metadata_get::get_info(&path1);
            let (name_2, ext_2, dir_2) = metadata_get::get_info(&path2);

            let (pre_name1, new_name1) = make_name(dir_1, name_2, ext_1);
            let (pre_name2, new_name2) = make_name(dir_2, name_1, ext_2);
            //println!("{:?} {:?}", &new_name1, &new_name2); //test
            let (exist_new_1, exist_new_2) = metadata_get::if_no_exist(&new_name1, &new_name2);
            if (!exist_new_1) || (!exist_new_2) {
                return 4;
            }

            let mode = if_root(&path1, &path2);

            if is_file1 && is_file2 {
                //all files
                rename_each(path1, new_name1, path2, new_name2, pre_name2, false)
            } else if (!is_file1) && (!is_file2) {
                //all dirs
                if mode == 1 {
                    //file1 contains file2
                    rename_each(path1, new_name1, path2, new_name2, pre_name2, true)
                } else if mode == 2 {
                    //file2 contains file1
                    rename_each(path2, new_name2, path1, new_name1, pre_name1, true)
                } else {
                    //no contains
                    rename_each(path2, new_name2, path1, new_name1, pre_name1, false)
                }
            } else {
                // one file and one dir
                if is_file1 {
                    //1 is file and 2 is dir so impossible 1 contains 2
                    if mode == 1 {
                        //file1 rename first
                        rename_each(path1, new_name1, path2, new_name2, pre_name2, true)
                    } else {
                        rename_each(path1, new_name1, path2, new_name2, pre_name2, false)
                    }
                } else {
                    //same
                    if mode == 2 {
                        //file2 rename first
                        rename_each(path2, new_name2, path1, new_name1, pre_name1, true)
                    } else {
                        //file2 rename first
                        rename_each(path2, new_name2, path1, new_name1, pre_name1, false)
                    }
                }
            }
        }
    }
    pub mod show_error {
        pub fn switch_error_msg(num: u8) -> String {
            match num {
                0 => String::from("\nALL SUCCESS!"),
                1 => String::from("\nFAILED: \nInput paths not exist."),
                2 => String::from("\nFAILED: \nInput paths are not absoltue path."),
                3 => String::from("\nFAILED: \nNo permission to rename files."),
                4 => String::from("\nFAILED: \nThere are files with the same name you want to use in the folder already."),
                255 => String::from("\nFAILED: \nUNKNOWN ERROR when rename."),
                _ => String::from("\nFAILED: \nUNKNOWN ERROR"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    fn clear_olds() {
        let test_path1 = String::from("file1.ext1");
        let test_path2 = String::from("file2.ext2");
        let test_new_path1 = String::from("file2.ext1");
        let test_new_path2 = String::from("file1.ext2");

        let _ = remove_file(&test_path1);
        let _ = remove_file(&test_path2);
        let _ = remove_file(&test_new_path1);
        let _ = remove_file(&test_new_path2);

        let mut new_file1 = std::fs::File::create(&test_path1).unwrap();
        let mut new_file2 = std::fs::File::create(&test_path2).unwrap();
        let _ = std::io::Write::write_all(&mut new_file1, b"");
        let _ = std::io::Write::write_all(&mut new_file2, b"");
    }
    #[test]
    fn it_works() {
        //1 no exist
        //2 not absolte
        //3 no permission
        //4 already exist
        //255 unknown error

        clear_olds();

        let test_path1 =
            CString::new("D:\\languagelearning\\Rust\\exchange_name_lib\\file1.ext1").unwrap();

        let test_path2 =
            CString::new("D:\\languagelearning\\Rust\\exchange_name_lib\\file2.ext2").unwrap();
        let test_path1_ptr = test_path1.as_ptr() as *mut i8;
        let test_path2_ptr = test_path2.as_ptr() as *mut i8;

        let run_result = exchange_inputs(test_path1_ptr, test_path2_ptr);
        let run_result = unsafe { CString::from_raw(run_result) }
            .to_string_lossy()
            .to_string();
        println!("{}", run_result);
        ()
    }
}
