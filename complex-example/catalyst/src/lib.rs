use struct_patch::Catalyst;
use substrate::Base;

#[derive(Catalyst)]
// #[catalyst(bind = Base)]
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
        let complex = AmyloidComplex { };
    }
}
