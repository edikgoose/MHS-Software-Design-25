// Integration tests for pipeline functionality
use cli_rust::modules::Environment;

#[test]
fn test_pipeline_parsing() {
    // Test that pipelines are correctly parsed into multiple commands
    use cli_rust::modules::input::InputProcessorBuilder;

    let processor = InputProcessorBuilder::new().build();
    let env = Environment::new();

    let cmds = processor.process("echo hello | cat | wc -l", &env).unwrap();

    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].name, "echo");
    assert_eq!(cmds[1].name, "cat");
    assert_eq!(cmds[2].name, "wc");
}

#[test]
fn test_pipeline_with_redirections() {
    // Test pipeline with input/output redirections
    use cli_rust::modules::input::InputProcessorBuilder;

    let processor = InputProcessorBuilder::new().build();
    let env = Environment::new();

    let cmds = processor
        .process(
            "cat < input.txt | grep pattern > output.txt 2> errors.log",
            &env,
        )
        .unwrap();

    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].name, "cat");
    assert_eq!(cmds[0].stdin, Some("input.txt".to_string()));
    assert_eq!(cmds[1].name, "grep");
    assert_eq!(cmds[1].stdout, Some("output.txt".to_string()));
    assert_eq!(cmds[1].stderr, Some("errors.log".to_string()));
}

#[test]
fn test_exit_in_pipeline() {
    // Test that exit in pipeline is detected
    use cli_rust::modules::input::InputProcessorBuilder;

    let processor = InputProcessorBuilder::new().build();
    let env = Environment::new();

    // Parse a pipeline with exit
    let cmds = processor.process("echo hello | exit | cat", &env).unwrap();

    assert_eq!(cmds.len(), 3);
    // The middle command is exit
    assert_eq!(cmds[1].name, "exit");
}

#[test]
fn test_pipeline_with_variables() {
    // Test pipeline with environment variable expansion
    use cli_rust::modules::input::InputProcessorBuilder;

    let mut env = Environment::new();
    env.set("FILE", "test.txt");
    env.set("PATTERN", "error");

    let processor = InputProcessorBuilder::new().build();

    let cmds = processor
        .process("cat $FILE | grep $PATTERN", &env)
        .unwrap();

    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].name, "cat");
    assert_eq!(cmds[0].args, vec!["test.txt"]);
    assert_eq!(cmds[1].name, "grep");
    assert_eq!(cmds[1].args, vec!["error"]);
}

#[test]
fn test_complex_pipeline_parsing() {
    // Test a more complex pipeline
    use cli_rust::modules::input::InputProcessorBuilder;

    let processor = InputProcessorBuilder::new().build();
    let env = Environment::new();

    let cmds = processor
        .process(
            "cat file1.txt file2.txt | grep -v '^#' | sort | uniq > output.txt",
            &env,
        )
        .unwrap();

    assert_eq!(cmds.len(), 4);
    assert_eq!(cmds[0].name, "cat");
    assert_eq!(cmds[0].args, vec!["file1.txt", "file2.txt"]);
    assert_eq!(cmds[1].name, "grep");
    assert_eq!(cmds[1].args, vec!["-v", "^#"]);
    assert_eq!(cmds[2].name, "sort");
    assert_eq!(cmds[3].name, "uniq");
    assert_eq!(cmds[3].stdout, Some("output.txt".to_string()));
}

#[test]
fn test_stderr_not_piped() {
    // Verify that stderr redirection is independent per command
    use cli_rust::modules::input::InputProcessorBuilder;

    let processor = InputProcessorBuilder::new().build();
    let env = Environment::new();

    let cmds = processor
        .process(
            "cmd1 2> err1.txt | cmd2 2> err2.txt | cmd3 2> err3.txt",
            &env,
        )
        .unwrap();

    assert_eq!(cmds.len(), 3);
    assert_eq!(cmds[0].stderr, Some("err1.txt".to_string()));
    assert_eq!(cmds[1].stderr, Some("err2.txt".to_string()));
    assert_eq!(cmds[2].stderr, Some("err3.txt".to_string()));
}

#[test]
fn test_pipeline_stdin_redirection_execution() {
    // Test that stdin redirection works correctly in pipelines
    use cli_rust::modules::{pipeline::Pipeline, runner::Runner, Environment};
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    // Create a temporary test file
    let test_dir = env::temp_dir().join("cli_pipeline_stdin_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let test_file = test_dir.join("test_input.txt");
    let test_content = "line1\nline2\nline3\n";
    fs::write(&test_file, test_content).expect("Failed to write test file");

    // Parse command: cat < test_file | wc (without -l flag, as our wc doesn't support it)
    let processor = cli_rust::modules::input::InputProcessorBuilder::new().build();
    let env = Environment::new();

    let input = format!("cat < {} | wc", test_file.display());
    let cmds = processor.process(&input, &env).unwrap();

    // Verify parsing
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].name, "cat");
    assert_eq!(cmds[0].stdin, Some(test_file.to_string_lossy().to_string()));

    // Execute the pipeline
    let bin_path = PathBuf::from("target/release");
    let runner = Runner::new(bin_path);
    let pipeline = Pipeline::new(runner);
    let current_dir = PathBuf::from(".");

    let result = pipeline.execute(cmds, &env, &current_dir);

    assert!(result.is_ok(), "Pipeline execution failed");
    let output = result.unwrap();

    // Output should contain the line count (3) at the beginning
    // Format: "       3        3       18"
    assert!(
        output.trim().starts_with("3") || output.contains("       3"),
        "Expected output to contain line count 3, got: {}",
        output
    );

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
