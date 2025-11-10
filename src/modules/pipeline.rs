use crate::modules::command::Command;
use crate::modules::environment::Environment;
use crate::modules::runner::Runner;

use std::fs;

/// Handles execution of command pipelines.
///
/// The `Pipeline` struct is responsible for chaining commands together,
/// where the standard output (stdout) of each command becomes the standard
/// input (stdin) of the next command.
///
/// # Pipeline Behavior
///
/// - **stdout**: Connected between commands (cmd1 stdout → cmd2 stdin)
/// - **stderr**: Independent for each command (not piped)
/// - **Error handling**: Commands continue executing even if previous ones fail
///
/// # Examples
///
/// ```no_run
/// use cli_rust::modules::{pipeline::Pipeline, runner::Runner, Environment};
/// use std::path::PathBuf;
///
/// let runner = Runner::new(PathBuf::from("target/release"));
/// let pipeline = Pipeline::new(runner);
/// let env = Environment::new();
///
/// // Pipeline will be executed when commands are provided
/// ```
pub struct Pipeline {
    runner: Runner,
}

impl Pipeline {
    /// Creates a new Pipeline executor.
    ///
    /// # Arguments
    ///
    /// * `runner` - The command executor to use for running each command
    ///
    /// # Returns
    ///
    /// A new `Pipeline` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::{pipeline::Pipeline, runner::Runner};
    /// use std::path::PathBuf;
    ///
    /// let runner = Runner::new(PathBuf::from("target/release"));
    /// let pipeline = Pipeline::new(runner);
    /// ```
    pub fn new(runner: Runner) -> Self {
        Self { runner }
    }

    /// Executes a pipeline of commands, connecting stdout of each to stdin of the next.
    ///
    /// For a pipeline like `cmd1 | cmd2 | cmd3`:
    /// - cmd1's stdout becomes cmd2's stdin
    /// - cmd2's stdout becomes cmd3's stdin  
    /// - cmd3's output is returned as a String
    /// - Each command's stderr goes to the terminal (not piped)
    ///
    /// If any command fails, the pipeline continues but the error is reported via stderr.
    ///
    /// # Arguments
    ///
    /// * `parsed_cmds` - Vector of parsed commands to execute in sequence
    /// * `env_vars` - Environment variables available to commands
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The final output of the last command
    /// * `Err(String)` - Error message if stdin file reading fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cli_rust::modules::{pipeline::Pipeline, runner::Runner, Environment};
    /// use cli_rust::modules::input::command::Command;
    /// use std::path::PathBuf;
    ///
    /// let runner = Runner::new(PathBuf::from("target/release"));
    /// let pipeline = Pipeline::new(runner);
    /// let env = Environment::new();
    ///
    /// let cmd1 = Command::new("echo".to_string(), vec!["hello".to_string()]);
    /// let cmd2 = Command::new("cat".to_string(), vec![]);
    ///
    /// // This would execute: echo hello | cat
    /// // let result = pipeline.execute(vec![cmd1, cmd2], &env);
    /// ```
    pub fn execute(
        &self,
        parsed_cmds: Vec<crate::modules::input::command::Command>,
        env_vars: &Environment,
    ) -> Result<String, String> {
        if parsed_cmds.is_empty() {
            return Ok(String::new());
        }

        // Single command - no pipeline
        if parsed_cmds.len() == 1 {
            let pc = &parsed_cmds[0];
            let cmd = self.prepare_command(pc)?;
            return self.execute_command(cmd, env_vars);
        }

        // Pipeline execution: chain commands
        let mut pipeline_input: Option<String> = None;
        let mut final_output = String::new();

        for (i, pc) in parsed_cmds.iter().enumerate() {
            let is_last = i == parsed_cmds.len() - 1;
            let mut cmd = Command::new(pc.name.clone(), pc.args.clone());

            // First command: handle stdin redirection from file if specified
            if i == 0 {
                if let Some(ref stdin_file) = pc.stdin {
                    match fs::read_to_string(stdin_file) {
                        Ok(content) => {
                            // First command reads from file - pass as stdin
                            cmd = cmd.with_stdin(content);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Error reading stdin file '{}': {}",
                                stdin_file, e
                            ));
                        }
                    }
                }
            } else {
                // Middle/last commands: use output from previous command
                if let Some(input) = pipeline_input.take() {
                    cmd = cmd.with_stdin(input);
                }
            }

            // Handle stderr redirection (independent of pipeline)
            if let Some(ref stderr_file) = pc.stderr {
                cmd = cmd
                    .with_stderr(stderr_file.clone())
                    .with_append_stderr(pc.append_stderr);
            }

