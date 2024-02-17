extern crate alloc;
use alloc::{vec, vec::Vec};
use core::cmp::{max, min};
use rand::RngCore;

use crate::contexts::{CtxAdaptor, WarriorContext};
use crate::errors::Error;
use crate::fight::pve::MapFightPVE;
use crate::fight::traits::{FightLog, SimplePVE};
use crate::systems::{GameSystem, SystemReturn};
use crate::wrappings::{
    randomized_selection, Card, Context, Item, ItemClass, LevelNode, LevelPartition, Node, Point,
};

pub enum MoveResult<'a> {
    Fight(MapFightPVE<'a>),
    MapLogs(Vec<FightLog>),
    Complete,
    Null,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MapSkeleton {
    width: i16,
    height: i16,
    skeleton: Vec<LevelNode>,
    player_point: Point,
}

impl<'a> MapSkeleton {
    pub fn randomized(
        player_point: Point,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<Self, Error> {
        let resource_pool = system.resource_pool();
        let rng = system.rng();
        let scene_pool = resource_pool.scene_pool();
        let scene = randomized_selection(scene_pool.len(), scene_pool, 1, rng)
            .first()
            .cloned()
            .ok_or(Error::ResourceBrokenScenePool)?;
        let mut skeleton = scene
            .fixed_nodes()
            .into_iter()
            .map(|node| LevelNode::fix_randomized(resource_pool, node, rng))
            .collect::<Result<Vec<_>, _>>()?;
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

    pub fn current_point(&self) -> (u8, u8) {
        let Point { x, y } = self.player_point;
        (x, y)
    }

    pub fn node_skeleton(&self) -> &Vec<LevelNode> {
        &self.skeleton
    }

    pub fn movable_range(&self, motion: u8) -> Vec<Point> {
        let motion = motion as i16;
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

    pub fn peak_upcoming_movment(
        &self,
        peak_point: Point,
        motion: u8,
    ) -> Result<Option<&LevelNode>, Error> {
        if !self.contains(&peak_point) {
            return Err(Error::ScenePlayerPointBeyondMap);
        }
        let movable_range = self.movable_range(motion);
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

    pub fn move_to(
        &'a mut self,
        player: &'a mut WarriorContext<'a>,
        player_point: Point,
        motion: u8,
        user_imported: Vec<usize>,
        system: &mut GameSystem<'a, impl RngCore>,
    ) -> Result<MoveResult<'a>, Error> {
        self.player_point = player_point;
        let Some(level) = self.peak_upcoming_movment(player_point, motion)? else {
            return Ok(MoveResult::Null);
        };
        let mut map_logs = vec![];
        match &level.node {
            Node::Barrier | Node::StartingPoint => return Err(Error::SceneInvalidMove),
            Node::TargetingPoint => return Ok(MoveResult::Complete),
            Node::RecoverPoint(percent) => {
                let max_hp = player.warrior.hp;
                let hp_recover = min(max_hp * (*percent / 100u8) as u16, max_hp - player.hp);
                player.hp += hp_recover;
                map_logs.push(FightLog::RecoverHp(hp_recover));
            }
            Node::Campsite(context) => {
                let mut logs = run_context(player, context, system)?;
                map_logs.append(&mut logs);
            }
            Node::Unknown(contexts) => {
                contexts
                    .iter()
                    .map(|context| {
                        let mut logs = run_context(player, context, system)?;
                        map_logs.append(&mut logs);
                        Ok(())
                    })
                    .collect::<Result<_, _>>()?;
            }
            Node::Enemy(enemies) => {
                let fight = MapFightPVE::create(player, enemies)?;
                return Ok(MoveResult::Fight(fight));
            }
            Node::ItemMerchant(items) => {
                collect_items(player, user_imported, items, true)?;
            }
            Node::CardMerchant(cards) => {
                purchase_cards(player, user_imported, cards)?;
            }
            Node::TreasureChest(items, pick_count) => {
                if user_imported.len() > *pick_count as usize {
                    return Err(Error::SceneTreasureChestOutOfBound);
                }
                collect_items(player, user_imported, items, false)?;
            }
        }
        Ok(MoveResult::MapLogs(map_logs))
    }
}

fn run_context<'a>(
    player: &mut WarriorContext<'a>,
    context: &Context,
    system: &mut GameSystem<'a, impl RngCore>,
) -> Result<Vec<FightLog>, Error> {
    let mut system_contexts: Vec<&mut dyn CtxAdaptor> = vec![player];
    let system_return =
        system.call(context.system_id, &context.args, &mut system_contexts, None)?;
    if let SystemReturn::FightLog(logs) = system_return {
        Ok(logs)
    } else {
        return Err(Error::SceneUnexpectedSystemReturn);
    }
}

fn purchase_cards<'a>(
    player: &mut WarriorContext<'a>,
    user_imported: Vec<usize>,
    cards: &'a Vec<Card>,
) -> Result<Vec<()>, Error> {
    user_imported
        .into_iter()
        .map(|index| {
            let card = cards.get(index).ok_or(Error::SceneUserImportOutOfIndex)?;
            if card.price > player.gold {
                return Err(Error::SceneMerchantInsufficientGold);
            }
            player.gold -= card.price;
            player.deck.push(card);
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()
}

fn collect_items<'a>(
    player: &mut WarriorContext<'a>,
    user_imported: Vec<usize>,
    items: &'a Vec<Item>,
    purchase: bool,
) -> Result<Vec<()>, Error> {
    user_imported
        .into_iter()
        .map(|index| {
            let item = items.get(index).ok_or(Error::SceneUserImportOutOfIndex)?;
            if purchase {
                if item.price > player.gold {
                    return Err(Error::SceneMerchantInsufficientGold);
                }
                player.gold -= item.price;
            }
            match item.class {
                ItemClass::Equipment => player.equipment_list.push(item),
                ItemClass::Props => player.props_list.push(item),
            }
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()
}
