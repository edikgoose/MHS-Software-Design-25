use std::collections::HashMap;
use std::path::PathBuf;

use crate::modules::input::{Environment, InputProcessor, InputProcessorBuilder};

/// Initialization configuration for the CLI shell.
///
/// This struct serves as the single source of truth for the shell's configuration,
/// holding environment variables and the path to custom command binaries.
///
/// # Fields
///
/// * `env_vars` - The environment variables available to the shell
/// * `bin_path` - Path to the directory containing custom command implementations
///
/// # Examples
///
/// ```
/// use cli_rust::modules::init::Init;
///
/// // Create with system environment
/// let init = Init::new();
///
/// // Access environment variables
/// if let Some(home) = init.get_env("HOME") {
///     println!("Home directory: {}", home);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Init {
    /// Environment variables for the shell session
    pub env_vars: Environment,
    /// Path to directory containing custom command binaries
    pub bin_path: PathBuf,
}

impl Init {
    /// Creates a new Init instance with system environment variables.
    ///
    /// This constructor:
    /// 1. Captures all current system environment variables
    /// 2. Determines the binary path in this order:
    ///    - Uses `CLI_BIN_PATH` environment variable if set
    ///    - Falls back to `target/release` if it contains the echo binary
    ///    - Otherwise uses `target/debug`
    ///
    /// # Returns
    ///
    /// A new `Init` instance configured with system environment and binary path
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let init = Init::new();
    /// println!("Using binary path: {:?}", init.bin_path);
    /// ```
    pub fn new() -> Self {
        let env_vars = Environment::capture_current();

        // Check for CLI_BIN_PATH env var
        let bin_path = if let Ok(custom_path) = std::env::var("CLI_BIN_PATH") {
            PathBuf::from(custom_path)
        } else {
            // Try default path
            let debug_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("target")
                .join("debug");

            let release_path = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("target")
                .join("release");

            // Prefer release
            if release_path.join("echo").exists() {
                release_path
            } else {
                debug_path
            }
        };

        Init { env_vars, bin_path }
    }

    /// Creates a new Init with custom environment variables and binary path.
    ///
    /// This constructor is primarily intended for testing scenarios where you need
    /// full control over the environment configuration.
    ///
    /// # Arguments
    ///
    /// * `env_vars` - Pre-configured environment variables
    /// * `bin_path` - Path to the directory containing command binaries
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use cli_rust::modules::{Environment, init::Init};
    ///
    /// let mut env = Environment::new();
    /// env.set("TEST_VAR", "value");
    /// let init = Init::with_config(env, PathBuf::from("/custom/bin"));
    /// ```
    pub fn with_config(env_vars: Environment, bin_path: PathBuf) -> Self {
        Init { env_vars, bin_path }
    }

    /// Creates a new Init with environment variables from a HashMap and binary path.
    ///
    /// This is a convenience constructor for testing that accepts a HashMap
    /// and converts it to an Environment internally.
    ///
    /// # Arguments
    ///
    /// * `env_vars` - HashMap of environment variable key-value pairs
    /// * `bin_path` - Path to the directory containing command binaries
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use cli_rust::modules::init::Init;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("USER".to_string(), "alice".to_string());
    /// let init = Init::with_config_map(vars, PathBuf::from("/bin"));
    /// ```
    pub fn with_config_map(env_vars: HashMap<String, String>, bin_path: PathBuf) -> Self {
        Init {
            env_vars: Environment::with_vars(env_vars),
            bin_path,
        }
    }

    /// Retrieves the value of an environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the environment variable
    ///
    /// # Returns
    ///
    /// `Some(&str)` if the variable exists, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let init = Init::new();
    /// if let Some(path) = init.get_env("PATH") {
    ///     println!("PATH is: {}", path);
    /// }
    /// ```
    pub fn get_env(&self, key: &str) -> Option<&str> {
        self.env_vars.get(key)
    }

    /// Sets or updates an environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the environment variable
    /// * `value` - The value to set
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let mut init = Init::new();
    /// init.set_env("MY_VAR".to_string(), "my_value".to_string());
    /// assert_eq!(init.get_env("MY_VAR"), Some("my_value"));
    /// ```
    pub fn set_env(&mut self, key: String, value: String) {
        self.env_vars.set(key, value);
    }

    /// Returns a reference to all environment variables.
    ///
    /// # Returns
    ///
    /// A reference to the `Environment` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let init = Init::new();
    /// for (key, value) in init.env_vars().iter() {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn env_vars(&self) -> &Environment {
        &self.env_vars
    }
}

impl Default for Init {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_input_processor() -> InputProcessor {
    InputProcessorBuilder::new().build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_creation() {
        let init = Init::new();
        assert!(!init.env_vars.is_empty());
        assert!(init.bin_path.to_string_lossy().contains("target"));
    }

    #[test]
    fn test_with_config_init() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "42".to_string());
        let bin_path = PathBuf::from("/test_path/bin");

        let init = Init::with_config_map(env_vars, bin_path.clone());
        assert_eq!(init.get_env("TEST_VAR"), Some("42"));
        assert_eq!(init.bin_path, bin_path);
    }

    #[test]
    fn test_set_env() {
        let mut init = Init::new();
        init.set_env("NEW_VAR".to_string(), "new_value".to_string());
        assert_eq!(init.get_env("NEW_VAR"), Some("new_value"));
    }
}
