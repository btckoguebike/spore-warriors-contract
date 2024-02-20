mod card;
mod enemy;
mod warrior;

pub use card::*;
pub use enemy::*;
pub use warrior::*;

use crate::errors::Error;

#[macro_export]
macro_rules! change_value {
    ($this:ident.$field:ident, $diff:ident, $conv:ty, $ori:ty) => {{
        let value = max($this.$field as $conv + $diff as $conv, 0);
        let value = min(value, <$ori>::MAX as $conv);
        $this.$field = value as $ori;
    }};
}

pub enum ContextType {
    Warrior,
    Enemy,
    Card,
}

pub trait CtxAdaptor<'a> {
    fn context_type(&self) -> ContextType;

    fn offset(&self) -> usize;

    fn warrior(&mut self) -> Result<&mut WarriorContext<'a>, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }

    fn enemy(&mut self) -> Result<&mut EnemyContext<'a>, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }

    fn card(&mut self) -> Result<&mut CardContext<'a>, Error> {
        Err(Error::BattleUnexpectedSystemContext)
    }
}
