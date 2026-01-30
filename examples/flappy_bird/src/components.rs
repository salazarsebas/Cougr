use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol};

/// ComponentTrait from cougr-core
/// Components must implement serialization for on-chain storage
pub trait ComponentTrait {
    #[allow(dead_code)]
    fn component_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self>
    where
        Self: Sized;
}

/// Bird state component - tracks whether the bird is alive
#[contracttype]
#[derive(Clone, Debug)]
pub struct BirdState {
    pub is_alive: bool,
}

impl BirdState {
    pub fn new(is_alive: bool) -> Self {
        Self { is_alive }
    }
}

impl ComponentTrait for BirdState {
    fn component_type() -> Symbol {
        symbol_short!("birdstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let value: u8 = if self.is_alive { 1 } else { 0 };
        bytes.append(&Bytes::from_array(env, &[value]));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let is_alive = data.get(0).unwrap() != 0;
        Some(Self { is_alive })
    }
}

/// Pipe configuration component - stores pipe properties
#[contracttype]
#[derive(Clone, Debug)]
pub struct PipeConfig {
    pub gap_size: i32,
    pub gap_center_y: i32,
}

impl PipeConfig {
    pub fn new(gap_size: i32, gap_center_y: i32) -> Self {
        Self {
            gap_size,
            gap_center_y,
        }
    }
}

impl ComponentTrait for PipeConfig {
    fn component_type() -> Symbol {
        symbol_short!("pipeconf")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let gap_size_bytes = Bytes::from_array(env, &self.gap_size.to_be_bytes());
        let gap_center_bytes = Bytes::from_array(env, &self.gap_center_y.to_be_bytes());
        bytes.append(&gap_size_bytes);
        bytes.append(&gap_center_bytes);
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let gap_size = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let gap_center_y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        Some(Self {
            gap_size,
            gap_center_y,
        })
    }
}

/// Pipe marker component - identifies an entity as a pipe
#[contracttype]
#[derive(Clone, Debug)]
pub struct PipeMarker {
    pub passed: bool, // Has the bird passed this pipe?
}

impl PipeMarker {
    pub fn new() -> Self {
        Self { passed: false }
    }
}

impl ComponentTrait for PipeMarker {
    fn component_type() -> Symbol {
        symbol_short!("pipemark")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        let value: u8 = if self.passed { 1 } else { 0 };
        bytes.append(&Bytes::from_array(env, &[value]));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 1 {
            return None;
        }
        let passed = data.get(0).unwrap() != 0;
        Some(Self { passed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bird_state_serialization() {
        let env = Env::default();
        let bird_state = BirdState::new(true);

        let serialized = bird_state.serialize(&env);
        let deserialized = BirdState::deserialize(&env, &serialized).unwrap();

        assert_eq!(bird_state.is_alive, deserialized.is_alive);
    }

    #[test]
    fn test_pipe_config_serialization() {
        let env = Env::default();
        let pipe_config = PipeConfig::new(100, 200);

        let serialized = pipe_config.serialize(&env);
        let deserialized = PipeConfig::deserialize(&env, &serialized).unwrap();

        assert_eq!(pipe_config.gap_size, deserialized.gap_size);
        assert_eq!(pipe_config.gap_center_y, deserialized.gap_center_y);
    }

    #[test]
    fn test_pipe_marker_serialization() {
        let env = Env::default();
        let marker = PipeMarker::new();

        let serialized = marker.serialize(&env);
        let deserialized = PipeMarker::deserialize(&env, &serialized).unwrap();

        assert_eq!(marker.passed, deserialized.passed);
    }
}
