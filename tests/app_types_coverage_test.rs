use std::path::PathBuf;
use sweepx::models::InstallMethod;
use sweepx::scanner::desktop_entry::parse_desktop_entry_content;

#[test]
fn test_flatpak_app_desktop_parsing() {
    let desktop_content = r#"[Desktop Entry]
Name=Brave Web Browser
Exec=/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=brave com.brave.Browser
Icon=com.brave.Browser
Type=Application
"#;
    let desktop_file = PathBuf::from("/var/lib/flatpak/exports/share/applications/com.brave.Browser.desktop");
    let app = parse_desktop_entry_content(desktop_content, &desktop_file).expect("Must parse Flatpak desktop entry");

    assert_eq!(app.id, "com.brave.Browser");
    assert_eq!(app.install_method, InstallMethod::Flatpak);
    assert_eq!(app.display_name, "Brave Web Browser");
}

#[test]
fn test_bottles_wine_app_desktop_parsing() {
    let desktop_content = r#"[Desktop Entry]
Name=EA Desktop
Comment=Origin Replacement Client
Exec=flatpak run --command=bottles-cli com.usebottles.bottles run -b 'Gaming' -p 'EADesktop' -- %u
Type=Application
"#;
    let desktop_file = PathBuf::from("/home/user/.local/share/applications/ea-desktop.desktop");
    let app = parse_desktop_entry_content(desktop_content, &desktop_file).expect("Must parse Bottles desktop entry");

    assert_eq!(app.install_method, InstallMethod::Bottles);
    assert!(app.description.as_ref().unwrap().contains("Gaming"));
}

#[test]
fn test_appimage_desktop_parsing() {
    let desktop_content = r#"[Desktop Entry]
Name=Cursor IDE
Exec=/home/user/Applications/Cursor.AppImage --no-sandbox
Type=Application
"#;
    let desktop_file = PathBuf::from("/home/user/.local/share/applications/cursor.desktop");
    let app = parse_desktop_entry_content(desktop_content, &desktop_file).expect("Must parse AppImage desktop entry");

    assert_eq!(app.install_method, InstallMethod::AppImage);
    assert_eq!(app.display_name, "Cursor IDE");
}

#[test]
fn test_manual_opt_app_desktop_parsing() {
    let desktop_content = r#"[Desktop Entry]
Name=Sublime Text
Exec=/opt/sublime_text/sublime_text %F
Type=Application
"#;
    let desktop_file = PathBuf::from("/usr/share/applications/sublime_text.desktop");
    let app = parse_desktop_entry_content(desktop_content, &desktop_file).expect("Must parse Manual Opt desktop entry");

    assert_eq!(app.install_method, InstallMethod::ManualOpt);
    assert_eq!(app.name, "Sublime Text");
}

#[test]
fn test_all_install_methods_categorization_and_labels() {
    let methods = vec![
        InstallMethod::Flatpak,
        InstallMethod::Snap,
        InstallMethod::Bottles,
        InstallMethod::AppImage,
        InstallMethod::NativeDnf,
        InstallMethod::NativeApt,
        InstallMethod::NativePacman,
        InstallMethod::ManualOpt,
        InstallMethod::CustomDesktop,
    ];

    for m in methods {
        assert!(!m.badge_label().is_empty(), "Badge label must not be empty for {:?}", m);
    }
}
