#![cfg_attr(not(feature = "debug"), cfg_attr(not(feature = "json_ser"), no_std))]

pub mod battle;
pub mod contexts;
pub mod errors;
pub mod game;
pub mod map;
pub mod systems;
pub mod wrappings;
