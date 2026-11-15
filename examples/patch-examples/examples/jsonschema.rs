use schemars::{schema_for, JsonSchema, Schema};
use serde_json::Value;
use struct_patch::Patch;

/// The documentation for my structure.
#[derive(JsonSchema, Patch)]
#[patch(attribute(derive(JsonSchema)))]
#[patch(append_to_doc = "Append something.")]
struct MyStruct {
    /// The documentation for my field 1.
    my_field_1: bool,
    /// The documentation for my field 2.
    #[patch(prepend_to_doc = "Prepend something.")]
    #[patch(append_to_doc = "Append something.")]
    my_field_2: bool,
    /// The documentation for my field 3.
    #[patch(append_to_doc = "Append something part 1.")]
    #[patch(append_to_doc = "Append something part 2.")]
    my_field_3: bool,
    /// The original documentation for my field 4 is ignored.
    #[patch(doc = "Set the doc part 1.")]
    #[patch(doc = "Set the doc part 2.")]
    #[patch(append_to_doc = "This is appended.")]
    my_field_4: bool,
    /// The original documentation for my field 5 is ignored.
    #[patch(skip_doc)]
    my_field_5: bool,
}

fn get_description<'a>(schema: &'a Schema, field: &'static str) -> Option<&'a Value> {
    schema
        .get("properties")
        .unwrap()
        .get(field)
        .unwrap()
        .get("description")
}

fn main() {
    let schema = schema_for!(MyStruct);
    assert_eq!(schema.get("title").unwrap(), "MyStruct");
    assert_eq!(
        schema.get("description").unwrap(),
        "The documentation for my structure."
    );
    assert_eq!(
        get_description(&schema, "my_field_1").unwrap(),
        "The documentation for my field 1."
    );
    assert_eq!(
        get_description(&schema, "my_field_2").unwrap(),
        "The documentation for my field 2."
    );
    assert_eq!(
        get_description(&schema, "my_field_3").unwrap(),
        "The documentation for my field 3."
    );
    assert_eq!(
        get_description(&schema, "my_field_4").unwrap(),
        "The original documentation for my field 4 is ignored."
    );
    assert_eq!(
        get_description(&schema, "my_field_5").unwrap(),
        "The original documentation for my field 5 is ignored."
    );
    println!("{schema:#?}");

    let schema = schema_for!(MyStructPatch);
    assert_eq!(schema.get("title").unwrap(), "MyStructPatch");
    assert_eq!(
        schema.get("description").unwrap(),
        "The documentation for my structure.\nAppend something."
    );
    assert_eq!(
        get_description(&schema, "my_field_1").unwrap(),
        "The documentation for my field 1."
    );
    assert_eq!(
        get_description(&schema, "my_field_2").unwrap(),
        "Prepend something.\nThe documentation for my field 2.\nAppend something."
    );
    assert_eq!(
        get_description(&schema, "my_field_3").unwrap(),
        "The documentation for my field 3.\nAppend something part 1.\nAppend something part 2."
    );
    assert_eq!(
        get_description(&schema, "my_field_4").unwrap(),
        "Set the doc part 1.\nSet the doc part 2.\nThis is appended."
    );
    assert!(get_description(&schema, "my_field_5").is_none());
    println!("{schema:#?}");
}
