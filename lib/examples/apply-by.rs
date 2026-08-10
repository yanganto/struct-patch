use struct_patch::Patch;

fn concat_list(original: &mut Vec<i32>, additional: Vec<i32>) {
    original.extend(additional);
}

#[derive(Debug, Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Config {
    #[patch(apply_by(concat_list))]
    items: Vec<i32>,
    name: String,
}

fn main() {
    let mut config = Config {
        items: vec![1, 2, 3],
        name: "base".to_string(),
    };

    // Patch with apply_by: items are concatenated instead of replaced.
    config.apply(ConfigPatch {
        items: Some(vec![4, 5, 6]),
        name: None,
    });
    assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6]);
    println!("After first patch: {:?}", config.items);

    // A second patch appends more items.
    config.apply(ConfigPatch {
        items: Some(vec![7, 8]),
        name: Some("updated".to_string()),
    });
    assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    println!("After second patch: {:?}", config.items);
    println!("Name: {}", config.name);

    // None patch leaves the field unchanged.
    config.apply(ConfigPatch {
        items: None,
        name: None,
    });
    assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    println!("After empty patch: {:?}", config.items);
}
