use soroban_sdk::{contracttype, Bytes, Env, Symbol};

#[contracttype]
#[derive(Debug, Clone)]
pub struct Resource {
    pub resource_type: Symbol,
    pub data: Bytes,
}
impl Resource {
    pub fn new(resource_type: Symbol, data: Bytes) -> Self {
        Self {
            resource_type,
            data,
        }
    }
    pub fn resource_type(&self) -> &Symbol {
        &self.resource_type
    }
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut Bytes {
        &mut self.data
    }
}

pub trait ResourceTrait: Send + Sync + 'static {
    fn resource_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self>
    where
        Self: Sized;
}

#[contracttype]
#[derive(Clone)]
pub struct GameState {
    pub score: i32,
    pub level: i32,
    pub is_game_over: bool,
}
impl GameState {
    pub fn new() -> Self {
        Self {
            score: 0,
            level: 1,
            is_game_over: false,
        }
    }
    pub fn increment_score(&mut self, points: i32) {
        self.score += points;
    }
    pub fn next_level(&mut self) {
        self.level += 1;
    }
    pub fn game_over(&mut self) {
        self.is_game_over = true;
    }
}
impl_resource!(GameState, "gamestate", { score: i32, level: i32, is_game_over: bool });
impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_resource_creation() {
        let env = Env::default();
        let resource_type = symbol_short!("testres");
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &[1, 2, 3, 4]));
        let resource = Resource::new(resource_type, data.clone());

        assert_eq!(resource.resource_type(), &symbol_short!("testres"));
        assert_eq!(resource.data(), &data);
    }

    #[test]
    fn test_game_state_serialization() {
        let env = Env::default();
        let mut game_state = GameState::new();
        game_state.increment_score(100);
        game_state.next_level();

        let data = game_state.serialize(&env);
        let deserialized = GameState::deserialize(&env, &data).unwrap();

        assert_eq!(game_state.score, deserialized.score);
        assert_eq!(game_state.level, deserialized.level);
        assert_eq!(game_state.is_game_over, deserialized.is_game_over);
    }
}
