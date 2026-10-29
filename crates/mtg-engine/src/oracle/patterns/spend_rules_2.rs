//! Spending rules (CR 609.4b, 106.6):
//!
//! * "You may spend white mana as though it were red mana." (Sunglasses of Urza), "You may
//!   spend white mana as though it were mana of any color. You may spend other mana only as
//!   though it were colorless mana." (Celestial Dawn): the player's spending rules
//!   (`kw/spend_rules.rs`);
//! * "{T}: Add one mana of any color. Spend this mana only to cast a legendary spell, and
//!   that spell can't be countered." (Delighted Halfling, Cavern of Souls): restricted mana
//!   with an additional effect as it's spent on a spell (CR 106.6): that spell can't be
//!   countered (not a triggered ability: it applies as soon as the mana is spent).

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::kw::spend_rules::{as_though, only_colorless};
use crate::mana::ManaType;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::Color;

/// "white mana" → W.
fn color_mana(s: &str) -> Option<ManaType> {
    let c = Color::from_word(s.strip_suffix(" mana")?)?;
    Some(ManaType::from_color(c))
}

const COLORED: [ManaType; 5] = [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G];

/// "you may spend white mana as though it were red mana", "... mana of any color".
fn as_though_sentence(s: &str) -> Option<(ManaType, Vec<ManaType>)> {
    let r = s.strip_prefix("you may spend ")?;
    let (from, to) = r.split_once(" as though it were ")?;
    let from = color_mana(from)?;
    let to = if to == "mana of any color" {
        COLORED.to_vec()
    } else {
        vec![color_mana(to)?]
    };
    Some((from, to))
}

fn custom(name: String, text: &str) -> Ability {
    AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name.into()))),
        text,
    )
}

fn s_spend_rules(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (first, second) = match l.split_once(". ") {
        Some((a, b)) => (a, Some(b)),
        None => (l, None),
    };
    let (from, to) = as_though_sentence(first)?;
    let mut out = vec![custom(as_though(from, &to), text)];
    match second {
        None => {}
        // "You may spend other mana only as though it were colorless mana."
        Some("you may spend other mana only as though it were colorless mana") => {
            let others: Vec<ManaType> = COLORED.iter().copied().filter(|t| *t != from).collect();
            out.push(custom(only_colorless(&others), text));
        }
        Some(_) => return None,
    }
    Some(out)
}

inventory::submit! { StaticPattern { name: "spend rules: you may spend [color] mana as though it were [color] mana", priority: 100, parse: s_spend_rules } }

/// "Spend this mana only to cast a legendary spell, and that spell can't be countered."
fn a_restricted_mana_uncounterable(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let cut = lower.rfind(", and that spell can't be countered.")?;
    if !lower[..cut].contains(". spend this mana only to cast ")
        || cut + ", and that spell can't be countered.".len() != lower.len()
    {
        return None;
    }
    let head = format!("{}.", &t[..cut]);
    let abilities = crate::oracle::parse_ability(&head, ctx)?;
    let mut out = Vec::new();
    for a in abilities {
        let AbilityKind::Activated(act) = &a.kind else {
            out.push(a);
            continue;
        };
        let Effect::AddMana {
            restriction: Some(_),
            ..
        } = &act.body.effect
        else {
            return None;
        };
        let mut act = act.clone();
        act.body.effect = Effect::AddManaWithSpentTrigger {
            add: Box::new(act.body.effect.clone()),
            spell_filter: Filter::Spell,
            body: Box::new(Body::effect(Effect::Modify {
                what: Sel::TriggerSpell,
                mods: vec![Modification::AddAbility(
                    super::restriction_grammar::cant_be_countered_ability(),
                )],
                duration: Duration::Permanent,
            })),
            abilities: false,
            additional: true,
        };
        act.is_mana_ability = act.body.targets.is_empty();
        out.push(AbilityDef::with_link(AbilityKind::Activated(act), t, a.link));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "spend rules: spend this mana only to cast X, and that spell can't be countered", priority: 60, parse: a_restricted_mana_uncounterable } }
