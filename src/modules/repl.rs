use crate::modules::init::Init;
use crate::modules::input::{InputProcessor, InputProcessorBuilder};
use crate::modules::pipeline::Pipeline;
use crate::modules::runner::Runner;

use std::io::{self, Write};
use std::path::PathBuf;

/// Read-Eval-Print Loop (REPL) for the CLI shell.
///
/// The `Repl` struct provides an interactive shell session where users can:
/// - Execute commands (both custom and system)
/// - Set and use environment variables
/// - Use I/O redirection
/// - Execute pipelines
/// - View help information
/// - Exit the shell
///
/// # Architecture
///
/// The REPL coordinates three main components:
/// - `InputProcessor` - Parses command input
/// - `Pipeline` - Executes command pipelines
/// - `Init` - Provides environment and configuration
///
/// # Built-in Commands
///
/// - `exit` - Exits the shell (accepts any arguments, but ignores them)
/// - `help` - Displays available commands
/// - `NAME=VALUE` - Sets an environment variable
///
/// # Examples
///
/// ```no_run
/// use cli_rust::modules::{init::Init, repl::Repl};
///
/// let init = Init::new();
/// let mut repl = Repl::new(&init);
/// // Uncomment to run interactively:
/// // repl.run(&mut init);
/// ```
pub struct Repl {
    /// Path to custom command binaries
    bin_path: PathBuf,
    /// Pipeline executor
    pipeline: Pipeline,
    /// Command parser and processor
    input_processor: InputProcessor,
}

impl Repl {
    /// Creates a new REPL instance from initialization configuration.
    ///
    /// # Arguments
    ///
    /// * `init` - Configuration containing environment variables and binary path
    ///
    /// # Returns
    ///
    /// A new `Repl` instance ready to start an interactive session
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::{init::Init, repl::Repl};
    ///
    /// let init = Init::new();
    /// let repl = Repl::new(&init);
    /// ```
    pub fn new(init: &Init) -> Self {
        let bin_path = init.bin_path.clone();
        let runner = Runner::new(bin_path.clone());
        let pipeline = Pipeline::new(runner);

        let input_processor = InputProcessorBuilder::new().build();

        Repl {
            bin_path,
            pipeline,
            input_processor,
        }
    }

    pub fn run(&mut self, init: &mut Init) {
        println!("CLI Shell started with bin path: {:?}", self.bin_path);
        println!("Type 'exit' to quit or 'help' for available commands.");

        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    if input.is_empty() {
                        continue;
                    }

                    if input == "exit" {
                        println!("Goodbye!");
                        break;
                    }

                    if input == "help" {
                        self.show_help();
                        continue;
                    }

                    // Check if it's a variable assignment (NAME=VALUE)
                    if self.is_variable_assignment(input) {
                        self.handle_variable_assignment(input, init);
                        continue;
                    }

                    // Process as command
                    match self.input_processor.process(input, init.env_vars()) {
                        Ok(parsed_cmds) => {
                            // Check if any command in the pipeline is 'exit'
                            // For exit in a pipeline, we exit the shell
                            if parsed_cmds.iter().any(|cmd| cmd.name == "exit") {
                                println!("Goodbye!");
                                break;
                            }

                            // Check if it's a single 'help' command
                            if parsed_cmds.len() == 1
                                && parsed_cmds[0].name == "help"
                                && parsed_cmds[0].args.is_empty()
                            {
                                self.show_help();
                                continue;
                            }

                            // Execute the pipeline
                            match self.pipeline.execute(parsed_cmds, init.env_vars()) {
                                Ok(output) => {
                                    if !output.trim().is_empty() {
                                        print!("{}", output);
                                    }
                                }
                                Err(e) => eprintln!("{}", e),
                            }
                        }
                        Err(e) => eprintln!("parse error: {e}"),
                    }
                }
                Err(error) => {
                    eprintln!("Error reading input: {}", error);
                    break;
                }
            }
        }
    }

    fn is_variable_assignment(&self, input: &str) -> bool {
        // Simple check for pattern NAME=VALUE where NAME is a valid identifier
        if let Some(eq_pos) = input.find('=') {
            let name_part = &input[..eq_pos];
            // Check if name part is a valid identifier (starts with letter/underscore, contains alphanumeric/underscore)
            if !name_part.is_empty()
                && name_part.chars().all(|c| c.is_alphanumeric() || c == '_')
                && (name_part.chars().next().unwrap().is_alphabetic() || name_part.starts_with('_'))
            {
                return true;
            }
        }
        false
    }

    fn handle_variable_assignment(&mut self, input: &str, init: &mut Init) {
        if let Some(eq_pos) = input.find('=') {
            let name = &input[..eq_pos];
            let value = &input[eq_pos + 1..];

            // Update init's environment (single source of truth)
            init.set_env(name.to_string(), value.to_string());
            println!("Set {}={}", name, value);
        }
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("Built-in commands:");
        println!("  echo [args...]     - Print arguments to stdout");
        println!("  cat [files...]     - Display file contents or read from stdin");
        println!("  wc [files...]      - Count lines, words, and bytes in files or stdin");
        println!("  pwd               - Print current working directory");
        println!("  help              - Show this help message");
        println!("  exit              - Exit the shell");
        println!();
        println!("Shell features:");
        println!("  NAME=VALUE         - Set environment variable");
        println!("  $VAR or ${{VAR}}     - Variable expansion");
        println!("  cmd < file         - Redirect stdin from file");
        println!("  cmd > file         - Redirect stdout to file (overwrite)");
        println!("  cmd >> file        - Redirect stdout to file (append)");
        println!("  cmd 2> file        - Redirect stderr to file (overwrite)");
        println!("  cmd 2>> file       - Redirect stderr to file (append)");
        // println!("  cmd1 | cmd2        - Pipe output between commands");
        println!("  [command]          - Execute any system command or fallback to built-in");
    }
}

