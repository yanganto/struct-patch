use struct_patch::Patch;

#[cfg(not(feature = "nesting"))]
fn log_patch_field(field: &str) {
    println!("[default_log] patch field: {field}");
}

#[cfg(feature = "nesting")]
fn log_patch_field(prefixes: &[&str], field: &str) {
    let path = if prefixes.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", prefixes.join("."), field)
    };
    println!("[default_log] patch field: {path}");
}

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(default_log(log_patch_field))]
struct Config {
    host: String,
    port: u16,
    #[cfg(feature = "nesting")]
    #[patch(nesting)]
    logging: Logging,
}

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
struct Server {
    name: String,
    #[patch(nesting)]
    config: Config,
}

#[cfg(not(feature = "nesting"))]
fn main() {
    println!("--- Patch: apply() with default_log ---");
    let mut config = Config::default();
    config.apply(ConfigPatch {
        host: Some("localhost".into()),
        port: Some(8080),
    });
    // Prints:
    //   [default_log] patch field: host
    //   [default_log] patch field: port

    println!(
        "host={}, port={}",
        config.host, config.port
    );

    println!("\n--- Patch: apply_with_log() with custom format ---");
    config.apply_with_log(
        ConfigPatch {
            host: None,
            port: None,
        },
        |field| {
            println!("[custom_log] patch field '{}' was updated", field);
        },
    );
    // Prints:
    //   [custom_log] patch field 'debug' was updated

    println!(
        "host={}, port={}",
        config.host, config.port
    );
}

#[cfg(feature = "nesting")]
fn main() {
    println!("\n--- Patch: apply() with nesting and default_log ---");
    let mut server = Server::default();
    server.apply(ServerPatch {
        name: Some("prod-server".into()),
        config: ConfigPatch {
            host: Some("192.168.1.1".into()),
            port: Some(443),
            logging: LoggingPatch::default(),
        },
    });
    // Prints:
    //   [default_log] patch field: name
    //   [default_log] patch field: config.host
    //   [default_log] patch field: config.port

    println!(
        "name={}, config.host={}, config.port={}",
        server.name, server.config.host, server.config.port
    );

    println!("\n--- Patch: apply_with_log() with nesting and prefix path ---");
    let mut server2 = Server::default();
    server2.apply_with_log(
        ServerPatch {
            name: None,
            config: ConfigPatch {
                host: Some("10.0.0.1".into()),
                port: None,
                logging: LoggingPatch::default(),
            },
        },
        |prefixes, field| {
            let path = if prefixes.is_empty() {
                field.to_string()
            } else {
                format!("{}.{}", prefixes.join("."), field)
            };
            println!("[custom_log] patch field: '{path}'");
        },
    );
    // Prints:
    //   [custom_log] patch field: 'config.host'

    println!(
        "name={}, config.host={}, config.port={}",
        server2.name, server2.config.host, server2.config.port
    );

    println!("\n--- Patch: apply() with deep nesting and default_log ---");
    let mut server3 = Server::default();
    server3.apply(ServerPatch {
        name: Some("app-server".into()),
        config: ConfigPatch {
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
    //   [default_log] patch field: config.host
    //   [default_log] patch field: config.port
    //   [default_log] patch field: config.logging.level
    //   [default_log] patch field: config.logging.format

    println!(
        "name={}, config.host={}, config.port={}, config.logging.level={}, config.logging.format={}",
        server3.name,
        server3.config.host,
        server3.config.port,
        server3.config.logging.level,
        server3.config.logging.format
    );

    println!("\n--- Patch: apply_with_log() with deep nesting and full path ---");
    let mut server4 = Server::default();
    server4.apply_with_log(
        ServerPatch {
            name: None,
            config: ConfigPatch {
                host: None,
                port: Some(9000),
                logging: LoggingPatch {
                    level: Some("warn".into()),
                    format: None,
                },
            },
        },
        |prefixes, field| {
            let path = if prefixes.is_empty() {
                field.to_string()
            } else {
                format!("{}.{}", prefixes.join("."), field)
            };
            println!("[custom_log] patch field: '{path}'");
        },
    );
    // Prints:
    //   [custom_log] patch field: 'config.port'
    //   [custom_log] patch field: 'config.logging.level'

    println!(
        "name={}, config.host={}, config.port={}, config.logging.level={}, config.logging.format={}",
        server4.name,
        server4.config.host,
        server4.config.port,
        server4.config.logging.level,
        server4.config.logging.format
    );
}
