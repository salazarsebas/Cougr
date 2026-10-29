use soroban_sdk::{contracttype, symbol_short, Bytes, Env, Symbol, TryFromVal, Val, Vec};

#[contracttype]
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: Symbol,
    pub data: Bytes,
    pub timestamp: u64,
}
impl Event {
    pub fn new(event_type: Symbol, data: Bytes) -> Self {
        Self {
            event_type,
            data,
            timestamp: 0,
        }
    }
    pub fn with_timestamp(event_type: Symbol, data: Bytes, timestamp: u64) -> Self {
        Self {
            event_type,
            data,
            timestamp,
        }
    }
    pub fn event_type(&self) -> &Symbol {
        &self.event_type
    }
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

pub struct EventReader<'a> {
    events: &'a [Event],
    event_type: Symbol,
    read_index: usize,
}
impl<'a> EventReader<'a> {
    pub fn new(events: &'a [Event], event_type: Symbol) -> Self {
        Self {
            events,
            event_type,
            read_index: 0,
        }
    }
    pub fn read(&mut self) -> Option<&Event> {
        while self.read_index < self.events.len() {
            let event = &self.events[self.read_index];
            self.read_index += 1;
            if event.event_type() == &self.event_type {
                return Some(event);
            }
        }
        None
    }
    pub fn has_more(&self) -> bool {
        self.read_index < self.events.len()
    }
    pub fn reset(&mut self) {
        self.read_index = 0;
    }
}

pub struct EventWriter<'a> {
    events: &'a mut Vec<Event>,
}
impl<'a> EventWriter<'a> {
    pub fn new(events: &'a mut Vec<Event>) -> Self {
        Self { events }
    }
    pub fn send(&mut self, event: Event) {
        self.events.push_back(event);
    }
    pub fn send_with_data(&mut self, event_type: Symbol, data: Bytes) {
        let event = Event::new(event_type, data);
        self.send(event);
    }
    pub fn send_batch(&mut self, events: Vec<Event>) {
        for event in events {
            self.send(event);
        }
    }
}

pub trait EventTrait {
    fn event_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self>
    where
        Self: Sized;
}

