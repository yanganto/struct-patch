/// A struct that a patch can be applied to
///
/// Deriving [`Patch`] will generate a patch struct and an accompanying trait impl so that it can be applied to the original struct.
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// struct Item {
///     field_bool: bool,
///     field_int: usize,
///     field_string: String,
/// }
///
/// // Generated struct
/// // struct ItemPatch {
/// //     field_bool: Option<bool>,
/// //     field_int: Option<usize>,
/// //     field_string: Option<String>,
/// // }
/// ```
/// ## Container attributes
/// ### `#[patch(attribute(derive(...)))]`
/// Use this attribute to derive traits on the generated patch struct
/// ```rust
/// # use struct_patch::Patch;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Patch)]
/// #[patch(attribute(derive(Debug, Default, Deserialize, Serialize)))]
/// struct Item;
///
/// // Generated struct
/// // #[derive(Debug, Default, Deserialize, Serialize)]
/// // struct ItemPatch {}
/// ```
///
/// ### `#[patch(attribute(...))]`
/// Use this attribute to pass the attributes on the generated patch struct
/// ```compile_fail
/// // This example need `serde` and `serde_with` crates
/// # use struct_patch::Patch;
/// #[derive(Patch, Debug)]
/// #[patch(attribute(derive(Serialize, Deserialize, Default)))]
/// #[patch(attribute(skip_serializing_none))]
/// struct Item;
///
/// // Generated struct
/// // #[derive(Default, Deserialize, Serialize)]
/// // #[skip_serializing_none]
/// // struct ItemPatch {}
/// ```
///
/// ### `#[patch(name = "...")]`
/// Use this attribute to change the name of the generated patch struct
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// #[patch(name = "ItemOverlay")]
/// struct Item { }
///
/// // Generated struct
/// // struct ItemOverlay {}
/// ```
///
/// ### `#[patch(default_log(fn_path))]`
/// Automatically call `fn_path` with patched field information inside
/// every generated `apply` call. Has no effect on `apply_with_log`. The path
/// may be any function path visible at the call site.
///
/// When the `nesting` feature is enabled, `fn_path` receives both a prefix path and field name.
/// When `nesting` is disabled, `fn_path` receives only the field name.
///
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Default, Patch)]
/// struct Item {
///     field_int: usize,
///     field_string: String,
/// }
///
/// let mut item = Item::default();
/// let patch = ItemPatch { field_int: Some(1), field_string: None };
///
/// #[cfg(feature = "nesting")]
/// {
///     fn log_field(prefixes: &[&str], field: &str) {
///         let path = if prefixes.is_empty() {
///             field.to_string()
///         } else {
///             format!("{}.{}", prefixes.join("."), field)
///         };
///         println!("patched: {path}");
///     }
///
///     #[derive(Default, Patch)]
///     #[patch(default_log(log_field))]
///     struct Config {
///         field_int: usize,
///         field_string: String,
///     }
///     let mut config = Config::default();
///     config.apply(ConfigPatch { field_int: Some(1), field_string: None });
///     // log_field(&[], "field_int") is called automatically
/// }
///
/// #[cfg(not(feature = "nesting"))]
/// {
///     fn log_field(field: &str) {
///         println!("patched: {field}");
///     }
///
///     #[derive(Default, Patch)]
///     #[patch(default_log(log_field))]
///     struct Config {
///         field_int: usize,
///         field_string: String,
///     }
///     let mut config = Config::default();
///     config.apply(ConfigPatch { field_int: Some(1), field_string: None });
///     // log_field("field_int") is called automatically
/// }
/// ```
///
/// ## Field attributes
/// ### `#[patch(skip)]`
/// If you want certain fields to be unpatchable, you can let the derive macro skip certain fields when creating the patch struct
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// struct Item {
///     #[patch(skip)]
///     id: String,
///     data: String,
/// }
///
/// // Generated struct
/// // struct ItemPatch {
/// //     data: Option<String>,
/// // }
/// ```
///
/// ### `#[patch(skip_wrap)]`
/// Keep the field type as-is in the generated patch struct (no extra `Option`
/// wrapping). This is useful for fields that are already `Option<...>`,
/// typically `Option<Vec<_>>`, where the default double-`Option` in the patch
/// is unwanted. With `skip_wrap`, `None` in the patch means "no change" and
/// `Some(v)` sets the field to `Some(v)` (including `Some(vec![])` to clear
/// the vector). Cannot be combined with `empty_value`.
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Default, Patch)]
/// struct Item {
///     #[patch(skip_wrap)]
///     tags: Option<Vec<String>>,
/// }
///
/// // Generated struct
/// // struct ItemPatch {
/// //     tags: Option<Vec<String>>, // not wrapped again
/// // }
///
/// let mut item = Item { tags: Some(vec!["a".into()]) };
///
/// // `None` in the patch keeps the field unchanged.
/// item.apply(ItemPatch { tags: None });
/// assert_eq!(item.tags, Some(vec!["a".into()]));
///
/// // `Some(vec![])` still applies and clears the list.
/// item.apply(ItemPatch { tags: Some(vec![]) });
/// assert_eq!(item.tags, Some(vec![]));
/// ```
pub trait Patch<P> {
    /// Apply a patch
    fn apply(&mut self, patch: P);

