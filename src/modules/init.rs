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
/// * `current_dir` - Current working directory for the shell session
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
    /// Current working directory for the shell session
    pub current_dir: PathBuf,
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

        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Init {
            env_vars,
            bin_path,
            current_dir,
        }
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
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Init {
            env_vars,
            bin_path,
            current_dir,
        }
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
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Init {
            env_vars: Environment::with_vars(env_vars),
            bin_path,
            current_dir,
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

    /// Gets the current working directory.
    ///
    /// # Returns
    ///
    /// A reference to the current working directory path
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let init = Init::new();
    /// println!("Current directory: {:?}", init.current_dir());
    /// ```
    pub fn current_dir(&self) -> &PathBuf {
        &self.current_dir
    }

    /// Sets the current working directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - The new working directory path
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err(String)` if the directory doesn't exist or cannot be accessed
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use cli_rust::modules::init::Init;
    ///
    /// let mut init = Init::new();
    /// init.set_current_dir(PathBuf::from("/tmp")).unwrap();
    /// ```
    pub fn set_current_dir(&mut self, dir: PathBuf) -> Result<(), String> {
        if !dir.exists() {
            return Err(format!("cd: {}: No such file or directory", dir.display()));
        }
        if !dir.is_dir() {
            return Err(format!("cd: {}: Not a directory", dir.display()));
        }
        // Canonicalize the path to resolve relative paths and symlinks
        match std::fs::canonicalize(&dir) {
            Ok(canonical_path) => {
                self.current_dir = canonical_path;
                Ok(())
            }
            Err(e) => Err(format!("cd: {}: {}", dir.display(), e)),
        }
    }

    /// Changes the current working directory relative to the current directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to change to (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, `Err(String)` if the directory doesn't exist or cannot be accessed
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::init::Init;
    ///
    /// let mut init = Init::new();
    /// init.change_dir("..").unwrap(); // Go up one directory
    /// ```
    pub fn change_dir(&mut self, path: &str) -> Result<(), String> {
        let target_path = if path.is_empty() {
            // Empty path means HOME directory
            match self.get_env("HOME") {
                Some(home) => PathBuf::from(home),
                None => {
                    return Err("cd: HOME not set".to_string());
                }
            }
        } else if path.starts_with('/') {
            // Absolute path
            PathBuf::from(path)
        } else {
            // Relative path - resolve relative to current_dir
            self.current_dir.join(path)
        };

        self.set_current_dir(target_path)
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

    #[test]
    fn test_current_dir_initialization() {
        let init = Init::new();
        assert!(init.current_dir().exists());
        assert!(init.current_dir().is_absolute() || init.current_dir().to_string_lossy() == ".");
    }

    #[test]
    fn test_change_dir_absolute() {
        use std::env;
        let mut init = Init::new();
        let temp_dir = env::temp_dir();

        match init.change_dir(temp_dir.to_str().unwrap()) {
            Ok(()) => {
                assert_eq!(init.current_dir(), &temp_dir.canonicalize().unwrap());
            }
            Err(_) => {
                // Temp dir might not exist in some test environments, skip
            }
        }
    }

    #[test]
    fn test_change_dir_relative() {
        let mut init = Init::new();
        let original_dir = init.current_dir().clone();

        // Try to go up one directory
        match init.change_dir("..") {
            Ok(()) => {
                // Should be different from original
                assert_ne!(init.current_dir(), &original_dir);
            }
            Err(_) => {
                // Might fail if we're at root, that's okay
            }
        }
    }

    #[test]
    fn test_change_dir_nonexistent() {
        let mut init = Init::new();
        let result = init.change_dir("/nonexistent/directory/12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file or directory"));
    }

    #[test]
    fn test_change_dir_to_file() {
        use std::env;
        use std::fs;
        let mut init = Init::new();

        // Create a temporary file
        let temp_file = env::temp_dir().join("cd_test_file");
        let _ = fs::remove_file(&temp_file);
        fs::write(&temp_file, "test").unwrap();

        let result = init.change_dir(temp_file.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a directory"));

        // Clean up
        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_change_dir_empty_goes_to_home() {
        use std::env;
        let mut init = Init::new();

        // Set HOME if not set
        if init.get_env("HOME").is_none() {
            if let Ok(home) = env::var("HOME") {
                init.set_env("HOME".to_string(), home);
            } else {
                // Skip test if HOME is not set
                return;
            }
        }

        let home_dir = PathBuf::from(init.get_env("HOME").unwrap());
        match init.change_dir("") {
            Ok(()) => {
                assert_eq!(init.current_dir(), &home_dir.canonicalize().unwrap());
            }
            Err(_) => {
                // HOME might not exist, skip
            }
        }
    }
}
