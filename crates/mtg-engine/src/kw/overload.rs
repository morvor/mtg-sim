//! CR 702.96 Overload.
//!
//! "Overload [cost]" means "You may choose to pay [cost] rather than pay this spell's mana
//! cost" and "If you chose to pay this spell's overload cost, change its text by
//! replacing all instances of the word 'target' with the word 'each.'" (CR 702.96a). The
//! first is an alternative cost ([`KeywordRules::cast_options`]); the second a
//! text-changing effect (CR 702.96c) applied to the spell in layer 3
//! ([`KeywordRules::spell_text_change`]) from the moment it's cast with that cost
//! (CR 601.2a), so it has no targets (CR 702.96b).
//!
//! The ability language stores text structurally: "target [thing]" is a target slot with
//! the effect referring to [`Sel::Target`]. "Each [thing]" instead selects every object or
//! player matching the slot's description as the spell resolves: the overloaded spell
//! stores them in a variable first, and its effect refers to that variable.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use serde_json::Value as J;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// The name recorded in `CastInfo::paid` when a spell is cast for its overload cost.
pub const OVERLOAD: &str = "overload";

/// The variable holding the objects and players "each" of target slot `i` selects.
fn slot_var(i: usize) -> Var {
    vars::USER + 96 + i as Var
}

pub struct Overload;

impl KeywordRules for Overload {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Overload]
    }

    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        let o = g.obj(card);
        let Some(cost) = kw.cost.clone() else {
            return vec![];
        };
        let mut opt = CastOption::normal(FaceState::Front);
        opt.method = CastMethod::Keyword(KeywordKind::Overload);
        // Only from a zone the card could be cast from.
        if o.zone != Zone::Hand(p) {
            let chars = g.option_characteristics(card, &opt);
            if !g.permitted_cards(p).contains(&card) || !g.permission_allows(p, card, &chars, false)
            {
                return vec![];
            }
        }
        opt.alt_cost = Some(super::modified_keyword_cost(
            g,
            p,
            KeywordKind::Overload,
            &cost,
        ));
        opt.tag = Some(OVERLOAD);
        vec![opt]
    }

    fn spell_text_change(
        &self,
        _g: &Game,
        _spell: ObjectId,
        _kw: &Keyword,
        paid: &[SmolStr],
        chars: &mut Characteristics,
    ) {
        if !paid.iter().any(|p| p == OVERLOAD) {
            return;
        }
        chars.abilities = chars.abilities.iter().map(overloaded_ability).collect();
        let text = target_to_each(&chars.rules_text);
        if text != *chars.rules_text {
            chars.rules_text = std::sync::Arc::from(text.as_str());
        }
    }
}

/// "target" → "each" in display text.
fn target_to_each(s: &str) -> String {
    s.replace("target", "each").replace("Target", "Each")
}

/// A spell ability with "target" replaced by "each" (the same ability if it has no
/// targets, or if its effect refers to its targets in a way "each" can't express).
/// Cached so the changed ability keeps a stable identity across recomputations.
fn overloaded_ability(a: &Ability) -> Ability {
    static CACHE: OnceLock<Mutex<HashMap<u64, Ability>>> = OnceLock::new();
    let AbilityKind::Spell(sa) = &a.kind else {
        return a.clone();
    };
    let cache = CACHE.get_or_init(Default::default);
    if let Some(x) = cache.lock().unwrap().get(&a.uid) {
        return x.clone();
    }
    let result = match overloaded_body(&sa.body) {
        Some(body) => std::sync::Arc::new(AbilityDef {
            uid: next_ability_uid(),
            kind: AbilityKind::Spell(SpellAbility { body }),
            text: target_to_each(&a.text),
            link: a.link,
        }),
        None => a.clone(),
    };
    cache
        .lock()
        .unwrap()
        .entry(a.uid)
        .or_insert(result)
        .clone()
}

/// What "each [description of target slot]" selects as the spell resolves.
fn each_of(spec: &TargetSpec) -> Sel {
    match &spec.what {
        TargetKind::Object(f) => Sel::All(f.clone()),
        TargetKind::Player(pf) => Sel::Players(PlayerRef::Each(pf.clone())),
        // CR 115.4: creatures, players, planeswalkers, and battles.
        TargetKind::AnyTarget => Sel::Union(vec![
            Sel::All(Filter::Or(vec![
                Filter::Type(CardType::Creature),
                Filter::Type(CardType::Planeswalker),
                Filter::Type(CardType::Battle),
            ])),
            Sel::Players(PlayerRef::EachPlayer),
        ]),
        TargetKind::ObjectOrPlayer(f, pf) => Sel::Union(vec![
            Sel::All(f.clone()),
            Sel::Players(PlayerRef::Each(pf.clone())),
        ]),
        TargetKind::Spell(f) => Sel::All(Filter::and(vec![f.clone(), Filter::Spell])),
        TargetKind::Ability(f) | TargetKind::SpellOrAbility(f) => {
            Sel::All(Filter::and(vec![f.clone(), Filter::InZone(ZoneKind::Stack)]))
        }
    }
}

fn overloaded_body(b: &Body) -> Option<Body> {
    if b.targets.is_empty() || b.modal.is_some() {
        return None;
    }
    let n = b.targets.len();
    let effect = rewrite_targets(serde_json::to_value(&b.effect).ok()?, n)?;
    let effect: Effect = serde_json::from_value(effect).ok()?;
    let mut seq: Vec<Effect> = b
        .targets
        .iter()
        .enumerate()
        .map(|(i, spec)| Effect::Store {
            var: slot_var(i),
            sel: each_of(spec),
        })
        .collect();
    seq.push(effect);
    Some(Body {
        targets: vec![],
        effect: Effect::Seq(seq),
        modal: None,
    })
}

/// Replaces references to target slot `i` (`{"Target": i}`, a [`Sel`] or [`PlayerRef`])
/// with the slot's variable, and "all targets" with all of them. A reference the variable
/// can't stand for (e.g. a [`PlayerRel`]) fails to deserialize afterwards, leaving the
/// spell unchanged.
fn rewrite_targets(v: J, n: usize) -> Option<J> {
    Some(match v {
        J::Object(m) => {
            if m.len() == 1 {
                if let Some(J::Number(k)) = m.get("Target") {
                    let k = k.as_u64()? as usize;
                    if k >= n {
                        return None;
                    }
                    let mut out = serde_json::Map::new();
                    out.insert("Var".into(), J::from(slot_var(k)));
                    return Some(J::Object(out));
                }
            }
            let mut out = serde_json::Map::new();
            for (k, v) in m {
                out.insert(k, rewrite_targets(v, n)?);
            }
            J::Object(out)
        }
        J::Array(a) => J::Array(
            a.into_iter()
                .map(|x| rewrite_targets(x, n))
                .collect::<Option<Vec<_>>>()?,
        ),
        J::String(s) if s == "AllTargets" => {
            let vars: Vec<J> = (0..n)
                .map(|i| {
                    let mut m = serde_json::Map::new();
                    m.insert("Var".into(), J::from(slot_var(i)));
                    J::Object(m)
                })
                .collect();
            let mut m = serde_json::Map::new();
            m.insert("Union".into(), J::Array(vars));
            J::Object(m)
        }
        other => other,
    })
}

inventory::submit! { KeywordRegistration(&Overload) }
