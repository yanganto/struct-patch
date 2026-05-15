#![allow(unused)]
use struct_patch::Substrate;
use serde::Deserialize;


#[derive(Deserialize, Default, Substrate)]
pub struct Base {
    #[serde(default)]
    pub field_bool: bool,
    pub field_string: String,
    pub field_option: Option<usize>,
}

impl Base {
    pub fn has_bool(&self) -> bool {
        self.field_bool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_works() {
        assert_eq!(
            Base::expose_content(),
            r#"{"named":[{"attrs":[{"style":"outer","meta":{"list":{"path":{"segments":[{"ident":"serde"}]},"delimiter":"paren","tokens":[{"ident":"default"}]}}}],"vis":"pub","ident":"field_bool","colon_token":true,"ty":{"path":{"segments":[{"ident":"bool"}]}}},{"vis":"pub","ident":"field_string","colon_token":true,"ty":{"path":{"segments":[{"ident":"String"}]}}},{"vis":"pub","ident":"field_option","colon_token":true,"ty":{"path":{"segments":[{"ident":"Option","arguments":{"angle_bracketed":{"args":[{"type":{"path":{"segments":[{"ident":"usize"}]}}}]}}}]}}}]}"#
        );

        let _fields: syn::Fields = syn_serde::json::from_str(&Base::expose_content()).unwrap();
    }
}