            // Last command: handle stdout redirection if specified
            if is_last {
                if let Some(ref stdout_file) = pc.stdout {
                    cmd = cmd
                        .with_stdout(stdout_file.clone())
                        .with_append_stdout(pc.append_stdout);
                }
            }

            // Execute the command
            match self.runner.execute(cmd, env_vars) {
                Ok(output) => {
                    if is_last {
                        final_output = output;
                    } else {
                        // Not last: pass output to next command
                        pipeline_input = Some(output);
                    }
                }
                Err(error) => {
                    eprintln!("Error executing '{}': {}", pc.name, error);
                    // Pipeline continues, but without output from this command
                    pipeline_input = Some(String::new());
                    if is_last {
                        final_output = String::new();
                    }
                }
            }
        }

        Ok(final_output)
    }

    /// Prepares a Command from a parsed command, handling file redirections.
    ///
    /// # Arguments
    ///
    /// * `pc` - The parsed command from InputProcessor
    ///
    /// # Returns
    ///
    /// * `Ok(Command)` - A Command ready for execution
    /// * `Err(String)` - Error message if stdin file cannot be read
    fn prepare_command(
        &self,
        pc: &crate::modules::input::command::Command,
    ) -> Result<Command, String> {
        let mut cmd = Command::new(pc.name.clone(), pc.args.clone());

        // Handle stdin redirection from file
        if let Some(ref stdin_file) = pc.stdin {
            match fs::read_to_string(stdin_file) {
                Ok(content) => {
                    cmd = cmd.with_stdin(content);
                }
                Err(e) => {
                    return Err(format!("Error reading stdin file '{}': {}", stdin_file, e));
                }
            }
        }

        // Handle stdout redirection
        if let Some(ref stdout_file) = pc.stdout {
            cmd = cmd
                .with_stdout(stdout_file.clone())
                .with_append_stdout(pc.append_stdout);
        }

        // Handle stderr redirection
        if let Some(ref stderr_file) = pc.stderr {
            cmd = cmd
                .with_stderr(stderr_file.clone())
                .with_append_stderr(pc.append_stderr);
        }

        Ok(cmd)
    }

    /// Executes a single command and returns its output.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `env_vars` - Environment variables available to the command
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The command's output
    /// * `Err(String)` - Error message if execution fails
    fn execute_command(&self, command: Command, env_vars: &Environment) -> Result<String, String> {
        self.runner
            .execute(command, env_vars)
            .map_err(|e| format!("Error executing command: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::input::command::Command as ParsedCommand;
    use std::path::PathBuf;

    #[test]
    fn test_pipeline_creation() {
        let runner = Runner::new(PathBuf::from("target/release"));
        let _pipeline = Pipeline::new(runner);
        // Should not panic
    }

    #[test]
    fn test_empty_pipeline() {
        let runner = Runner::new(PathBuf::from("target/release"));
        let pipeline = Pipeline::new(runner);
        let env = Environment::new();

        let result = pipeline.execute(vec![], &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_single_command_pipeline() {
        let runner = Runner::new(PathBuf::from("target/release"));
        let pipeline = Pipeline::new(runner);
        let env = Environment::new();

        let cmd = ParsedCommand::new("echo".to_string(), vec!["test".to_string()]);
        let result = pipeline.execute(vec![cmd], &env);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test"));
    }

    #[test]
    fn test_prepare_command_with_redirections() {
        let runner = Runner::new(PathBuf::from("target/release"));
        let pipeline = Pipeline::new(runner);

        let mut pc = ParsedCommand::new("cat".to_string(), vec![]);
        pc.stdout = Some("output.txt".to_string());
        pc.append_stdout = false;
        pc.stderr = Some("errors.txt".to_string());
        pc.append_stderr = false;

        let result = pipeline.prepare_command(&pc);
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.name, "cat");
        assert_eq!(cmd.stdout, Some("output.txt".to_string()));
        assert_eq!(cmd.stderr, Some("errors.txt".to_string()));
    }

    #[test]
    fn test_prepare_command_stdin_error() {
        let runner = Runner::new(PathBuf::from("target/release"));
        let pipeline = Pipeline::new(runner);

        let mut pc = ParsedCommand::new("cat".to_string(), vec![]);
        pc.stdin = Some("/nonexistent/file.txt".to_string());

        let result = pipeline.prepare_command(&pc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Error reading stdin file"));
    }
}
