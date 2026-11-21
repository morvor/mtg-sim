//! Fear of Sleep Paralysis: "Stun counters can't be removed from permanents your opponents
//! control." Their stun counters' replacement effect still applies (CR 122.1d): such a
//! permanent doesn't untap and keeps the counter (ruling); the counters can't be moved
//! either (ruling).

use super::{active, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{counters, Entity};

const STUCK: &str = "card:Fear of Sleep Paralysis:stun counters can't be removed from opponents' permanents";
const TEXT: &str = "Stun counters can't be removed from permanents your opponents control.";

inventory::submit! { ManualAbility {
    card: "Fear of Sleep Paralysis",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(STUCK, TEXT)],
    reason: "stun counters can't be removed from opponents' permanents: the only rule stopping counters from being removed",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn counters_cant_be_removed(&self, g: &Game, target: Entity, kind: &str) -> bool {
        let Entity::Object(o) = target else {
            return false;
        };
        if kind != counters::STUN || g.obj(o).zone != Zone::Battlefield {
            return false;
        }
        let ctl = g.obj(o).controller;
        active(g, STUCK)
            .into_iter()
            .any(|(_, you)| g.are_opponents(you, ctl))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
