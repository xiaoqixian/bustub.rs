//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// shell.rs
//
// Identification: src/shell.rs
//
// Copyright (c) 2015-2024, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::env;
use std::io::{self, BufRead, Write};

use rustyline::error::ReadlineError;
use rustyline::config::Configurer;
use rustyline::DefaultEditor;

use bustub::common::bustub_instance::BustubInstance;
use bustub::common::result_writer::TableWriter;
use bustub::concurrency::transaction::TransactionState;

/// Main entry point for the BusTub shell.
fn main() {
    // Create a BusTub instance with the default database file.
    let mut bustub = match BustubInstance::new("test.bustub") {
        Ok(x) => x,
        Err(e) => panic!("{}", e)
    };

    let default_prompt = "bustub> ";
    let emoji_prompt = "\u{1f6c1}> "; // bathtub emoji
    let mut use_emoji_prompt = false;
    let mut disable_tty = false;

    // Parse command line arguments.
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        if arg == "--emoji-prompt" {
            use_emoji_prompt = true;
        }
        if arg == "--disable-tty" {
            disable_tty = true;
        }
    }

    // Generate mock tables.
    if let Err(e) = bustub.create_mock_table() {
        eprintln!("Error creating mock tables: {}", e);
        return;
    }

    // TODO: generate_test_table() - not yet implemented in Rust.
    // The C++ version conditionally calls GenerateTestTable() when the buffer
    // pool manager is available:
    //   if (bustub->buffer_pool_manager_ != nullptr) {
    //       bustub->GenerateTestTable();
    //   }

    bustub.enable_managed_txn_mode();

    println!("Welcome to the BusTub shell! Type \\help to learn more.\n");

    let prompt = if use_emoji_prompt {
        emoji_prompt
    } else {
        default_prompt
    };

    if disable_tty {
        run_non_interactive(&mut bustub, prompt);
    } else {
        run_interactive(&mut bustub, prompt);
    }
}

/// Run the shell in interactive (TTY) mode using rustyline.
fn run_interactive(bustub: &mut BustubInstance, prompt: &str) {
    // Create a rustyline editor with default configuration.
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to create editor: {}", e);
            return;
        }
    };

    // Set the maximum history size to 1024 (matching the C++ linenoise config).
    if let Err(e) = rl.set_max_history_size(1024) {
        eprintln!("Warning: failed to set max history size: {}", e);
    }

    loop {
        let mut query = String::new();
        let mut first_line = true;

        loop {
            // Build a context-aware prompt based on the current transaction state.
            let context_prompt = match bustub.current_managed_txn() {
                Some(txn) => {
                    let state = txn.get_state();
                    let txn_id = txn.get_transaction_id_human_readable();
                    if state != TransactionState::Running {
                        format!("txn{} ({:?})> ", txn_id, state)
                    } else {
                        format!("txn{}> ", txn_id)
                    }
                }
                None => prompt.to_string(),
            };

            let line_prompt = if first_line { &context_prompt } else { "... " };

            match rl.readline(line_prompt) {
                Ok(line) => {
                    query.push_str(&line);
                    // A query is complete when it ends with ';' or starts with '\'.
                    if query.ends_with(';') || query.starts_with('\\') {
                        break;
                    }
                    query.push(' ');
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                    return;
                }
                Err(e) => {
                    eprintln!("Readline error: {}", e);
                    return;
                }
            }

            first_line = false;
        }

        // Add the completed query to the history.
        if let Err(e) = rl.add_history_entry(&query) {
            eprintln!("Warning: failed to add history entry: {}", e);
        }

        // Execute the SQL query and print the result table.
        let mut writer = TableWriter::new();
        match bustub.execute_sql(&query, &mut writer) {
            Ok(_) => {
                println!("{}", writer.to_string());
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

/// Run the shell in non-interactive (pipe/file) mode.
///
/// This mode uses standard I/O directly instead of rustyline, which is useful
/// for scripting and piping input.
fn run_non_interactive(bustub: &mut BustubInstance, prompt: &str) {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        let mut query = String::new();
        let mut first_line = true;

        loop {
            // Build a context-aware prompt based on the current transaction state.
            let context_prompt = match bustub.current_managed_txn() {
                Some(txn) => {
                    let state = txn.get_state();
                    let txn_id = txn.get_transaction_id_human_readable();
                    if state != TransactionState::Running {
                        format!("txn{} ({:?})> ", txn_id, state)
                    } else {
                        format!("txn{}> ", txn_id)
                    }
                }
                None => prompt.to_string(),
            };

            let line_prompt = if first_line { &context_prompt } else { "... " };

            print!("{}", line_prompt);
            if io::stdout().flush().is_err() {
                return;
            }

            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return,  // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    return;
                }
            }

            // Remove trailing newline/carriage return.
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
            query.push_str(trimmed);

            // A query is complete when it ends with ';' or starts with '\'.
            if query.ends_with(';') || query.starts_with('\\') {
                break;
            }
            query.push('\n');
            first_line = false;
        }

        // Execute the SQL query and print the result table.
        let mut writer = TableWriter::new();
        match bustub.execute_sql(&query, &mut writer) {
            Ok(_) => {
                println!("{}", writer.to_string());
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
