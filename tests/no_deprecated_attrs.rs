use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    for entry in fs::read_dir(root).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn source_tree_has_no_deprecated_attributes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let offenders: Vec<String> = files
        .into_iter()
        .filter_map(|path| {
            let contents = fs::read_to_string(&path).expect("read_to_string failed");
            if contents.contains("#[deprecated") {
                Some(
                    path.strip_prefix(&manifest_dir)
                        .unwrap_or(&path)
                        .display()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "unexpected deprecated attributes in source tree: {offenders:?}"
    );
}
