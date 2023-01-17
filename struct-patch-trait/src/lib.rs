/// Define the behavior between patch struct and original sturct
pub mod traits {
    pub trait Patch<P: Default> {
        /// apply the patch, only update the existing fields
        fn apply(&mut self, patch: P);

        /// get an empty patch instance
        fn default_patch() -> P {
            P::default()
        }
    }
}
