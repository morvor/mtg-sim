//! CR 702.195 Storied.
//!
//! * Storied is a static ability: "Any time you control three or more permanents that are
//!   artifacts, Sagas, and/or legendary and you don't have an enduring story, you have an
//!   enduring story for the rest of the game." (CR 702.195a). It doesn't use the stack;
//!   each permanent counts once, whatever qualities it has. Checked as each event is
//!   processed (before triggered abilities are detected for it) and whenever state-based
//!   actions are checked (before them).
//! * An enduring story is a designation of a player with no rules meaning of its own
//!   (CR 702.195b), kept in `KeywordState::enduring_story` for the rest of the game.
//! * After a player gets an enduring story, continuous effects are reapplied before the
//!   game checks whether any trigger conditions have been met (CR 702.195c).
//! * "You have an enduring story" is the condition [`HAS_ENDURING_STORY`].

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::types::*;

/// `Condition::Custom`: the ability's controller has an enduring story.
pub const HAS_ENDURING_STORY: &str = "storied:you have an enduring story";

/// Whether `p` has an enduring story (CR 702.195b).
pub fn has_enduring_story(g: &Game, p: PlayerId) -> bool {
    g.kw_state.enduring_story.contains(&p)
}

/// Gives each player who controls a permanent with storied and three or more artifacts,
/// Sagas, and/or legendary permanents an enduring story; reapplies continuous effects if
/// anyone got one (CR 702.195c).
fn check(g: &mut Game) {
    // Per controller: whether they control a permanent with storied, and how many
    // artifacts, Sagas, and/or legendary permanents they control.
    let mut seen: Vec<(PlayerId, bool, usize)> = Vec::new();
    for o in g.permanents() {
        if has_enduring_story(g, o.controller) {
            continue;
        }
        let i = match seen.iter().position(|(p, ..)| *p == o.controller) {
            Some(i) => i,
            None => {
                seen.push((o.controller, false, 0));
                seen.len() - 1
            }
        };
        seen[i].1 |= o.has_keyword(KeywordKind::Storied);
        if o.is(CardType::Artifact)
            || o.chars.has_subtype("Saga")
            || o.chars.has_supertype(Supertype::Legendary)
        {
            seen[i].2 += 1;
        }
    }
    let got: Vec<PlayerId> = seen
        .into_iter()
        .filter(|(_, storied, n)| *storied && *n >= 3)
        .map(|(p, ..)| p)
        .collect();
    if got.is_empty() {
        return;
    }
    for p in got {
        g.kw_state.enduring_story.insert(p);
        g.log(|_| format!("{p} has an enduring story"));
    }
    g.dirty = true;
    g.recompute();
}

pub struct Storied;

impl KeywordRules for Storied {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Storied]
    }

    fn on_event(&self, g: &mut Game, _ev: &Event) {
        check(g);
    }

    fn static_state_checks(&self, g: &mut Game) {
        check(g);
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        (name == HAS_ENDURING_STORY).then(|| has_enduring_story(g, ctx.controller))
    }
}

inventory::submit! { KeywordRegistration(&Storied) }
