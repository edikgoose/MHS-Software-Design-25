Project of a simple CLI on Rust

[![CI](https://github.com/fatalem0/MHS-Software-Design-25/actions/workflows/rust.yml/badge.svg?branch=task%2Fhw2)](https://github.com/fatalem0/MHS-Software-Design-25/actions/workflows/rust.yml)

Test coverage (covered regions | executed functions | covered lines): 
![Static Badge](https://img.shields.io/badge/cov-88%25%20%E2%94%82%2091%25%20%E2%94%82%2084%25-21b577.svg)

## Supported functionality:
- can run own implementations of `wc`, `echo`, `cat`, `pwd`, `grep`
- can run other commands if there is no own implementation
- `exit`, `help`
- setting environment variables
- redirecting `stdin`, `stdout`, `stderr`
- substition of environment variables in weak quotes and in cases without qoutes
- pipelines: chain commands together with `|` operator; the standard output (stdout) of each command is connected to the standard input (stdin) of the next command in the pipeline

### Built-in Commands Features

#### grep
A powerful text search utility with the following features:
- **Regular expressions**: Full regex pattern support using Rust's `regex` crate
- **Word boundaries** (`-w`): Search for whole words only (Unicode-aware word boundaries)
- **Case-insensitive search** (`-i`): Ignore case when matching
- **Context printing** (`-A NUM`): Print NUM lines after each match
- **Multiple file support**: Search across multiple files with filename prefixes
- **Stdin support**: Read from stdin when no files are specified

#### Argument Parsing Library Choice

For the `grep` command implementation, we chose **clap v4.5** as the argument parsing library. Here's why:

**Alternatives considered:**
- `getopts`: Built into Rust std library, minimal dependencies, but very basic functionality
- `structopt`: Wrapper around clap v2, now deprecated in favor of clap's derive API
- `argh`: Simple derive-based argument parser, good for basic use cases

**Why clap:**
- **Most popular and actively maintained**: The de-facto standard for CLI argument parsing in Rust
- **Rich feature set**: Supports all required functionality (flags, options, positional arguments, help generation)
- **Excellent documentation**: Comprehensive docs with examples
- **Performance**: Fast compilation and runtime performance
- **Derive API**: Clean, declarative syntax using `#[derive(Parser)]`
- **Future-proof**: Actively developed with regular releases

**Trade-offs:**
- Adds dependency size vs getopts (but clap is widely used so likely cached)
- More complex API vs simpler alternatives (but provides much more functionality)

## Build and run instructions:
- install Rust toolchain (https://rustup.rs/)
- In the root of the project execute `cargo build -r`
- Run the executable by `./target/release/cli-shell`

## Usage examples:
```
>echo "Hello, world!"
Hello, world!
```

```
> echo Some example text > example.txt
> FILE=example.txt
Set FILE=example.txt
> wc < example.txt
       1        3       18
> cat $FILE
Some example text
> rm example.txt
```

```
> x=ex
Set x=ex
> y=it
Set y=it
> $x$y
Goodbye!
```

```
> cat file.txt | wc -l
> cat file1.txt file2.txt | grep "pattern" > results.txt
> cat data.txt | grep "error" | wc -l
```

#### grep Examples

```
> grep "Минимальный" README.md
Минимальный синтаксис grep
```

```
> grep "Минимальный$" README.md
```

```
> grep "^Минимальный" README.md
Минимальный синтаксис grep
```

```
> grep -i "минимальный" README.md
Минимальный синтаксис grep
```

```
> grep -w "Минимал" README.md
```

```
> grep -A 1 "II" README.md
```

```
> echo -e "line1\nerror line2\nline3\nerror line4" | grep -A 1 "error"
error line2
line3
--
error line4
```

## Testing

### Running Tests

Execute the full test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run specific test:

```bash
cargo test test_name
```

### Test Coverage

This project supports test coverage analysis using either `cargo-llvm-cov`.

#### Using cargo-llvm-cov

Install cargo-llvm-cov:

```bash
cargo install cargo-llvm-cov
```

Generate coverage report:

```bash
cargo llvm-cov --html
```

View the report:

```bash
# Opens the HTML report in your default browser
cargo llvm-cov --open
```

For terminal output:

```bash
cargo llvm-cov
```

Generate detailed lcov format (for CI/CD integration):

```bash
cargo llvm-cov --lcov --output-path lcov.info
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.
