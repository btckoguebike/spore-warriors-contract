extern crate alloc;
use alloc::vec::Vec;
use core::cmp::max;
use core::sync::atomic::AtomicUsize;
use rand::RngCore;
use rlp::{RlpDecodable, RlpEncodable};
use spore_warriors_generated as generated;

use crate::errors::Error;

static OFFSET: AtomicUsize = AtomicUsize::new(10);

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
            .collect::<Result<Vec<_>, _>>()
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

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct Value(pub u16);

impl Value {
    pub fn randomized(value: generated::Value, rng: &mut impl RngCore) -> Self {
        match value.to_enum() {
            generated::ValueUnion::Number(v) => Self(v.into()),
            generated::ValueUnion::RandomNumber(v) => Self(randomized_number(v, rng)),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, Copy)]
pub enum RequireTarget {
    Owner,
    Opponent,
    RandomOpponent,
    AllOpponents,
    AllCharactors,
}

impl TryFrom<u8> for RequireTarget {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Owner),
            1 => Ok(Self::Opponent),
            3 => Ok(Self::RandomOpponent),
            4 => Ok(Self::AllOpponents),
            5 => Ok(Self::AllCharactors),
            _ => Err(Error::ResourceBrokenTargetPosition),
        }
    }
}

impl rlp::Encodable for RequireTarget {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let target = self.clone() as u8;
        s.begin_list(1).append(&target);
    }
}

impl rlp::Decodable for RequireTarget {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u8 = rlp.val_at(0)?;
        let class: Self = value
            .try_into()
            .map_err(|_| rlp::DecoderError::Custom("Invalid RequireTarget"))?;
        Ok(class)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
#[repr(u16)]
pub enum SystemId {
    InstantDamage,
    InstantMultipleDamage,
    InstantArmorUp,
    InstantArmorDown,
    InstantShieldUp,
    InstantShieldDown,
    InstantHealing,
    InstantDrawCountUp,
    InstantDrawCountDown,
    InstantAttackPowerUp,
    InstantDefensePowerUp,
    InstantAttackPowerWeak,
    InstantDefensePowerWeak,
    InstantDrawCards,

    TriggerRecoverHp,
}

impl From<SystemId> for u16 {
    fn from(value: SystemId) -> Self {
        value as u16
    }
}

impl TryFrom<u16> for SystemId {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::InstantDamage),
            1 => Ok(Self::InstantMultipleDamage),
            2 => Ok(Self::InstantArmorUp),
            3 => Ok(Self::InstantArmorDown),
            4 => Ok(Self::InstantShieldUp),
            5 => Ok(Self::InstantShieldDown),
            6 => Ok(Self::InstantHealing),
            7 => Ok(Self::InstantDrawCountUp),
            8 => Ok(Self::InstantDrawCountDown),
            9 => Ok(Self::InstantAttackPowerUp),
            10 => Ok(Self::InstantDefensePowerUp),
            11 => Ok(Self::InstantAttackPowerWeak),
            12 => Ok(Self::InstantDefensePowerWeak),
            13 => Ok(Self::InstantDrawCards),
            _ => Err(Error::ResourceBrokenSystemId),
        }
    }
}

impl rlp::Encodable for SystemId {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let id = self.clone() as u16;
        s.begin_list(1).append(&id);
    }
}

impl rlp::Decodable for SystemId {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u16 = rlp.val_at(0)?;
        let class: Self = value
            .try_into()
            .map_err(|_| rlp::DecoderError::Custom("Invalid SystemId"))?;
        Ok(class)
    }
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, Copy, RlpEncodable, RlpDecodable)]
pub struct Duration {
    pub trigger: u16,
    pub count: u16,
}

