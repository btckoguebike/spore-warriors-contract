extern crate alloc;
use alloc::{vec, vec::Vec};
use rand::RngCore;

use crate::battle::pve::{FightView, Instruction, MapBattlePVE};
use crate::battle::traits::{FightLog, IterationOutput};
use crate::contexts::CtxAdaptor;
use crate::errors::Error;
use crate::systems::{Command, SystemController, SystemInput, SystemReturn};
use crate::wrappings::RequireTarget;

impl<'a> MapBattlePVE<'a> {
    pub(super) fn player_draw(
        &mut self,
        draw_count: u8,
        controller: &mut SystemController,
    ) -> Result<(), Error> {
        if draw_count == 0 {
            return Err(Error::BattleUnexpectedDrawCount);
        }
        for _ in 0..draw_count {
            if self.player_deck.deck.is_empty() {
                let mut grave_cards = self.player_deck.grave_deck.drain(..).collect::<Vec<_>>();
                if grave_cards.is_empty() {
                    return Err(Error::BattleInternalError);
                }
                self.player_deck.deck.append(&mut grave_cards);
                self.trigger_log(FightLog::RecoverGraveDeck)?;
            }
            let card_index = controller.rng.next_u32() as usize % self.player_deck.deck.len();
            let card = self.player_deck.deck.remove(card_index);
            self.trigger_log(FightLog::Draw(card.offset()))?;
            self.player_deck.hand_deck.push(card);
        }
        Ok(())
    }

    pub(super) fn player_select_draw(&mut self, card_offsets: Vec<usize>) -> Result<(), Error> {
        for offset in card_offsets {
            if !self.player_deck.selection_pool.contains(&offset) {
                return Err(Error::BattleUnexpectedCardOffset);
            }
            for (deck, remove) in [
                (&mut self.player_deck.deck, true),
                (&mut self.player_deck.grave_deck, true),
                (&mut self.player_deck.unbelonging_deck, false),
            ] {
                let Some((index, card)) =
                    deck.iter().enumerate().find(|(_, v)| v.offset() == offset)
                else {
                    return Err(Error::BattleCardOffsetNotFound);
                };
                if remove {
                    self.player_deck.hand_deck.push(deck.remove(index));
                } else {
                    self.player_deck.hand_deck.push(card.clone());
                }
            }
            self.trigger_log(FightLog::Draw(offset))?;
        }
        self.player_deck.selection_pool.clear();
        Ok(())
    }

    pub(super) fn player_select_discard(
        &mut self,
        discard_offsets: Vec<usize>,
        grave: bool,
    ) -> Result<(), Error> {
        if discard_offsets.is_empty() {
            return Err(Error::BattleUnexpectedDiscardCount);
        }
        for offset in discard_offsets {
            let Some((index, _)) = self
                .player_deck
                .hand_deck
                .iter()
                .enumerate()
                .find(|(_, card)| card.offset() == offset)
            else {
                break;
            };
            let hand_card = self.player_deck.hand_deck.remove(index);
            self.trigger_log(FightLog::DiscardHandDeck(hand_card.offset()))?;
            if grave {
                self.player_deck.grave_deck.push(hand_card);
            } else {
                self.player_deck.unavaliable_deck.push(hand_card);
            }
        }
        Ok(())
    }

    pub(super) fn player_random_discard(
        &mut self,
        discard_count: u8,
        grave: bool,
        controller: &mut SystemController,
    ) -> Result<(), Error> {
        if discard_count == 0 {
            return Err(Error::BattleUnexpectedDiscardCount);
        }
        for _ in 0..discard_count {
            if self.player_deck.hand_deck.is_empty() {
                break;
            }
            let card_index = controller.rng.next_u32() as usize % self.player_deck.hand_deck.len();
            let hand_card = self.player_deck.hand_deck.remove(card_index);
            self.trigger_log(FightLog::DiscardHandDeck(hand_card.offset()))?;
            if grave {
                self.player_deck.grave_deck.push(hand_card);
            } else {
                self.player_deck.unavaliable_deck.push(hand_card);
            }
        }
        Ok(())
    }

    pub(super) fn collect_system_caster_offset(
        &self,
        view: FightView,
        target: RequireTarget,
        target_offset: Option<usize>,
    ) -> Result<usize, Error> {
        match (view, target) {
            (FightView::Card(offset), _) => Ok(offset),
            (FightView::Player, _) => Ok(self.player.offset()),
            (FightView::Enemy, _) => {
                let Some(offset) = target_offset else {
                    return Err(Error::BattleSelectionError);
                };
                Ok(offset)
            }
        }
    }

