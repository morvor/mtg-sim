//! CR 702.153 Casualty: "Casualty N" means "As an additional cost to cast this spell, you
//! may sacrifice a creature with power N or greater," and "When you cast this spell, if a
//! casualty cost was paid for it, copy it. If the spell has any targets, you may choose new
//! targets for the copy." (CR 702.153a). Paying the casualty cost follows the rules for
//! additional costs (CR 601.2b, 601.2f–h): one creature is sacrificed, and the spell is
//! copied once. The copy is created on the stack, not cast (CR 707.10), and it resolves
//! first.
//!
//! Each instance is paid separately and triggers based on its own payment (CR 702.153b):
//! paying the `i`th casualty cost of a spell (counting from 1) is recorded as
//! `"casualty#i"` in `CastInfo::paid`, and the `i`th "Casualty" triggered ability of the
//! spell checks it. If the spell loses that instance before its triggered ability resolves
//! (e.g. it loses all abilities), the ability still copies it (CR 113.7a).
//!
//! "Casualty X" (Ob Nixilis, the Adversary): as the spell is cast, its controller chooses
//! whether to pay the casualty cost and the value of X (CR 601.2b), the spell's X; then they
//! sacrifice a creature with power X or greater. "The copy isn't legendary and has
//! starting loyalty X." are exceptions to the copy effect (CR 707.9), part of the copy's
//! copiable values, which the token it becomes as it resolves keeps.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::{Affected, ContinuousEffect, Game, Layer1};
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;
use smol_str::SmolStr;

/// `Condition::Custom`: the casualty cost of the resolving (or triggering) casualty
/// ability's instance was paid.
pub const PAID: &str = "casualty:a casualty cost was paid for it";

/// The name recorded in `CastInfo::paid` for the `i`th (0-based) casualty cost of a spell.
pub fn cost_name(i: usize) -> SmolStr {
    SmolStr::new(format!("casualty#{}", i + 1))
}

/// `Effect::Custom` prefix of the copy made by a casualty ability whose keyword lists
/// exceptions for the copy ("The copy isn't legendary and has starting loyalty X."); the
/// exceptions follow, separated by commas.
pub const COPY_WITH_EXCEPTIONS: &str = "casualty:copy it with exceptions:";
/// Copy exception: the copy isn't legendary.
const NOT_LEGENDARY: &str = "not legendary";
/// Copy exception: the copy has starting loyalty X.
const LOYALTY_X: &str = "starting loyalty x";

/// "Sacrifice a creature with power N or greater." (for "Casualty X", power X or greater:
/// X is the value announced for the spell).
pub fn casualty_cost(n: i32) -> Cost {
    let power = if n < 0 { Value::X } else { Value::c(n) };
    Cost::free().with(CostPart::Sacrifice {
        filter: Filter::and(vec![
            Filter::creature(),
            Filter::Power(Cmp::Ge, Box::new(power)),
        ]),
        count: Value::c(1),
    })
}

/// The exceptions a casualty keyword's text lists for the copy ("Casualty X. The copy
/// isn't legendary and has starting loyalty X.").
fn copy_exceptions(kw: &Keyword) -> Vec<&'static str> {
    let text = kw.text.as_deref().unwrap_or("").to_lowercase();
    let Some((_, rest)) = text.split_once(". the copy ") else {
        return vec![];
    };
    let mut out = Vec::new();
    if rest.contains("isn't legendary") {
        out.push(NOT_LEGENDARY);
    }
    if rest.contains("has starting loyalty x") {
        out.push(LOYALTY_X);
    }
    out
}

pub struct Casualty;

impl KeywordRules for Casualty {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Casualty]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let exceptions = copy_exceptions(kw);
        // Copied from the spell as it last existed on the stack if it has left it.
        let copy = if exceptions.is_empty() {
            Effect::CopySpell {
                what: Sel::This,
                count: Value::c(1),
                new_targets: true,
            }
        } else {
            Effect::Custom(SmolStr::new(format!(
                "{COPY_WITH_EXCEPTIONS}{}",
                exceptions.join(",")
            )))
        };
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(copy),
        );
        t.zone = FunctionZone::Stack;
        t.intervening_if = Some(Condition::Custom(PAID.into()));
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Casualty.name(),
        )])
    }

    fn spell_optional_costs(&self, g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
        g.obj(spell)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Casualty)
            .enumerate()
            // "Casualty X" (N is -1): power X or greater.
            .filter_map(|(i, k)| Some((cost_name(i), casualty_cost(k.n?), false)))
            .collect()
    }

    /// "Copy it. [The copy has these exceptions.]" (CR 707.9): the copy's copiable values
    /// are the spell's with the exceptions.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(exceptions) = name.strip_prefix(COPY_WITH_EXCEPTIONS) else {
            return false;
        };
        let Some(spell) = ctx.source else {
            return true;
        };
        let x = g.cast_info(ctx).and_then(|c| c.x).unwrap_or(0).max(0);
        let Some(copy) = crate::copy::copy_spell(g, spell, ctx.controller, true) else {
            return true;
        };
        let mut values = g.obj(copy).copiable.clone();
        for e in exceptions.split(',') {
            match e {
                NOT_LEGENDARY => values.supertypes.remove(Supertype::Legendary),
                LOYALTY_X => values.loyalty = Some(x),
                _ => {}
            }
        }
        let id = g.new_effect_id();
        let timestamp = g.new_timestamp();
        let created_turn = g.turn.number;
        g.effects.push(ContinuousEffect {
            id,
            source: Some(copy),
            controller: ctx.controller,
            timestamp,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![copy]),
            mods: vec![],
            layer1: Some(Layer1::Copy {
                values: Box::new(values),
                exceptions: vec![],
            }),
            created_turn,
        });
        g.dirty = true;
        g.recompute();
        true
    }

    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        if name != PAID {
            return None;
        }
        // Which of the spell's casualty triggered abilities this is.
        let index = ctx.source.and_then(|s| {
            g.obj(s)
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(a.kind, AbilityKind::Triggered(_))
                        && a.text == KeywordKind::Casualty.name()
                })
                .position(|a| a.uid == ctx.ability_uid)
        });
        let Some(i) = index else {
            // The spell no longer has this ability (e.g. it lost all abilities). Once
            // triggered, the ability exists independently of its source (CR 113.7a); it
            // triggered because this instance's casualty cost was paid, and that doesn't
            // change: as it resolves (CR 603.4), it still was.
            return Some(ctx.stack_obj.is_some());
        };
        let paid = cost_name(i);
        Some(
            g.cast_info(ctx)
                .is_some_and(|c| c.paid.iter().any(|p| *p == paid)),
        )
    }
}

inventory::submit! { KeywordRegistration(&Casualty) }
