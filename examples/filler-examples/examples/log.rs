use struct_patch::Filler;

// TODO really implement nesting feature for filler,
// currently the log x nesting feature works, but no nesting feature for filler lol

#[cfg(not(feature = "nesting"))]
fn log_filler_field(field: &str) {
    println!("[default_log] filler field: {field}");
}

#[cfg(feature = "nesting")]
fn log_filler_field(prefixes: &[&str], field: &str) {
    let path = if prefixes.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefixes.join("."), field)
    };
    println!("[default_log] filler field: {path}");
}

#[derive(Default, Filler)]
#[filler(attribute(derive(Debug, Default)))]
#[filler(default_log(log_filler_field))]
struct Settings {
    theme: Option<String>,
    max_connections: Option<u16>,
}

fn main() {
    // --- Filler with default_log ---
    println!("--- Filler: apply() with default_log ---");
    let mut settings = Settings::default();
    settings.apply(SettingsFiller {
        theme: Some("dark".into()),
        max_connections: Some(100),
    });
    // Prints:
    //   [default_log] filler field: theme
    //   [default_log] filler field: max_connections

    println!(
        "theme={:?}, max_connections={:?}",
        settings.theme, settings.max_connections
    );

    // Applying again has no effect because the fields are already filled.
    println!("\n--- Filler: apply() again (fields already filled, no log) ---");
    settings.apply(SettingsFiller {
        theme: Some("light".into()),
        max_connections: Some(999),
    });
    println!(
        "theme={:?}, max_connections={:?}",
        settings.theme, settings.max_connections
    );

    #[cfg(not(feature = "nesting"))]
    {
        // --- Filler with apply_with_log (custom format) ---
        println!("\n--- Filler: apply_with_log() with custom format ---");
        let mut settings2 = Settings::default();
        settings2.apply_with_log(
            SettingsFiller {
                theme: Some("light".into()),
                max_connections: None,
            },
            |field| {
                println!("[custom_log] filler field '{}' was filled", field);
            },
        );
        // Prints:
        //   [custom_log] filler field 'theme' was filled

        println!(
            "theme={:?}, max_connections={:?}",
            settings2.theme, settings2.max_connections
        );
    }
}
