# Rust Code Translator

![GitHub License](https://img.shields.io/badge/license-MIT-blue.svg)

Rust Code Translator is a versatile program that translates code written in a custom PHP-like language into other programming languages. This repository includes a lexer and parser for the source code and provides translation functionality for two target languages: Python 3 and x86_64 assembly for Linux using FASM.

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [Supported Targets](#supported-targets)
- [Author](#author)

## Features

- Translate custom PHP-like code into Python 3 or x86_64 assembly for Linux.
- Lexer and parser for the source code.
- Support for functions, function calls, string literals, numbers, and control flow statements.

## Getting Started

### Prerequisites

Before you begin, ensure you have met the following requirements:

- [Rust](https://www.rust-lang.org/tools/install) installed on your system.
- [FASM](https://flatassembler.net/download.php) (only for assembly translation) installed on your system.

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/your-github-username/rust-code-translator.git
2. Build the Rust program using Cargo:
    cd rust-code-translator
    cargo build --release
  This will create an executable in the target/release directory.
### Usage

  -You can run the program using the following command:
    ./target/release/rust_code_translator [OPTIONS] <input.c>
  Replace <input.c> with the path to your input code file. Here are some options you can use:

  -target <target>: Specify the compilation target. Use python3 for Python 3 translation (default) or fasm-x86_64-linux for x86_64 assembly for Linux.
  -help: Print the usage message.
### Supported Targets

  Python 3: Translates the code into Python 3, providing similar functionality.

  x86_64 Assembly for Linux (FASM): Translates the code into x86_64 assembly for Linux using the Flat Assembler (FASM). The generated assembly code can b assembled 
  and linked with FASM to create an executable.
### Author

  Gastordia
