//! Natural sorting utilities for paths and strings

use std::cmp::Ordering;
use std::path::Path;

/// Sort paths using depth-first directory traversal with natural ordering
///
/// This function compares paths like a file manager with "folders first":
/// 1. Compare directory paths component by component using natural sort
/// 2. Subdirectory contents come before files in the parent directory
/// 3. Files in the same directory are sorted naturally
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
///
/// let mut paths = vec![
///     PathBuf::from("story.twee"),
///     PathBuf::from("assets/001-intro.twee"),
///     PathBuf::from("assets/chapter1/scene1.twee"),
///     PathBuf::from("module/helper.twee"),
/// ];
///
/// paths.sort_by(|a, b| tweers_core::util::sort::compare_paths(a, b));
///
/// // Result (depth-first, natural order):
/// // assets/chapter1/scene1.twee
/// // assets/001-intro.twee
/// // module/helper.twee
/// // story.twee
/// ```
pub fn compare_paths<P: AsRef<Path>>(a: P, b: P) -> Ordering {
    let a_str = a.as_ref().to_string_lossy();
    let b_str = b.as_ref().to_string_lossy();

    // Normalize backslashes to forward slashes for WASM compatibility
    let a_normalized = a_str.replace('\\', "/");
    let b_normalized = b_str.replace('\\', "/");

    let a_path = Path::new(&a_normalized);
    let b_path = Path::new(&b_normalized);

    let a_components: Vec<_> = a_path.components().collect();
    let b_components: Vec<_> = b_path.components().collect();

    let a_len = a_components.len();
    let b_len = b_components.len();

    // Compare component by component
    for i in 0..a_len.min(b_len) {
        let a_comp = a_components[i].as_os_str().to_string_lossy();
        let b_comp = b_components[i].as_os_str().to_string_lossy();

        let is_a_last = i == a_len - 1;
        let is_b_last = i == b_len - 1;

        // If components are equal, check depth difference
        if a_comp == b_comp {
            // Same component, continue to next
            continue;
        }

        // Components differ - check if one is a file and other has subdirectory
        if is_a_last && !is_b_last {
            return Ordering::Greater; // b has subdirectory, b comes first
        }
        if !is_a_last && is_b_last {
            return Ordering::Less; // a has subdirectory, a comes first
        }

        // Both are directories or both are files at this level
        return natord::compare(&a_comp, &b_comp);
    }

    // Paths are equal up to the shorter one's length
    a_len.cmp(&b_len)
}
