use std::collections::HashMap;

/// Environment variable container providing type-safe access to shell environment variables.
///
/// This struct wraps a `HashMap<String, String>` and provides a convenient API for managing
/// environment variables throughout the CLI application. It ensures consistent handling of
/// environment variables across different modules.
///
/// # Examples
///
/// ```
/// use cli_rust::modules::Environment;
///
/// let mut env = Environment::new();
/// env.set("USER", "alice");
/// assert_eq!(env.get("USER"), Some("alice"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Environment {
    /// Internal storage for environment variables
    vars: HashMap<String, String>,
}

impl Environment {
    /// Creates a new empty environment.
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let env = Environment::new();
    /// assert!(env.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Creates an environment from an existing HashMap of variables.
    ///
    /// # Arguments
    ///
    /// * `vars` - A HashMap containing environment variable key-value pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use cli_rust::modules::Environment;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("HOME".to_string(), "/home/user".to_string());
    /// let env = Environment::with_vars(vars);
    /// assert_eq!(env.get("HOME"), Some("/home/user"));
    /// ```
    pub fn with_vars(vars: HashMap<String, String>) -> Self {
        Self { vars }
    }

    /// Retrieves the value of an environment variable.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the environment variable to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&str)` if the variable exists, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let mut env = Environment::new();
    /// env.set("PATH", "/usr/bin");
    /// assert_eq!(env.get("PATH"), Some("/usr/bin"));
    /// assert_eq!(env.get("NONEXISTENT"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(|s| s.as_str())
    }

    /// Sets or updates an environment variable.
    ///
    /// # Arguments
    ///
    /// * `k` - The key (variable name), can be any type that converts to String
    /// * `v` - The value, can be any type that converts to String
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let mut env = Environment::new();
    /// env.set("DEBUG", "1");
    /// env.set("PORT".to_string(), "8080".to_string());
    /// ```
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, k: K, v: V) {
        self.vars.insert(k.into(), v.into());
    }

    /// Removes an environment variable.
    ///
    /// # Arguments
    ///
    /// * `k` - The key (variable name) to remove
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let mut env = Environment::new();
    /// env.set("TEMP", "value");
    /// env.remove("TEMP");
    /// assert_eq!(env.get("TEMP"), None);
    /// ```
    pub fn remove(&mut self, k: &str) {
        self.vars.remove(k);
    }

    /// Captures the current system environment variables.
    ///
    /// Creates an Environment instance populated with all environment variables
    /// from the current process's environment.
    ///
    /// # Returns
    ///
    /// A new Environment containing all current system environment variables
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let env = Environment::capture_current();
    /// // Will contain system variables like PATH, HOME, etc.
    /// ```
    pub fn capture_current() -> Self {
        let vars = std::env::vars().collect::<HashMap<_, _>>();
        Self { vars }
    }

    /// Returns an iterator over environment variable key-value pairs.
    ///
    /// # Returns
    ///
    /// An iterator yielding references to (key, value) pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let mut env = Environment::new();
    /// env.set("A", "1");
    /// env.set("B", "2");
    ///
    /// for (key, value) in env.iter() {
    ///     println!("{} = {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.vars.iter()
    }

    /// Checks if the environment contains no variables.
    ///
    /// # Returns
    ///
    /// `true` if there are no environment variables, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::Environment;
    ///
    /// let mut env = Environment::new();
    /// assert!(env.is_empty());
    /// env.set("KEY", "value");
    /// assert!(!env.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }
}
