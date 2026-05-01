use struct_patch::Substrate;

#[derive(Substrate)]
pub struct Base {
    field_bool: bool,
    field_string: String,
    field_option: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_works() {
        assert_eq!(Base::expose(), 
            r#""{\"named\":[{\"ident\":\"field_bool\",\"colon_token\":true,\"ty\":{\"path\":{\"segments\":[{\"ident\":\"bool\"}]}}},{\"ident\":\"field_string\",\"colon_token\":true,\"ty\":{\"path\":{\"segments\":[{\"ident\":\"String\"}]}}},{\"ident\":\"field_option\",\"colon_token\":true,\"ty\":{\"path\":{\"segments\":[{\"ident\":\"Option\",\"arguments\":{\"angle_bracketed\":{\"args\":[{\"type\":{\"path\":{\"segments\":[{\"ident\":\"usize\"}]}}}]}}}]}}}]}""#);
    }
}
