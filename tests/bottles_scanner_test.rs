use sweepx::models::{ArtifactKind, InstallMethod};
use sweepx::scanner::bottles::{
    calculate_dir_size, find_bottle_program_dir, parse_bottle_invocation,
};
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;
use tempfile::tempdir;

#[test]
fn test_parse_bottle_invocation_patterns() {
    // Standard CLI invocation with double quotes
    let cmd1 = "bottles-cli run -b \"MathType\" -p \"MathType\"";
    let res1 = parse_bottle_invocation(cmd1);
    assert_eq!(res1, Some(("MathType".to_string(), "MathType".to_string())));

    // Flatpak wrapper invocation
    let cmd2 = "flatpak run --command=bottles-cli com.usebottles.bottles run -b \"Work\" -p \"mathtype\" \"$@\"";
    let res2 = parse_bottle_invocation(cmd2);
    assert_eq!(res2, Some(("Work".to_string(), "mathtype".to_string())));

    // Unquoted flag syntax
    let cmd3 = "bottles-cli run -b Gaming -p Steam";
    let res3 = parse_bottle_invocation(cmd3);
    assert_eq!(res3, Some(("Gaming".to_string(), "Steam".to_string())));

    // Equals-sign flag syntax
    let cmd4 = "bottles-cli run --bottle=Design --program=Photoshop";
    let res4 = parse_bottle_invocation(cmd4);
    assert_eq!(res4, Some(("Design".to_string(), "Photoshop".to_string())));

    // Direct bottles executable flag
    let cmd5 = "bottles -b \"Utilities\" -e \"C:\\Program Files\\MathType\\MathType.exe\"";
    let res5 = parse_bottle_invocation(cmd5);
    assert_eq!(res5, Some(("Utilities".to_string(), "MathType".to_string())));

    // Real Flatpak Bottles bash wrapper script (WINEPREFIX pattern)
    let bash_script = "#!/usr/bin/env bash\nexec flatpak run --command=bash com.usebottles.bottles -c '\n  WINEPREFIX=/var/data/bottles/bottles/MathType /app/bin/wine \"C:\\Program Files (x86)\\MathType\\MathType.exe\" \"$@\"\n' \"$@\"\n";
    let res_bash = parse_bottle_invocation(bash_script);
    assert_eq!(res_bash, Some(("MathType".to_string(), "MathType".to_string())));

    // Non-bottles invocation
    let cmd_none = "/usr/bin/google-chrome --enable-features=UseOzonePlatform";
    assert_eq!(parse_bottle_invocation(cmd_none), None);
}

#[test]
fn test_find_bottle_program_dir_and_size() {
    let temp = tempdir().unwrap();
    let bottle_dir = temp.path().join("MathType");
    let program_dir = bottle_dir.join("drive_c/Program Files (x86)/MathType");
    std::fs::create_dir_all(&program_dir).unwrap();

    let exe_file = program_dir.join("MathType.exe");
    std::fs::write(&exe_file, vec![0u8; 1024 * 512]).unwrap(); // 512 KB

    let font_file = program_dir.join("symbol.ttf");
    std::fs::write(&font_file, vec![0u8; 1024 * 256]).unwrap(); // 256 KB

    let found_prog = find_bottle_program_dir(&bottle_dir, "MathType");
    assert!(found_prog.is_some());
    assert_eq!(found_prog.unwrap(), program_dir);

    let total_size = calculate_dir_size(&program_dir);
    assert_eq!(total_size, 1024 * 768); // 768 KB
}

#[test]
fn test_desktop_entry_with_bottles_wrapper() {
    let temp = tempdir().unwrap();
    let wrapper_path = temp.path().join("mathtype-wrapper");
    let wrapper_content = "#!/bin/sh\nflatpak run --command=bottles-cli com.usebottles.bottles run -b \"MathType\" -p \"MathType\" \"$@\"\n";
    std::fs::write(&wrapper_path, wrapper_content).unwrap();

    let desktop_file = temp.path().join("mathtype.desktop");
    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=MathType 6.0\n\
         Exec=\"{exec}\" %U\n\
         Icon=mathtype\n\
         Comment=Equation Editor\n",
        exec = wrapper_path.display()
    );
    std::fs::write(&desktop_file, &desktop_content).unwrap();

    let app = parse_desktop_entry_content(&desktop_content, &desktop_file).expect("Failed to parse desktop entry");
    assert_eq!(app.install_method, InstallMethod::Bottles);
    assert_eq!(app.install_method.badge_label(), "Bottles");
    assert_eq!(app.name, "MathType 6.0");
    assert!(app.description.unwrap().contains("MathType"));
}

#[test]
fn test_real_bottles_bash_wrapper_integration() {
    let temp = tempdir().unwrap();
    let wrapper_path = temp.path().join("mathtype");
    let wrapper_content = "#!/usr/bin/env bash\nexec flatpak run --command=bash com.usebottles.bottles -c '\n  WINEPREFIX=/var/data/bottles/bottles/MathType /app/bin/wine \"C:\\Program Files (x86)\\MathType\\MathType.exe\" \"$@\"\n' \"$@\"\n";
    std::fs::write(&wrapper_path, wrapper_content).unwrap();

    let desktop_file = temp.path().join("mathtype.desktop");
    let desktop_content = format!(
        "[Desktop Entry]\n\
         Name=MathType 6.0\n\
         Comment=Equation Editor\n\
         Exec={}\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Office;Education;Science;Math;\n\
         Icon=accessories-calculator\n",
        wrapper_path.display()
    );
    std::fs::write(&desktop_file, &desktop_content).unwrap();

    let app = parse_desktop_entry_content(&desktop_content, &desktop_file).expect("Failed to parse desktop entry");
    assert_eq!(app.install_method, InstallMethod::Bottles);
    assert_eq!(app.install_method.badge_label(), "Bottles");
    assert_eq!(app.name, "MathType 6.0");
}

#[test]
fn test_artifact_kind_icon_display_name() {
    assert_eq!(ArtifactKind::Icon.display_name(), "Application Icon");
    assert_eq!(ArtifactKind::DesktopEntry.display_name(), "Desktop Launcher (.desktop)");
}
