use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Parses `~/.boukensharc`, a small persistent config file that can set
/// `BOUKENSHA_PATH` and/or `BOUKENSHA_DIR` so you don't have to export
/// them in every shell session.
///
/// Format — one `KEY=VALUE` per line, `#` comments and blank lines
/// ignored:
///
///   BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop
///   BOUKENSHA_DIR=~/projects/mybot/.boukensha
///
/// Legacy format: a file containing just a bare path (no `=` on any
/// non-comment line) is treated as `BOUKENSHA_PATH`, e.g.:
///
///   echo ~/Sites/boukensha/07_the_repl_loop > ~/.boukensharc
///
/// Returns an empty map if the file doesn't exist or is empty.
pub fn read() -> HashMap<String, String> {
    let Some(path) = rc_path() else {
        return HashMap::new();
    };
    let Ok(contents) = fs::read_to_string(&path) else {
        return HashMap::new();
    };

    let lines: Vec<&str> = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    if lines.is_empty() {
        return HashMap::new();
    }

    if !lines.iter().any(|line| line.contains('=')) {
        // Legacy format: the whole file is a bare BOUKENSHA_PATH value.
        let mut config = HashMap::new();
        config.insert("BOUKENSHA_PATH".to_string(), lines.join(" ").trim().to_string());
        return config;
    }

    let mut config = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    config
}

fn rc_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".boukensharc"))
}