    /// Apply a patch, calling `log` with each patched field name.
    ///
    /// The default implementation ignores `log` and delegates to [`apply`](Patch::apply).
    /// The derive macro generates an override that calls `log` once per field that is
    /// actually changed.
    ///
    /// When the `nesting` feature is enabled, `log` receives both a prefix path and field name.
    /// When `nesting` is disabled, `log` receives only the field name.
    ///
    /// ```rust
    /// # use struct_patch::Patch;
    /// #[derive(Default, Patch)]
    /// struct Item {
    ///     field_int: usize,
    ///     field_string: String,
    /// }
    ///
    /// let mut item = Item::default();
    /// let patch = ItemPatch { field_int: Some(42), field_string: None };
    ///
    /// let mut patched_fields = Vec::new();
    /// #[cfg(feature = "nesting")]
    /// item.apply_with_log(patch, |prefixes, field| {
    ///     let path = if prefixes.is_empty() {
    ///         field.to_string()
    ///     } else {
    ///         format!("{}.{}", prefixes.join("."), field)
    ///     };
    ///     patched_fields.push(path);
    /// });
    ///
    /// #[cfg(not(feature = "nesting"))]
    /// item.apply_with_log(patch, |field| {
    ///     patched_fields.push(field.to_string());
    /// });
    ///
    /// assert_eq!(patched_fields, vec!["field_int"]);
    /// ```
    #[cfg(feature = "nesting")]
    fn apply_with_log<F: FnMut(&[&str], &str)>(&mut self, patch: P, _log: F) {
        self.apply(patch);
    }

    #[cfg(not(feature = "nesting"))]
    fn apply_with_log<F: FnMut(&str)>(&mut self, patch: P, _log: F) {
        self.apply(patch);
    }

    /// Returns a patch that when applied turns any struct of the same type into `Self`
    fn into_patch(self) -> P;

    /// Returns a patch that when applied turns `previous_struct` into `Self`
    fn into_patch_by_diff(self, previous_struct: Self) -> P;

    /// Get an empty patch instance
    fn new_empty_patch() -> P;
}

pub trait Filler<F> {
    /// Apply a filler
    fn apply(&mut self, filler: F);

    /// Apply a filler, calling `log` with each field name that is actually filled.
    ///
    /// The default implementation ignores `log` and delegates to [`apply`](Filler::apply).
    /// The derive macro generates an override that calls `log` once per field that is
    /// actually filled (i.e. the field was empty and the filler supplied a value).
    ///
    /// When the `nesting` feature is enabled, `log` receives both a prefix path and field name.
    /// When `nesting` is disabled, `log` receives only the field name.
    ///
    /// ```rust
    /// # use struct_patch::Filler;
    /// #[derive(Default, Filler)]
    /// struct Item {
    ///     value: Option<usize>,
    /// }
    ///
    /// let mut item = Item::default();
    /// let filler = ItemFiller { value: Some(42) };
    ///
    /// let mut filled_fields = Vec::new();
    /// #[cfg(feature = "nesting")]
    /// item.apply_with_log(filler, |prefixes, field| {
    ///     let path = if prefixes.is_empty() {
    ///         field.to_string()
    ///     } else {
    ///         format!("{}.{}", prefixes.join("."), field)
    ///     };
    ///     filled_fields.push(path);
    /// });
    ///
    /// #[cfg(not(feature = "nesting"))]
    /// item.apply_with_log(filler, |field| {
    ///     filled_fields.push(field.to_string());
    /// });
    ///
    /// assert_eq!(filled_fields, vec!["value"]);
    /// ```
    #[cfg(feature = "nesting")]
    fn apply_with_log<L: FnMut(&[&str], &str)>(&mut self, filler: F, _log: L) {
        self.apply(filler);
    }

    #[cfg(not(feature = "nesting"))]
    fn apply_with_log<L: FnMut(&str)>(&mut self, filler: F, _log: L) {
        self.apply(filler);
    }

    /// Get an empty filler instance
    fn new_empty_filler() -> F;
}

#[cfg(feature = "status")]
/// A patch struct with extra status information
pub trait Status {
    /// Returns `true` if all fields are `None`, `false` otherwise.
    fn is_empty(&self) -> bool;
}

#[cfg(feature = "merge")]
/// A patch struct that can be merged to another one
pub trait Merge {
    fn merge(self, other: Self) -> Self;
}

#[cfg(feature = "substrate")]
/// A substrate struct that can expose the fields information thereof
pub trait Substrate {
    fn expose_content() -> &'static str;

    /// Expose the field information, by call this function in Build.rs
    fn expose();
}

#[cfg(feature = "catalyst")]
/// A catalyst struct that can expose the fields information thereof
pub trait Catalyst<S, Cpx> {
    /// catalyst bind on substrate and generate complex
    fn bind(self, substrate: S) -> Cpx;
}

#[cfg(feature = "catalyst")]
/// A complex struct that can decouple return catalyst and substrate
pub trait Complex<Cat, S> {
    /// complex decouple to catalyst and substrate
    fn decouple(self) -> (Cat, S);
}
