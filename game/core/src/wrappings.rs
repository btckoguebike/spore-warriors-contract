extern crate alloc;
use core::cmp::max;

use alloc::vec::Vec;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::errors::Error;

macro_rules! randomized_pool {
    ($val:ident.$meth:ident(), $pool:ident.$pmeth:ident(), $retn:ty, $rng:ident) => {{
        let indexes: Vec<u16> = $val.$meth().into();
        $pool
            .$pmeth()
            .into_iter()
            .filter_map(|v| {
                if indexes.contains(&v.id().into()) {
                    Some(<$retn>::randomized($pool, v, $rng))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }};
}

fn randomized_number(value: generated::RandomNumber, rng: &mut impl RngCore) -> u16 {
    let lower_bound: u16 = value.lower_bound().into();
    let upper_bound: u16 = value.upper_bound().into();
    let offset = rng.next_u32() % (upper_bound - lower_bound) as u32;
    lower_bound + offset as u16
}

fn randomized_byte(value: generated::RandomByte, rng: &mut impl RngCore) -> u8 {
    let lower_bound: u8 = value.lower_bound().into();
    let upper_bound: u8 = value.upper_bound().into();
    let offset = rng.next_u32() % (upper_bound - lower_bound) as u32;
    lower_bound + offset as u8
}

pub fn randomized_selection<T: IntoIterator>(
    mut size: usize,
    value: T,
    count: u8,
    rng: &mut impl RngCore,
) -> Vec<T::Item> {
    let mut pool = value.into_iter();
    (0..count)
        .filter_map(|_| {
            if size == 0 {
                return None;
            }
            let i = rng.next_u32() as usize % size;
            size -= 1;
            pool.nth(i)
        })
        .collect::<Vec<_>>()
}

pub enum Value {
    Resource(u16),
    System(u16),
    Number(u16),
}

impl Value {
    pub fn randomized(value: generated::Value, rng: &mut impl RngCore) -> Self {
        match value.to_enum() {
            generated::ValueUnion::Number(v) => Self::Number(v.into()),
            generated::ValueUnion::RandomNumber(v) => Self::Number(randomized_number(v, rng)),
            generated::ValueUnion::ResourceId(v) => Self::Resource(v.into()),
            generated::ValueUnion::SystemId(v) => Self::System(v.into()),
        }
    }
}

pub struct Context {
    pub scene_id: u8,
    pub system_id: u16,
    pub args: Vec<Value>,
}

impl Context {
    pub fn randomized(value: generated::Context, rng: &mut impl RngCore) -> Self {
        Self {
            scene_id: value.charactor_id().into(),
            system_id: value.system_id().into(),
            args: value
                .args()
                .into_iter()
                .map(|v| Value::randomized(v, rng))
                .collect(),
        }
    }
}

pub enum Duration {
    Round(u16),
    LifePoint(u16, u8, bool),
}

impl From<generated::Duration> for Duration {
    fn from(value: generated::Duration) -> Self {
        match value.to_enum() {
            generated::DurationUnion::Number(v) => Self::Round(v.into()),
            generated::DurationUnion::LifePoint(v) => Self::LifePoint(
                v.listen_system_id().into(),
                v.point().into(),
                u8::from(v.round_recover()) == 1u8,
            ),
        }
    }
}

pub struct Effect {
    pub on_trigger: Option<Context>,
    pub on_execution: Option<Context>,
    pub on_discard: Option<Context>,
    pub duration: Option<Duration>,
}

impl Effect {
    pub fn randomized(
        _: &generated::ResourcePool,
        value: generated::Effect,
        rng: &mut impl RngCore,
    ) -> Self {
        fn casting(context: generated::ContextOpt, rng: &mut impl RngCore) -> Option<Context> {
            context.to_opt().map(|v| Context::randomized(v, rng))
        }
        Self {
            on_trigger: casting(value.trigger(), rng),
            on_execution: casting(value.execution(), rng),
            on_discard: casting(value.discard(), rng),
            duration: value.duration().to_opt().map(Into::into),
        }
    }
}

pub struct Item {
    pub class: u8,
    pub quality: u8,
    pub weight: u8,
    pub price: u16,
    pub effect_pool: Vec<Effect>,
}

impl Item {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Item,
        rng: &mut impl RngCore,
    ) -> Self {
        let effect_pool = randomized_pool!(
            value.effect_pool(),
            resource_pool.effect_pool(),
            Effect,
            rng
        );
        Self {
            class: value.class().into(),
            quality: value.quality().into(),
            weight: randomized_byte(value.random_weight(), rng),
            price: randomized_number(value.price(), rng),
            effect_pool,
        }
    }
}

pub enum Score {
    Function(Context),
    Number(u16),
}

impl Score {
    pub fn randomized(value: generated::Score, rng: &mut impl RngCore) -> Self {
        match value.to_enum() {
            generated::ScoreUnion::Context(v) => Self::Function(Context::randomized(v, rng)),
            generated::ScoreUnion::RandomNumber(v) => Self::Number(randomized_number(v, rng)),
        }
    }
}

pub struct Loot {
    pub gold: u16,
    pub score: Score,
    pub card_pool: Vec<Item>,
    pub props_pool: Vec<Item>,
    pub equipment_pool: Vec<Item>,
}

impl Loot {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Loot,
        rng: &mut impl RngCore,
    ) -> Self {
        fn package_unpack(
            resource_pool: &generated::ResourcePool,
            package: Option<generated::Package>,
            rng: &mut impl RngCore,
        ) -> Vec<Item> {
            let Some(package) = package else {
                return Default::default();
            };
            randomized_pool!(package.item_pool(), resource_pool.item_pool(), Item, rng)
        }
        Self {
            gold: randomized_number(value.gold(), rng),
            score: Score::randomized(value.score(), rng),
            card_pool: package_unpack(resource_pool, Some(value.card_pool()), rng),
            props_pool: package_unpack(resource_pool, value.props_pool().to_opt(), rng),
            equipment_pool: package_unpack(resource_pool, value.equipment_pool().to_opt(), rng),
        }
    }
}

pub struct Action {
    pub random_select: bool,
    pub pointer: u8,
    pub effect_pool: Vec<Effect>,
}

impl Action {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Action,
        rng: &mut impl RngCore,
    ) -> Self {
        let effect_pool = randomized_pool!(
            value.effect_pool(),
            resource_pool.effect_pool(),
            Effect,
            rng
        );
        Self {
            random_select: u8::from(value.random()) == 1u8,
            pointer: 0,
            effect_pool,
        }
    }
}