impl From<generated::Duration> for Duration {
    fn from(value: generated::Duration) -> Self {
        Self {
            trigger: u8::from(value.trigger()) as u16,
            count: u8::from(value.count()) as u16,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct System {
    pub id: u16,
    pub system_id: SystemId,
    pub args: Vec<Value>,
    pub duration: Option<Duration>,
    pub target_type: RequireTarget,
}

impl System {
    pub fn randomized(
        _: &generated::ResourcePool,
        value: generated::System,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let duration: Option<Duration> = value.duration().to_opt().map(Into::into);
        if let Some(duration) = duration.as_ref() {
            if duration.count == 0 {
                return Err(Error::ResourceBrokenDurationCount);
            }
        }
        Ok(Self {
            id: value.id().into(),
            system_id: u16::from(value.system_id()).try_into()?,
            target_type: u8::from(value.target_type()).try_into()?,
            args: value
                .args()
                .into_iter()
                .map(|v| Value::randomized(v, rng))
                .collect(),
            duration,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum ItemClass {
    Equipment,
    Props,
}

impl TryFrom<u8> for ItemClass {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            Ok(Self::Equipment)
        } else if value == 1 {
            Ok(Self::Props)
        } else {
            Err(Error::ResourceBrokenItemClass)
        }
    }
}

impl rlp::Encodable for ItemClass {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let class = self.clone() as u8;
        s.begin_list(1).append(&class);
    }
}

impl rlp::Decodable for ItemClass {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value: u8 = rlp.val_at(0)?;
        let class: Self = value
            .try_into()
            .map_err(|_| rlp::DecoderError::Custom("invalid ItemClass"))?;
        Ok(class)
    }
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpDecodable, RlpEncodable)]
pub struct Item {
    pub id: u16,
    pub class: ItemClass,
    pub quality: u8,
    pub weight: u8,
    pub price: u16,
    pub system_pool: Vec<System>,
}

impl Item {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Item,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let system_pool = randomized_pool!(
            value.system_pool(),
            resource_pool.system_pool(),
            System,
            rng
        )?;
        Ok(Self {
            id: value.id().into(),
            class: u8::from(value.class()).try_into()?,
            quality: value.quality().into(),
            weight: randomized_byte(value.random_weight(), rng),
            price: randomized_number(value.price(), rng),
            system_pool,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Loot {
    pub gold: u16,
    pub score: u16,
    pub card_pool: Vec<Item>,
    pub props_pool: Vec<Item>,
    pub equipment_pool: Vec<Item>,
}

impl Loot {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Loot,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        fn package_unpack(
            resource_pool: &generated::ResourcePool,
            package: Option<generated::Package>,
            rng: &mut impl RngCore,
        ) -> Result<Vec<Item>, Error> {
            let Some(package) = package else {
                return Ok(Default::default());
            };
            randomized_pool!(package.item_pool(), resource_pool.item_pool(), Item, rng)
        }
        Ok(Self {
            gold: randomized_number(value.gold(), rng),
            score: randomized_number(value.score(), rng),
            card_pool: package_unpack(resource_pool, Some(value.card_pool()), rng)?,
            props_pool: package_unpack(resource_pool, value.props_pool().to_opt(), rng)?,
            equipment_pool: package_unpack(resource_pool, value.equipment_pool().to_opt(), rng)?,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Action {
    pub random_select: bool,
    pub system_pool: Vec<System>,
}

impl Action {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Action,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let system_pool = randomized_pool!(
            value.system_pool(),
            resource_pool.system_pool(),
            System,
            rng
        )?;
        Ok(Self {
            random_select: u8::from(value.random()) == 1u8,
            system_pool,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ActionStrategy {
    pub random_select: bool,
    pub actions: Vec<Action>,
}

impl ActionStrategy {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::ActionContext,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let actions = randomized_pool!(
            value.action_pool(),
            resource_pool.action_pool(),
            Action,
            rng
        )?;
        Ok(Self {
            random_select: u8::from(value.random()) == 1u8,
            actions,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Enemy {
    pub id: u16,
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
    ) -> Result<Self, Error> {
        let rewards = randomized_pool!(value.loot_pool(), resource_pool.loot_pool(), Loot, rng)?;
        let strategy = ActionStrategy::randomized(resource_pool, value.action_strategy(), rng)?;
        Ok(Self {
            id: value.id().into(),
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
        })
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl From<(u8, u8)> for Point {
    fn from(value: (u8, u8)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Point {
    pub fn from_xy(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SizedPoint {
    pub point: Point,
    x_size: u8,
    y_size: u8,
}

impl From<generated::Coordinate> for SizedPoint {
    fn from(value: generated::Coordinate) -> Self {
        Self {
            point: Point::from_xy(value.x().into(), value.y().into()),
            x_size: 0,
            y_size: 0,
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

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct Card {
    pub offset: usize,
    pub id: u16,
    pub class: u8,
    pub power_cost: u8,
    pub price: u16,
    pub system_pool: Vec<System>,
}

impl Card {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Card,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let system_pool = randomized_pool!(
            value.system_pool(),
            resource_pool.system_pool(),
            System,
            rng
        )?;
        Ok(Self {
            offset: OFFSET.fetch_add(1, core::sync::atomic::Ordering::SeqCst),
            id: value.id().into(),
            class: value.class().into(),
            power_cost: value.cost().into(),
            price: randomized_number(value.price(), rng),
            system_pool,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone, RlpEncodable, RlpDecodable)]
pub struct Warrior {
    pub id: u16,
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
        let draw_count: u8 = value.draw_count().into();
        let deck_status =
            randomized_pool!(value.deck_status(), resource_pool.card_pool(), Card, rng)?;
        if draw_count as usize > deck_status.len() {
            return Err(Error::ResourceBrokenPlayerDeck);
        }
        let package_status =
            randomized_pool!(value.package_status(), resource_pool.item_pool(), Item, rng)?;
        Ok(Self {
            id: value.id().into(),
            charactor_card: Card::randomized(resource_pool, charactor_card, rng)?,
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
            draw_count,
            deck_status,
            package_status,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Node {
    Enemy(Vec<Enemy>),
    TreasureChest(Vec<Item>, u8),
    RecoverPoint(u8),
    ItemMerchant(Vec<Item>),
    CardMerchant(Vec<Card>),
    Unknown(Vec<System>),
    Campsite(System),
    Barrier,
    StartingPoint,
    TargetingPoint,
}

#[cfg_attr(feature = "debug", derive(Debug))]
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
    ) -> Result<Self, Error> {
        Ok(Self {
            visible: u8::from(value.visible()) == 1u8,
            point: value.size().into(),
            node: match value.node().to_enum() {
                generated::NodeInstanceUnion::NodeEnemy(value) => {
                    let enemies = randomized_pool!(
                        value.enemy_pool(),
                        resource_pool.enemy_pool(),
                        Enemy,
                        rng
                    )?;
                    let randomized_enemies =
                        randomized_selection(enemies.len(), enemies, value.count().into(), rng);
                    Node::Enemy(randomized_enemies)
                }
                generated::NodeInstanceUnion::NodeRecoverPoint(value) => {
                    let percent: u8 = value.hp_percent().into();
                    if percent > 100 {
                        return Err(Error::ResourceBrokenHpPercent);
                    }
                    Node::RecoverPoint(percent)
                }
                generated::NodeInstanceUnion::NodeItemMerchant(value) => {
                    let goods =
                        randomized_pool!(value.item_pool(), resource_pool.item_pool(), Item, rng)?;
                    let randomized_goods =
                        randomized_selection(goods.len(), goods, value.count().into(), rng);
                    Node::ItemMerchant(randomized_goods)
                }
                generated::NodeInstanceUnion::NodeCardMerchant(value) => {
                    let goods =
                        randomized_pool!(value.card_pool(), resource_pool.card_pool(), Card, rng)?;
                    let randomized_goods =
                        randomized_selection(goods.len(), goods, value.count().into(), rng);
                    Node::CardMerchant(randomized_goods)
                }
                generated::NodeInstanceUnion::NodeCampsite(value) => {
                    let system = resource_pool
                        .system_pool()
                        .into_iter()
                        .find(|v| v.id().raw_data() == value.card_system().raw_data())
                        .ok_or(Error::ResourceBrokenSystemPool)?;
                    Node::Campsite(System::randomized(resource_pool, system, rng)?)
                }
                generated::NodeInstanceUnion::NodeUnknown(value) => {
                    let unknowns = randomized_pool!(
                        value.system_pool(),
                        resource_pool.system_pool(),
                        System,
                        rng
                    )?;
                    let randomized_contexts =
                        randomized_selection(unknowns.len(), unknowns, value.count().into(), rng);
                    Node::Unknown(randomized_contexts)
                }
                generated::NodeInstanceUnion::NodeTreasureChest(value) => {
                    let items =
                        randomized_pool!(value.item_pool(), resource_pool.item_pool(), Item, rng)?;
                    let randomized_items =
                        randomized_selection(items.len(), items, value.count().into(), rng);
                    Node::TreasureChest(randomized_items, value.pick().into())
                }
                generated::NodeInstanceUnion::NodeBarrier(_) => Node::Barrier,
                generated::NodeInstanceUnion::NodeStartingPoint(_) => Node::StartingPoint,
                generated::NodeInstanceUnion::NodeTargetingPoint(_) => Node::TargetingPoint,
            },
        })
    }

    pub fn fix_randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::FixedLevelNode,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let mut node = LevelNode::randomized(resource_pool, value.node(), rng)?;
        node.point = node
            .point
            .shift(value.point().x().into(), value.point().y().into());
        Ok(node)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
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
            .collect::<Result<Vec<_>, _>>()?;
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Potion {
    pub count: u8,
    pub hp: u8,
    pub gold: u8,
    pub power: u8,
    pub motion: u8,
    pub view_range: u8,
    pub armor: u8,
    pub shield: u8,
    pub attack: u8,
    pub defense: u8,
    pub physique: u8,
    pub draw_count: u8,
    pub deck_status: Vec<Card>,
    pub package_status: Vec<Item>,
}

impl Potion {
    pub fn randomized(
        resource_pool: &generated::ResourcePool,
        value: generated::Potion,
        rng: &mut impl RngCore,
    ) -> Result<Self, Error> {
        let deck_status =
            randomized_pool!(value.deck_status(), resource_pool.card_pool(), Card, rng)?;
        let package_status =
            randomized_pool!(value.package_status(), resource_pool.item_pool(), Item, rng)?;
        Ok(Self {
            count: value.count().into(),
            hp: value.hp().into(),
            gold: value.gold().into(),
            power: value.power().into(),
            motion: value.motion().into(),
            view_range: value.view_range().into(),
            armor: value.armor().into(),
            shield: value.shield().into(),
            attack: value.attack().into(),
            defense: value.defense().into(),
            physique: value.physique().into(),
            draw_count: value.draw_count().into(),
            deck_status,
            package_status,
        })
    }
}
