#[cfg(test)]
mod test {
    use lazy_static::lazy_static;
    use spore_warriors_core::battle::pve::MapBattlePVE;
    use spore_warriors_core::battle::traits::{IterationInput, Selection, SimplePVE};
    use spore_warriors_core::game::Game;
    use spore_warriors_core::wrappings::{Enemy, Point};

    lazy_static! {
        pub static ref RAW_RESOURCE_POOL: Vec<u8> =
            std::fs::read("./resources.bin").expect("load resources.bin");
    }

    #[test]
    fn test_map_skeleton() -> eyre::Result<()> {
        let point = Point::from_xy(1, 0);
        let mut game = Game::new(&RAW_RESOURCE_POOL, None, 10000, 5001).unwrap();
        let (mut map, mut controller, mut player) = game.new_session(point).unwrap();
        println!("[map] = {:?}", map);
        println!(
            "[node] = {:?}",
            map.peak_upcoming_movment(&player, (1, 1).into())
        );
        map.move_to(&mut player, point, vec![], &mut controller)
            .unwrap();
        Ok(())
    }

    #[test]
    fn test_pve_fight() -> eyre::Result<()> {
        let mut game = Game::new(&RAW_RESOURCE_POOL, None, 10000, 5001).unwrap();
        let enemies = {
            let resource_pool = &game.resource_pool;
            let enemy = resource_pool.enemy_pool().get_unchecked(0);
            vec![Enemy::randomized(resource_pool, enemy, &mut game.rng).unwrap()]
        };
        let point = Point::from_xy(1, 0);
        let (_, mut controller, mut player) = game.new_session(point).unwrap();
        let mut battle = MapBattlePVE::create(&mut player, &enemies).unwrap();
        let (output, logs) = battle.start(&mut controller).unwrap();
        println!("===START===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(
                vec![IterationInput::HandCardUse(
                    Selection::SingleCard(0),
                    Some(0),
                )],
                &mut controller,
            )
            .unwrap();
        println!("===PLAYER TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(vec![IterationInput::EnemyTurn], &mut controller)
            .unwrap();
        println!("===ENEMY TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        Ok(())
    }
}
