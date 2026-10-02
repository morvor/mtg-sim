//! Engine support for basic effect verbs with qualified objects (see
//! `oracle/patterns/basic_effects_*.rs`):
//!
//! * "Counter target spell that's the second spell cast this turn." (Second Guess): the
//!   second spell any player cast this turn, by cast order (copies aren't cast, CR 707.10).
//! * The permanent whose ability is in a target slot ("If a permanent's ability is
//!   countered this way, destroy that permanent."): the object that was the ability's
//!   source, if it's still that object (CR 400.7); a stack ability's source is the object
//!   it came from (CR 113.7, 113.7a).

use super::{KeywordRegistration, KeywordRules};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;

/// `Filter::Custom`: the second spell cast this turn.
pub const SECOND_SPELL_CAST_THIS_TURN: &str = "basic_effects:second spell cast this turn";

/// `Filter::Custom`: a source that dealt damage this turn.
pub const DEALT_DAMAGE_THIS_TURN: &str = "basic_effects:dealt damage this turn";
/// `Filter::Custom`: a creature that blocked this turn.
pub const BLOCKED_THIS_TURN: &str = "basic_effects:blocked this turn";
const BLOCKED_OR_WAS_BLOCKED_BY: &str = "basic_effects:blocked or was blocked by:";

/// `Filter::Custom` name: a creature that blocked, or was blocked by, a creature matching
/// `by` this turn.
pub fn blocked_or_was_blocked_by(by: &crate::ability::Filter) -> String {
    format!(
        "{BLOCKED_OR_WAS_BLOCKED_BY}{}",
        serde_json::to_string(by).unwrap_or_default()
    )
}

/// The (blocker, attacker) pairs of blocks made this turn.
fn blocks_this_turn(g: &Game) -> Vec<(ObjectId, ObjectId)> {
    let mut v = Vec::new();
    for e in &g.turn_events {
        match e {
            Event::BlockersDeclared { blocks } => v.extend(blocks.iter().copied()),
            Event::BlockAdded {
                blocker, attacker, ..
            } => v.push((*blocker, *attacker)),
            _ => {}
        }
    }
    v
}

/// `Effect::Custom` name prefix: a player looks at the top cards of a library, exiles
/// some of them, and puts the rest back in any order or leaves them (see [`LookAtTop`]).
const LOOK_AT_TOP: &str = "basic_effects:look at top:";

/// "Look at the top three cards of target player's library, then put them back in any
/// order", "That player looks at the top three cards of your library, then puts them back
/// in any order", "Look at the top two cards of target opponent's library, then exile one
/// of them": the player looking (not necessarily the library's owner) makes the choices.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LookAtTop {
    pub library: crate::ability::PlayerRef,
    pub looker: crate::ability::PlayerRef,
    pub n: crate::ability::Value,
    /// How many of them the looker exiles.
    pub exile: u32,
    /// The rest go back on top in the order the looker chooses (else they stay as they
    /// are).
    pub reorder: bool,
}

/// The `Effect::Custom` for a [`LookAtTop`].
pub fn look_at_top(l: &LookAtTop) -> crate::ability::Effect {
    crate::ability::Effect::Custom(
        format!("{LOOK_AT_TOP}{}", serde_json::to_string(l).unwrap_or_default()).into(),
    )
}

fn perform_look_at_top(g: &mut Game, l: &LookAtTop, ctx: &mut Ctx) {
    let Some(owner) = g.eval_player(&l.library, ctx) else {
        return;
    };
    let Some(looker) = g.eval_player(&l.looker, ctx) else {
        return;
    };
    let n = g.eval_value(&l.n, ctx).max(0) as u32;
    let cards = crate::library::top_cards(g, owner, n);
    let k = l.exile.min(cards.len() as u32);
    let exiled = g.ask_objects(looker, ctx.source, "Choose cards to exile", cards.clone(), k, k);
    let rest: Vec<ObjectId> = cards.iter().copied().filter(|c| !exiled.contains(c)).collect();
    if !exiled.is_empty() {
        let moved = g.move_to_destination(
            exiled,
            &crate::ability::Destination::zone(crate::ability::ZoneKind::Exile),
            ctx,
        );
        ctx.set_var(
            crate::ability::vars::IT,
            moved.into_iter().map(Entity::Object).collect(),
        );
    }
    if l.reorder && rest.len() > 1 {
        let names = rest
            .iter()
            .map(|c| g.obj(*c).chars.name.to_string())
            .collect();
        let order: Vec<ObjectId> = g
            .ask_order(looker, "Order the cards to put on top (top first)", names)
            .into_iter()
            .map(|i| rest[i])
            .collect();
        crate::library::put_on_top(g, owner, &order);
    }
}

