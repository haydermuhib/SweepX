use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Information discovered about an application installed inside a Bottles Wine environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottleAppInfo {
    pub bottle_name: String,
    pub program_name: String,
    pub bottle_dir: Option<PathBuf>,
    pub program_dir: Option<PathBuf>,
    pub payload_size_bytes: u64,
}

/// Known root directories where Bottles stores bottles (Wine prefixes).
pub fn get_bottles_root_directories() -> Vec<PathBuf> {
    let mut dirs_list = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Flatpak Bottles (standard location)
        dirs_list.push(home.join(".var/app/com.usebottles.bottles/data/bottles/bottles"));
        // Flatpak Bottles (direct data directory fallback)
        dirs_list.push(home.join(".var/app/com.usebottles.bottles/data/bottles"));
        // Native Bottles (XDG data location)
        dirs_list.push(home.join(".local/share/bottles/bottles"));
        dirs_list.push(home.join(".local/share/bottles"));
    }

    dirs_list
}

/// Splits a command line string into tokens respecting single and double quotes.
fn tokenize_command(cmd: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for ch in cmd.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if in_quote && c == quote_char => {
                in_quote = false;
            }
            ' ' | '\t' | '\n' | '\r' if !in_quote => {
                if !current.is_empty() {
                    tokens.push(current);
                    current = String::new();
                }
            }
            c => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Attempts to parse the bottle name and program name from command strings or wrapper script contents.
///
/// Handles patterns like:
/// - `bottles-cli run -b "BottleName" -p "ProgramName"`
/// - `WINEPREFIX=/var/data/bottles/bottles/MathType /app/bin/wine "C:\Program Files\..."`
/// - `flatpak run --command=bash com.usebottles.bottles -c 'WINEPREFIX=...'`
/// - `bottles -b "BottleName" -e "..."`
pub fn parse_bottle_invocation(content: &str) -> Option<(String, String)> {
    if !content.contains("bottles") && !content.contains("com.usebottles.bottles") && !content.contains("WINEPREFIX") {
        return None;
    }

    let mut bottle_name = None;
    let mut program_name = None;

    // 1. Check for WINEPREFIX=... pattern common in flatpak bash wrapper scripts:
    // e.g. WINEPREFIX=/var/data/bottles/bottles/MathType /app/bin/wine "C:\Program Files (x86)\MathType\MathType.exe"
    if let Some(pos) = content.find("WINEPREFIX=") {
        let remainder = &content[pos + "WINEPREFIX=".len()..];
        let prefix_token = remainder
            .split(|c: char| c.is_whitespace() || c == '\'' || c == '"')
            .find(|s| !s.is_empty())
            .unwrap_or("");

        if !prefix_token.is_empty() {
            let b_name = prefix_token
                .trim_end_matches('/')
                .split('/')
                .last()
                .unwrap_or(prefix_token);
            if !b_name.is_empty() {
                bottle_name = Some(b_name.to_string());
            }
        }
    }

    // 2. Check for .exe executable path if program_name is not yet known
    if let Some(exe_pos) = content.to_lowercase().find(".exe") {
        let before = &content[..exe_pos];
        let start = before
            .rfind(|c: char| c == '"' || c == '\'' || c.is_whitespace())
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let exe_token = &content[start..exe_pos + 4];
        let clean_exe = clean_arg_token(exe_token);
        let filename = clean_exe.split(['/', '\\']).last().unwrap_or(&clean_exe);
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);
        if !stem.is_empty() {
            program_name = Some(stem.to_string());
        }
    }

    // 3. Fallback to token-based flags (-b, --bottle, -p, --program, -e, --executable)
    let tokens = tokenize_command(content);
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        if token == "-b" || token == "--bottle" {
            if i + 1 < tokens.len() {
                bottle_name = Some(clean_arg_token(&tokens[i + 1]));
                i += 1;
            }
        } else if token.starts_with("-b=") || token.starts_with("--bottle=") {
            if let Some((_, val)) = token.split_once('=') {
                bottle_name = Some(clean_arg_token(val));
            }
        } else if token == "-p" || token == "--program" {
            if i + 1 < tokens.len() {
                program_name = Some(clean_arg_token(&tokens[i + 1]));
                i += 1;
            }
        } else if token.starts_with("-p=") || token.starts_with("--program=") {
            if let Some((_, val)) = token.split_once('=') {
                program_name = Some(clean_arg_token(val));
            }
        } else if token == "-e" || token == "--executable" {
            if i + 1 < tokens.len() {
                let exec_val = clean_arg_token(&tokens[i + 1]);
                let filename = exec_val.split(['/', '\\']).last().unwrap_or(&exec_val);
                let stem = Path::new(filename)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(filename);
                program_name = Some(stem.to_string());
                i += 1;
            }
        }
        i += 1;
    }

    match (bottle_name, program_name) {
        (Some(b), Some(p)) if !b.is_empty() && !p.is_empty() => Some((b, p)),
        (Some(b), None) if !b.is_empty() => Some((b.clone(), b)),
        _ => None,
    }
}

