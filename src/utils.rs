#![allow(dead_code)]

use serenity::model::channel::ReactionType;

pub const GREEN_CIRCLE: char = '\u{1F7E2}';
pub const RED_CIRCLE: char = '\u{1F534}';
pub const SKULL: char = '\u{1F480}';
pub const HALO_SMILEY: char = '\u{1F607}';
pub const ARROW_LOOP: char = '\u{1F504}';

pub fn react_success() -> ReactionType {
    ReactionType::Unicode(GREEN_CIRCLE.to_string())
}

pub fn react_failure() -> ReactionType {
    ReactionType::Unicode(RED_CIRCLE.to_string())
}

pub fn react_skull() -> ReactionType {
    ReactionType::Unicode(SKULL.to_string())
}

pub fn react_halo() -> ReactionType {
    ReactionType::Unicode(HALO_SMILEY.to_string())
}

pub fn react_loop() -> ReactionType {
    ReactionType::Unicode(ARROW_LOOP.to_string())
}


#[macro_export]
macro_rules! some_or_return {
    ($expression:expr) => {
        if let Some(value) = $expression {
            value
        } else {
            return;
        }
    };
}

#[macro_export]
macro_rules! get_data {
    ($expression:expr, $type:ty) => {
        $expression.data.read().get::<$type>().unwrap()
    };
}

#[macro_export]
macro_rules! get_data_mut {
    ($expression:expr, $type:ty) => {
        $expression.data.write().get_mut::<$type>().unwrap()
    };
}
