extern crate alloc;
use alloc::{vec, vec::Vec};
use core::cmp::{max, min};

use crate::battle::pve::MapBattlePVE;
use crate::battle::traits::{FightLog, SimplePVE};
use crate::contexts::{CardContext, CtxAdaptor, WarriorContext, WarriorDeckContext};
use crate::errors::Error;
use crate::systems::{Command, SystemController, SystemReturn};
use crate::wrappings::{
    randomized_selection, Card, Item, ItemClass, LevelNode, LevelPartition, Node, Point, System,
};

#[cfg(feature = "json_ser")]
use serde::Serialize;

fn run_context(
    player: &mut WarriorContext,
    system: System,
    controller: &mut SystemController,
) -> Result<Vec<FightLog>, Error> {
    let player_offset = player.offset();
    let mut system_contexts: Vec<&mut dyn CtxAdaptor> = vec![player];
    let system_return = controller.system_call(
        system.into(),
        player_offset,
        vec![],
        &mut system_contexts,
        None,
    )?;
    if let SystemReturn::Continue(cmds) = system_return {
        let mut context_logs = vec![];
        cmds.into_iter().for_each(|v| {
            if let Command::AddLogs(mut logs) = v {
                context_logs.append(&mut logs);
            }
        });
        Ok(context_logs)
    } else {
        return Err(Error::SceneUnexpectedSystemReturn);
    }
}

fn purchase_cards(
    player: &mut WarriorContext,
    player_deck: &mut WarriorDeckContext,
    user_imported: Vec<usize>,
    cards: &Vec<Card>,
) -> Result<Vec<()>, Error> {
    user_imported
        .into_iter()
        .map(|index| {
            let card = cards.get(index).ok_or(Error::SceneUserImportOutOfIndex)?;
            if card.price > player.gold {
                return Err(Error::SceneMerchantInsufficientGold);
            }
            player.gold -= card.price;
            player_deck.deck.push(CardContext::new(card.clone()));
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()
}

fn collect_items(
    player: &mut WarriorContext,
    user_imported: Vec<usize>,
    items: &Vec<Item>,
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
                if item.weight > player.physique {
                    return Err(Error::SceneMerchantInsufficientPhysique);
                }
                player.physique -= item.weight;
            }
            match item.class {
                ItemClass::Equipment => player.equipment_list.push(item.clone()),
                ItemClass::Props => player.props_list.push(item.clone()),
            }
            Ok(())
        })
        .collect::<Result<Vec<_>, _>>()
}

pub enum MoveResult<T: SimplePVE> {
    Fight(T),
    MapLogs(Vec<FightLog>),
    Complete,
    Skip,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "json_ser", derive(Serialize))]
pub struct MapSkeleton {
    pub id: u16,
    pub width: i16,
    pub height: i16,
    pub skeleton: Vec<LevelNode>,
    pub player_point: Point,
}

impl<'a> MapSkeleton {
    pub fn randomized(controller: &mut SystemController) -> Result<Self, Error> {
        let resource_pool = &controller.resource_pool;
        let rng = &mut controller.rng;
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
        Ok(Self {
            id: scene.id().into(),
            width: u8::from(scene.width()) as i16,
            height: u8::from(scene.height()) as i16,
            skeleton,
            player_point: Point::default(),
        })
    }

    pub fn place_player(
        &mut self,
        player_point: Point,
        check_start_point: bool,
    ) -> Result<(), Error> {
        if check_start_point {
            let good_start = self.skeleton.iter().any(|level| {
                if let Node::StartingPoint = level.node {
                    return level.point.contains(&player_point);
                }
                false
            });
            if !good_start {
                return Err(Error::ScenePlayerPointInvalid);
            }
        }
        self.player_point = player_point;
        Ok(())
    }

    pub fn peak_upcoming_movment(
        &self,
        player: &WarriorContext,
        peak_point: Point,
    ) -> Result<Option<&LevelNode>, Error> {
        if !self.contains(&peak_point) {
            return Err(Error::ScenePlayerPointBeyondMap);
        }
        let movable_range = self.movable_range(player.warrior.motion);
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
        &mut self,
        player: &mut WarriorContext,
        player_deck: &mut WarriorDeckContext,
        player_point: Point,
        user_imported: Vec<usize>,
        controller: &mut SystemController,
    ) -> Result<MoveResult<impl SimplePVE>, Error> {
        self.player_point = player_point;
        let Some(level) = self.peak_upcoming_movment(player, player_point)? else {
            return Ok(MoveResult::Skip);
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
                let mut logs = run_context(player, context.clone(), controller)?;
                map_logs.append(&mut logs);
            }
            Node::Unknown(contexts) => {
                contexts
                    .iter()
                    .map(|context| {
                        let mut logs = run_context(player, context.clone(), controller)?;
                        map_logs.append(&mut logs);
                        Ok(())
                    })
                    .collect::<Result<_, _>>()?;
            }
            Node::Enemy(enemies) => {
                let fight =
                    MapBattlePVE::create(player.clone(), player_deck.clone(), enemies.clone())?;
                return Ok(MoveResult::Fight(fight));
            }
            Node::ItemMerchant(items) => {
                collect_items(player, user_imported, items, true)?;
            }
            Node::CardMerchant(cards) => {
                purchase_cards(player, player_deck, user_imported, cards)?;
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

    fn contains(&self, point: &Point) -> bool {
        (point.x as i16) < self.width && (point.y as i16) < self.height
    }

    fn movable_range(&self, motion: u8) -> Vec<Point> {
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

    fn filter_nonempty_nodes(&self, points: &Vec<Point>) -> Vec<&LevelNode> {
        self.skeleton
            .iter()
            .filter(|node| points.iter().any(|point| node.point.contains(point)))
            .collect()
    }
}
