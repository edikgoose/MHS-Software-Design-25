use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::process;

/// Grep command implementation with regex support, word boundaries, case-insensitive search, and context printing
#[derive(Parser)]
#[command(author, version, about = "Print lines matching a pattern", long_about = None)]
struct Args {
    /// Pattern to search for (supports regular expressions)
    pattern: String,

    /// Files to search in. If no files are specified, reads from stdin
    files: Vec<String>,

    /// Search for whole words only (word boundaries)
    #[arg(short = 'w', long)]
    word_regexp: bool,

    /// Perform case-insensitive matching
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Print NUM lines of trailing context after matching lines
    #[arg(short = 'A', long, value_name = "NUM")]
    after_context: Option<usize>,
}

/// Represents a match result with context information
#[derive(Debug, Clone)]
struct MatchResult {
    content: String,
    is_match: bool,
}

impl MatchResult {
    fn new(_line_number: usize, content: String, is_match: bool) -> Self {
        Self { content, is_match }
    }
}

/// Main grep functionality
struct GrepEngine {
    regex: Regex,
    after_context: Option<usize>,
    multiple_files: bool,
}

impl GrepEngine {
    fn new(
        pattern: &str,
        word_regexp: bool,
        ignore_case: bool,
        after_context: Option<usize>,
        multiple_files: bool,
    ) -> Result<Self, String> {
        let mut regex_pattern = pattern.to_string();

        if word_regexp {
            regex_pattern = format!(r"\b(?:{})\b", regex_pattern);
        }

        let mut regex_builder = regex::RegexBuilder::new(&regex_pattern);
        regex_builder.case_insensitive(ignore_case);

        let regex = regex_builder
            .build()
            .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;

        Ok(Self {
            regex,
            after_context,
            multiple_files,
        })
    }

    fn process_input<R: BufRead>(&self, reader: R, filename: Option<&str>) -> Result<(), String> {
        let mut matches = Vec::new();
        let mut line_number = 0;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Error reading input: {}", e))?;
            line_number += 1;

            let is_match = self.regex.is_match(&line);
            matches.push(MatchResult::new(line_number, line, is_match));
        }

