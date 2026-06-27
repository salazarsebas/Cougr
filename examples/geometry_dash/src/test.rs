#[cfg(test)]
mod tests {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_initialization() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);

        let status = client.get_state(&player);
        assert_eq!(status, GameStatus::Playing);

        let score = client.get_score(&player);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_movement() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);

        // After one tick, distance should be 10 (TICK_MOVEMENT_X is 10000, scaled by 1000)
        client.update_tick(&player);

        let score = client.get_score(&player);
        assert_eq!(score, 10);
    }

    #[test]
    fn test_jump_physics() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);

        // Initial y is GROUND_Y
        client.jump(&player);
        client.update_tick(&player);

        let (_, y) = client.get_pos(&player);
        assert!(y < 400_000); // Should have moved up
    }

    #[test]
    fn test_collision() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);

        // The spike is at 100,000. Player starts at 0 and moves 10,000 per tick.
        // At 10 ticks, player is exactly at 100,000.
        for _ in 0..10 {
            client.update_tick(&player);
        }

        let status = client.get_state(&player);
        assert_eq!(status, GameStatus::Crashed);
    }

    #[test]
    fn test_mode_switch() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);

        // Jump over the spike at tick 9-11
        for i in 0..30 {
            if (8..=12).contains(&i) {
                client.jump(&player);
            }
            client.update_tick(&player);
            let (px, _) = client.get_pos(&player);
            let pm = client.get_mode(&player);
            let ps = client.get_state(&player);

            // At tick 30, we expect mode switch
            if i == 29 {
                assert_eq!(px, 300_000, "Position X mismatch at tick 30");
                assert_eq!(ps, GameStatus::Playing, "Game should be playing at tick 30");
                assert_eq!(pm, 1, "Mode should be Ship (1) at tick 30");
            }
        }
    }

    #[test]
    fn test_invalid_action_after_crash_is_ignored() {
        let env = Env::default();
        let player = Address::generate(&env);
        let contract_id = env.register(GeometryDashContract, ());
        let client = GeometryDashContractClient::new(&env, &contract_id);

        client.init_game(&player, &0);
        for _ in 0..10 {
            client.update_tick(&player);
        }
        assert_eq!(client.get_state(&player), GameStatus::Crashed);
        let pos = client.get_pos(&player);
        client.jump(&player);
        client.update_tick(&player);
        assert_eq!(client.get_pos(&player), pos);
    }

    #[test]
    fn test_gameapp_tick_integration() {
        let env = Env::default();
        crate::systems::run_gameapp_tick(&env);
    }
}
