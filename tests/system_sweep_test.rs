use sweepx::cleaner::system_sweep::{parse_snap_disabled_revisions, SweepCategory};

#[test]
fn test_parse_snap_disabled_revisions() {
    let output = "Name                          Version                         Rev    Tracking       Publisher   Notes\n\
                  bare                          1.0                             5      latest/stable  canonical✓  base\n\
                  core24                        20260410                        1643   latest/stable  canonical✓  base,disabled\n\
                  core24                        20260824                        2124   latest/stable  canonical✓  base\n\
                  snapd                         2.76.2                          27710  latest/stable  canonical✓  snapd,disabled\n\
                  snapd                         2.76.3                          27738  latest/stable  canonical✓  snapd\n\
                  notepad-plus-plus             8.9.6.4                         444    latest/stable  mmtrt       -";

    let items = parse_snap_disabled_revisions(output);
    assert_eq!(items.len(), 2);

    assert_eq!(items[0].id, "snap-rev-core24-1643");
    assert_eq!(items[0].category, SweepCategory::SnapDisabledRevisions);
    assert!(items[0].command.as_ref().unwrap().contains("snap remove core24 --revision=1643"));

    assert_eq!(items[1].id, "snap-rev-snapd-27710");
    assert_eq!(items[1].category, SweepCategory::SnapDisabledRevisions);
    assert!(items[1].command.as_ref().unwrap().contains("snap remove snapd --revision=27710"));
}
