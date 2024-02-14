extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::contexts::{EnemyContext, WarriorContext};
use crate::errors::Error;
use crate::fight::traits::{IterationInput, IterationOutput, Selection, SimplePVE, Target};
use crate::wrappings::{Context, Effect, Enemy, ItemClass, Potion, RequireTarget, Warrior};

macro_rules! iterate_deck_select {
    ($which:ident, $this:ident.$deck:ident$(.$ext:ident)?, $output:ident) => {{
        let Selection::Deck(indexes) = $which else {
            return Err(Error::BattleSelectionMismatch);
        };
        if indexes.iter().any(|v| *v >= $this.$deck$(.$ext)?.len()) {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::$output != $this.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        $this.operate_pending_instructions(Some(indexes))
    }};
}

enum Instruction<'a> {
    SystemCall(&'a Context),
    PositionChange(Target),
}

pub struct MapFightPVE<'a, R: RngCore> {
    player: WarriorContext<'a>,
    opponents: Vec<EnemyContext<'a>>,
    round: u8,
    last_output: IterationOutput,
    last_position: Target,
    pending_instructions: Vec<Instruction<'a>>,
    rng: &'a mut R,
}

impl<'a, T: RngCore> SimplePVE<'a, T> for MapFightPVE<'a, T> {
    fn create(
        player: &'a Warrior,
        potion: Option<&'a Potion>,
        enemies: &'a [Enemy],
        rng: &'a mut T,
    ) -> Result<Self, Error> {
        let mut warrior_context = WarriorContext::new(player);
        if let Some(potion) = potion {
            warrior_context.hp += potion.hp as u16;
            warrior_context.power += potion.power;
            warrior_context.armor += potion.armor;
            warrior_context.shield += potion.shield;
            warrior_context.attack += potion.attack;
            warrior_context.draw_count += potion.draw_count;
            warrior_context
                .props_list
                .append(&mut potion.package_status.iter().collect());
            warrior_context
                .deck
                .append(&mut potion.deck_status.iter().collect());
        }

        Ok(Self {
            player: warrior_context,
            opponents: enemies.iter().map(EnemyContext::new).collect(),
            round: 0,
            last_output: IterationOutput::Continue,
            last_position: Target::Player,
            pending_instructions: vec![],
            rng,
        })
    }

    fn start(&mut self) -> Result<IterationOutput, Error> {
        if self.round != 0 {
            return Err(Error::BattleRepeatStart);
        }
        let mut equipment_effects = vec![];
        self.player.warrior.package_status.iter().for_each(|v| {
            if v.class == ItemClass::Equipment {
                v.effect_pool
                    .iter()
                    .for_each(|effect| equipment_effects.push(effect));
            }
        });
        self.round = 1;
        self.operate_effects(Target::Player, &equipment_effects)
    }

    fn run(&mut self, operations: Vec<IterationInput>) -> Result<IterationOutput, Error> {
        if self.round == 0 {
            return Err(Error::BattleNotStarted);
        }
        for operation in operations {
            self.iterate(operation)?;
        }
        Ok(self.last_output)
    }

    fn peak_target(&self, hand_card_selection: Selection) -> Result<Vec<&'a RequireTarget>, Error> {
        let Selection::Deck(indexes) = hand_card_selection else {
            return Err(Error::BattleOperationMismatch);
        };
        if indexes.len() != 1 {
            return Err(Error::BattleSelectionError);
        }
        let required_targets = self
            .player
            .hand_deck
            .get(indexes[0])
            .ok_or(Error::BattleSelectionError)?
            .effect_pool
            .iter()
            .filter_map(|effect| effect.on_execution.as_ref())
            .map(|ctx| &ctx.target_position)
            .collect::<Vec<_>>();
        Ok(required_targets)
    }
}

impl<'a, T: RngCore> MapFightPVE<'a, T> {
    fn iterate(&mut self, operation: IterationInput) -> Result<IterationOutput, Error> {
        match operation {
            IterationInput::ItemUse(which, target) => self.iterate_item_use(which, target),
            IterationInput::HandCardUse(which, target) => self.iterate_hand_card_use(which, target),
            IterationInput::SpecialCardUse(target) => self.iterate_special_card_use(target),
            IterationInput::PlayerRoundEnd => self.iterate_player_round_end(),
            IterationInput::EnemyTurn => self.iterate_enemy_turn(),
            IterationInput::HandCardSelect(which) => {
                iterate_deck_select!(which, self.player.hand_deck, RequireHandCardSelect)
            }
            IterationInput::DeckCardSelect(which) => {
                iterate_deck_select!(which, self.player.deck, RequireDeckCardSelect)
            }
            IterationInput::GraveCardSelect(which) => {
                iterate_deck_select!(which, self.player.grave_deck, RequireGraveCardSelect)
            }
            IterationInput::OutsideCardSelect(which) => {
                iterate_deck_select!(which, self.player.outside_deck, RequireOutsideCardSelect)
            }
        }
    }

