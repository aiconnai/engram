//! Demo do OutputFilter inspirado no RTK
//!
//! Este binário demonstra como o OutputFilter reduz tokens para LLMs
//! executando filtros em saídas típicas de comandos Bash.

use engram::intelligence::output_filter::OutputFilter;

fn main() {
    let filter = OutputFilter::new();

    // Exemplo 1: ls -la (45 linhas -> ~150 tokens)
    let ls_output = "drwxr-xr-x  15 user staff 480 Jan 1 10:00 src\n\
                     -rw-r--r--   1 user staff 123 Jan 1 10:00 Cargo.toml\n\
                     -rw-r--r--   1 user staff 456 Jan 1 10:00 README.md\n\
                     drwxr-xr-x   5 user staff 160 Jan 1 10:00 tests\n\
                     -rw-r--r--   1 user staff 789 Jan 1 10:00 .gitignore\n";
    let filtered = filter.filter("ls -la", ls_output);
    println!("LS original: {} chars", ls_output.len());
    println!("LS filtrado: {} chars\n", filtered.len());
    println!("Conteúdo filtrado:\n{}\n", filtered);

    // Exemplo 2: git status (15 linhas -> ~10 tokens)
    let git_output = "On branch main\n\
                      Changes not staged for commit:\n\
                        modified:   src/main.rs\n\
                        deleted:    old_file.rs\n\
                      Untracked files:\n\
                        new_file.rs\n";
    let filtered = filter.filter("git status", git_output);
    println!("Git original: {} chars", git_output.len());
    println!("Git filtrado: {} chars\n", filtered.len());
    println!("Conteúdo filtrado:\n{}\n", filtered);

    // Exemplo 3: cargo test (200+ linhas -> ~20 linhas)
    let test_output = "running 15 tests\n\
                       test test_one ... ok\n\
                       test test_two ... FAILED\n\
                       test test_three ... ok\n\
                       \n\
                       failures:\n\
                       \n\
                       ---- test_two stdout ----\n\
                       thread 'test_two' panicked at 'assertion failed'\n\
                       \n\
                       \n\
                       failures:\n\
                       test_two\n\
                       \n\
                       FAILED\n";
    let filtered = filter.filter("cargo test", test_output);
    println!("Test original: {} chars", test_output.len());
    println!("Test filtrado: {} chars\n", filtered.len());
    println!("Conteúdo filtrado:\n{}\n", filtered);
}
