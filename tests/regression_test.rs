use std::path::Path;
use std::process::Command;

#[test]
fn regression_test_against_binary() {
    let binary = env!("CARGO_BIN_EXE_sqlfmt");
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut tested = false;

    for entry in std::fs::read_dir(&tests_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "sql") {
            tested = true;
            let out_path = path.with_extension("out");

            let output = Command::new(&binary)
                .arg(&path)
                .output()
                .unwrap_or_else(|e| panic!("Failed to run sqlfmt on {:?}: {}", path, e));

            assert!(
                output.status.success(),
                "sqlfmt exited with error on {:?}:\n{}",
                path,
                String::from_utf8_lossy(&output.stderr),
            );

            let formatted = String::from_utf8(output.stdout)
                .unwrap_or_else(|e| panic!("Output is not valid UTF-8: {}", e));
            let expected = std::fs::read_to_string(&out_path)
                .unwrap_or_else(|e| panic!("Failed to read expected output {:?}: {}", out_path, e));

            assert_eq!(
                formatted,
                expected,
                "Formatting mismatch for {:?}",
                path.file_name().unwrap(),
            );
        }
    }

    assert!(tested, "No .sql files found in tests/ directory");
}
