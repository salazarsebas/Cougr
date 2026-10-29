use cougr_core::component::ComponentTrait;
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

/// RaceComponent - wraps Race data for Cougr ECS integration
#[derive(Clone, Debug)]
pub struct RaceComponent {
    pub race_id: u32,
    pub season_id: u32,
    pub entrants_count: u32,
    pub phase: u32,
    pub start_height: u32,
    pub duration: u32,
}

impl ComponentTrait for RaceComponent {
    fn component_type() -> Symbol {
        symbol_short!("race")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.race_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.season_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.entrants_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.phase.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.start_height.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.duration.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 24 {
            return None;
        }
        Some(RaceComponent {
            race_id: u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]),
            season_id: u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]),
            entrants_count: u32::from_be_bytes([
                data.get(8)?,
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
            ]),
            phase: u32::from_be_bytes([data.get(12)?, data.get(13)?, data.get(14)?, data.get(15)?]),
            start_height: u32::from_be_bytes([
                data.get(16)?,
                data.get(17)?,
                data.get(18)?,
                data.get(19)?,
            ]),
            duration: u32::from_be_bytes([
                data.get(20)?,
                data.get(21)?,
                data.get(22)?,
                data.get(23)?,
            ]),
        })
    }
}

/// VehicleComponent - wraps VehicleState for Cougr ECS
#[derive(Clone, Debug)]
pub struct VehicleComponent {
    pub speed: u32,
    pub boost_state_type: u32,
    pub boost_active: bool,
    pub penalty_count: u32,
}

impl ComponentTrait for VehicleComponent {
    fn component_type() -> Symbol {
        symbol_short!("vehicle")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.speed.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &self.boost_state_type.to_be_bytes(),
        ));
        let bool_byte = if self.boost_active { 1u8 } else { 0u8 };
        bytes.append(&Bytes::from_array(env, &[bool_byte]));
        bytes.append(&Bytes::from_array(env, &self.penalty_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 13 {
            return None;
        }
        Some(VehicleComponent {
            speed: u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]),
            boost_state_type: u32::from_be_bytes([
                data.get(4)?,
                data.get(5)?,
                data.get(6)?,
                data.get(7)?,
            ]),
            boost_active: data.get(8)? == 1u8,
            penalty_count: u32::from_be_bytes([
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
                data.get(12)?,
            ]),
        })
    }
}

/// BoostComponent - wraps BoostState for Cougr ECS
#[derive(Clone, Debug)]
pub struct BoostComponent {
    pub boost_type: u32,
    pub status: u32,
    pub activation_height: u32,
}

impl ComponentTrait for BoostComponent {
    fn component_type() -> Symbol {
        symbol_short!("boost")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.boost_type.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &self.activation_height.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        Some(BoostComponent {
            boost_type: u32::from_be_bytes([
                data.get(0)?,
                data.get(1)?,
                data.get(2)?,
                data.get(3)?,
            ]),
            status: u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]),
            activation_height: u32::from_be_bytes([
                data.get(8)?,
                data.get(9)?,
                data.get(10)?,
                data.get(11)?,
            ]),
        })
    }
}
