#[cfg(test)]
mod test {
    use molecule::prelude::Entity;
    use rand::SeedableRng;
    use spore_warriors_core::map::MapSkeleton;
    use spore_warriors_core::wrappings::{Point, Warrior};
    use spore_warriors_generated as generated;

    #[test]
    fn test_map_skeleton() -> eyre::Result<()> {
        let raw_resource_pool = std::fs::read("./resources.bin")?;
        let resource_pool = generated::ResourcePool::new_unchecked(raw_resource_pool.into());

        let mut rng = rand::rngs::SmallRng::seed_from_u64(10086);
        let player = {
            let warrior = resource_pool.warrior_pool().get_unchecked(0);
            Warrior::randomized(&resource_pool, warrior, &mut rng).unwrap()
        };
        let point = Point::from_xy(1, 0);
        let map = MapSkeleton::randomized(&resource_pool, player, point, &mut rng).unwrap();
        println!("[map] = {map:?}");
        println!("[avaliable_range] = {:?}", map.movable_range());
        println!("[node] = {:?}", map.peak_upcoming_movment((1, 1).into()));
        Ok(())
    }
}
