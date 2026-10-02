//! Runtime support for the zone-move grammar (`oracle/patterns/zone_move_grammar.rs`):
//! "return a creature card at random from your graveyard to your hand" — the cards are
//! picked at random among those that match as the instruction is performed (CR 608.2c).

use super::{KeywordRegistration, KeywordRules};
use crate::types::Entity;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::events::{Event, MoveCause};
use crate::oracle::patterns::zone_move_grammar::{
    ACTIVATED_ABILITIES_OF_EXILED, DEALT_DAMAGE_THIS_TURN, DISCARDED_BY_YOU_THIS_TURN,
    ENTERED_UNDER_YOUR_CONTROL_THIS_TURN, MILLED_THIS_TURN, RANDOM_COUNT, RANDOM_PICK,
    RANDOM_POOL,
};
use crate::types::ObjectId;

/// `Effect::Custom`: picks [`RANDOM_COUNT`] of the objects in [`RANDOM_POOL`] at random
/// (all of them if there are fewer) and stores them in [`RANDOM_PICK`].
pub const PICK_AT_RANDOM: &str = "zone move: pick at random";

pub struct ZoneMoves;

impl KeywordRules for ZoneMoves {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != PICK_AT_RANDOM {
            return false;
        }
        use rand::seq::SliceRandom;
        let mut pool: Vec<Entity> = ctx.vars.get(&RANDOM_POOL).cloned().unwrap_or_default();
        let n = ctx.nums.get(&RANDOM_COUNT).copied().unwrap_or(0).max(0) as usize;
        pool.shuffle(&mut g.rng);
        pool.truncate(n);
        ctx.vars.insert(RANDOM_PICK, pool);
        true
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::ZoneChange { new, cause, by, .. } = ev {
            match cause {
                MoveCause::Mill => g.history.milled.push(*new),
                MoveCause::Discard => {
                    if let Some(p) = by {
                        g.history.discarded.push((*p, *new));
                    }
                }
                _ => {}
            }
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        match name {
            DEALT_DAMAGE_THIS_TURN => Some(g.history.damage_sources.iter().any(|(s, _)| *s == id)),
            MILLED_THIS_TURN => Some(g.history.milled.contains(&id)),
            DISCARDED_BY_YOU_THIS_TURN => {
                Some(g.history.discarded.contains(&(ctx.controller, id)))
            }
            ENTERED_UNDER_YOUR_CONTROL_THIS_TURN => Some(
                g.history
                    .permanents_entered
                    .iter()
                    .any(|e| e.id == id && e.controller == ctx.controller),
            ),
            _ => None,
        }
    }

    /// "Has all activated abilities of all [kind] cards exiled with ~" (layer 6): the
    /// activated abilities of the cards the source's linked abilities exiled that are
    /// still in exile (CR 607.2a), including activated abilities of keywords.
    fn custom_modification(
        &self,
        g: &Game,
        name: &str,
        chars: &mut crate::object::Characteristics,
        ctx: &Ctx,
        _target: ObjectId,
    ) -> bool {
        let Some(kind) = name.strip_prefix(ACTIVATED_ABILITIES_OF_EXILED) else {
            return false;
        };
        let card_type = crate::types::CardType::from_word(kind);
        let mut gained = Vec::new();
        for e in g.eval_sel(&crate::ability::Sel::Linked, ctx) {
            let Entity::Object(c) = e else { continue };
            let o = g.obj(c);
            if !o.is_card() || !matches!(o.zone, crate::object::Zone::Exile) {
                continue;
            }
            let ok = kind.is_empty()
                || match card_type {
                    Some(t) => o.chars.card_types.contains(t),
                    None => o.chars.has_subtype(kind),
                };
            if !ok {
                continue;
            }
            for a in &o.chars.abilities {
                match &a.kind {
                    crate::ability::AbilityKind::Activated(_) => gained.push(a.clone()),
                    crate::ability::AbilityKind::Keyword(k) => gained.extend(
                        crate::keyword_impls::derived_abilities(k).into_iter().filter(|d| {
                            matches!(d.kind, crate::ability::AbilityKind::Activated(_))
                        }),
                    ),
                    _ => {}
                }
            }
        }
        chars.abilities.extend(gained);
        true
    }
}

inventory::submit! { KeywordRegistration(&ZoneMoves) }
