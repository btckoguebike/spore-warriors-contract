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

struct BytesPusher(Vec<u8>);

impl BytesPusher {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push_u8(&mut self, data: u8) -> &mut Self {
        self.0.push(data);
        self
    }

    pub fn push_u16(&mut self, data: u16) -> &mut Self {
        self.0.append(&mut data.to_le_bytes().to_vec());
        self
    }

    pub fn push_usize(&mut self, data: usize) -> &mut Self {
        self.0.append(&mut data.to_le_bytes().to_vec());
        self
    }

    pub fn push_bytes(&mut self, mut data: Vec<u8>) -> &mut Self {
        self.0.append(&mut data);
        self
    }

    pub fn data(self) -> Vec<u8> {
        self.0
    }
}

struct BytesExtractor<'a> {
    source: &'a mut Vec<u8>,
    pointer: usize,
}

impl<'a> BytesExtractor<'a> {
    pub fn new(data: &'a mut Vec<u8>) -> Self {
        Self {
            source: data,
            pointer: 0,
        }
    }

    pub fn pop_u8(&mut self) -> Result<u8, Error> {
        if self.pointer >= self.source.len() {
            return Err(Error::SystemDeserializeError);
        }
        let value = self.source[self.pointer];
        self.pointer += 1;
        Ok(value)
    }

    pub fn pop_u16(&mut self) -> Result<u16, Error> {
        let new_pointer = self.pointer + u16::BITS as usize;
        if new_pointer > self.source.len() {
            return Err(Error::SystemDeserializeError);
        }
        let value = u16::from_le_bytes(self.source[self.pointer..new_pointer].try_into().unwrap());
        self.pointer = new_pointer;
        Ok(value)
    }

    pub fn pop_usize(&mut self) -> Result<usize, Error> {
        let new_pointer = self.pointer + usize::BITS as usize;
        if new_pointer > self.source.len() {
            return Err(Error::SystemDeserializeError);
        }
        let value =
            usize::from_le_bytes(self.source[self.pointer..new_pointer].try_into().unwrap());
        self.pointer = new_pointer;
        Ok(value)
    }
}
