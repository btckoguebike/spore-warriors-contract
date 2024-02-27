#[cfg(test)]
mod test {
    use lazy_static::lazy_static;
    use spore_warriors_core::battle::pve::MapBattlePVE;
    use spore_warriors_core::battle::traits::{IterationInput, Selection, SimplePVE};
    use spore_warriors_core::contexts::WarriorContext;
    use spore_warriors_core::game::Game;
    use spore_warriors_core::wrappings::{Enemy, Point};

    lazy_static! {
        pub static ref RAW_RESOURCE_POOL: Vec<u8> =
            std::fs::read("./resources.bin").expect("load resources.bin");
    }

    #[test]
    fn test_map_skeleton() {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, None, 10000).unwrap();
        let mut player = game.new_session(5001, point).unwrap();
        println!("[map] = {:?}", game.map);
        println!(
            "[node] = {:?}",
            game.map.peak_upcoming_movment(&player, (1, 1).into())
        );
        game.map
            .move_to(&mut player, point, vec![], &mut game.controller)
            .unwrap();
    }

    #[test]
    fn test_pve_fight() {
        let mut game = Game::new(&RAW_RESOURCE_POOL, None, 10000).unwrap();
        let enemies = {
            let resource_pool = &game.controller.resource_pool;
            let enemy = resource_pool.enemy_pool().get_unchecked(0);
            vec![Enemy::randomized(resource_pool, enemy, &mut game.controller.rng).unwrap()]
        };
        let point = Point::from_xy(1, 0);
        let mut player = game.new_session(5001, point).unwrap();
        let mut battle = MapBattlePVE::create(&mut player, enemies).unwrap();
        let (output, logs) = battle.start(&mut game.controller).unwrap();
        println!("===START===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(
                vec![IterationInput::HandCardUse(
                    Selection::SingleCard(0),
                    Some(0),
                )],
                &mut game.controller,
            )
            .unwrap();
        println!("===PLAYER TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(vec![IterationInput::EnemyTurn], &mut game.controller)
            .unwrap();
        println!("===ENEMY TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
    }

    #[test]
    fn test_context_encode_decode() {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, None, 10086).unwrap();
        let player = game.new_session(5001, point).unwrap();
        let context_bytes = rlp::encode(&player);
        let decoded_player: WarriorContext = rlp::decode(&context_bytes.to_vec()).unwrap();
        assert_eq!(player, decoded_player);
    }
}