    pub(super) fn collect_system_target_offsets(
        &mut self,
        view: FightView,
        target: RequireTarget,
        target_offset: Option<usize>,
        controller: &mut SystemController,
    ) -> Result<Vec<usize>, Error> {
        let mut targets = vec![];
        match (view, target) {
            (FightView::Card(offset), _) => {
                let card = self
                    .player_deck
                    .refer_card(offset)
                    .ok_or(Error::BattleInternalError)?;
                targets.push(card.offset());
            }
            (FightView::Player, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::Owner) => {
                let Some(offset) = target_offset else {
                    return Err(Error::BattleSelectionError);
                };
                let enemy = self
                    .opponents
                    .get_mut(offset)
                    .ok_or(Error::BattleEnemyNotFound)?;
                targets.push(enemy.offset());
            }
            (FightView::Player, RequireTarget::AllOpponents) => {
                self.opponents
                    .iter_mut()
                    .for_each(|v| targets.push(v.offset()));
            }
            (FightView::Player, RequireTarget::AllCharactors)
            | (FightView::Enemy, RequireTarget::AllCharactors) => {
                self.opponents
                    .iter_mut()
                    .for_each(|v| targets.push(v.offset()));
                targets.push(self.player.offset());
            }
            (FightView::Player, RequireTarget::RandomOpponent) => {
                let offset = controller.rng.next_u32() as usize % self.opponents.len();
                let enemy = self.opponents.get_mut(offset).unwrap();
                targets.push(enemy.offset());
            }
            (FightView::Player, RequireTarget::Owner)
            | (FightView::Enemy, RequireTarget::Opponent)
            | (FightView::Enemy, RequireTarget::RandomOpponent)
            | (FightView::Enemy, RequireTarget::AllOpponents) => targets.push(self.player.offset()),
        };
        Ok(targets)
    }

    pub(super) fn operate_pending_instructions(
        &mut self,
        controller: &mut SystemController,
    ) -> Result<IterationOutput, Error> {
        let mut game_over = false;
        self.last_output = IterationOutput::Continue;
        while let Some(Instruction {
            view,
            ctx,
            target,
            mut system_input,
        }) = self.pending_instructions.pop_front()
        {
            let target_type = ctx.system.target_type;
            let caster = self.collect_system_caster_offset(view, target_type, target)?;
            self.trigger_log(FightLog::CallSystem(caster, ctx.clone()))?;
            let targets =
                self.collect_system_target_offsets(view, target_type, target, controller)?;
            let mut objects: Vec<&mut dyn CtxAdaptor> = vec![self.player];
            self.opponents.iter_mut().for_each(|v| objects.push(v));
            self.player_deck
                .collect_cards()
                .into_iter()
                .for_each(|v| objects.push(v));
            if game_over {
                system_input = Some(SystemInput::Trigger(FightLog::GameOver));
            }
            let system_return =
                controller.system_call(ctx, caster, targets, &mut objects, system_input.clone())?;
            if !game_over {
                if self.player.hp == 0 {
                    self.last_output = IterationOutput::GameLose;
                    self.trigger_log(FightLog::GameOver)?;
                    game_over = true;
                    continue;
                }
                if self.opponents.iter().all(|v| v.hp == 0) {
                    self.last_output = IterationOutput::GameWin;
                    self.trigger_log(FightLog::GameOver)?;
                    game_over = true;
                    continue;
                }
                self.operate_system_return(system_return, view, controller)?;
                if let IterationOutput::RequireCardSelect(_, _) = self.last_output {
                    break;
                }
            }
        }
        if game_over {
            self.player.reset();
        }
        Ok(self.last_output)
    }

    fn operate_system_return(
        &mut self,
        system_return: SystemReturn,
        view: FightView,
        controller: &mut SystemController,
    ) -> Result<(), Error> {
        let mut return_cmds = vec![];
        match system_return {
            SystemReturn::Continue(cmds) => return_cmds = cmds,
            SystemReturn::RequireCardSelect(select_count, is_draw, operator) => {
                if let FightView::Player = view {
                    return Err(Error::ResourceEffectCardSelectInEnemy);
                }
                self.last_output = IterationOutput::RequireCardSelect(select_count, is_draw);
                if let Some(mut changer) = operator {
                    let logs = changer(self.player_deck);
                    return_cmds = vec![Command::AddLogs(logs)];
                }
            }
            SystemReturn::RequireDeckChange(mut changer) => {
                let logs = changer(self.player_deck);
                return_cmds = vec![Command::AddLogs(logs)];
            }
        };
        for cmd in return_cmds {
            match cmd {
                Command::AddLogs(mut logs) => self.fight_logs.append(&mut logs),
                Command::DrawCards(count) => self.player_draw(count, controller)?,
                Command::DiscardHandCards(count, to_grave) => {
                    self.player_random_discard(count, to_grave, controller)?
                }
            }
        }
        Ok(())
    }
}
