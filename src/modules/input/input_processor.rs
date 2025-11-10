use crate::modules::input::{
    command::Command,
    errors::{CliError, Result},
    expander::Expander,
    quote_handler::QuoteHandler,
    tokenizer::Tokenizer,
    Environment,
};

/// Converts parsed tokens into Command objects with I/O redirection support.
///
/// `CommandProducer` is responsible for the final stage of command parsing:
/// taking a vector of expanded token strings and producing a `Command` object
/// that includes proper configuration for stdin, stdout, and stderr redirections.
///
/// # Redirection Support
///
/// The producer handles the following redirection operators:
/// - `<` or `0<` - Redirect stdin from file
/// - `>` or `1>` - Redirect stdout to file (overwrite)
/// - `>>` or `1>>` - Redirect stdout to file (append)
/// - `2>` - Redirect stderr to file (overwrite)
/// - `2>>` - Redirect stderr to file (append)
///
/// # Examples
///
/// ```ignore
/// let pieces = vec!["cat".to_string(), ">".to_string(), "output.txt".to_string()];
/// let cmd = CommandProducer::produce_command(pieces).unwrap();
/// assert_eq!(cmd.name, "cat");
/// assert_eq!(cmd.stdout, Some("output.txt".to_string()));
/// ```
pub struct CommandProducer;

impl CommandProducer {
    /// Produces a Command object from tokenized command pieces, handling redirection operators.
    ///
    /// This method parses redirection operators and builds a Command with appropriate
    /// stdin, stdout, and stderr configurations. Tokens that are not redirection operators
    /// or their associated filenames become command arguments.
    ///
    /// # Arguments
    ///
    /// * `pieces` - Vector of tokenized strings representing the command and its arguments/redirections
    ///
    /// # Returns
    ///
    /// - `Ok(Command)` - Successfully parsed command with redirections
    /// - `Err(CliError::EmptyCommand)` - If pieces vector is empty
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Simple command with no redirection
    /// let cmd = CommandProducer::produce_command(vec!["echo".to_string(), "hello".to_string()]).unwrap();
    ///
    /// // Command with output redirection
    /// let cmd = CommandProducer::produce_command(
    ///     vec!["cat".to_string(), "file.txt".to_string(), ">".to_string(), "output.txt".to_string()]
    /// ).unwrap();
    /// ```
    pub fn produce_command(mut pieces: Vec<String>) -> Result<Command> {
        if pieces.is_empty() {
            return Err(CliError::EmptyCommand);
        }
        let name = pieces.remove(0);
        let mut args = Vec::<String>::new();
        let mut stdin = None;
        let mut stdout = None;
        let mut append_stdout = false;
        let mut stderr = None;
        let mut append_stderr = false;

        let mut it = pieces.into_iter().peekable();
        while let Some(p) = it.next() {
            match p.as_str() {
                "<" | "0<" => stdin = it.next(),
                ">" | "1>" => {
                    append_stdout = false;
                    stdout = it.next();
                }
                ">>" | "1>>" => {
                    append_stdout = true;
                    stdout = it.next();
                }
                "2>" => {
                    append_stderr = false;
                    stderr = it.next();
                }
                "2>>" => {
                    append_stderr = true;
                    stderr = it.next();
                }
                _ => {
                    // Check for patterns like "3>", "4>>", etc.
                    if let Some(fd_redirect) = parse_fd_redirect(&p) {
                        let target_file = it.next();
                        match fd_redirect {
                            (0, false) => stdin = target_file, // "0>"  (unusual but possible)
                            (1, false) => {
                                append_stdout = false;
                                stdout = target_file;
                            } // "1>"
                            (1, true) => {
                                append_stdout = true;
                                stdout = target_file;
                            } // "1>>"
                            (2, false) => {
                                append_stderr = false;
                                stderr = target_file;
                            } // "2>"
                            (2, true) => {
                                append_stderr = true;
                                stderr = target_file;
                            } // "2>>"
                            _ => {
                                // For fd >= 3, we could extend Command struct to support them
                                // For now, ignore or add to args
                                args.push(p);
                                if let Some(file) = target_file {
                                    args.push(file);
                                }
                            }
                        }
                    } else {
                        args.push(p);
                    }
                }
            }
        }
        let mut cmd = Command::new(name, args);
        cmd.stdin = stdin;
        cmd.stdout = stdout;
        cmd.append_stdout = append_stdout;
        cmd.stderr = stderr;
        cmd.append_stderr = append_stderr;
        Ok(cmd)
    }
}

/// Builder for creating `InputProcessor` instances.
///
/// Provides a fluent interface for constructing an `InputProcessor` with
/// default or custom configurations.
///
/// # Examples
///
/// ```
/// use cli_rust::modules::input::InputProcessorBuilder;
///
/// let processor = InputProcessorBuilder::new().build();
/// ```
pub struct InputProcessorBuilder {}

