```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    fn clear_olds(test_path1: &String, test_path2: &String) {
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
        let test_path1 = String::from("D:\\languagelearning\\Rust\\exchange_name_lib\\file1.ext1");
        let test_path2 = String::from("D:\\languagelearning\\Rust\\exchange_name_lib\\file2.ext2");

        clear_olds(&test_path1, &test_path2);

        let run_result = exchange_inputs(test_path1, test_path2);
        println!("{}", run_result);
    }
}
```
