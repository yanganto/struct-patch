use serde::{Deserialize, Serialize};
use struct_patch::Catalyst;
use substrate::{Base, PhoneNumber};

#[derive(Default, Catalyst)]
#[catalyst(bind = Base, src = "substrate:/src/lib.rs")]
#[catalyst(keep_field_attribute)]
#[complex(attribute(derive(Debug, Deserialize, Serialize)))]
#[complex(override_field_attribute("filed_numbers", serde(default)))]
#[allow(dead_code)]
struct Amyloid {
    pub extra_bool: bool,
    #[complex(attribute(serde(default = "default_str")))]
    pub extra_string: String,
    pub extra_option: Option<usize>,
    #[complex(attribute(serde(default = "default_extra_private_number")))]
    extra_private_number: u8,
}

fn default_str() -> String {
    "default".to_string()
}

fn default_extra_private_number() -> u8 {
    7
}

#[allow(dead_code)]
impl AmyloidComplex {
    fn private_number_sum(&self) -> u8 {
        self.private_number + self.extra_private_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_from_binding_src_works() {
        let substrate = Base::__substrate_new(true, String::new(), None, 1u8, PhoneNumber::default());
        let amyloid = Amyloid::default();
        let complex = amyloid.bind(substrate);
        assert_eq!(complex.field_bool, true);
        assert_eq!(complex.private_number_sum(), 1);

        let toml_str = toml::to_string_pretty(&complex).unwrap();
        assert_eq!(
            toml_str,
            r#"field_bool = true
field_string = ""
private_number = 1
extra_bool = false
extra_string = ""
extra_private_number = 0

[filed_numbers]
country_code = 0
local_numbers = ""
"#
        );

        let toml_str = r#"field_bool = true
field_string = ""
private_number = 1
extra_bool = true
"#;
        let complex: AmyloidComplex = toml::from_str(toml_str).unwrap();
        assert_eq!(complex.extra_string, "default");
        // extra_private_number defaults to 7 via default_extra_private_number
        assert_eq!(complex.private_number_sum(), 8);
    }
}
