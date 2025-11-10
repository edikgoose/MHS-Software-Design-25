Project of a simple CLI on Rust

[![CI](https://github.com/fatalem0/MHS-Software-Design-25/actions/workflows/rust.yml/badge.svg?branch=task%2Fhw2)](https://github.com/fatalem0/MHS-Software-Design-25/actions/workflows/rust.yml)

![Static Badge](https://img.shields.io/badge/cov-87%25%20%E2%94%82%2091%25%20%E2%94%82%2082%25-21b577.svg)


## Supported functionality:
- can run own implementations of `wc`, `echo`, `cat`, `pwd`
- can run other commands if there is no own implementation
- `exit`, `help`
- setting environment variables
- redirecting `stdin`, `stdout`, `stderr`
- substition of environment variables in weak quotes and in cases without qoutes

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
