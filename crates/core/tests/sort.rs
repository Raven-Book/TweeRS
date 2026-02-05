// Unit tests for path sorting utilities
use std::path::PathBuf;
use tweers_core::util::sort::compare_paths;

#[test]
fn test_debug_path_components() {
    // Debug test to understand how paths are parsed on different platforms
    let paths = vec![
        PathBuf::from("assets/css/inventory.css"),
        PathBuf::from("module/00-core/00-lib/combat-log.js"),
        PathBuf::from("all-spells.js"),
    ];

    for path in &paths {
        println!("\nPath: {:?}", path);
        println!("Display: {}", path.display());
        for (i, comp) in path.components().enumerate() {
            println!(
                "  Component[{}]: {:?} -> '{}'",
                i,
                comp,
                comp.as_os_str().to_string_lossy()
            );
        }
    }
}

#[test]
fn test_depth_first_sorting() {
    let mut paths = vec![
        PathBuf::from("story.twee"),
        PathBuf::from("assets/001-intro.twee"),
        PathBuf::from("assets/chapter1/scene1.twee"),
        PathBuf::from("module/helper.twee"),
        PathBuf::from("assets/002-main.twee"),
        PathBuf::from("assets/chapter1/scenes/intro.twee"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    // Expected order: DFS traversal with natural sort
    // assets/ -> assets/chapter1/ -> assets/chapter1/scenes/ -> files
    assert_eq!(
        paths,
        vec![
            PathBuf::from("assets/chapter1/scenes/intro.twee"),
            PathBuf::from("assets/chapter1/scene1.twee"),
            PathBuf::from("assets/001-intro.twee"),
            PathBuf::from("assets/002-main.twee"),
            PathBuf::from("module/helper.twee"),
            PathBuf::from("story.twee"),
        ]
    );
}

#[test]
fn test_natural_sorting_with_numbers() {
    let mut paths = vec![
        PathBuf::from("chapter10.twee"),
        PathBuf::from("chapter2.twee"),
        PathBuf::from("chapter1.twee"),
        PathBuf::from("chapter20.twee"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    // Natural sort: 1 < 2 < 10 < 20 (not lexicographic 1 < 10 < 2 < 20)
    assert_eq!(
        paths,
        vec![
            PathBuf::from("chapter1.twee"),
            PathBuf::from("chapter2.twee"),
            PathBuf::from("chapter10.twee"),
            PathBuf::from("chapter20.twee"),
        ]
    );
}

#[test]
fn test_natural_sorting_with_leading_zeros() {
    let mut paths = vec![
        PathBuf::from("010-end.twee"),
        PathBuf::from("002-middle.twee"),
        PathBuf::from("001-start.twee"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    assert_eq!(
        paths,
        vec![
            PathBuf::from("001-start.twee"),
            PathBuf::from("002-middle.twee"),
            PathBuf::from("010-end.twee"),
        ]
    );
}

#[test]
fn test_alphabetical_sorting_same_depth() {
    let mut paths = vec![
        PathBuf::from("module/helper.twee"),
        PathBuf::from("assets/intro.twee"),
        PathBuf::from("config/settings.twee"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    // Same depth (2 levels), alphabetical order
    assert_eq!(
        paths,
        vec![
            PathBuf::from("assets/intro.twee"),
            PathBuf::from("config/settings.twee"),
            PathBuf::from("module/helper.twee"),
        ]
    );
}

#[test]
fn test_mixed_depth_and_natural_sort() {
    let mut paths = vec![
        PathBuf::from("001-root.twee"),
        PathBuf::from("dir/010-file.twee"),
        PathBuf::from("002-root.twee"),
        PathBuf::from("dir/002-file.twee"),
        PathBuf::from("dir/subdir/001-deep.twee"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    // DFS: dir/ -> dir/subdir/ -> dir files -> root files
    assert_eq!(
        paths,
        vec![
            PathBuf::from("dir/subdir/001-deep.twee"),
            PathBuf::from("dir/002-file.twee"),
            PathBuf::from("dir/010-file.twee"),
            PathBuf::from("001-root.twee"),
            PathBuf::from("002-root.twee"),
        ]
    );
}

#[test]
fn test_cross_platform_paths() {
    // Test that sorting works consistently across platforms
    // On Unix: uses forward slashes
    // On Windows: uses backslashes
    #[cfg(unix)]
    let mut paths = vec![
        PathBuf::from("assets/css/inventory.css"),
        PathBuf::from("assets/css/task.css"),
        PathBuf::from("assets/js/00-macro-utils.js"),
        PathBuf::from("assets/js/01-seed.js"),
        PathBuf::from("module/00-core/00-lib/combat-log.js"),
        PathBuf::from("module/01-ui/01-ui.js"),
        PathBuf::from("all-spells.js"),
        PathBuf::from("index.js"),
    ];

    #[cfg(windows)]
    let mut paths = vec![
        PathBuf::from(r"assets\css\inventory.css"),
        PathBuf::from(r"assets\css\task.css"),
        PathBuf::from(r"assets\js\00-macro-utils.js"),
        PathBuf::from(r"assets\js\01-seed.js"),
        PathBuf::from(r"module\00-core\00-lib\combat-log.js"),
        PathBuf::from(r"module\01-ui\01-ui.js"),
        PathBuf::from("all-spells.js"),
        PathBuf::from("index.js"),
    ];

    paths.sort_by(|a, b| compare_paths(a, b));

    // Expected: DFS with natural sort (same order on both platforms)
    #[cfg(unix)]
    let expected = vec![
        PathBuf::from("assets/css/inventory.css"),
        PathBuf::from("assets/css/task.css"),
        PathBuf::from("assets/js/00-macro-utils.js"),
        PathBuf::from("assets/js/01-seed.js"),
        PathBuf::from("module/00-core/00-lib/combat-log.js"),
        PathBuf::from("module/01-ui/01-ui.js"),
        PathBuf::from("all-spells.js"),
        PathBuf::from("index.js"),
    ];

    #[cfg(windows)]
    let expected = vec![
        PathBuf::from(r"assets\css\inventory.css"),
        PathBuf::from(r"assets\css\task.css"),
        PathBuf::from(r"assets\js\00-macro-utils.js"),
        PathBuf::from(r"assets\js\01-seed.js"),
        PathBuf::from(r"module\00-core\00-lib\combat-log.js"),
        PathBuf::from(r"module\01-ui\01-ui.js"),
        PathBuf::from("all-spells.js"),
        PathBuf::from("index.js"),
    ];

    assert_eq!(paths, expected);
}
