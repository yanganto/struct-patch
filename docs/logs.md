# Case Study - Log which fields were patched or filled

Both `Patch` and `Filler` support two ways to observe which fields are changed.
**Requires the `log` feature.** default enabled.

**Note:** If the `log` feature is not enabled, there is no difference between the `nesting`
and `simple-nesting` features—both lack the ability to observe field changes.

**Error:** `nesting` and `simple-nesting` features cannot be enabled at the same time.
They are mutually exclusive. Choose one or the other based on your use case:
- Use `nesting` for full multi-level nesting support with complete path visibility
- Use `simple-nesting` for lightweight single-level nesting in `no_std` environments

**Ad-hoc at the call site** — use `apply_with_log`, which takes a closure that
is called with each patched/filled field name:

```rust
use struct_patch::Patch;

#[derive(Default, Patch)]
struct Item {
    field_int: usize,
    field_string: String,
}

let mut item = Item::default();
let patch = ItemPatch { field_int: Some(42), field_string: None };

let mut patched_fields = Vec::new();
item.apply_with_log(patch, |field| patched_fields.push(field.to_string()));

assert_eq!(patched_fields, vec!["field_int"]);
assert_eq!(item.field_int, 42);
```

**Function Signature for logging functions:**
- **Without `nesting` or `simple-nesting` feature**: Define your logging
  function as `fn(&str)` that takes only the field name.
- **With `nesting` feature enabled**: Define your logging function as
  `fn(&[&str], &str)` where the first parameter contains path segments for 
  nested fields (e.g., `["config", "logging"]` for a nested field), and the
  second parameter is the field name. This allows you to see the complete path
  through nested structures.
- **With `simple-nesting` feature enabled**: Define your logging function as
  `fn(&str, &str)` where the first parameter is a single prefix string for one
  level of nesting (e.g., `"config"` for nested fields, `""` for top-level), and
  the second parameter is the field name. This allows you to observe the path
  through single-level nested structures without heap allocation, making it ideal
  for `no_std` environments.

**Always-on via struct attribute** — use `#[patch(default_log(fn_path))]` or
`#[filler(default_log(fn_path))]` to wire a specific function into `apply`
itself. Every call to `apply` on that struct will automatically invoke the
function for each field that is changed, with no extra effort at call sites.
Has no effect on `apply_with_log`.

Example without `nesting` or `simple-nesting` feature:

```rust
use struct_patch::Filler;

// Your clean logging function that takes only the field name
#[cfg(not(any(feature = "nesting", feature = "simple-nesting")))]
fn my_filler_log(field: &str) {
    println!("filled: {field}");
}


#[derive(Default, Filler)]
#[filler(default_log(my_filler_log))]
struct Settings {
    theme: Option<String>,
}

let mut settings = Settings::default();
settings.apply(SettingsFiller { theme: Some("dark".into()) });
// prints: filled: theme
```

Example with `nesting` feature:

```rust
use struct_patch::Patch;

fn my_log(prefixes: &[&str], field: &str) {
    let path = if prefixes.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefixes.join("."), field)
    };
    println!("patched: {path}");
}

#[derive(Default, Patch)]
#[patch(default_log(my_log))]
struct Config {
    retries: usize,
    timeout: u64,
}

let mut cfg = Config::default();
cfg.apply(ConfigPatch { retries: Some(3), timeout: None });
// prints: patched: retries
```

The path may be any item path (`crate::logging::log_field`,
`tracing::debug!` wrapped in a thin function, etc.).

Example with `simple-nesting` feature:

The `simple-nesting` feature provides lightweight nesting support for
single-level nested patches without heap allocation. It's ideal for `no_std`
environments where you want nesting but cannot afford the allocation overhead
of the full `nesting` feature.

** Note: ** `simple-nesting` still works for multi-level nesting, but the
logging function only works well with one layer.

```rust
use struct_patch::Patch;

fn log_field(prefix: &str, field: &str) {
    // prefix is a single string like "inner" or empty ""
    // field is the current field name like "value"
    let path = if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefix, field)
    };
    println!("patched: {path}");
}

#[derive(Clone, Debug, Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Item {
    value: u32,
    #[patch(nesting)]
    config: Config,
}

#[derive(Clone, Debug, Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Config {
    timeout: u32,
    retries: u32,
}

// Creating a nested patch
let item_a = Item::default();
let item_b = Item {
    value: 42,
    config: Config {
        timeout: 5000,
        retries: 3,
    },
};

let patch: ItemPatch = item_b.clone().into_patch_by_diff(item_a);

// Applying with logging
let mut item = Item::default();
item.apply_with_log(patch, |prefix, field| {
    let path = if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefix, field)
    };
    println!("patched: {path}");
});

// Output:
// patched: value
// patched: config.timeout
// patched: config.retries
```
