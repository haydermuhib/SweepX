use std::path::Path;
use sweepx::models::InstallMethod;
use sweepx::scanner::desktop_entry::{
    extract_binary_from_exec, parse_desktop_entry_content, resolve_binary_path,
};

#[test]
fn test_parse_desktop_entry_basic() {
    let desktop_content = r#"
[Desktop Entry]
Version=1.0
Type=Application
Name=Visual Studio Code
GenericName=Text Editor
Comment=Code Editing. Redefined.
Exec=/usr/share/code/code --unity-launch %F
Icon=vscode
Categories=Utility;Development;IDE;
MimeType=text/plain;inode/directory;
Actions=new-empty-window;
Keywords=vscode;
"#;

    let path = Path::new("/usr/share/applications/code.desktop");
    let app = parse_desktop_entry_content(desktop_content, path).expect("Should parse desktop entry");

    assert_eq!(app.id, "code");
    assert_eq!(app.name, "Visual Studio Code");
    assert_eq!(app.display_name, "Visual Studio Code (Text Editor)");
    assert_eq!(app.icon.as_deref(), Some("vscode"));
    assert_eq!(app.description.as_deref(), Some("Code Editing. Redefined."));
}

#[test]
fn test_parse_desktop_entry_flatpak_and_opt() {
    let flatpak_desktop = r#"
[Desktop Entry]
Type=Application
Name=Spotify
Exec=/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=spotify com.spotify.Client @@u %U @@
Icon=com.spotify.Client
"#;
    let path = Path::new("/var/lib/flatpak/exports/share/applications/com.spotify.Client.desktop");
    let app = parse_desktop_entry_content(flatpak_desktop, path).expect("Should parse");
    assert_eq!(app.install_method, InstallMethod::Flatpak);
    assert_eq!(app.id, "com.spotify.Client");

    let opt_desktop = r#"
[Desktop Entry]
Type=Application
Name=Postman
Exec=/opt/Postman/Postman %U
Icon=/opt/Postman/app/resources/app/assets/icon.png
"#;
    let path = Path::new("/usr/share/applications/postman.desktop");
    let app = parse_desktop_entry_content(opt_desktop, path).expect("Should parse");
    assert_eq!(app.install_method, InstallMethod::ManualOpt);
}

#[test]
fn test_extract_binary_from_exec() {
    assert_eq!(
        extract_binary_from_exec("/usr/bin/google-chrome-stable %U"),
        "/usr/bin/google-chrome-stable"
    );
    assert_eq!(
        extract_binary_from_exec("\"/opt/My App/bin/launcher\" --flag arg"),
        "/opt/My App/bin/launcher"
    );
    assert_eq!(
        extract_binary_from_exec("flatpak run com.obsproject.Studio"),
        "flatpak"
    );
}

#[test]
fn test_resolve_binary_path() {
    // Should resolve standard binaries like sh or bash in PATH
    assert!(resolve_binary_path("sh").is_some());
    assert!(resolve_binary_path("/bin/sh").is_some() || resolve_binary_path("/usr/bin/sh").is_some());
}
