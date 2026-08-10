# Case Study - Log which fields were patched or filled

Both `Patch` and `Filler` support two ways to observe which fields are changed:

**Ad-hoc at the call site** — use `apply_with_log`, which takes a closure that
is called with each patched/filled field name:

```rust
use struct_patch::{Filler, Patch};

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

#[derive(Default, Filler)]
struct Settings {
    theme: Option<String>,
}

let mut settings = Settings::default();
let mut filled_fields = Vec::new();
settings.apply_with_log(
    SettingsFiller { theme: Some("dark".into()) },
    |field| filled_fields.push(field.to_string()),
);
assert_eq!(filled_fields, vec!["theme"]);
```

For structs using `#[patch(nesting)]`, the log closure is threaded into nested
patches so you receive field names from all levels of nesting.

**Always-on via struct attribute** — use `#[patch(default_log(fn_path))]` or
`#[filler(default_log(fn_path))]` to wire a specific function into `apply`
itself. Every call to `apply` on that struct will automatically invoke the
function for each field that is changed, with no extra effort at call sites.
Has no effect on `apply_with_log`.

```rust
use struct_patch::{Filler, Patch};

fn my_log(field: &str) {
    println!("patched: {field}");
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

The path may be any item path (`crate::logging::log_field`,
`tracing::debug!` wrapped in a thin function, etc.).
