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

// --- Patch with nesting example ---

#[cfg(feature = "nesting")]
#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Logging {
    level: String,
    format: String,
}

#[cfg(feature = "nesting")]
#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(default_log(log_patch_field))]
struct ConfigWithLogging {
    host: String,
    port: u16,
    #[patch(nesting)]
    logging: Logging,
}

#[cfg(feature = "nesting")]
#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(default_log(log_patch_field))]
struct Server {
    name: String,
    #[patch(nesting)]
    config: ConfigWithLogging,
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

    // --- Patch with nesting and default_log ---
    #[cfg(feature = "nesting")]
    {
        println!("\n--- Patch: apply() with nesting and default_log ---");
        let mut server = Server::default();
        server.apply(ServerPatch {
            name: Some("prod-server".into()),
            config: ConfigWithLoggingPatch {
                host: Some("192.168.1.1".into()),
                port: Some(443),
                logging: LoggingPatch::default(),
            },
        });
        // Prints:
        //   [default_log] patch field: name
        //   [default_log] patch field: host
        //   [default_log] patch field: port

        println!(
            "name={}, config.host={}, config.port={}",
            server.name, server.config.host, server.config.port
        );

        // --- Patch with nesting and apply_with_log ---
        println!("\n--- Patch: apply_with_log() with nesting ---");
        let mut server2 = Server::default();
        server2.apply_with_log(
            ServerPatch {
                name: None,
                config: ConfigWithLoggingPatch {
                    host: Some("10.0.0.1".into()),
                    port: None,
                    logging: LoggingPatch::default(),
                },
            },
            |field| println!("[custom_log] patch field: '{field}'"),
        );
        // Prints:
        //   [custom_log] patch field: 'host'

        println!(
            "name={}, config.host={}, config.port={}",
            server2.name, server2.config.host, server2.config.port
        );

        // --- Patch with deep nesting (nesting within nesting) and default_log ---
        println!("\n--- Patch: apply() with deep nesting and default_log ---");
        let mut server3 = Server::default();
        server3.apply(ServerPatch {
            name: Some("app-server".into()),
            config: ConfigWithLoggingPatch {
                host: Some("localhost".into()),
                port: Some(8080),
                logging: LoggingPatch {
                    level: Some("debug".into()),
                    format: Some("json".into()),
                },
            },
        });
        // Prints:
        //   [default_log] patch field: name
        //   [default_log] patch field: host
        //   [default_log] patch field: port
        //   [default_log] patch field: level
        //   [default_log] patch field: format

        println!(
            "name={}, config.host={}, config.port={}, config.logging.level={}, config.logging.format={}",
            server3.name,
            server3.config.host,
            server3.config.port,
            server3.config.logging.level,
            server3.config.logging.format
        );

        // --- Patch with deep nesting and apply_with_log ---
        println!("\n--- Patch: apply_with_log() with deep nesting ---");
        let mut server4 = Server::default();
        server4.apply_with_log(
            ServerPatch {
                name: None,
                config: ConfigWithLoggingPatch {
                    host: None,
                    port: Some(9000),
                    logging: LoggingPatch {
                        level: Some("warn".into()),
                        format: None,
                    },
                },
            },
            |field| println!("[custom_log] patch field: '{field}'"),
        );
        // Prints:
        //   [custom_log] patch field: 'port'
        //   [custom_log] patch field: 'level'

        println!(
            "name={}, config.host={}, config.port={}, config.logging.level={}, config.logging.format={}",
            server4.name,
            server4.config.host,
            server4.config.port,
            server4.config.logging.level,
            server4.config.logging.format
        );
    }
}