impl Default for Repl {
    fn default() -> Self {
        let init = Init::new();
        Self::new(&init)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::command::Command;
    use std::env;
    use std::fs;

    #[test]
    fn test_repl_creation() {
        let init = Init::new();
        let repl = Repl::new(&init);

        assert_eq!(repl.bin_path, init.bin_path);
        // Environment variables are now managed by Init, not duplicated in Repl
    }

    #[test]
    fn test_repl_default() {
        let _repl = Repl::default();
        // Should not panic
    }

    #[test]
    fn test_variable_assignment_detection() {
        let repl = Repl::default();

        // Valid assignments
        assert!(repl.is_variable_assignment("VAR=value"));
        assert!(repl.is_variable_assignment("PATH=/usr/bin"));
        assert!(repl.is_variable_assignment("_PRIVATE=secret"));
        assert!(repl.is_variable_assignment("VAR123=test"));

        // Invalid assignments
        assert!(!repl.is_variable_assignment("echo hello"));
        assert!(!repl.is_variable_assignment("=value"));
        assert!(!repl.is_variable_assignment("123VAR=value"));
        assert!(!repl.is_variable_assignment("VAR-NAME=value"));
        assert!(!repl.is_variable_assignment(""));
    }

    #[test]
    fn test_stdin_file_reading() {
        let test_dir = env::temp_dir().join("cli_repl_test_stdin");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");

        let input_file = test_dir.join("test_input.txt");
        let test_content = "Test file content\nLine 2\nLine 3\n";
        fs::write(&input_file, test_content).expect("Failed to write test file");

        // Test file reading functionality indirectly by checking if fs::read_to_string works
        // (The actual REPL stdin file reading is tested in integration tests)
        let content = fs::read_to_string(&input_file).expect("Failed to read test file");
        assert_eq!(content, test_content);
        assert!(content.contains("Test file content"));
        assert!(content.contains("Line 2"));
        assert!(content.contains("Line 3"));

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_stdin_file_reading_error_handling() {
        // Test reading non-existent file
        let result = fs::read_to_string("/nonexistent/file.txt");
        assert!(result.is_err());

        // The error handling in REPL will print an error message and continue
        // This behavior is tested in integration tests
    }

    #[test]
    fn test_command_creation_with_redirection() {
        // Test the command creation logic that the REPL uses
        let name = "cat".to_string();
        let args = vec![];
        let mut cmd = Command::new(name.clone(), args.clone());

        // Simulate stdin redirection
        let stdin_content = "file content".to_string();
        cmd = cmd.with_stdin(stdin_content.clone());

        // Simulate stdout redirection
        let stdout_file = "output.txt".to_string();
        cmd = cmd
            .with_stdout(stdout_file.clone())
            .with_append_stdout(false);

        assert_eq!(cmd.name, name);
        assert_eq!(cmd.args, args);
        assert_eq!(cmd.stdin, Some(stdin_content));
        assert_eq!(cmd.stdout, Some(stdout_file));
        assert!(!cmd.append_stdout);
    }

    #[test]
    fn test_command_creation_with_append_redirection() {
        let name = "echo".to_string();
        let args = vec!["test".to_string()];
        let mut cmd = Command::new(name.clone(), args.clone());

        // Simulate stdout append redirection
        let stdout_file = "output.txt".to_string();
        cmd = cmd
            .with_stdout(stdout_file.clone())
            .with_append_stdout(true);

        assert_eq!(cmd.name, name);
        assert_eq!(cmd.args, args);
        assert!(cmd.stdin.is_none());
        assert_eq!(cmd.stdout, Some(stdout_file));
        assert!(cmd.append_stdout);
    }

    #[test]
    fn test_environment_variable_handling() {
        let mut init = Init::new();
        let _repl = Repl::new(&init);

        // Test setting environment variable through init (single source of truth)
        init.set_env("TEST_VAR".to_string(), "test_value".to_string());

        // Test getting environment variable
        let value = init.get_env("TEST_VAR");
        assert_eq!(value, Some("test_value"));

        // Test non-existent variable
        let no_value = init.get_env("NONEXISTENT_VAR");
        assert_eq!(no_value, None);
    }
}
