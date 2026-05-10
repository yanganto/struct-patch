use struct_patch::Catalyst;

#[derive(Catalyst)]
#[catalyst(bind = substrate::Base)]
// #[complex(name = Complex)]
// #[complex(attribute(derive(Default)))]
struct Amyloid {
    extra_bool: bool,
    extra_string: String,
    extra_option: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_works() {
        let _complex = AmyloidComplex {
            field_bool: false,
            field_string: String::new(),
            field_option: None,
            extra_bool: false,
            extra_string: String::new(),
            extra_option: None,
        };
    }
}
