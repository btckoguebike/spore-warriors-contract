#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use lazy_static::lazy_static;
    use molecule::prelude::Entity;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use spore_warriors_core::fight::pve::MapFightPVE;
    use spore_warriors_core::fight::traits::{IterationInput, Selection, SimplePVE};
    use spore_warriors_core::map::MapSkeleton;
    use spore_warriors_core::systems::GameSystem;
    use spore_warriors_core::wrappings::{Enemy, Point, Warrior};
    use spore_warriors_generated as generated;

    lazy_static! {
        pub static ref RESOURCE_POOL: generated::ResourcePool = {
            let raw_resource_pool = std::fs::read("./resources.bin").expect("load resources.bin");
            generated::ResourcePool::new_unchecked(raw_resource_pool.into())
        };
        pub static ref RNG: Mutex<SmallRng> =
            Mutex::new(rand::rngs::SmallRng::seed_from_u64(10000));
    }

    #[test]
    fn test_map_skeleton() -> eyre::Result<()> {
        let rng = &mut RNG.lock().unwrap().to_owned();
        let player = {
            let warrior = RESOURCE_POOL.warrior_pool().get_unchecked(0);
            Warrior::randomized(&RESOURCE_POOL, warrior, rng).unwrap()
        };
        let point = Point::from_xy(1, 0);
        let map = MapSkeleton::randomized(&RESOURCE_POOL, &player, point, rng).unwrap();
        println!("[map] = {map:?}");
        println!("[avaliable_range] = {:?}", map.movable_range());
        println!("[node] = {:?}", map.peak_upcoming_movment((1, 1).into()));
        Ok(())
    }

    #[test]
    fn test_pve_fight() -> eyre::Result<()> {
        let rng = &mut RNG.lock().unwrap().to_owned();
        let player = {
            let warrior = RESOURCE_POOL.warrior_pool().get_unchecked(0);
            Warrior::randomized(&RESOURCE_POOL, warrior, rng).unwrap()
        };
        let enemies = {
            let enemy = RESOURCE_POOL.enemy_pool().get_unchecked(0);
            vec![Enemy::randomized(&RESOURCE_POOL, enemy, rng).unwrap()]
        };
        let mut game_system = GameSystem::new(&RESOURCE_POOL, rng);
        let mut battle = MapFightPVE::create(&player, None, &enemies).unwrap();
        let (output, logs) = battle.start(&mut game_system).unwrap();
        println!("===START===");
        println!("[output] = {output:?}");
        println!("[logs] = {logs:?}");
        let (output, logs) = battle
            .run(
                vec![IterationInput::HandCardUse(
                    Selection::SingleCard(0),
                    Some(0),
                )],
                &mut game_system,
            )
            .unwrap();
        println!("===RUN===");
        println!("[output] = {output:?}");
        println!("[logs] = {logs:?}");
        Ok(())
    }
}
