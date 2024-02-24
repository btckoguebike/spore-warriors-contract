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
        let mut session = game.new_context(point).unwrap();
        let motion = session.player.warrior.motion;
        println!("[map] = {:?}", session.map);
        println!(
            "[avaliable_range] = {:?}",
            session.map.movable_range(motion)
        );
        println!(
            "[node] = {:?}",
            session.map.peak_upcoming_movment((1, 1).into(), motion)
        );
        session
            .map
            .move_to(&mut session.player, point, vec![], &mut session.system)
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
        let mut session = game.new_context(point).unwrap();
        let mut battle = MapBattlePVE::create(&mut session.player, &enemies).unwrap();
        let (output, logs) = battle.start(&mut session.system).unwrap();
        println!("===START===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(
                vec![IterationInput::HandCardUse(
                    Selection::SingleCard(0),
                    Some(0),
                )],
                &mut session.system,
            )
            .unwrap();
        println!("===PLAYER TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        let (output, logs) = battle
            .run(vec![IterationInput::EnemyTurn], &mut session.system)
            .unwrap();
        println!("===ENEMY TURN===");
        println!("[logs] = {logs:?}");
        println!("[output] = {output:?}");
        Ok(())
    }
}
