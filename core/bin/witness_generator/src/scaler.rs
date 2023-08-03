//! Module with utilities for prover scaler service.

// Workspace deps
use crate::database_interface::DatabaseInterface;
/// Scaler oracle provides information for prover scaler
/// service about required amount of provers for server
/// to operate optimally.
#[derive(Debug)]
pub struct ScalerOracle<DB: DatabaseInterface> {
    /// Database access to gather the information about amount of pending blocks.
    db: DB,

    /// Number of idle provers running for faster up-scaling.
    idle_provers: u32,
}

impl<DB: DatabaseInterface> ScalerOracle<DB> {
    pub fn new(db: DB, idle_provers: u32) -> Self {
        Self { db, idle_provers }
    }
}
