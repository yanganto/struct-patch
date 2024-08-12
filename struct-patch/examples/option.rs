#[cfg(feature = "option")]
use struct_patch::Patch;

#[cfg(feature = "option")]
fn main() {
    #[derive(Default, Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug, Default)))]
    struct User {
        name: String,
        #[patch(name = "Option<AddressPatch>")]
        address: Option<Address>,
    }

    #[derive(Default, Debug, PartialEq, Patch)]
    #[patch(attribute(derive(Debug, Default)))]
    struct Address {
        street: Option<String>,
        country: String,
    }

    impl From<AddressPatch> for Address {
        fn from(patch: AddressPatch) -> Self {
            let mut address = Address::default();
            address.apply(patch);
            address
        }
    }

    let mut user = User {
        name: String::from("Thomas"),
        address: None,
    };
    let mut patch: UserPatch = User::new_empty_patch();

    patch.address = Some(Some(AddressPatch {
        country: Some("France".to_string()),
        ..Default::default()
    }));

    user.apply(patch);

    assert_eq!(
        user,
        User {
            name: String::from("Thomas"),
            address: Some(Address {
                street: None,
                country: String::from("France"),
            }),
        }
    );
}

#[cfg(not(feature = "option"))]
fn main() {
    println!("Please enable the 'option' feature to run this example");
}