#[contracttype]
#[derive(Clone)]
pub struct CollisionEvent {
    pub entity_a: u64,
    pub entity_b: u64,
    pub collision_type: Symbol,
}
impl CollisionEvent {
    pub fn new(entity_a: u64, entity_b: u64, collision_type: Symbol) -> Self {
        Self {
            entity_a,
            entity_b,
            collision_type,
        }
    }
}
impl EventTrait for CollisionEvent {
    fn event_type() -> Symbol {
        symbol_short!("collision")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_slice(env, &self.entity_a.to_be_bytes()));
        bytes.append(&Bytes::from_slice(env, &self.entity_b.to_be_bytes()));
        // Serialize the collision_type symbol by converting to Val and then to u64
        let symbol_val: Val = self.collision_type.to_val();
        let symbol_bits = symbol_val.get_payload();
        bytes.append(&Bytes::from_slice(env, &symbol_bits.to_be_bytes()));
        bytes
    }
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 24 {
            return None;
        }
        let entity_a = u64::from_be_bytes([
            data.get(0)?,
            data.get(1)?,
            data.get(2)?,
            data.get(3)?,
            data.get(4)?,
            data.get(5)?,
            data.get(6)?,
            data.get(7)?,
        ]);
        let entity_b = u64::from_be_bytes([
            data.get(8)?,
            data.get(9)?,
            data.get(10)?,
            data.get(11)?,
            data.get(12)?,
            data.get(13)?,
            data.get(14)?,
            data.get(15)?,
        ]);
        // Deserialize the symbol from its Val representation
        let symbol_bits = u64::from_be_bytes([
            data.get(16)?,
            data.get(17)?,
            data.get(18)?,
            data.get(19)?,
            data.get(20)?,
            data.get(21)?,
            data.get(22)?,
            data.get(23)?,
        ]);
        let symbol_val = Val::from_payload(symbol_bits);
        let collision_type: Symbol = Symbol::try_from_val(env, &symbol_val).ok()?;
        Some(Self {
            entity_a,
            entity_b,
            collision_type,
        })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct DamageEvent {
    pub target_entity: u64,
    pub damage_amount: i32,
    pub damage_type: Symbol,
}
impl DamageEvent {
    pub fn new(target_entity: u64, damage_amount: i32, damage_type: Symbol) -> Self {
        Self {
            target_entity,
            damage_amount,
            damage_type,
        }
    }
}
impl EventTrait for DamageEvent {
    fn event_type() -> Symbol {
        symbol_short!("damage")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_slice(env, &self.target_entity.to_be_bytes()));
        bytes.append(&Bytes::from_slice(env, &self.damage_amount.to_be_bytes()));
        // Serialize the damage_type symbol by converting to Val and then to u64
        let symbol_val: Val = self.damage_type.to_val();
        let symbol_bits = symbol_val.get_payload();
        bytes.append(&Bytes::from_slice(env, &symbol_bits.to_be_bytes()));
        bytes
    }
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }
        let target_entity = u64::from_be_bytes([
            data.get(0)?,
            data.get(1)?,
            data.get(2)?,
            data.get(3)?,
            data.get(4)?,
            data.get(5)?,
            data.get(6)?,
            data.get(7)?,
        ]);
        let damage_amount =
            i32::from_be_bytes([data.get(8)?, data.get(9)?, data.get(10)?, data.get(11)?]);
        // Deserialize the symbol from its Val representation
        let symbol_bits = u64::from_be_bytes([
            data.get(12)?,
            data.get(13)?,
            data.get(14)?,
            data.get(15)?,
            data.get(16)?,
            data.get(17)?,
            data.get(18)?,
            data.get(19)?,
        ]);
        let symbol_val = Val::from_payload(symbol_bits);
        let damage_type: Symbol = Symbol::try_from_val(env, &symbol_val).ok()?;
        Some(Self {
            target_entity,
            damage_amount,
            damage_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec as StdVec;
    use soroban_sdk::{symbol_short, Env, Vec};

    #[test]
    fn test_event_creation() {
        let env = Env::default();
        let event_type = symbol_short!("testevent");
        let mut data = Bytes::new(&env);
        data.append(&Bytes::from_array(&env, &[1, 2, 3, 4]));
        let event = Event::new(event_type, data.clone());

        assert_eq!(event.event_type(), &symbol_short!("testevent"));
        assert_eq!(event.data(), &data);
        assert_eq!(event.timestamp(), 0);
    }

    #[test]
    fn test_collision_event_serialization() {
        let env = Env::default();
        let collision_event = CollisionEvent::new(123, 456, symbol_short!("physical"));

        let data = collision_event.serialize(&env);
        let deserialized = CollisionEvent::deserialize(&env, &data).unwrap();

        assert_eq!(collision_event.entity_a, deserialized.entity_a);
        assert_eq!(collision_event.entity_b, deserialized.entity_b);
        assert_eq!(collision_event.collision_type, deserialized.collision_type);
    }

    #[test]
    fn test_damage_event_serialization() {
        let env = Env::default();
        let damage_event = DamageEvent::new(789, 50, symbol_short!("fire"));

        let data = damage_event.serialize(&env);
        let deserialized = DamageEvent::deserialize(&env, &data).unwrap();

        assert_eq!(damage_event.target_entity, deserialized.target_entity);
        assert_eq!(damage_event.damage_amount, deserialized.damage_amount);
        assert_eq!(damage_event.damage_type, deserialized.damage_type);
    }

    #[test]
    fn test_event_writer_and_reader_filter_by_type() {
        let env = Env::default();
        let mut events = Vec::new(&env);
        {
            let mut writer = EventWriter::new(&mut events);
            writer.send_with_data(symbol_short!("spawn"), Bytes::from_array(&env, &[1]));
            writer.send_with_data(symbol_short!("move"), Bytes::from_array(&env, &[2]));
            writer.send_with_data(symbol_short!("spawn"), Bytes::from_array(&env, &[3]));
        }

        let mut snapshot = StdVec::new();
        for i in 0..events.len() {
            snapshot.push(events.get(i).unwrap());
        }
        let mut reader = EventReader::new(&snapshot, symbol_short!("spawn"));

        let first = reader.read().cloned().unwrap();
        let second = reader.read().cloned().unwrap();

        assert_eq!(first.event_type(), &symbol_short!("spawn"));
        assert_eq!(first.data(), &Bytes::from_array(&env, &[1]));
        assert_eq!(second.event_type(), &symbol_short!("spawn"));
        assert_eq!(second.data(), &Bytes::from_array(&env, &[3]));
        assert!(reader.read().is_none());
        assert!(!reader.has_more());
    }

    #[test]
    fn test_event_reader_reset_rewinds_sequence() {
        let env = Env::default();
        let event = Event::with_timestamp(symbol_short!("tick"), Bytes::from_array(&env, &[9]), 42);
        let events = [event.clone()];
        let mut reader = EventReader::new(&events, symbol_short!("tick"));

        assert_eq!(reader.read().unwrap().timestamp(), 42);
        assert!(reader.read().is_none());

        reader.reset();

        assert_eq!(reader.read().unwrap().timestamp(), 42);
    }
}
