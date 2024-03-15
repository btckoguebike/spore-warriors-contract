#[cfg(test)]
mod test {
    use lazy_static::lazy_static;
    use spore_warriors_core::battle::pve::MapBattlePVE;
    use spore_warriors_core::battle::traits::{IterationInput, Selection, SimplePVE};
    use spore_warriors_core::contexts::{WarriorContext, WarriorDeckContext};
    use spore_warriors_core::game::Game;
    use spore_warriors_core::wrappings::{Enemy, Point};

    lazy_static! {
        pub static ref RAW_RESOURCE_POOL: Vec<u8> =
            std::fs::read("./resources.bin").expect("load resources.bin");
    }

    #[test]
    fn test_map_skeleton() -> eyre::Result<()> {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, 10000)?;
        let (mut player, mut player_deck) = game.new_session(5001, point, None)?;
        println!("[map] = {:?}", game.map);
        println!(
            "[node] = {:?}",
            game.map.peak_upcoming_movment(&player, (1, 1).into())
        );
        game.map.move_to(
            &mut player,
            &mut player_deck,
            point,
            vec![],
            &mut game.controller,
        )?;
        Ok(())
    }

    #[test]
    fn test_pve_fight() -> eyre::Result<()> {
        let mut game = Game::new(&RAW_RESOURCE_POOL, 10000)?;
        let enemies = {
            let resource_pool = &game.controller.resource_pool;
            let enemy = resource_pool.enemy_pool().get_unchecked(0);
            let enemy = Enemy::randomized(resource_pool, enemy, &mut game.controller.rng)?;
            vec![enemy]
        };
        let point = Point::from_xy(1, 0);
        let (mut player, mut player_deck) = game.new_session(5001, point, None)?;
        let mut battle = MapBattlePVE::create(&mut player, &mut player_deck, enemies)?;
        let (output, logs) = battle.start(&mut game.controller)?;
        println!("===START===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle.run(
            vec![IterationInput::HandCardUse(
                Selection::SingleCard(0),
                Some(0),
            )],
            &mut game.controller,
        )?;
        println!("===PLAYER TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle.run(vec![IterationInput::EnemyTurn], &mut game.controller)?;
        println!("===ENEMY TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        Ok(())
    }

    #[test]
    fn test_context_encode_decode() -> eyre::Result<()> {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, 10086)?;
        let (player, player_deck) = game.new_session(5001, point, None)?;
        let decoded_player: WarriorContext = rlp::decode(&rlp::encode(&player).to_vec())?;
        assert_eq!(player, decoded_player);
        let decoded_player_deck: WarriorDeckContext =
            rlp::decode(&rlp::encode(&player_deck).to_vec())?;
        assert_eq!(player_deck, decoded_player_deck);
        Ok(())
    }

    #[test]
    fn test_json_encode() -> eyre::Result<()> {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, 10086)?;
        let (player, player_deck) = game.new_session(5001, point, None)?;
        println!("[MAP] = {}", serde_json::to_string_pretty(&game.map)?);
        println!("[PLAYER] = {}", serde_json::to_string_pretty(&player)?);
        println!("[DECK] = {}", serde_json::to_string_pretty(&player_deck)?);
        Ok(())
    }
}
