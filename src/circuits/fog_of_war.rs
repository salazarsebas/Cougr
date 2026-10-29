use soroban_sdk::Env;

use crate::zk::ZKError;

use super::spec::{pipeline_vk, CircuitId, CircuitParams, GameCircuitSpec, PublicInputLayout};

/// Build a fog-of-war exploration circuit for a `map_width` × `map_height` board.
///
/// Verification delegates to [`crate::zk::experimental::FogOfWarCircuit`]; public inputs
/// match its frozen layout.
pub fn fog_of_war(
    env: &Env,
    map_width: u32,
    map_height: u32,
    visibility_radius: u32,
) -> Result<GameCircuitSpec, ZKError> {
    if map_width == 0 || map_height == 0 || visibility_radius == 0 {
        return Err(ZKError::InvalidInput);
    }

    let layout = PublicInputLayout::fog_of_war(env);
    Ok(GameCircuitSpec {
        circuit_id: CircuitId::FogOfWar,
        layout: layout.clone(),
        verification_key: pipeline_vk(env, CircuitId::FogOfWar),
        params: CircuitParams::FogOfWar(map_width, map_height, visibility_radius),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::spec::CircuitParams;
    use crate::zk::experimental::{FogOfWarSnapshot, FogOfWarTransition};
    use crate::zk::types::{G1Point, G2Point, Groth16Proof};
    use soroban_sdk::BytesN;

    #[test]
    fn fog_of_war_rejects_zero_dimensions() {
        let env = Env::default();
        assert!(fog_of_war(&env, 0, 32, 3).is_err());
        assert!(fog_of_war(&env, 32, 0, 3).is_err());
        assert!(fog_of_war(&env, 32, 32, 0).is_err());
    }

    #[test]
    fn fog_of_war_delegates_to_fog_circuit() {
        let env = Env::default();
        let spec = fog_of_war(&env, 32, 32, 3).unwrap();
        assert_eq!(spec.circuit_id, CircuitId::FogOfWar);
        assert_eq!(spec.layout.public_input_count(), 8);
        assert_eq!(spec.params, CircuitParams::FogOfWar(32, 32, 3));

        let snapshot = FogOfWarSnapshot {
            map_root: BytesN::from_array(&env, &[1u8; 32]),
            explored_root: BytesN::from_array(&env, &[2u8; 32]),
            origin_x: 0,
            origin_y: 0,
            visibility_radius: 4,
        };
        let transition = FogOfWarTransition {
            prior_explored_root: snapshot.explored_root.clone(),
            next_explored_root: BytesN::from_array(&env, &[3u8; 32]),
            tile_x: 1,
            tile_y: 1,
        };

        let g1 = G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 128]),
        };
        let proof = Groth16Proof {
            a: g1.clone(),
            b: g2,
            c: g1,
        };

        let result = spec.verify_fog_exploration(&env, &proof, &snapshot, &transition);
        assert_eq!(result, Err(crate::zk::ZKError::InvalidVisibility));
    }
}
