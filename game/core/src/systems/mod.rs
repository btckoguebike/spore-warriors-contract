extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use rand::RngCore;
use spore_warriors_generated as generated;

use crate::contexts::CtxAdaptor;
use crate::wrappings::Value;

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum SystemId {
    NormalDamage,
}

pub enum SystemReturn {
    Null,
}

pub struct SystemImport<'a, T: RngCore> {
    resource_pool: &'a generated::ResourcePool,
    rng: &'a mut T,
    args: Vec<Value>,
    contexts: Vec<Box<dyn CtxAdaptor>>,
    user_imported: Vec<usize>,
}

pub type SystemCallback<'a, T> = fn(SystemImport<'a, T>) -> SystemReturn;

pub fn initialize_system_callbacks<'a, T: RngCore>() -> BTreeMap<SystemId, SystemCallback<'a, T>> {
    let mut system_callbacks = BTreeMap::new();
    system_callbacks.insert(
        SystemId::NormalDamage,
        normal_attack as SystemCallback<'a, T>,
    );
    system_callbacks
}

fn normal_attack<'a, T: RngCore>(input: SystemImport<'a, T>) -> SystemReturn {
    SystemReturn::Null
}
