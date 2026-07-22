use tiler_ir::semantic::{
    SemanticAdmissionProvenanceIdentity, SemanticDefinitionProjectionIdentity,
    SemanticGraphIdentity, SemanticIdentity, SemanticRegistrySnapshotIdentity,
};

fn graph() -> SemanticGraphIdentity { panic!() }
fn definitions() -> SemanticDefinitionProjectionIdentity { panic!() }
fn admission() -> SemanticAdmissionProvenanceIdentity { panic!() }
fn snapshot() -> SemanticRegistrySnapshotIdentity { panic!() }

fn main() {
    let _ = SemanticIdentity::new(graph(), definitions(), admission(), snapshot());
}
