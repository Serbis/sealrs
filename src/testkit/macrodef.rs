/// Common testkit macros

/// Print fmt info text of the test case
#[macro_export]
macro_rules! test_case {
    ($l:expr) => {
        println!("Run test case: {}", $l);
    };
}