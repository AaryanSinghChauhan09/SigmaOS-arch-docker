// Integration test for SigmaOS-arch-docker
// Verifies the crate builds and links as a dependency.

use sigmaos_arch_docker::{ContainerRuntime, ContainerSpec};

#[test]
fn crate_links_and_runs() {
    // Arrange
    let mut runtime = ContainerRuntime::new();
    runtime.pull_image("ubuntu:22.04").unwrap();

    // Act
    let id = runtime.create(ContainerSpec::new("web", "ubuntu:22.04"));

    // Assert
    assert!(id.is_ok());
    assert_eq!(runtime.list().len(), 1);
}