impl InputProcessorBuilder {
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Builds an `InputProcessor` with the configured settings.
    ///
    /// # Returns
    ///
    /// A new `InputProcessor` instance ready for parsing commands
    pub fn build(self) -> InputProcessor {
        InputProcessor {
            expander: Expander::default(),
        }
    }
}

impl Default for InputProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Facade for parsing and processing shell command input.
///
/// `InputProcessor` coordinates the entire command parsing pipeline:
/// 1. **Tokenization** - Splits input into tokens, respecting quotes and escapes
/// 2. **Variable Expansion** - Expands environment variables like `$VAR` and `${VAR}`
/// 3. **Command Production** - Converts expanded tokens into `Command` objects with I/O redirection
///
/// The processor handles complex shell features including:
/// - Quoted strings (single and double quotes)
/// - Escape sequences with backslash
/// - Environment variable expansion
/// - I/O redirection operators
/// - Pipeline support (commands separated by `|`)
///
/// # Architecture
///
/// This is a facade pattern that delegates to specialized components:
/// - `Tokenizer` - Breaks input into tokens
/// - `QuoteHandler` - Processes quotes and escapes
/// - `Expander` - Performs variable substitution
/// - `CommandProducer` - Builds final Command objects
///
/// # Examples
///
/// ```
/// use cli_rust::modules::{Environment, input::InputProcessorBuilder};
///
/// let processor = InputProcessorBuilder::new().build();
/// let mut env = Environment::new();
/// env.set("NAME", "Alice");
///
/// let commands = processor.process(r#"echo "Hello, $NAME!""#, &env).unwrap();
/// assert_eq!(commands[0].name, "echo");
/// assert_eq!(commands[0].args[0], "Hello, Alice!");
/// ```
#[derive(Clone)]
pub struct InputProcessor {
    /// Variable expander for environment variable substitution
    expander: Expander,
}

impl InputProcessor {
    /// Processes a command line input string into one or more executable commands.
    ///
    /// This is the main entry point for command parsing. It performs the complete
    /// parsing pipeline:
    /// 1. Tokenizes the input string (handles quotes and escapes)
    /// 2. Splits on pipe operators (`|`) to identify separate commands
    /// 3. Handles quotes and escapes for each command segment
    /// 4. Expands environment variables in each token
    /// 5. Produces final Command objects with I/O redirections
    ///
    /// # Arguments
    ///
    /// * `line` - The raw input string to parse (e.g., `"echo $USER > file.txt"`)
    /// * `env_vars` - Environment variables for variable expansion
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Command>)` - Vector of parsed commands (multiple if pipes are used)
    /// - `Err(CliError)` - If parsing fails at any stage
    ///
    /// # Examples
    ///
    /// ```
    /// use cli_rust::modules::{Environment, input::InputProcessorBuilder};
    ///
    /// let processor = InputProcessorBuilder::new().build();
    /// let env = Environment::new();
    ///
    /// // Simple command
    /// let cmds = processor.process("echo hello", &env).unwrap();
    /// assert_eq!(cmds.len(), 1);
    ///
    /// // Pipeline with multiple commands
    /// let cmds = processor.process("cat file.txt | grep pattern", &env).unwrap();
    /// assert_eq!(cmds.len(), 2);
    /// ```
    pub fn process(&self, line: &str, env_vars: &Environment) -> Result<Vec<Command>> {
        // 1) Токенизируем всю строку (учитывая кавычки/экраны)
        let raw = Tokenizer::tokenize(line)?;

        // 2) Делим список токенов по некавычённому токену "|"
        let parts = split_on_pipes_tokens(&raw);

        // 3) Обрабатываем каждую часть отдельной командой
        let mut cmds = Vec::with_capacity(parts.len());
        for raw_part in parts {
            let tokens = QuoteHandler::handle(&raw_part)?;
            let pieces = self.expander.expand_tokens(env_vars, tokens)?;
            cmds.push(CommandProducer::produce_command(pieces)?);
        }
        Ok(cmds)
    }
}

/// Parse file descriptor redirection patterns like "2>", "3>>", "1>", etc.
/// Returns Some((fd_number, is_append)) if the string matches a pattern, None otherwise.
fn parse_fd_redirect(s: &str) -> Option<(u32, bool)> {
    if s.ends_with(">>") {
        if let Some(fd_part) = s.strip_suffix(">>") {
            if let Ok(fd) = fd_part.parse::<u32>() {
                return Some((fd, true)); // append mode
            }
        }
    } else if let Some(fd_part) = s.strip_suffix('>') {
        // Pattern like "2>" or "1>"
        if let Ok(fd) = fd_part.parse::<u32>() {
            return Some((fd, false)); // overwrite mode
        }
    }
    None
}

/// Делит уже токенизированную строку на команды по токену "|".
/// Токен "|" будет отдельным элементом только если он вне кавычек.
fn split_on_pipes_tokens(raw: &[String]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for t in raw {
        if t == "|" {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(t.clone());
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
