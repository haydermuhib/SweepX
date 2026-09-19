use std::collections::HashSet;
use sweepx::models::InstallMethod;
use sweepx::scanner::native::apt::AptManager;
use sweepx::scanner::native::dnf::DnfManager;
use sweepx::scanner::native::pacman::PacmanManager;

#[test]
fn test_parse_dpkg_query_output() {
    let output = "curl\t7.88.1-10+deb12u5\t450\tweb\tCommand line tool for transferring data with URLs\n\
                  libc6\t2.36-9+deb12u4\t12450\tlibs\tGNU C Library: Shared libraries\n\
                  git\t1:2.39.2-1.1\t38500\tvcs\tfast, scalable, distributed revision control system";

    let mut manual = HashSet::new();
    manual.insert("curl".to_string());
    manual.insert("git".to_string());

    let apps = AptManager::parse_dpkg_query_output(output, &manual);
    assert_eq!(apps.len(), 3);

    assert_eq!(apps[0].id, "curl");
    assert_eq!(apps[0].install_method, InstallMethod::NativeApt);
    assert_eq!(apps[0].total_size_bytes, 450 * 1024);
    assert!(!apps[0].is_system);

    assert_eq!(apps[1].id, "libc6");
    assert!(apps[1].is_system);

    assert_eq!(apps[2].id, "git");
    assert_eq!(apps[2].version.as_deref(), Some("1:2.39.2-1.1"));
    assert!(!apps[2].is_system);
}

#[test]
fn test_parse_rpm_query_output() {
    let output = "ripgrep\t14.1.0-1.fc40\t6423980\tDevelopment/Tools\tLine-oriented search tool\n\
                  glibc\t2.39-13.fc40\t29840210\tSystem Environment/Libraries\tThe GNU libc libraries\n\
                  htop\t3.3.0-2.fc40\t390234\tApplications/System\tInteractive process viewer";

    let mut user_installed = HashSet::new();
    user_installed.insert("ripgrep".to_string());
    user_installed.insert("htop".to_string());

    let apps = DnfManager::parse_rpm_query_output(output, &user_installed);
    assert_eq!(apps.len(), 3);

    assert_eq!(apps[0].id, "ripgrep");
    assert_eq!(apps[0].install_method, InstallMethod::NativeDnf);
    assert_eq!(apps[0].total_size_bytes, 6423980);
    assert!(!apps[0].is_system);

    assert_eq!(apps[1].id, "glibc");
    assert!(apps[1].is_system);

    assert_eq!(apps[2].id, "htop");
    assert_eq!(apps[2].version.as_deref(), Some("3.3.0-2.fc40"));
    assert!(!apps[2].is_system);
}

#[test]
fn test_parse_pacman_qi_output() {
    let output = "Name            : neovim\n\
                  Version         : 0.9.5-1\n\
                  Description     : Vim-fork focused on extensibility and usability\n\
                  Installed Size  : 23.45 MiB\n\
                  Install Reason  : Explicitly installed\n\
                  \n\
                  Name            : glibc\n\
                  Version         : 2.39-1\n\
                  Description     : GNU C Library\n\
                  Installed Size  : 45.20 MiB\n\
                  Install Reason  : Installed as a dependency for another package\n";

    let apps = PacmanManager::parse_pacman_qi_output(output);
    assert_eq!(apps.len(), 2);

    assert_eq!(apps[0].id, "neovim");
    assert_eq!(apps[0].install_method, InstallMethod::NativePacman);
    assert_eq!(apps[0].version.as_deref(), Some("0.9.5-1"));
    assert!(!apps[0].is_system);

    assert_eq!(apps[1].id, "glibc");
    assert!(apps[1].is_system);
}
