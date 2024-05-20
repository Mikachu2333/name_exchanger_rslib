```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //-1 no exist
        //-2 not absolte
        //-3 no permission
        process_input::change_name::exchange(
            String::from(r"PATH1"),
            String::from(r"PATH2"),
        );
    }
}
```
