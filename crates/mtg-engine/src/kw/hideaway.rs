//! CR 702.75 Hideaway: "Hideaway N" means "When this permanent enters, look at the top N
//! cards of your library. Exile one of them face down and put the rest on the bottom of
//! your library in a random order. The exiled card gains 'The player who controls the
//! permanent that exiled this card may look at this card in the exile zone.'"
//! (CR 702.75a). The exiled card is linked to the permanent (CR 607.2a): its other
//! abilities refer to it as "the exiled card".
//!
//! Older cards printed with "Hideaway" and no number have Oracle errata to "Hideaway 4"
//! and "[This permanent] enters tapped." (CR 702.75b).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::Zone;
use crate::types::*;

/// `Value::Custom`: the number of creatures its controller attacked with this turn
/// (Windbrisk Heights).
pub const ATTACKERS_THIS_TURN: &str = "hideaway:creatures you attacked with this turn";
/// `Value::Custom`: the most damage dealt to one of its controller's opponents this turn
/// (Spinerock Knoll).
pub const MOST_DAMAGE_TO_AN_OPPONENT: &str = "hideaway:most damage dealt to an opponent this turn";
/// `Value::Custom`: the number of cards in the smallest library (Shelldock Isle).
pub const SMALLEST_LIBRARY: &str = "hideaway:cards in the smallest library";

pub struct Hideaway;

impl KeywordRules for Hideaway {
    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let p = ctx.controller;
        Some(match name {
            ATTACKERS_THIS_TURN => {
                let mut seen: Vec<ObjectId> = Vec::new();
                for a in &g.history.attackers {
                    if g.obj(*a).controller == p && !seen.contains(a) {
                        seen.push(*a);
                    }
                }
                seen.len() as i64
            }
            MOST_DAMAGE_TO_AN_OPPONENT => g
                .history
                .damage_dealt_to_players
                .iter()
                .filter(|(q, _)| g.are_opponents(p, **q))
                .map(|(_, n)| *n as i64)
                .max()
                .unwrap_or(0),
            SMALLEST_LIBRARY => g
                .players
                .iter()
                .filter(|q| q.in_game())
                .map(|q| q.library.len() as i64)
                .min()
                .unwrap_or(0),
            _ => return None,
        })
    }

    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Hideaway]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let n = kw.n.unwrap_or(4);
        let mut exile = Destination::zone(ZoneKind::Exile);
        exile.face_down = true;
        let mut bottom = Destination::library_bottom();
        bottom.position = LibraryPosition::BottomRandom;
        let t = TriggeredAbility::new(
            TriggerCond::EntersBattlefield(Filter::Source),
            Body::effect(Effect::Seq(vec![
                Effect::Dig {
                    who: PlayerRef::You,
                    n: Value::c(n),
                    reveal: false,
                    filter: Filter::Any,
                    take: Value::c(1),
                    take_up_to: false,
                    take_to: exile,
                    rest_to: bottom,
                },
                // The player who controls the permanent may look at the exiled card.
                Effect::Custom(crate::zones::MAY_LOOK_AT_EXILED.into()),
            ])),
        );
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            format!("Hideaway {n}"),
        )])
    }

    /// Whoever controls a permanent with hideaway may look at the cards it exiled face
    /// down (a new controller too, for as long as the card stays exiled). Not a
    /// state-based action: the permission is just kept up to date whenever they're
    /// checked.
    fn state_based_actions(&self, g: &mut Game) -> bool {
        let mut allow: Vec<(PlayerId, ObjectId)> = Vec::new();
        for o in g.permanents() {
            if !o.chars.has_keyword(KeywordKind::Hideaway) {
                continue;
            }
            for c in o.linked.values().flatten() {
                let c = g.current(*c);
                let e = g.obj(c);
                if e.zone == Zone::Exile && e.face_down && !crate::zones::may_look(g, o.controller, c)
                {
                    allow.push((o.controller, c));
                }
            }
        }
        for (p, c) in allow {
            crate::zones::allow_look(g, p, c);
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&Hideaway) }
