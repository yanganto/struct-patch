use struct_patch::Patch;

fn concat_list(original: &mut Vec<i32>, additional: Vec<i32>) {
    original.extend(additional);
}

fn merge_tags(original: &mut String, additional: String) {
    for c in additional.chars() {
        if !original.contains(c) {
            original.push(c);
        }
    }
}

#[derive(Debug, Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Config {
    #[patch(apply_by(concat_list))]
    items: Vec<i32>,
    name: String,
    // skip_wrap keeps the patch field as `String` (no Option wrapping).
    // merge_tags is always called unconditionally on every apply.
    #[patch(skip_wrap, apply_by(merge_tags))]
    tags: String,
}

fn main() {
    let mut config = Config {
        items: vec![1, 2, 3],
        name: "base".to_string(),
        tags: "a".to_string(),
    };

    // Patch with apply_by: items are concatenated instead of replaced.
    // tags patch field is plain String (not Option<String>) due to skip_wrap.
    config.apply(ConfigPatch {
        items: Some(vec![4, 5, 6]),
        name: None,
        tags: "ab".to_string(),
    });
    assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(config.tags, "ab");
    println!("After first patch: {:?}", config.items);
    println!("Tags after merge: {:?}", config.tags);

    // A second patch appends more items; tags is always applied.
    config.apply(ConfigPatch {
        items: Some(vec![7, 8]),
        name: Some("updated".to_string()),
        tags: "c".to_string(),
    });
    assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(config.tags, "abc");
    println!("After second patch: {:?}", config.items);
    println!("Tags after second merge: {:?}", config.tags);
    println!("Name: {}", config.name);
}
