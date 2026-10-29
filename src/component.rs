use alloc::vec::Vec;
use soroban_sdk::{contracttype, Bytes, BytesN, Env, IntoVal, Symbol, TryFromVal, Val};

/// A unique identifier for a component type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId {
    id: u32,
}

impl ComponentId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}
// Soroban SDK trait implementations for ComponentId
impl IntoVal<Env, Val> for ComponentId {
    fn into_val(&self, env: &Env) -> Val {
        self.id.into_val(env)
    }
}

impl TryFromVal<Env, Val> for ComponentId {
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let id: u32 = TryFromVal::try_from_val(env, val)?;
        Ok(ComponentId::new(id))
    }
}

#[contracttype]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentStorage {
    #[default]
    Table = 0,
    Sparse = 1,
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct Component {
    pub component_type: Symbol,
    pub data: Bytes,
    pub storage: ComponentStorage,
}

impl Component {
    pub fn new(component_type: Symbol, data: Bytes) -> Self {
        Self {
            component_type,
            data,
            storage: ComponentStorage::default(),
        }
    }
    pub fn with_storage(component_type: Symbol, data: Bytes, storage: ComponentStorage) -> Self {
        Self {
            component_type,
            data,
            storage,
        }
    }
    pub fn component_type(&self) -> &Symbol {
        &self.component_type
    }
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut Bytes {
        &mut self.data
    }
    pub fn storage(&self) -> ComponentStorage {
        self.storage
    }
    pub fn set_storage(&mut self, storage: ComponentStorage) {
        self.storage = storage;
    }
}

/// Registry for managing component types
#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    next_id: u32,
    components: Vec<(Symbol, ComponentId)>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            next_id: 1,
            components: Vec::new(),
        }
    }

    /// Register a new component type
    pub fn register_component(&mut self, component_type: Symbol) -> ComponentId {
        // Check if component type is already registered
        for (ctype, id) in &self.components {
            if ctype == &component_type {
                return *id;
            }
        }

        let id = ComponentId::new(self.next_id);
        self.next_id += 1;
        self.components.push((component_type, id));
        id
    }

    /// Get the component ID for a component type
    pub fn get_component_id(&self, component_type: &Symbol) -> Option<ComponentId> {
        for (ctype, id) in &self.components {
            if ctype == component_type {
                return Some(*id);
            }
        }
        None
    }

    /// Get the component type for a component ID
    pub fn get_component_type(&self, component_id: ComponentId) -> Option<Symbol> {
        for (ctype, id) in &self.components {
            if id == &component_id {
                return Some(ctype.clone());
            }
        }
        None
    }

    /// Get the number of registered component types
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Check if a component type is registered
    pub fn is_registered(&self, component_type: &Symbol) -> bool {
        for (ctype, _) in &self.components {
            if ctype == component_type {
                return true;
            }
        }
        false
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ComponentTrait {
    fn component_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self>
    where
        Self: Sized;
    fn default_storage() -> ComponentStorage {
        ComponentStorage::Table
    }
}

#[contracttype]
#[derive(Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl_component!(Position, "position", Table, { x: i32, y: i32 });

#[contracttype]
#[derive(Clone)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}
impl Velocity {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl_component!(Velocity, "velocity", Table, { x: i32, y: i32 });

// ─── Test types for macro-generated components ────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct Health {
    pub current: u128,
    pub max: u128,
}
impl_component!(Health, "health", Table, { current: u128, max: u128 });

#[contracttype]
#[derive(Clone, Debug)]
pub struct Token {
    pub amount: u32,
    pub hash: BytesN<32>,
}
impl_component!(Token, "token", Table, { amount: u32, hash: bytes32 });

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, BytesN, Env};

    #[test]
    fn test_component_id_creation() {
        let id = ComponentId::new(1);
        assert_eq!(id.id(), 1);
    }

    #[test]
    fn test_component_creation() {
        let env = Env::default();
        let component_type = symbol_short!("test");
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &[1, 2, 3, 4]));
        let component = Component::new(component_type, data.clone());

        assert_eq!(component.component_type(), &symbol_short!("test"));
        assert_eq!(component.data(), &data);
        assert_eq!(component.storage(), ComponentStorage::Table);
    }

    #[test]
    fn test_component_registry() {
        let mut registry = ComponentRegistry::new();
        assert_eq!(registry.component_count(), 0);

        let component_type = symbol_short!("test");
        let id = registry.register_component(component_type.clone());
        assert_eq!(registry.component_count(), 1);
        assert!(registry.is_registered(&component_type));

        let retrieved_id = registry.get_component_id(&component_type);
        assert_eq!(retrieved_id, Some(id));
    }

    #[test]
    fn test_position_component() {
        let env = Env::default();
        let position = Position::new(100, 200);
        let data = position.serialize(&env);
        let deserialized = Position::deserialize(&env, &data).unwrap();

        assert_eq!(position.x, deserialized.x);
        assert_eq!(position.y, deserialized.y);
    }

    // ─── Extended macro type tests ────────────────────────────────

    #[test]
    fn test_u128_component() {
        let env = Env::default();
        let health = Health {
            current: 999_999_999_999,
            max: 1_000_000_000_000,
        };
        let data = health.serialize(&env);
        assert_eq!(data.len(), 32); // 16 + 16 bytes
        let deserialized = Health::deserialize(&env, &data).unwrap();
        assert_eq!(deserialized.current, 999_999_999_999);
        assert_eq!(deserialized.max, 1_000_000_000_000);
    }

    #[test]
    fn test_bytes32_component() {
        let env = Env::default();
        let hash = BytesN::from_array(&env, &[0xABu8; 32]);
        let token = Token {
            amount: 42,
            hash: hash.clone(),
        };
        let data = token.serialize(&env);
        assert_eq!(data.len(), 36); // 4 + 32 bytes
        let deserialized = Token::deserialize(&env, &data).unwrap();
        assert_eq!(deserialized.amount, 42);
        assert_eq!(deserialized.hash, hash);
    }

    #[test]
    fn test_u128_zero() {
        let env = Env::default();
        let health = Health { current: 0, max: 0 };
        let data = health.serialize(&env);
        let deserialized = Health::deserialize(&env, &data).unwrap();
        assert_eq!(deserialized.current, 0);
        assert_eq!(deserialized.max, 0);
    }

    #[test]
    fn test_u128_max() {
        let env = Env::default();
        let health = Health {
            current: u128::MAX,
            max: u128::MAX,
        };
        let data = health.serialize(&env);
        let deserialized = Health::deserialize(&env, &data).unwrap();
        assert_eq!(deserialized.current, u128::MAX);
        assert_eq!(deserialized.max, u128::MAX);
    }
}
