use std::path::Path;
use std::process::Command;

fn decode_stdout(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_owned();
    }
    let (decoded, _, _) = encoding_rs::GB18030.decode(bytes);
    decoded.into_owned()
}

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
            println!("checking {}", path.file_name().unwrap().to_string_lossy());

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

            let formatted = decode_stdout(&output.stdout);
            let expected_bytes = std::fs::read(&out_path)
                .unwrap_or_else(|e| panic!("Failed to read expected output {:?}: {}", out_path, e));
            let expected = decode_stdout(&expected_bytes);

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
