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
const PAID_TIMES: &str = "spell cost:times paid:";
const MANA_COST_IS: &str = "spell cost:mana cost is:";
const PAY_ANY: &str = "spell cost:pay any amount:";
const UNLOCK_LESS: &str = "spell cost:unlock costs less:";

/// `Value::Custom`: how many of the source's chosen colors ("As ~ enters, choose two
/// colors") the spell being cast or the triggering object is.
pub const CHOSEN_COLORS_IT_IS: &str = "spell cost:chosen colors it is";

/// `StaticEffect::Custom` name: "Unlock costs you pay cost {n} less."
pub fn unlock_costs_less(n: u32) -> String {
    format!("{UNLOCK_LESS}{n}")
}

/// How much less the unlock costs `p` pays cost (CR 709.5e, 118.7a: generic mana only).
pub fn unlock_cost_reduction(g: &Game, p: PlayerId) -> u32 {
    g.statics
        .customs
        .iter()
        .filter(|(_, ctl, _)| *ctl == p)
        .filter_map(|(_, _, n)| n.strip_prefix(UNLOCK_LESS)?.parse::<u32>().ok())
        .sum()
}

/// `Effect::Custom` name: the player chooses an amount of `kind` ("life", "mana", or a mana
/// symbol such as "{r}") and pays it (`may`: they may choose not to). The amount paid is
/// the ability's X from then on ("draw that many cards").
pub fn pay_any_amount(kind: &str, may: bool) -> String {
    format!("{PAY_ANY}{kind}:{}", if may { "may" } else { "must" })
}

/// The amount paid by [`pay_any_amount`], for a reflexive triggered ability's "that much"
/// (CR 603.12).
pub const AMOUNT_PAID: Var = crate::ability::vars::USER + 6011;

pub fn is_pay_any_amount(name: &str) -> bool {
    name.starts_with(PAY_ANY)
}

/// `Filter::Custom` name: the object's mana cost is exactly `m` ("with mana cost {0}").
pub fn mana_cost_is(m: &crate::mana::ManaCost) -> String {
    format!("{MANA_COST_IS}{m}")
}

/// `Value::Custom` name: how many times the spell `ctx.source`'s controller announced
/// they'd pay its repeatable additional cost `name` (CR 601.2b).
pub fn paid_times(name: &str) -> String {
    format!("{PAID_TIMES}{name}")
}

/// `Value::Custom`: how many targets the spell `ctx.source` has (each time an object or
/// player is chosen counts), once they're chosen (CR 601.2c); none before that.
pub const OWN_TARGETS: &str = "spell cost:own targets";

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

/// Locks in the amounts of the cost changes granted to "the next spell you cast" (CR
/// 611.2c): "costs {X} less to cast, where X is the number of cards looked at while
/// scrying this way".
pub fn lock_granted_cost_changes(g: &Game, mods: &mut [Modification], ctx: &Ctx) {
    for m in mods.iter_mut() {
        let Modification::AddAbility(a) = m else {
            continue;
        };
        let AbilityKind::Static(s) = &a.kind else {
            continue;
        };
        let StaticEffect::CostModifier(cm) = &s.effect else {
            continue;
        };
        if !matches!(cm.applies_to, CostTarget::ThisSpell) {
            continue;
        }
        let mut cm = cm.clone();
        let mut pm = PlayerModification::CostModifier(cm.clone());
        lock_player_effect(g, &mut pm, ctx);
        if let PlayerModification::CostModifier(locked) = pm {
            cm = locked;
        }
        let mut s = s.clone();
        s.effect = StaticEffect::CostModifier(cm);
        *a = AbilityDef::new(AbilityKind::Static(s), &a.text);
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

/// The names of the spell's own additional costs paid any number of times ("As an
/// additional cost to cast this spell, you may sacrifice any number of creatures."): each
/// is announced as the spell is cast, as a number of times (CR 601.2b).
fn repeatable_costs(chars: &Characteristics) -> Vec<smol_str::SmolStr> {
    let mut out = Vec::new();
    for a in &chars.abilities {
        let AbilityKind::Static(StaticAbility {
            effect:
                StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    change: CostChange::AdditionalCost(c),
                    ..
                }),
            ..
        }) = &a.kind
        else {
            continue;
        };
        for p in &c.parts {
            if let CostPart::Repeated {
                times: Value::Custom(n),
                ..
            } = p
            {
                if let Some(name) = n.strip_prefix(PAID_TIMES) {
                    out.push(name.into());
                }
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
        if let Some(m) = name.strip_prefix(MANA_COST_IS) {
            let o = g.obj(id);
            return Some(o.chars.mana_cost.as_ref().is_some_and(|c| format!("{c}") == m));
        }
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

    fn spell_optional_costs(
        &self,
        g: &Game,
        spell: ObjectId,
    ) -> Vec<(smol_str::SmolStr, Cost, bool)> {
        repeatable_costs(&g.obj(spell).chars)
            .into_iter()
            .map(|n| (n, Cost::free(), true))
            .collect()
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(r) = name.strip_prefix(PAY_ANY) else {
            return false;
        };
        let Some((kind, _may)) = r.split_once(':') else {
            return false;
        };
        let p = ctx.controller;
        let max = match kind {
            "life" => g.player(p).life.max(0) as i64,
            _ => g.max_mana_available(p) as i64,
        };
        let src = ctx.source.or(ctx.stack_obj).unwrap_or(ObjectId(0));
        let n = match g.ask(p, crate::decision::Decision::ChooseX { source: src, min: 0, max }) {
            crate::decision::Answer::Number(n) if n >= 0 => n.min(max),
            _ => 0,
        };
        let cost = match kind {
            "life" => Cost::free().with(CostPart::PayLife(Value::c(n as i32))),
            "mana" => Cost::mana(crate::mana::ManaCost::generic(n as u32)),
            sym => {
                let Some(one) = crate::mana::ManaCost::parse(&sym.to_uppercase()) else {
                    return true;
                };
                let mut m = crate::mana::ManaCost::default();
                for _ in 0..n {
                    m.add(&one);
                }
                Cost::mana(m)
            }
        };
        let paid = n > 0 && g.pay_cost(p, &cost, ctx.source, ctx);
        let n = if paid { n } else { 0 };
        ctx.x = n as i32;
        ctx.x_defined = true;
        ctx.nums.insert(AMOUNT_PAID, n);
        ctx.prev_happened = paid;
        true
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == CHOSEN_COLORS_IT_IS {
            let Some(chosen) = g.source_choices(ctx).and_then(|c| c.colors) else {
                return Some(0);
            };
            let Some(obj) = ctx.event.as_ref().and_then(|e| e.spell.or(e.object)) else {
                return Some(0);
            };
            let colors = g.obj(obj).chars.colors;
            return Some(chosen.iter().filter(|c| colors.contains(*c)).count() as i64);
        }
        if let Some(n) = name.strip_prefix(PAID_TIMES) {
            let times = ctx
                .source
                .and_then(|s| g.obj(s).stack.as_deref())
                .map_or(0, |si| si.cast.paid.iter().filter(|p| p.as_str() == n).count());
            return Some(times as i64);
        }
        if name == OWN_TARGETS {
            let n: usize = ctx
                .source
                .and_then(|s| g.obj(s).stack.as_deref())
                .map_or(0, |si| {
                    si.chosen
                        .iter()
                        .map(|cm| cm.targets.iter().map(Vec::len).sum::<usize>())
                        .sum()
                });
            return Some(n as i64);
        }
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