pub struct ActionStrategy {
    pub random_select: bool,
    pub actions: Vec<Action>,
}

impl ActionStrategy {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::ActionContext,
        rng: &mut impl RngCore,
    ) -> Self {
        let actions = randomized_pool!(
            value.action_pool(),
            resource_pool.action_pool(),
            Action,
            rng
        );
        Self {
            random_select: u8::from(value.random()) == 1u8,
            actions,
        }
    }
}

pub struct Enemy {
    pub scene_id: u8,
    pub rank: u8,
    pub hp: u16,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub rewards: Vec<Loot>,
    pub strategy: ActionStrategy,
}

impl Enemy {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Enemy,
        rng: &mut impl RngCore,
    ) -> Self {
        let rewards = randomized_pool!(value.loot_pool(), resource_pool.loot_pool(), Loot, rng);
        let strategy = ActionStrategy::randomized(resource_pool, value.action_strategy(), rng);
        Self {
            scene_id: 0,
            rank: value.rank().into(),
            hp: value.hp().into(),
            armor: value.rank().into(),
            shield: value.shield().into(),
            attack: value.attack().into(),
            attack_weak: value.attack_weak().into(),
            defense: value.defense().into(),
            defense_weak: value.defense_weak().into(),
            rewards,
            strategy,
        }
    }
}

#[derive(PartialEq, Eq, Default)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    pub fn from_xy(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(PartialEq, Eq)]
pub struct SizedPoint {
    pub point: Point,
    x_size: u8,
    y_size: u8,
}