/// `Modification::Custom` name prefix: the object loses the keyword instances described
/// (see [`remove_keyword`]).
const REMOVE_KEYWORD: &str = "basic_effects:remove keyword:";

/// Which instances of a keyword an object loses: "loses forestwalk" (that landwalk only,
/// not islandwalk, CR 702.14), "loses all \"bands with other\" abilities" (every banding
/// with a quality, not banding itself, CR 702.22).
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RemoveKeyword {
    pub kind: KeywordKind,
    /// `Some(filter)`: the instances with exactly that parameter; `None`: every instance
    /// with a parameter.
    pub filter: Option<crate::ability::Filter>,
}

/// The `Modification::Custom` that removes keyword instances (layer 6).
pub fn remove_keyword(r: &RemoveKeyword) -> crate::ability::Modification {
    crate::ability::Modification::Custom {
        name: format!("{REMOVE_KEYWORD}{}", serde_json::to_string(r).unwrap_or_default()).into(),
        layer: crate::ability::Layer::L6Ability,
    }
}

const SOURCE_OF_SLOT: &str = "basic_effects:source of the ability in target slot ";

/// `Filter::Custom` name: the source of the ability chosen in target slot `slot`.
pub fn source_of_slot(slot: u8) -> String {
    format!("{SOURCE_OF_SLOT}{slot}")
}

pub struct BasicEffects;

impl KeywordRules for BasicEffects {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_modification(
        &self,
        _g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        _ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        let Some(json) = name.strip_prefix(REMOVE_KEYWORD) else {
            return false;
        };
        let Ok(r) = serde_json::from_str::<RemoveKeyword>(json) else {
            return true;
        };
        let want = r.filter.as_ref().map(|f| serde_json::to_string(f).unwrap_or_default());
        chars.abilities.retain(|a| {
            let Some(k) = a.keyword() else {
                return true;
            };
            let same = k.kind == r.kind
                && match (&want, &k.filter) {
                    (Some(w), Some(f)) => serde_json::to_string(f).is_ok_and(|j| &j == w),
                    (None, Some(_)) => true,
                    _ => false,
                };
            !same
        });
        true
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(json) = name.strip_prefix(LOOK_AT_TOP) else {
            return false;
        };
        if let Ok(l) = serde_json::from_str::<LookAtTop>(json) {
            perform_look_at_top(g, &l, ctx);
        }
        true
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name == DEALT_DAMAGE_THIS_TURN {
            return Some(g.turn_events.iter().any(|e| {
                matches!(e, Event::Damage { source, amount, .. } if *source == id && *amount > 0)
            }));
        }
        if name == BLOCKED_THIS_TURN {
            return Some(blocks_this_turn(g).iter().any(|(b, _)| *b == id));
        }
        if let Some(json) = name.strip_prefix(BLOCKED_OR_WAS_BLOCKED_BY) {
            let by: crate::ability::Filter = serde_json::from_str(json).ok()?;
            let other_ok = |o: ObjectId| {
                g.matches_view(&crate::eval::Current, o, &by, ctx)
            };
            return Some(blocks_this_turn(g).iter().any(|(b, a)| {
                (*b == id && other_ok(*a)) || (*a == id && other_ok(*b))
            }));
        }
        if name == SECOND_SPELL_CAST_THIS_TURN {
            return Some(g.history.spells_cast.get(1).is_some_and(|(_, s)| *s == id));
        }
        if let Some(slot) = name.strip_prefix(SOURCE_OF_SLOT) {
            let slot: usize = slot.parse().ok()?;
            let targets = ctx.targets.get(slot)?;
            return Some(targets.iter().any(|t| {
                let Entity::Object(a) = t else {
                    return false;
                };
                matches!(
                    g.obj(*a).stack.as_deref().map(|si| &si.kind),
                    Some(StackKind::Activated { source, .. } | StackKind::Triggered { source, .. })
                        if *source == id
                )
            }));
        }
        None
    }
}

inventory::submit! { KeywordRegistration(&BasicEffects) }
