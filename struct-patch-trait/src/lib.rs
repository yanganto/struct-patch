pub mod traits {
    pub trait Patch<P: Default> {
        fn apply(&mut self, patch: P);

        fn default_patch() -> P {
            P::default()
        }
    }
}