impl From<generated::Coordinate> for SizedPoint {
    fn from(value: generated::Coordinate) -> Self {
        Self {
            point: Point::from_xy(value.x().into(), value.y().into()),
            x_size: 1,
            y_size: 1,
        }
    }
}

impl From<generated::Size> for SizedPoint {
    fn from(value: generated::Size) -> Self {
        Self {
            point: Default::default(),
            x_size: value.x().into(),
            y_size: value.y().into(),
        }
    }
}

impl SizedPoint {
    pub fn x(&self) -> u8 {
        self.point.x
    }

    pub fn y(&self) -> u8 {
        self.point.y
    }

    pub fn shift(&self, x: u8, y: u8) -> Self {
        Self {
            point: Point::from_xy(self.point.x + x, self.point.y + y),
            x_size: self.x_size,
            y_size: self.y_size,
        }
    }

    pub fn contains(&self, point: &Point) -> bool {
        let x = self.point.x as i16;
        let y = self.point.y as i16;
        let y_size = self.y_size as i16;
        let x_size = self.x_size as i16;
        (-y_size..(y_size + 1)).any(|y_shift| {
            let new_y = max(y + y_shift, 0);
            (-x_size..(x_size + 1)).any(|x_shift| {
                let new_x = max(x + x_shift, 0);
                return point.x == new_x as u8 && point.y == new_y as u8;
            })
        })
    }
}

pub struct Card {
    pub class: u8,
    pub power_cost: u8,
    pub merchant_price: u16,
    pub effect_pool: Vec<Effect>,
}

impl Card {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Card,
        rng: &mut impl RngCore,
    ) -> Card {
        let effect_pool = randomized_pool!(
            value.effect_pool(),
            resource_pool.effect_pool(),
            Effect,
            rng
        );
        Self {
            class: value.class().into(),
            power_cost: value.cost().into(),
            merchant_price: randomized_number(value.price(), rng),
            effect_pool,
        }
    }
}

pub struct Warrior {
    pub charactor_card: Card,
    pub hp: u16,
    pub gold: u16,
    pub power: u8,
    pub motion: u8,
    pub view_range: u8,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub attack_weak: u8,
    pub defense: u8,
    pub defense_weak: u8,
    pub physique: u8,
    pub draw_count: u8,
    pub deck_status: Vec<Card>,
    pub package_status: Vec<Item>,
}

impl Warrior {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Warrior,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let card_id =
            randomized_selection(value.special_cards().len(), value.special_cards(), 1, rng)
                .first()
                .cloned()
                .ok_or(Error::ResourceBrokenCharactorCard)?;
        let charactor_card = resource_pool
            .card_pool()
            .into_iter()
            .find(|card| card.id().raw_data() == card_id.raw_data())
            .ok_or(Error::ResourceBrokenCardPool)?;
        let deck_status =
            randomized_pool!(value.deck_status(), resource_pool.card_pool(), Card, rng);
        let package_status =
            randomized_pool!(value.package_status(), resource_pool.item_pool(), Item, rng);
        Ok(Self {
            charactor_card: Card::randomized(resource_pool, charactor_card, rng),
            hp: value.hp().into(),
            gold: value.gold().into(),
            power: value.power().into(),
            motion: value.motion().into(),
            view_range: value.view_range().into(),
            armor: value.armor().into(),
            shield: value.shield().into(),
            attack: value.attack().into(),
            attack_weak: value.attack_weak().into(),
            defense: value.defense().into(),
            defense_weak: value.defense_weak().into(),
            physique: value.physique().into(),
            draw_count: value.draw_count().into(),
            deck_status,
            package_status,
        })
    }
}

pub enum Node {
    Enemy(Vec<Enemy>),
    TreasureChest(Vec<Item>, u8),
    RecoverPoint(u8),
    Merchant(Vec<Item>),
    Unknown(Vec<Context>),
    Campsite(Context),
    Barrier(),
    StartingPoint(),
    TargetingPoint(),
}

