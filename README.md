# N-gram Context Analyzer

A final bachelor degree project that corrects contextual errors in Croatian text using statistical n-gram frequencies.

## Table of Contents
- [About The Project](#about-the-project)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Technologies](#technologies)
- [Configuration](#configuration)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## About The Project

The **N-gram Context Analyzer** is a tool developed as a final bachelor degree project. Its main goal is to detect and correct contextual errors in natural Croatian text by leveraging historical n-gram frequency data. The analyzer uses statistical methods to suggest improvements where contextual errors are detected, thus enhancing the overall quality of written language.

## Features

- **Contextual Error Detection:** Identifies contextual mistakes in Croatian texts.
- **Statistical Correction:** Utilizes historical n-gram frequency data to propose corrections.
- **Academic Project:** Developed as a bachelor’s degree final project.
- **Rust Implementation:** Built with Rust for performance and reliability.

## Installation

Ensure you have [Rust](https://www.rust-lang.org/) and Cargo installed on your system, then follow the steps below:

Clone the repository

```bash 
git clone https://github.com/TeoMatosevic/ngram-context-analyzer.git
```

Navigate to the project directory

```bash
cd ngram-context-analyzer
```

Build the project in release mode

```bash
cargo build --release
```

## Usage

This application uses a ScyllaDB database to store n-gram data and thus it cannot be run locally because the data is not for public use and huge (> 2 billion n-grams). The application is designed to be run on a server with a ScyllaDB database containing the necessary data. The hardware this app runs on is a supercomputer provided by ![SUPEK](https://www.srce.unizg.hr/hr-zoo/napredno-racunanje).

## Technologies

- **Rust:** Core programming language used for implementation.
- **Cargo:** Rust’s package manager and build tool.
- **Statistical N-gram Analysis:** Methodology for assessing contextual language patterns.
- **ScyllaDB:** Database for storing n-gram frequency data.

## Configuration

The project utilizes several data files essential for its operation:
- `confusion_set.txt`: Contains mappings for commonly confused words or phrases.
- `number_of_distinct_n_grams.txt`: Provides the count of unique n-grams.
- `number_of_ngrams.txt`: Contains frequency statistics for n-grams.

Ensure these files are kept up-to-date with the latest statistical data to maintain the tool's accuracy.

## License

Distributed under the MIT License. See the `LICENSE` file for more information.

## Acknowledgements

- Heartfelt thanks to mentors, peers, and academic advisors for their guidance.
- Gratitude to the open-source community for providing the tools and inspiration for this project.