fn clean_arg_token(token: &str) -> String {
    token
        .trim_matches(|c| c == '\'' || c == '"' || c == ';' || c == ',')
        .to_string()
}

/// Locates the bottle directory for a given bottle name.
pub fn find_bottle_directory(bottle_name: &str) -> Option<PathBuf> {
    let roots = get_bottles_root_directories();
    let target_clean = bottle_name.to_lowercase();

    for root in roots {
        if !root.exists() {
            continue;
        }

        // Direct exact match
        let direct = root.join(bottle_name);
        if direct.is_dir() && direct.join("drive_c").exists() {
            return Some(direct);
        }

        // Case-insensitive match among children
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let file_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if file_name == target_clean && path.join("drive_c").exists() {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Finds the specific program directory inside `drive_c` of a bottle.
pub fn find_bottle_program_dir(bottle_dir: &Path, program_name: &str) -> Option<PathBuf> {
    let drive_c = bottle_dir.join("drive_c");
    if !drive_c.exists() {
        return None;
    }

    let program_clean = program_name.to_lowercase().replace(' ', "");

    // Search common Windows program locations in drive_c
    let search_roots = [
        drive_c.join("Program Files"),
        drive_c.join("Program Files (x86)"),
        drive_c.join("users"),
        drive_c.clone(),
    ];

    for root in &search_roots {
        if !root.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_lowercase().replace(' ', ""))
                        .unwrap_or_default();

                    if dir_name.contains(&program_clean) || program_clean.contains(&dir_name) {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// Calculates the recursive disk usage of a directory in bytes.
pub fn calculate_dir_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Probes a given desktop Exec or binary path to determine if it is a Bottles application.
/// If verified, returns the full `BottleAppInfo` with resolved bottle and payload size.
pub fn probe_bottle_app(
    raw_exec: Option<&str>,
    resolved_bin: Option<&Path>,
) -> Option<BottleAppInfo> {
    let mut invocation = None;

    // 1. Try parsing from raw Exec line
    if let Some(exec_str) = raw_exec {
        invocation = parse_bottle_invocation(exec_str);
    }

    // 2. If not found in Exec line, check if the resolved binary is a shell script wrapper
    if invocation.is_none() {
        if let Some(bin_path) = resolved_bin {
            if bin_path.exists() {
                if let Ok(content) = std::fs::read_to_string(bin_path) {
                    invocation = parse_bottle_invocation(&content);
                }
            }
        }
    }

    let (bottle_name, program_name) = invocation?;
    let bottle_dir = find_bottle_directory(&bottle_name);

    let program_dir = bottle_dir
        .as_ref()
        .and_then(|dir| find_bottle_program_dir(dir, &program_name));

    let payload_size_bytes = match (&program_dir, &bottle_dir) {
        (Some(p_dir), _) => calculate_dir_size(p_dir),
        (None, Some(b_dir)) => calculate_dir_size(&b_dir.join("drive_c")),
        _ => 0,
    };

    Some(BottleAppInfo {
        bottle_name,
        program_name,
        bottle_dir,
        program_dir,
        payload_size_bytes,
    })
}
