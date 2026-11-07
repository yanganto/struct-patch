use schemars::{schema_for, JsonSchema};
use struct_patch::Patch;

/// The documentation for my structure.
#[derive(JsonSchema, Patch)]
#[patch(attribute(derive(JsonSchema)))]
struct MyStruct {
    /// The documentation for my field.
    my_field: bool,
}

fn main() {
    let schema = schema_for!(MyStruct);
    assert_eq!(schema.get("title").unwrap(), "MyStruct");
    assert_eq!(
        schema.get("description").unwrap(),
        "The documentation for my structure."
    );
    assert_eq!(
        schema
            .get("properties")
            .unwrap()
            .get("my_field")
            .unwrap()
            .get("description")
            .unwrap(),
        "The documentation for my field."
    );
    dbg!(schema);

    let schema = schema_for!(MyStructPatch);
    assert_eq!(schema.get("title").unwrap(), "MyStructPatch");
    assert_eq!(
        schema.get("description").unwrap(),
        "The documentation for my structure."
    );
    assert_eq!(
        schema
            .get("properties")
            .unwrap()
            .get("my_field")
            .unwrap()
            .get("description")
            .unwrap(),
        "The documentation for my field."
    );
    dbg!(schema);
}