    fn iterate_item_use(
        &mut self,
        which: Selection,
        target: Target,
    ) -> Result<IterationOutput, Error> {
        let Selection::Item(index) = which else {
            return Err(Error::BattleSelectionMismatch);
        };
        if index >= self.player.props_list.len() {
            return Err(Error::BattleSelectionError);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        }
        let props_item = self.player.props_list.remove(index);
        let effects = props_item.effect_pool.iter().collect::<Vec<_>>();
        self.operate_effects(target, &effects)
    }

    fn iterate_hand_card_use(
        &mut self,
        which: Selection,
        target: Target,
    ) -> Result<IterationOutput, Error> {
        let index = {
            let Selection::Deck(indexes) = which else {
                return Err(Error::BattleSelectionMismatch);
            };
            indexes.first().cloned().unwrap_or(usize::MAX)
        };
        if index >= self.player.hand_deck.len() {
            return Err(Error::BattleSelectionError);
        }
        let card = self.player.hand_deck.remove(index);
        if self.player.power < card.power_cost {
            return Err(Error::BattlePowerInsufficient);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        self.player.power -= card.power_cost;
        self.player.grave_deck.push(card);
        let effects = card.effect_pool.iter().collect::<Vec<_>>();
        self.operate_effects(target, &effects)
    }

    fn iterate_special_card_use(&mut self, target: Target) -> Result<IterationOutput, Error> {
        if self.player.power < self.player.special_card.power_cost {
            return Err(Error::BattlePowerInsufficient);
        }
        if IterationOutput::Continue != self.last_output {
            return Err(Error::BattleUnexpectedOutput);
        };
        self.player.power -= self.player.special_card.power_cost;
        let effects = self
            .player
            .special_card
            .effect_pool
            .iter()
            .collect::<Vec<_>>();
        self.operate_effects(target, &effects)
    }

    fn iterate_player_round_end(&mut self) -> Result<IterationOutput, Error> {
        Ok(IterationOutput::Continue)
    }

    fn iterate_enemy_turn(&mut self) -> Result<IterationOutput, Error> {
        Ok(IterationOutput::GameLose)
    }

    fn fight_log() -> Result<(), Error> {
        Ok(())
    }

    fn operate_effects(
        &mut self,
        target: Target,
        effects: &[&'a Effect],
    ) -> Result<IterationOutput, Error> {
        let mut queue = match target {
            Target::Player => vec![&mut self.player.fight_effects],
            Target::Enemy(offset) => {
                let enemy = self
                    .opponents
                    .get_mut(offset)
                    .ok_or(Error::BattleSelectionError)?;
                vec![&mut enemy.fight_effects]
            }
            Target::AllEnemy => self
                .opponents
                .iter_mut()
                .map(|enemy| &mut enemy.fight_effects)
                .collect(),
            Target::AllCharactor => {
                let mut queue = self
                    .opponents
                    .iter_mut()
                    .map(|enemy| &mut enemy.fight_effects)
                    .collect::<Vec<_>>();
                queue.append(&mut vec![&mut self.player.fight_effects]);
                queue
            }
            _ => return Err(Error::BattleUnexpectedPosition),
        };
        self.pending_instructions
            .push(Instruction::PositionChange(target));
        effects
            .iter()
            .map(|v| {
                if v.on_trigger.is_none() {
                    if let Some(execute) = &v.on_execution {
                        self.pending_instructions
                            .push(Instruction::SystemCall(execute));
                    } else {
                        if v.on_discard.is_none() || v.duration.is_none() {
                            return Err(Error::ResourceEffectSetupConflict);
                        }
                    }
                }
                queue.iter_mut().for_each(|delay_queue| delay_queue.push(v));
                Ok(())
            })
            .collect::<Result<_, _>>()?;
        self.operate_pending_instructions(None)
    }

    fn operate_pending_instructions(
        &mut self,
        selected_indexes: Option<Vec<usize>>,
    ) -> Result<IterationOutput, Error> {
        if self.pending_instructions.is_empty() {
            return Err(Error::BattleNoPendingContext);
        }
        // TODO: continue effect
        let effect = self.pending_instructions.remove(0);

        let pending_effects = self.pending_instructions.drain(..).collect::<Vec<_>>();
        Ok(IterationOutput::Continue)
    }
}
