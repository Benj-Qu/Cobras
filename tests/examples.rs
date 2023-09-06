use snake::runner;

macro_rules! mk_test {
    ($test_name:ident, $file_name:expr, $expected_output:expr) => {
        #[test]
        fn $test_name() -> std::io::Result<()> {
            test_example_file($file_name, $expected_output)
        }
    };
}

macro_rules! mk_fail_test {
    ($test_name:ident, $file_name:expr, $expected_output:expr) => {
        #[test]
        fn $test_name() -> std::io::Result<()> {
            test_example_fail($file_name, $expected_output)
        }
    };
}

/*
 * YOUR TESTS GO HERE
 */

// Check Basic functionality
mk_test!(dummy_class, "simple.garter", "483");
// Multiple objects with different classes
mk_test!(multiple_classes, "multi_class.garter", "477");
// Multiple objects with the same class
// Show that different objects in the same class won't affect each other
// Check Constructor
mk_test!(multiple_objects, "multi_objs.garter", "2");
// An example class called car
mk_test!(car, "car.garter", "8000\ntrue");
// A more complicated case on traingle and normal_traingle classes
mk_test!(triangle, "triangle.garter", "true\n30\n60\n30");
// Error: Calling method from another class
mk_fail_test!(
    wrong_method,
    "wrong_method.garter",
    "calling method from another class"
);
// Error: Calling method on non-object.
// Handled same as calling method from another class
mk_fail_test!(
    wrong_object,
    "wrong_object.garter",
    "calling method from another class"
);
// Error: Constructing object with wrong number of fields
mk_fail_test!(
    wrong_field,
    "wrong_field.garter",
    "Wrong number of fields applied to construct object"
);
// Error: Modifying non-field in class method
mk_fail_test!(set_non_field, "set_nonfield.garter", "Undefined field");
// Error: Duplicate field
mk_fail_test!(
    dup_field,
    "dup_field.garter",
    "multiple defined class field"
);

// IMPLEMENTATION
fn test_example_file(f: &str, expected_str: &str) -> std::io::Result<()> {
    use std::path::Path;
    let p_name = format!("examples/{}", f);
    let path = Path::new(&p_name);

    let tmp_dir = tempfile::TempDir::new()?;
    let mut w = Vec::new();
    match runner::compile_and_run_file(&path, tmp_dir.path(), &mut w) {
        Ok(()) => {
            let stdout = std::str::from_utf8(&w).unwrap();
            assert_eq!(stdout.trim(), expected_str)
        }
        Err(e) => {
            assert!(false, "Expected {}, got an error: {}", expected_str, e)
        }
    }
    Ok(())
}

fn test_example_fail(f: &str, includes: &str) -> std::io::Result<()> {
    use std::path::Path;
    let p_name = format!("examples/{}", f);
    let path = Path::new(&p_name);

    let tmp_dir = tempfile::TempDir::new()?;
    let mut w_run = Vec::new();
    match runner::compile_and_run_file(
        &Path::new(&format!("examples/{}", f)),
        tmp_dir.path(),
        &mut w_run,
    ) {
        Ok(()) => {
            let stdout = std::str::from_utf8(&w_run).unwrap();
            assert!(false, "Expected a failure but got: {}", stdout.trim())
        }
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains(includes),
                "Expected error message to include the string \"{}\" but got the error: {}",
                includes,
                msg
            )
        }
    }
    Ok(())
}