        self.print_matches_with_context(&matches, filename);
        Ok(())
    }

    fn print_matches_with_context(&self, matches: &[MatchResult], filename: Option<&str>) {
        if matches.is_empty() {
            return;
        }

        let context_lines = self.after_context.unwrap_or(0);

        if context_lines == 0 {
            for match_result in matches {
                if match_result.is_match {
                    self.print_line(match_result, filename);
                }
            }
            return;
        }

        let mut i = 0;
        while i < matches.len() {
            if matches[i].is_match {
                let mut end_idx = (i + context_lines + 1).min(matches.len());

                let mut j = i + 1;
                while j < matches.len() && j < end_idx {
                    if matches[j].is_match {
                        end_idx = (j + context_lines + 1).min(matches.len());
                    }
                    j += 1;
                }

                for item in matches.iter().take(end_idx).skip(i) {
                    self.print_line(item, filename);
                }

                i = end_idx;

                if i < matches.len() {
                    println!("--");
                }
            } else {
                i += 1;
            }
        }
    }

    fn print_line(&self, match_result: &MatchResult, filename: Option<&str>) {
        if self.multiple_files {
            if let Some(fname) = filename {
                println!("{}:{}", fname, match_result.content);
            } else {
                println!("{}", match_result.content);
            }
        } else {
            println!("{}", match_result.content);
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.pattern.is_empty() {
        eprintln!("grep: empty pattern not allowed");
        process::exit(1);
    }

    let multiple_files = args.files.len() > 1;

    let engine = match GrepEngine::new(
        &args.pattern,
        args.word_regexp,
        args.ignore_case,
        args.after_context,
        multiple_files,
    ) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("grep: {}", e);
            process::exit(1);
        }
    };

    if args.files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        if let Err(e) = engine.process_input(reader, None) {
            eprintln!("grep: {}", e);
            process::exit(1);
        }
    } else {
        let mut has_errors = false;
        for filename in &args.files {
            match fs::File::open(filename) {
                Ok(file) => {
                    let reader = io::BufReader::new(file);
                    if let Err(e) = engine.process_input(reader, Some(filename)) {
                        eprintln!("grep: {}: {}", filename, e);
                        has_errors = true;
                    }
                }
                Err(e) => {
                    eprintln!("grep: {}: {}", filename, e);
                    has_errors = true;
                }
            }
        }
        if has_errors {
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process::Command;

    fn get_grep_binary_path() -> std::path::PathBuf {
        let mut path = env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("grep");
        path
    }

    fn run_grep(args: &[&str]) -> Result<String, String> {
        let output = Command::new(get_grep_binary_path())
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute grep: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn run_grep_with_input(pattern: &str, input: &str, args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new(get_grep_binary_path());
        cmd.arg(pattern)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn grep: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(input.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to get output: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[test]
    fn test_grep_basic_pattern() {
        let input = "line1\nline with test\nline3\ntest line\n";
        let result = run_grep_with_input("test", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("line with test"));
        assert!(lines[1].contains("test line"));
    }

    #[test]
    fn test_grep_regex_pattern() {
        let input = "test123\ntest456\nno match\n";
        let result = run_grep_with_input(r"test\d+", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("test123"));
        assert!(lines[1].contains("test456"));
    }

    #[test]
    fn test_grep_case_insensitive() {
        let input = "Test\nTEST\ntest\nNoMatch\n";
        let result = run_grep_with_input("test", input, &["-i"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Test"));
        assert!(lines[1].contains("TEST"));
        assert!(lines[2].contains("test"));
    }

    #[test]
    fn test_grep_word_boundary() {
        let input = "test\ntesting\ntest123\ntest_word\nno test here\n";
        let result = run_grep_with_input("test", input, &["-w"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("test"));
        assert!(lines[1].contains("test"));
    }

    #[test]
    fn test_grep_after_context() {
        let input = "line1\nmatch\nline3\nline4\nmatch\nline6\n";
        let result = run_grep_with_input("match", input, &["-A", "1"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert!(lines[0].contains("match"));
        assert!(lines[1].contains("line3"));
        assert_eq!(lines[2], "--");
        assert!(lines[3].contains("match"));
        assert!(lines[4].contains("line6"));
    }

    #[test]
    fn test_grep_after_context_zero() {
        let input = "line1\nmatch\nline3\n";
        let result = run_grep_with_input("match", input, &["-A", "0"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("match"));
    }

    #[test]
    fn test_grep_overlapping_context() {
        let input = "match1\ncontext1\nmatch2\ncontext2\n";
        let result = run_grep_with_input("match", input, &["-A", "2"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert!(lines[0].contains("match1"));
        assert!(lines[1].contains("context1"));
        assert!(lines[2].contains("match2"));
        assert!(lines[3].contains("context2"));
    }

    #[test]
    fn test_grep_multiple_files() {
        let test_dir = env::temp_dir().join("grep_test_multi");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");

        fs::write(&file1, "test in file1\n").unwrap();
        fs::write(&file2, "test in file2\n").unwrap();

        let result = run_grep(&["test", file1.to_str().unwrap(), file2.to_str().unwrap()]).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("file1.txt:"));
        assert!(lines[1].contains("file2.txt:"));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_grep_file_not_found() {
        let result = run_grep(&["pattern", "nonexistent.txt"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent.txt"));
    }

    #[test]
    fn test_grep_invalid_regex() {
        let result = run_grep(&["[invalid", "Cargo.toml"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid regex pattern"));
    }

    #[test]
    fn test_grep_empty_pattern() {
        let result = run_grep(&["", "Cargo.toml"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty pattern"));
    }

    #[test]
    fn test_grep_unicode_support() {
        let input = "hello мир\ntest 世界\n";
        let result = run_grep_with_input("мир", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("hello мир"));
    }

    #[test]
    fn test_grep_case_insensitive_unicode() {
        let input = "HELLO мир\nhello МИР\ntest\n";
        let result = run_grep_with_input("мир", input, &["-i"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("HELLO мир"));
        assert!(lines[1].contains("hello МИР"));
    }

    #[test]
    fn test_grep_line_anchors() {
        let input = "test at start\nmiddle test\nend test";
        let result = run_grep_with_input("^test", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("test at start"));
    }

    #[test]
    fn test_grep_end_anchor() {
        let input = "start test\ntest middle\ntest";
        let result = run_grep_with_input("test$", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("start test"));
        assert!(lines[1].contains("test"));
    }

    #[test]
    fn test_grep_complex_regex() {
        let input = "email@test.com\ninvalid-email\nuser+tag@domain.org\n@domain.com";
        let result = run_grep_with_input(
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
            input,
            &[],
        )
        .unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("email@test.com"));
        assert!(lines[1].contains("user+tag@domain.org"));
    }

    #[test]
    fn test_grep_no_matches() {
        let input = "line1\nline2\nline3";
        let result = run_grep_with_input("nomatch", input, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_grep_context_with_separator() {
        let input = "before1\nmatch1\nafter1\nbefore2\nmatch2\nafter2";
        let result = run_grep_with_input("match", input, &["-A", "1"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert!(lines[0].contains("match1"));
        assert!(lines[1].contains("after1"));
        assert_eq!(lines[2], "--");
        assert!(lines[3].contains("match2"));
        assert!(lines[4].contains("after2"));
    }

    #[test]
    fn test_grep_multiple_patterns_same_line() {
        let input = "test test\ntest other\ntest";
        let result = run_grep_with_input("test", input, &[]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("test test"));
        assert!(lines[1].contains("test other"));
        assert!(lines[2].contains("test"));
    }

    #[test]
    fn test_grep_case_insensitive_with_regex() {
        let input = "Test123\ntest456\nTEST789";
        let result = run_grep_with_input("test\\d+", input, &["-i"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("Test123"));
        assert!(lines[1].contains("test456"));
        assert!(lines[2].contains("TEST789"));
    }

    #[test]
    fn test_grep_word_boundary_unicode() {
        let input = "привет\nприветмир\nмир привет\nтест";
        let result = run_grep_with_input("привет", input, &["-w"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("привет"));
        assert!(lines[1].contains("мир привет"));
    }

    #[test]
    fn test_grep_word_boundary_with_alternation() {
        let input = "cat\ndog\nhotdog\nscat\ncatdog\ncat dog\n";
        let result = run_grep_with_input("cat|dog", input, &["-w"]).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("cat"));
        assert!(lines[1].contains("dog"));
        assert!(lines[2].contains("cat dog"));
        // Should NOT match: hotdog, scat, catdog
    }
}
