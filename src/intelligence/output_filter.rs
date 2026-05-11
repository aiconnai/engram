use std::collections::HashMap;

/// Filtro de saída de comandos para redução de tokens em LLMs
/// Inspirado no RTK (<https://github.com/rtk-ai/rtk>)
pub struct OutputFilter {
    max_chars: usize,
}

impl Default for OutputFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFilter {
    /// Cria um novo OutputFilter com configurações padrão
    pub fn new() -> Self {
        Self { max_chars: 5000 }
    }

    /// Filtra saída de comando Bash antes de enviar para LLM
    pub fn filter(&self, command: &str, output: &str) -> String {
        if output.len() <= self.max_chars {
            return output.to_string();
        }

        let filtered = self.apply_filter(command, output);

        if filtered.len() < output.len() * 3 / 4 {
            format!(
                "[engram-filtered: {} chars -> {} chars]\n{}",
                output.len(),
                filtered.len(),
                filtered
            )
        } else {
            output.to_string()
        }
    }

    fn apply_filter(&self, command: &str, output: &str) -> String {
        if command.starts_with("ls") || command.starts_with("tree") {
            self.filter_listing(output)
        } else if command.starts_with("cat") || command.starts_with("read") {
            self.filter_file_content(output)
        } else if command.starts_with("git") {
            self.filter_git_output(command, output)
        } else if command.contains("grep") || command.contains("rg") {
            self.filter_search_output(output)
        } else if command.starts_with("cargo") {
            self.filter_build_output(output)
        } else if command.starts_with("npm") || command.starts_with("pnpm") {
            self.filter_npm_output(output)
        } else if command.contains("test") {
            self.filter_test_output(output)
        } else {
            self.filter_generic(output)
        }
    }

    /// Mantém apenas nomes de arquivos, agrupa por diretório
    /// Redução: ~80%
    fn filter_listing(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        if lines.len() <= 20 {
            return output.to_string();
        }

        let mut dirs: HashMap<&str, Vec<&str>> = HashMap::new();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(pos) = trimmed.rfind('/') {
                let dir = &trimmed[..pos];
                let file = &trimmed[pos + 1..];
                dirs.entry(dir).or_default().push(file);
            } else {
                dirs.entry(".").or_default().push(trimmed);
            }
        }

        let mut result = Vec::new();
        for (dir, files) in dirs {
            result.push(format!("{} ({} files)", dir, files.len()));
        }
        result.join(", ")
    }

    /// Mantém apenas assinaturas de funções, ignora corpos
    /// Redução: ~70%
    fn filter_file_content(&self, output: &str) -> String {
        if output.len() <= 3000 {
            return output.to_string();
        }

        let mut declarations = Vec::new();
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("impl ")
            {
                let sig = trimmed.split('{').next().unwrap_or(trimmed);
                declarations.push(format!("{} {{...}}", sig));
            }
        }
        if declarations.is_empty() {
            output.to_string()
        } else {
            declarations.join("\n")
        }
    }

    /// Filtra saída do git conforme o subcomando
    /// Redução: ~80%
    fn filter_git_output(&self, cmd: &str, output: &str) -> String {
        if cmd.contains("status") {
            let mut changes = Vec::new();
            for line in output.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('M') || trimmed.starts_with('A') || trimmed.starts_with('D')
                {
                    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        changes.push(format!("{} {}", parts[0], parts[1]));
                    }
                }
            }
            changes.join(", ")
        } else if cmd.contains("log") {
            output
                .lines()
                .filter(|l| l.contains("commit ") || l.contains("Author:") || l.contains("Date:"))
                .take(5)
                .collect::<Vec<&str>>()
                .join("\n")
        } else if cmd.contains("diff") {
            output
                .lines()
                .filter(|l| l.starts_with("diff --git") || l.contains("@@"))
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            output.to_string()
        }
    }

    /// Agrupa resultados de busca por arquivo
    /// Redução: ~85%
    fn filter_search_output(&self, output: &str) -> String {
        let mut file_matches: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, line) in output.lines().enumerate() {
            if let Some(pos) = line.find(':') {
                let file = &line[..pos];
                file_matches.entry(file).or_default().push(i + 1);
            }
        }

        let mut result = Vec::new();
        for (file, lines) in file_matches {
            result.push(format!(
                "{}: {} matches (lines {})",
                file,
                lines.len(),
                lines
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if result.is_empty() {
            output.to_string()
        } else {
            result.join("\n")
        }
    }

    /// Mantém apenas erros e warnings
    /// Redução: ~90%
    fn filter_build_output(&self, output: &str) -> String {
        output
            .lines()
            .filter(|l| l.contains("error") || l.contains("warning"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Mantém apenas testes que falharam
    /// Redução: ~90%
    fn filter_test_output(&self, output: &str) -> String {
        if !output.contains("FAILED") {
            return output.to_string();
        }

        output
            .lines()
            .filter(|l| l.contains("FAILED") || l.contains("failures"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Mantém resumo de pacotes npm
    fn filter_npm_output(&self, output: &str) -> String {
        output
            .lines()
            .filter(|l| l.contains("dependencies") || l.contains("updated") || l.contains("added"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// Remove linhas vazias e espaços extras
    /// Redução: ~30%
    fn filter_generic(&self, output: &str) -> String {
        output
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim())
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_short_output() {
        let filter = OutputFilter::new();
        let output = "short output";
        assert_eq!(filter.filter("ls", output), output);
    }

    #[test]
    fn test_filter_listing() {
        let filter = OutputFilter::new();
        // Create output with more than 5000 chars to pass the max_chars check
        let output = (0..1000)
            .map(|i| format!("file{}.rs with some padding", i))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(output.len() > 5000, "Output should exceed max_chars");
        let filtered = filter.filter("ls -la", &output);
        // Should contain summary format
        assert!(
            filtered.contains("files)"),
            "Should contain file count summary"
        );
    }
    #[test]
    fn test_filter_git_status() {
        let filter = OutputFilter::new();
        let output = "On branch main\nChanges not staged:\n  M file.rs\n  A new_file.rs\n";
        let filtered = filter.filter("git status", output);
        assert!(filtered.contains("M file.rs"));
        assert!(filtered.contains("A new_file.rs"));
    }
}
