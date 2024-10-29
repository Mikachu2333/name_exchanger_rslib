use std::ffi::{c_char, CStr};
use std::path::PathBuf;

use file_rename::NameExchange;
use path_checkout::GetPathInfo;
mod file_rename;
mod path_checkout;

#[no_mangle]
/// # Safety
/// 最终暴露的执行函数
pub unsafe extern "C" fn exchange(path1: *const c_char, path2: *const c_char) -> u8 {
    let binding = std::env::current_exe().unwrap();
    let exe_dir = binding.parent().unwrap();

    let path1 = unsafe { CStr::from_ptr(path1).to_str().unwrap().to_owned() };
    let path2 = unsafe { CStr::from_ptr(path2) }
        .to_str()
        .unwrap()
        .to_owned();

    let mut all_infos = NameExchange::new();

    // 用于校验文件夹路径最后是否为斜杠与双引号的闭包
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
            s
        }
    };
    let mut packed_path = GetPathInfo {
        path1: dir_check(path1),
        path2: dir_check(path2),
    };

    (all_infos.f1.is_exist, all_infos.f2.is_exist) = (packed_path).if_no_exist(exe_dir);
    if all_infos.f1.is_exist || all_infos.f2.is_exist {
        return 1;
    }
    all_infos.f1.exchange.original_path = packed_path.path1.clone();
    all_infos.f2.exchange.original_path = packed_path.path2.clone();

    (all_infos.f1.is_file, all_infos.f2.is_file) = packed_path.if_file();
    (all_infos.f1.packed_info, all_infos.f2.packed_info) = packed_path.metadata_collect();

    (
        all_infos.f1.exchange.pre_path,
        all_infos.f1.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f1.packed_info.parent_dir,
        &all_infos.f2.packed_info.name,
        &all_infos.f1.packed_info.ext,
    );
    (
        all_infos.f2.exchange.pre_path,
        all_infos.f2.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f2.packed_info.parent_dir,
        &all_infos.f1.packed_info.name,
        &all_infos.f2.packed_info.ext,
    );
    let mut packed_path_new = GetPathInfo {
        //其实没必要mut
        path1: all_infos.f1.exchange.new_path.clone(),
        path2: all_infos.f2.exchange.new_path.clone(),
    };

    //println!("{:?} {:?}", &packed_path_new.path1, &packed_path_new.path2); //test
    let (exist_new_1, exist_new_2) = GetPathInfo::if_no_exist(&mut packed_path_new, exe_dir);
    if (!exist_new_1) || (!exist_new_2) {
        return 4;
    }

    //1 -> file1 should be renamed first
    let mode = packed_path.if_root();

    if all_infos.f1.is_file & all_infos.f2.is_file {
        //all files
        NameExchange::rename_each(&all_infos, false, true)
    } else if (!all_infos.f1.is_file) && (!all_infos.f2.is_file) {
        //all dirs
        if mode == 1 {
            //file1 contains file2
            NameExchange::rename_each(&all_infos, true, true)
        } else if mode == 2 {
            //file2 contains file1
            NameExchange::rename_each(&all_infos, true, false)
        } else {
            //no contains
            NameExchange::rename_each(&all_infos, false, true)
        }
    } else {
        // one file and one dir
        if all_infos.f1.is_file {
            //1 is file and 2 is dir so impossible 1 contains 2
            if mode == 1 {
                //file1 rename first
                NameExchange::rename_each(&all_infos, true, true)
            } else {
                NameExchange::rename_each(&all_infos, false, true)
            }
        } else {
            //same
            if mode == 2 {
                //file2 rename first
                NameExchange::rename_each(&all_infos, true, false)
            } else {
                //file2 rename first
                NameExchange::rename_each(&all_infos, false, false)
            }
        }
    }
}
