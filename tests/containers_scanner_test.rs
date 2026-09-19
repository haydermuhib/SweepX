use sweepx::models::InstallMethod;
use sweepx::scanner::flatpak::{parse_flatpak_list_output, parse_flatpak_size_string};
use sweepx::scanner::snap::parse_snap_list_output;

#[test]
fn test_parse_flatpak_list() {
    let output = "Spotify\tcom.spotify.Client\t1.2.31\t245.5 MB\tsystem\t\n\
                  GIMP\torg.gimp.GIMP\t2.10.36\t1.2 GB\tuser\t\n\
                  VLC\torg.videolan.VLC\t3.0.20\t80.0 MB\tsystem\t";

    let apps = parse_flatpak_list_output(output);
    assert_eq!(apps.len(), 3);

    assert_eq!(apps[0].id, "com.spotify.Client");
    assert_eq!(apps[0].name, "Spotify");
    assert_eq!(apps[0].version.as_deref(), Some("1.2.31"));
    assert_eq!(apps[0].install_method, InstallMethod::Flatpak);
    assert!(apps[0].total_size_bytes > 200 * 1024 * 1024);

    assert_eq!(apps[1].id, "org.gimp.GIMP");
    assert_eq!(apps[1].version.as_deref(), Some("2.10.36"));
}

#[test]
fn test_parse_flatpak_size_string() {
    assert_eq!(parse_flatpak_size_string("500 B"), 500);
    assert_eq!(parse_flatpak_size_string("10 kB"), 10 * 1024);
    assert_eq!(parse_flatpak_size_string("100 MB"), 100 * 1024 * 1024);
    assert_eq!(
        parse_flatpak_size_string("1.5 GB"),
        (1.5 * 1024.0 * 1024.0 * 1024.0) as u64
    );
}

#[test]
fn test_parse_snap_list() {
    let output = "Name       Version     Rev    Tracking       Publisher     Notes\n\
                  core20     20231123    2105   latest/stable  canonical✓    base\n\
                  postman    10.24.8     249    latest/stable  postman-inc✓  -\n\
                  vlc        3.0.19      3721   latest/stable  videolan✓     -\n\
                  snapd      2.61.2      21184  latest/stable  canonical✓    snapd";

    let apps = parse_snap_list_output(output);
    // core20 and snapd should be filtered out
    assert_eq!(apps.len(), 2);

    assert_eq!(apps[0].id, "postman");
    assert_eq!(apps[0].version.as_deref(), Some("10.24.8"));
    assert_eq!(apps[0].install_method, InstallMethod::Snap);

    assert_eq!(apps[1].id, "vlc");
}
