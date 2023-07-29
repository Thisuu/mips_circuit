use super::StorageProcessor;

/// `ChainIntermediator` is a structure providing methods to
/// obtain schemas declared in the `chain` module.
#[derive(Debug)]
pub struct ChainIntermediator<'a, 'c>(pub &'a mut StorageProcessor<'c>);

