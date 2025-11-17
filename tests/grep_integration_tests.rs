// Integration tests for grep command through CLI shell
// Tests grep functionality via pipelines and file redirections

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path(name: &str) -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push(name);
    path
}

fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let file_path = dir.join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

fn setup_test_dir(test_name: &str) -> PathBuf {
    let test_dir = env::temp_dir().join(format!("grep_integration_test_{}", test_name));
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

fn cleanup_test_dir(test_dir: &PathBuf) {
    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_grep_with_stdin_redirection() {
    let test_dir = setup_test_dir("stdin_redirect");
    let input_file = create_test_file(
        &test_dir,
        "input.txt",
        "line1\ntest line\nline3\nanother test\n",
    );
    let output_file = test_dir.join("output.txt");

    let grep_path = get_binary_path("grep");

    // grep "test" < input.txt > output.txt
    let output = Command::new(&grep_path)
        .arg("test")
        .stdin(fs::File::open(&input_file).unwrap())
        .stdout(fs::File::create(&output_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());

    let result = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("test line"));
    assert!(lines[1].contains("another test"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_through_pipe() {
    let echo_path = get_binary_path("echo");
    let grep_path = get_binary_path("grep");

    // echo "test\nline\ntest again" | grep test
    let echo_output = Command::new(&echo_path)
        .args(["test\nline\ntest again"])
        .output()
        .expect("Failed to execute echo");

    let grep_output = Command::new(&grep_path)
        .arg("test")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(&echo_output.stdout)?;
            }
            child.wait_with_output()
        })
        .expect("Failed to pipe to grep");

    assert!(grep_output.status.success());
    let result = String::from_utf8_lossy(&grep_output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("test"));
    assert!(lines[1].contains("test again"));
}

#[test]
fn test_grep_cat_pipe_chain() {
    let test_dir = setup_test_dir("cat_pipe");
    let input_file = create_test_file(
        &test_dir,
        "input.txt",
        "first line\nMATCH here\nsecond line\nANOTHER MATCH\n",
    );

    let cat_path = get_binary_path("cat");
    let grep_path = get_binary_path("grep");

    // cat input.txt | grep "MATCH"
    let cat_output = Command::new(&cat_path)
        .arg(input_file.to_str().unwrap())
        .output()
        .expect("Failed to execute cat");

    let grep_output = Command::new(&grep_path)
        .arg("MATCH")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(&cat_output.stdout)?;
            }
            child.wait_with_output()
        })
        .expect("Failed to pipe to grep");

    assert!(grep_output.status.success());
    let result = String::from_utf8_lossy(&grep_output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("MATCH here"));
    assert!(lines[1].contains("ANOTHER MATCH"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_word_boundary_integration() {
    let test_dir = setup_test_dir("word_boundary");
    let input_file = create_test_file(
        &test_dir,
        "words.txt",
        "test\ntesting\ntest123\ntest_word\nno test here\ntest\n",
    );

    let grep_path = get_binary_path("grep");

    // grep -w "test" < words.txt
    let output = Command::new(&grep_path)
        .args(["-w", "test"])
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should match only "test" and "no test here"
    assert_eq!(lines.len(), 3); // "test" appears twice + "no test here"
    assert!(lines
        .iter()
        .all(|line| line.trim() == "test" || line.contains("no test here")));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_case_insensitive_integration() {
    let test_dir = setup_test_dir("case_insensitive");
    let input_file = create_test_file(
        &test_dir,
        "mixed_case.txt",
        "Test\nTEST\ntest\ntesting\nNoMatch\n",
    );

    let grep_path = get_binary_path("grep");

    // grep -i "test" < mixed_case.txt
    let output = Command::new(&grep_path)
        .args(["-i", "test"])
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 4); // Test, TEST, test, testing
    assert!(lines[0].contains("Test"));
    assert!(lines[1].contains("TEST"));
    assert!(lines[2].contains("test"));
    assert!(lines[3].contains("testing"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_after_context_integration() {
    let test_dir = setup_test_dir("after_context");
    let input_file = create_test_file(
        &test_dir,
        "context.txt",
        "line1\nmatch\ncontext1\ncontext2\nline5\nmatch\ncontext3\n",
    );

    let grep_path = get_binary_path("grep");

    // grep -A 2 "match" < context.txt
    let output = Command::new(&grep_path)
        .args(["-A", "2", "match"])
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should have: match, context1, context2, --, match, context3
    assert!(lines.len() >= 6);
    assert!(lines[0].contains("match"));
    assert!(lines[1].contains("context1"));
    assert!(lines[2].contains("context2"));
    assert_eq!(lines[3], "--");
    assert!(lines[4].contains("match"));
    assert!(lines[5].contains("context3"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_regex_integration() {
    let test_dir = setup_test_dir("regex");
    let input_file = create_test_file(
        &test_dir,
        "regex.txt",
        "test123\ntest456\nno match\ntest\nabc123\n",
    );

    let grep_path = get_binary_path("grep");

    // grep "test\d+" < regex.txt (regex pattern)
    let output = Command::new(&grep_path)
        .arg(r"test\d+")
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("test123"));
    assert!(lines[1].contains("test456"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_line_anchors_integration() {
    let test_dir = setup_test_dir("line_anchors");
    let input_file = create_test_file(
        &test_dir,
        "anchors.txt",
        "test at start\nmiddle test\ntest\nend test\n",
    );

    let grep_path = get_binary_path("grep");

    // grep "^test" < anchors.txt (lines starting with "test")
    let output = Command::new(&grep_path)
        .arg("^test")
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("test at start"));
    assert_eq!(lines[1].trim(), "test");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_multiple_files_integration() {
    let test_dir = setup_test_dir("multiple_files");
    let file1 = create_test_file(&test_dir, "file1.txt", "test in file1\nline2\n");
    let file2 = create_test_file(&test_dir, "file2.txt", "line1\ntest in file2\n");
    let file3 = create_test_file(&test_dir, "file3.txt", "no match here\n");

    let grep_path = get_binary_path("grep");

    // grep "test" file1.txt file2.txt file3.txt
    let output = Command::new(&grep_path)
        .arg("test")
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .arg(file3.to_str().unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should have matches from file1 and file2 with filenames
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("file1.txt:"));
    assert!(lines[0].contains("test in file1"));
    assert!(lines[1].contains("file2.txt:"));
    assert!(lines[1].contains("test in file2"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_combined_flags_integration() {
    let test_dir = setup_test_dir("combined_flags");
    let input_file = create_test_file(
        &test_dir,
        "combined.txt",
        "Test word\nTEST\ntesting\ntest word here\ntest\nno match\n",
    );

    let grep_path = get_binary_path("grep");

    // grep -w -i "test" < combined.txt
    let output = Command::new(&grep_path)
        .args(["-w", "-i", "test"])
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should match: "Test word", "TEST", "test word here", "test"
    // Should NOT match: "testing" (not a whole word boundary after 'test')
    assert_eq!(lines.len(), 4);
    assert!(lines.iter().any(|l| l.contains("Test word")));
    assert!(lines.iter().any(|l| l.trim() == "TEST"));
    assert!(lines.iter().any(|l| l.contains("test word here")));
    assert!(lines.iter().any(|l| l.trim() == "test"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_unicode_integration() {
    let test_dir = setup_test_dir("unicode");
    let input_file = create_test_file(
        &test_dir,
        "unicode.txt",
        "hello world\nпривет мир\ntest 世界\nмир\n",
    );

    let grep_path = get_binary_path("grep");

    // grep "мир" < unicode.txt
    let output = Command::new(&grep_path)
        .arg("мир")
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("привет мир"));
    assert!(lines[1].contains("мир"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_empty_result_integration() {
    let test_dir = setup_test_dir("empty_result");
    let input_file = create_test_file(&test_dir, "no_match.txt", "line1\nline2\nline3\n");

    let grep_path = get_binary_path("grep");

    // grep "nomatch" < no_match.txt
    let output = Command::new(&grep_path)
        .arg("nomatch")
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    assert!(result.is_empty());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_grep_error_handling_invalid_file() {
    let grep_path = get_binary_path("grep");

    // grep "pattern" nonexistent.txt
    let output = Command::new(&grep_path)
        .args(["pattern", "/nonexistent/file/path.txt"])
        .output()
        .expect("Failed to execute grep");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nonexistent") || stderr.contains("No such file"));
}

#[test]
fn test_grep_overlapping_context_integration() {
    let test_dir = setup_test_dir("overlapping");
    let input_file = create_test_file(
        &test_dir,
        "overlap.txt",
        "match1\ncontext1\nmatch2\ncontext2\n",
    );

    let grep_path = get_binary_path("grep");

    // grep -A 2 "match" < overlap.txt
    // Should merge overlapping contexts
    let output = Command::new(&grep_path)
        .args(["-A", "2", "match"])
        .stdin(fs::File::open(&input_file).unwrap())
        .output()
        .expect("Failed to execute grep");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // All lines should be printed without duplication
    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains("match1"));
    assert!(lines[1].contains("context1"));
    assert!(lines[2].contains("match2"));
    assert!(lines[3].contains("context2"));
    // No separator because contexts overlap

    cleanup_test_dir(&test_dir);
}
