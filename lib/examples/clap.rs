use clap::Parser;
use struct_patch::Patch;

#[derive(Debug, Patch)]
#[patch(attribute(derive(Debug, Default, Parser)))]
struct Config {
    #[patch(attribute(arg(short, long)))]
    log_level: u8,
    #[patch(attribute(arg(short, long)))]
    debug: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            log_level: 10,
            debug: false,
        }
    }
}

fn main() {

    // NOTE: 
    // We patch from the patch instance, so the config can easily follow
    // Rust Default Trait by avoiding to set default from the clap macro
    // we can easily have the single source of default values
    
    let mut config = Config::default();
    config.apply(ConfigPatch::parse());

    println!("{config:#?}")
}
