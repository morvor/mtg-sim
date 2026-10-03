//! Rules behind the spell cost grammar (`oracle/patterns/spell_cost_grammar.rs`,
//! CR 601.2f):
//!
//! * While a spell's total cost is determined, a cost change another object makes for the
//!   spells a player casts sees the spell as "it" (`Sel::TriggerObject`) and its caster as
//!   "that player" / "its controller" (`PlayerRef::TriggerPlayer`); see
//!   `Game::base_total_cost`.
//! * [`SECOND_THIS_TURN`]: the spell is the second spell its caster casts this turn ("The
//!   second spell you cast each turn costs {1} less to cast." — Monk Class; a spell being
//!   cast isn't cast yet, CR 601.2i, and a copy was never cast). [`CASTER_NOT_ACTIVE`]:
//!   its caster isn't the active player ("except during its controller's turn").
//!   [`CASTER_ENCHANTED`]: its caster is the player the source enchants ("Spells with the
//!   chosen name enchanted player casts cost {2} more to cast.").
//! * "for each creature it targets": each time the spell targets a creature counts
//!   (Battlefield Thaumaturge ruling).
//! * A player effect that changes spells' costs for a while has its amount determined once,
//!   as the effect begins (CR 611.2c: "where X is the amount of life you lost this turn"
//!   is locked in, Rowan, Scion of War ruling) — see [`lock_player_effect`].
//! * "The next spell you cast this turn costs {2} less to cast" gives that spell the
//!   reduction as it's put on the stack (CR 601.2a, 611.2f); before then, a card that
//!   would be that spell is judged with it, so whether it can be cast accounts for it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;

pub const SECOND_THIS_TURN: &str = "spell cost:second spell this turn";
pub const CASTER_NOT_ACTIVE: &str = "spell cost:caster isn't the active player";
pub const CASTER_ENCHANTED: &str = "spell cost:caster is enchanted player";
const TARGETS_PREFIX: &str = "spell cost:targets of type:";

/// `Value::Custom` name: how many times the spell being cast targets an object of type `t`.
pub fn targets_of_type(t: CardType) -> String {
    format!("{TARGETS_PREFIX}{}", format!("{t:?}").to_lowercase())
}

/// The spell being cast, and its caster, as the cost change sees them.
fn spell_and_caster(ctx: &Ctx) -> (Option<ObjectId>, Option<PlayerId>) {
    match &ctx.event {
        Some(e) => (e.spell, e.player),
        None => (None, None),
    }
}

/// The quality of "the second [quality] spell" in the source's cost change.
fn quality(g: &Game, src: ObjectId) -> Option<Filter> {
    g.obj(src).chars.abilities.iter().find_map(|a| {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::And(parts)),
            ..
        }) = &s.effect
        else {
            return None;
        };
        let marker = |f: &Filter| matches!(f, Filter::Custom(n) if n == SECOND_THIS_TURN);
        if !parts.iter().any(marker) {
            return None;
        }
        Some(Filter::and(
            parts
                .iter()
                .filter(|f| !marker(f) && !matches!(f, Filter::Spell))
                .cloned()
                .collect(),
        ))
    })
}

/// Locks in the amounts of a player effect's spell cost change as the effect begins
/// (CR 611.2c).
pub fn lock_player_effect(g: &Game, effect: &mut PlayerModification, ctx: &Ctx) {
    let PlayerModification::CostModifier(cm) = effect else {
        return;
    };
    let lock = |v: &mut Value| {
        if !matches!(v, Value::Const(_)) {
            *v = Value::c(g.eval_value(v, ctx) as i32);
        }
    };
    match &mut cm.change {
        CostChange::IncreaseGeneric(v)
        | CostChange::ReduceGeneric(v)
        | CostChange::ReduceGenericMinOne(v)
        | CostChange::ReduceColored(_, v) => lock(v),
        _ => {}
    }
}

/// The reductions waiting "next spell" effects would give `card` if `p` cast it now.
fn pending_next_spell_reductions(g: &Game, p: PlayerId, card: ObjectId) -> Vec<(CostChange, Ctx)> {
    let turn = g.turn.number;
    let mut out = Vec::new();
    for e in &g.next_spell_effects {
        if e.player != p
            || (matches!(e.expires, Duration::EndOfTurn | Duration::ThisTurn)
                && e.created_turn != turn)
        {
            continue;
        }
        let ctx = Ctx::new(e.source, e.player);
        if !g.matches(card, &crate::casting::as_spell_filter(&e.filter), &ctx) {
            continue;
        }
        for m in &e.mods {
            let Modification::AddAbility(a) = m else {
                continue;
            };
            if let AbilityKind::Static(StaticAbility {
                effect:
                    StaticEffect::CostModifier(CostModifier {
                        applies_to: CostTarget::ThisSpell,
                        change: change @ (CostChange::ReduceGeneric(_) | CostChange::ReduceColored(..)),
                        ..
                    }),
                ..
            }) = &a.kind
            {
                out.push((change.clone(), Ctx::new(Some(card), p)));
            }
        }
    }
    out
}

pub struct SpellCostGrammar;

impl KeywordRules for SpellCostGrammar {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        match name {
            CASTER_NOT_ACTIVE => {
                let (_, caster) = spell_and_caster(ctx);
                Some(caster.is_some_and(|p| !g.is_active_player(p)))
            }
            CASTER_ENCHANTED => {
                let (_, caster) = spell_and_caster(ctx);
                let host = ctx
                    .source
                    .and_then(|s| g.obj(s).attached_to)
                    .and_then(|e| match e {
                        Entity::Player(p) => Some(p),
                        Entity::Object(_) => None,
                    });
                Some(caster.is_some() && caster == host)
            }
            SECOND_THIS_TURN => {
                let q = ctx.source.and_then(|s| quality(g, s))?;
                let caster = ctx.controller;
                let mut before = 0;
                for (p, s) in &g.history.spells_cast {
                    if *p != caster {
                        continue;
                    }
                    if *s == id {
                        return Some(before == 1);
                    }
                    if g.matches(*s, &q, ctx) {
                        before += 1;
                    }
                }
                Some(before == 1 && g.obj(id).kind != ObjKind::SpellCopy)
            }
            _ => None,
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        let t = CardType::from_word(name.strip_prefix(TARGETS_PREFIX)?)?;
        let (spell, _) = spell_and_caster(ctx);
        let Some(si) = spell.and_then(|s| g.obj(s).stack.as_deref()) else {
            return Some(0);
        };
        let n = si
            .chosen
            .iter()
            .flat_map(|cm| cm.targets.iter().flatten())
            .filter(|e| match e {
                Entity::Object(o) => {
                    g.is_live(*o) && g.obj(*o).chars.card_types.contains(t)
                }
                Entity::Player(_) => false,
            })
            .count();
        Some(n as i64)
    }

    fn global_spell_cost(&self, g: &Game, p: PlayerId, card: ObjectId, cost: &mut Cost) {
        // A card about to be cast: the reduction it would gain as it's put on the stack.
        if g.obj(card).zone == Zone::Stack {
            return;
        }
        for (change, ctx) in pending_next_spell_reductions(g, p, card) {
            let mut changes = crate::activation_costs::CostChanges::default();
            changes.add(g, cost, &change, &ctx);
            changes.apply(cost, |_, _, _| crate::cost_rules::Half::Colorless);
        }
    }
}

inventory::submit! { KeywordRegistration(&SpellCostGrammar) }
