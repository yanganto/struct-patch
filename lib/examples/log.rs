use struct_patch::{Filler, Patch};

fn log_patch_field(field: &str) {
    println!("[default_log] patch field: {field}");
}

fn log_filler_field(field: &str) {
    println!("[default_log] filler field: {field}");
}

// --- Patch example ---

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(default_log(log_patch_field))]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

// --- Filler example ---

#[derive(Default, Filler)]
#[filler(attribute(derive(Debug, Default)))]
#[filler(default_log(log_filler_field))]
struct Settings {
    theme: Option<String>,
    max_connections: Option<u16>,
}

fn main() {
    // --- Patch with default_log ---
    println!("--- Patch: apply() with default_log ---");
    let mut config = Config::default();
    config.apply(ConfigPatch {
        host: Some("localhost".into()),
        port: Some(8080),
        debug: None,
    });
    // Prints:
    //   [default_log] patch field: host
    //   [default_log] patch field: port

    println!(
        "host={}, port={}, debug={}",
        config.host, config.port, config.debug
    );

    // --- Patch with apply_with_log (custom format) ---
    println!("\n--- Patch: apply_with_log() with custom format ---");
    config.apply_with_log(
        ConfigPatch {
            host: None,
            port: None,
            debug: Some(true),
        },
        |field| println!("[custom_log] patch field '{}' was updated", field),
    );
    // Prints:
    //   [custom_log] patch field 'debug' was updated

    println!(
        "host={}, port={}, debug={}",
        config.host, config.port, config.debug
    );

    // --- Filler with default_log ---
    println!("\n--- Filler: apply() with default_log ---");
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

    // --- Filler with apply_with_log (custom format) ---
    println!("\n--- Filler: apply_with_log() with custom format ---");
    let mut settings2 = Settings::default();
    settings2.apply_with_log(
        SettingsFiller {
            theme: Some("light".into()),
            max_connections: None,
        },
        |field| println!("[custom_log] filler field '{}' was filled", field),
    );
    // Prints:
    //   [custom_log] filler field 'theme' was filled

    println!(
        "theme={:?}, max_connections={:?}",
        settings2.theme, settings2.max_connections
    );
}
