use struct_patch::Patch;

fn log_field(field: &str) {
    println!("[default_log] patched field: {field}");
}

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(default_log(log_field))]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

fn main() {
    let mut config = Config::default();

    // apply() calls log_field automatically via default_log
    println!("--- apply() with default_log ---");
    config.apply(ConfigPatch {
        host: Some("localhost".into()),
        port: Some(8080),
        debug: None,
    });
    // Prints:
    //   [default_log] patched field: host
    //   [default_log] patched field: port

    println!(
        "host={}, port={}, debug={}",
        config.host, config.port, config.debug
    );

    // apply_with_log() overrides the log callback with a custom format
    println!("\n--- apply_with_log() with custom format ---");
    config.apply_with_log(
        ConfigPatch {
            host: None,
            port: None,
            debug: Some(true),
        },
        |field| println!("[custom_log] field '{}' was updated", field),
    );
    // Prints:
    //   [custom_log] field 'debug' was updated

    println!(
        "host={}, port={}, debug={}",
        config.host, config.port, config.debug
    );
}
