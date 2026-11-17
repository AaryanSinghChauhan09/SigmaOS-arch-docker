//! Docker-like container management interface for SigmaOS.
//!
//! Provides container lifecycle: create, start, stop, inspect, and remove.
//! Simulates container runtime operations without actual cgroups/namespaces. Dependency-free.

use std::collections::HashMap;
use std::fmt;

/// Errors for container operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerError {
    ContainerNotFound(String),
    AlreadyRunning(String),
    NotRunning(String),
    ImageNotFound(String),
    InvalidName(String),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContainerNotFound(c) => write!(f, "container not found: {c}"),
            Self::AlreadyRunning(c) => write!(f, "container already running: {c}"),
            Self::NotRunning(c) => write!(f, "container not running: {c}"),
            Self::ImageNotFound(i) => write!(f, "image not found: {i}"),
            Self::InvalidName(n) => write!(f, "invalid container name: {n}"),
        }
    }
}

impl std::error::Error for ContainerError {}

/// Container state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
}

impl ContainerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }
}

/// Container specification.
#[derive(Debug, Clone, PartialEq)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub ports: Vec<String>,
    pub env: HashMap<String, String>,
}

impl ContainerSpec {
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            command: Vec::new(),
            ports: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn with_command(mut self, cmd: Vec<String>) -> Self {
        self.command = cmd;
        self
    }

    pub fn with_port(mut self, port: impl Into<String>) -> Self {
        self.ports.push(port.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

/// Container instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub id: String,
    pub spec: ContainerSpec,
    pub state: ContainerState,
    pub created_at: u64,
}

/// Container runtime.
#[derive(Debug, Clone, Default)]
pub struct ContainerRuntime {
    containers: HashMap<String, Container>,
    images: HashMap<String, String>, // image -> digest
    next_id: u64,
}

impl ContainerRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pull_image(&mut self, image: &str) -> Result<(), ContainerError> {
        self.images.insert(image.to_string(), format!("sha256:{}", image.len()));
        Ok(())
    }

    pub fn create(&mut self, spec: ContainerSpec) -> Result<String, ContainerError> {
        if spec.name.trim().is_empty() {
            return Err(ContainerError::InvalidName(spec.name.clone()));
        }
        if self.containers.contains_key(&spec.name) {
            return Err(ContainerError::AlreadyRunning(spec.name.clone()));
        }
        if !self.images.contains_key(&spec.image) {
            return Err(ContainerError::ImageNotFound(spec.image.clone()));
        }

        self.next_id += 1;
        let id = format!("{}{}", &spec.name[..3.min(spec.name.len())], self.next_id);
        let name = spec.name.clone();
        let container = Container {
            id: id.clone(),
            spec,
            state: ContainerState::Created,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.containers.insert(name, container);
        Ok(id)
    }

    pub fn start(&mut self, name: &str) -> Result<(), ContainerError> {
        let container = self.containers.get_mut(name)
            .ok_or_else(|| ContainerError::ContainerNotFound(name.to_string()))?;
        if container.state == ContainerState::Running {
            return Err(ContainerError::AlreadyRunning(name.to_string()));
        }
        container.state = ContainerState::Running;
        Ok(())
    }

    pub fn stop(&mut self, name: &str) -> Result<(), ContainerError> {
        let container = self.containers.get_mut(name)
            .ok_or_else(|| ContainerError::ContainerNotFound(name.to_string()))?;
        if container.state != ContainerState::Running {
            return Err(ContainerError::NotRunning(name.to_string()));
        }
        container.state = ContainerState::Stopped;
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<(), ContainerError> {
        let container = self.containers.get(name)
            .ok_or_else(|| ContainerError::ContainerNotFound(name.to_string()))?;
        if container.state == ContainerState::Running {
            return Err(ContainerError::AlreadyRunning(name.to_string()));
        }
        self.containers.remove(name);
        Ok(())
    }

    pub fn inspect(&self, name: &str) -> Result<&Container, ContainerError> {
        self.containers.get(name)
            .ok_or_else(|| ContainerError::ContainerNotFound(name.to_string()))
    }

    pub fn list(&self) -> Vec<&Container> {
        self.containers.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container_aaa() {
        // Arrange
        let mut runtime = ContainerRuntime::new();
        runtime.pull_image("ubuntu:22.04").unwrap();
        let spec = ContainerSpec::new("web", "ubuntu:22.04")
            .with_command(vec!["nginx".to_string()])
            .with_port("80:80".to_string());

        // Act
        let id = runtime.create(spec);

        // Assert
        assert!(id.is_ok());
        assert_eq!(runtime.list().len(), 1);
    }

    #[test]
    fn test_start_stop_container_aaa() {
        // Arrange
        let mut runtime = ContainerRuntime::new();
        runtime.pull_image("ubuntu:22.04").unwrap();
        runtime.create(ContainerSpec::new("web", "ubuntu:22.04")).unwrap();

        // Act
        runtime.start("web").unwrap();
        let state = runtime.inspect("web").unwrap().state;
        runtime.stop("web").unwrap();

        // Assert
        assert_eq!(state, ContainerState::Running);
        assert_eq!(runtime.inspect("web").unwrap().state, ContainerState::Stopped);
    }

    #[test]
    fn test_missing_image_fails_aaa() {
        // Arrange
        let mut runtime = ContainerRuntime::new();
        let spec = ContainerSpec::new("web", "nonexistent:latest");

        // Act
        let res = runtime.create(spec);

        // Assert
        assert!(res.is_err());
    }
}
