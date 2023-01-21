/// Define the behavior between patch struct and original sturct
pub mod traits {
    /// The trait can apply patch and generete corresponding patch instance
    pub trait Patch<P> {
        /// Apply the patch, only update the existing fields
        fn apply(&mut self, patch: P);

        /// Diff on a previous state and into the patch instance
        fn into_patch_by_diff(self, previous_struct: Self) -> P;

        /// Get an empty patch instance
        fn new_empty_patch() -> P;
    }

    #[cfg(feature = "status")]
    /// The trait can check on the status of patch instance
    pub trait PatchStatus {
        /// There is any field need to patch
        fn is_empty(&self) -> bool;
    }
}