pub struct LevelNode {
    pub visible: bool,
    pub point: SizedPoint,
    pub node: Node,
}

impl LevelNode {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::LevelNode,
        rng: &mut impl RngCore,
    ) -> Self {
        Self {
            visible: u8::from(value.visible()) == 1u8,
            point: value.size().into(),
            node: match value.node().to_enum() {
                generated::NodeInstanceUnion::NodeEnemy(value) => {
                    let enemies = randomized_pool!(
                        value.enemy_pool(),
                        resource_pool.enemy_pool(),
                        Enemy,
                        rng
                    );
                    let randomized_enemies =
                        randomized_selection(enemies.len(), enemies, value.count().into(), rng);
                    Node::Enemy(randomized_enemies)
                }
                generated::NodeInstanceUnion::NodeRecoverPoint(value) => {
                    Node::RecoverPoint(value.hp_percent().into())
                }
                generated::NodeInstanceUnion::NodeMerchant(value) => {
                    let goods =
                        randomized_pool!(value.item_pool(), resource_pool.item_pool(), Item, rng);
                    let randomized_goods =
                        randomized_selection(goods.len(), goods, value.count().into(), rng);
                    Node::Merchant(randomized_goods)
                }
                generated::NodeInstanceUnion::NodeCampsite(value) => {
                    Node::Campsite(Context::randomized(value.card_context(), rng))
                }
                generated::NodeInstanceUnion::NodeUnknown(value) => {
                    let contexts = value
                        .system_pool()
                        .into_iter()
                        .map(|context| Context::randomized(context, rng))
                        .collect::<Vec<_>>();
                    let randomized_contexts =
                        randomized_selection(contexts.len(), contexts, value.count().into(), rng);
                    Node::Unknown(randomized_contexts)
                }
                generated::NodeInstanceUnion::NodeTreasureChest(value) => {
                    let items =
                        randomized_pool!(value.item_pool(), resource_pool.item_pool(), Item, rng);
                    let randomized_items =
                        randomized_selection(items.len(), items, value.count().into(), rng);
                    Node::TreasureChest(randomized_items, value.pick().into())
                }
                generated::NodeInstanceUnion::NodeBarrier(_) => Node::Barrier(),
                generated::NodeInstanceUnion::NodeStartingPoint(_) => Node::StartingPoint(),
                generated::NodeInstanceUnion::NodeTargetingPoint(_) => Node::TargetingPoint(),
            },
        }
    }

    pub fn fix_randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::FixedLevelNode,
        rng: &mut impl RngCore,
    ) -> Self {
        let mut node = LevelNode::randomized(resource_pool, value.node(), rng);
        node.point = node
            .point
            .shift(value.point().x().into(), value.point().y().into());
        node
    }
}

pub struct LevelPartition {
    pub nodes: Vec<LevelNode>,
}

impl LevelPartition {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::ScenePartition,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let start: SizedPoint = value.start_point().into();
        let (x_diff, y_diff) = {
            let end: SizedPoint = value.end_point().into();
            if start.x() >= end.x() || start.y() >= end.y() {
                return Err(Error::ResourceBrokenPartitionRange);
            }
            (end.x() - start.x(), end.y() - start.y())
        };

        let sample_count = randomized_byte(value.count(), rng);
        let points = (0..sample_count)
            .map(|_| {
                let x_sample = rng.next_u32() % x_diff as u32;
                let y_sample = rng.next_u32() % y_diff as u32;
                start.shift(x_sample as u8, y_sample as u8)
            })
            .collect::<Vec<_>>();

        let nodes = value
            .node_pool()
            .into_iter()
            .map(|node| LevelNode::randomized(resource_pool, node, rng))
            .collect::<Vec<_>>();
        let randomized_nodes = randomized_selection(nodes.len(), nodes, sample_count, rng)
            .into_iter()
            .zip(points.into_iter())
            .map(|(mut node, point)| {
                node.point = point;
                node
            })
            .collect::<Vec<_>>();

        Ok(Self {
            nodes: randomized_nodes,
        })
    }
}
