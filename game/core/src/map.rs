extern crate alloc;
use core::cmp::{max, min};

use alloc::vec::Vec;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::errors::Error;
use crate::wrappings::{randomized_selection, LevelNode, LevelPartition, Node, Point, Warrior};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MapSkeleton {
    width: i16,
    height: i16,
    skeleton: Vec<LevelNode>,
    player: Warrior,
    player_point: Point,
}

impl MapSkeleton {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        player: Warrior,
        player_point: Point,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let scene_pool = resource_pool.scene_pool();
        let scene = randomized_selection(scene_pool.len(), scene_pool, 1, rng)
            .first()
            .cloned()
            .ok_or(Error::ResourceBrokenScenePool)?;
        let mut skeleton = scene
            .fixed_nodes()
            .into_iter()
            .map(|node| LevelNode::fix_randomized(resource_pool, node, rng))
            .collect::<Vec<_>>();
        scene
            .partition_list()
            .into_iter()
            .map(|partition| LevelPartition::randomized(resource_pool, partition, rng))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .for_each(|mut level| skeleton.append(&mut level.nodes));

        let good_start = skeleton.iter().any(|level| {
            if let Node::StartingPoint = level.node {
                return level.point.contains(&player_point);
            }
            false
        });
        if !good_start {
            return Err(Error::ScenePlayerPointInvalid);
        }

        Ok(Self {
            width: u8::from(scene.width()) as i16,
            height: u8::from(scene.height()) as i16,
            skeleton,
            player,
            player_point,
        })
    }

    pub fn contains(&self, point: &Point) -> bool {
        (point.x as i16) < self.width && (point.y as i16) < self.height
    }

    pub fn width(&self) -> u8 {
        self.width as u8
    }

    pub fn height(&self) -> u8 {
        self.height as u8
    }

    pub fn movable_range(&self) -> Vec<Point> {
        let motion = self.player.motion as i16;
        let mut points = Vec::<Point>::new();
        (1..motion).for_each(|y_step| {
            let y = y_step;
            (-y_step..(y_step + 1)).for_each(|x_step| {
                let x = min(
                    max((self.player_point.x as i16) + x_step, 0),
                    self.width - 1,
                );
                if !points
                    .iter()
                    .any(|point| point.x as i16 == x && point.y as i16 == y)
                {
                    points.push(Point::from_xy(x as u8, y as u8));
                }
            })
        });
        points
    }

    pub fn filter_nonempty_nodes(&self, points: &Vec<Point>) -> Vec<&LevelNode> {
        self.skeleton
            .iter()
            .filter(|node| points.iter().any(|point| node.point.contains(point)))
            .collect()
    }

    pub fn peak_upcoming_movment(&self, peak_point: Point) -> Result<Option<&LevelNode>, Error> {
        if !self.contains(&peak_point) {
            return Err(Error::ScenePlayerPointBeyondMap);
        }
        let movable_range = self.movable_range();
        let nodes = self.filter_nonempty_nodes(&movable_range);
        let mut peaked_node = None;
        if nodes.is_empty() {
            if !movable_range.into_iter().any(|point| point == peak_point) {
                return Err(Error::ScenePlayerPointInvalid);
            }
        } else {
            peaked_node = nodes
                .into_iter()
                .find(|node| node.point.contains(&peak_point));
        }
        Ok(peaked_node)
    }

    pub fn unchecked_move(&mut self, player_point: Point) {
        self.player_point = player_point;
    }
}
