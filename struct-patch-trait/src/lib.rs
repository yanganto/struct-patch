/// Define the behavior between patch struct and original sturct
pub mod traits {
    /// The trait can apply patch and generete corresponding patch instance
    pub trait Patch<P: Default> {
        /// Apply the patch, only update the existing fields
        fn apply(&mut self, patch: P);

        /// Get an empty patch instance
        fn default_patch() -> P {
            P::default()
        }
    }

    /// The trait can check on the status of patch instance
    pub trait PatchStruct {
        /// There is any field need to patch
        fn is_empty(&self) -> bool;
    }
}
