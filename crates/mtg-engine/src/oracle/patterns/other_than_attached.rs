//! "Other creatures" on a noncreature Aura or Equipment: "Enchanted creature gets +1/+1
//! for each other creature you control." (Bravado), "Other creatures you control get
//! -1/-1." (Vampirism), "Whenever equipped creature attacks alone, it gets +1/+1 until
//! end of turn for each other creature you control." (Sigil of Valor). The Aura or
//! Equipment isn't a creature, so "other" can't mean other than it: it means other than
//! the creature it's attached to. The phrase parsers read "other" as "not the source"
//! ([`Filter::Other`]); this pass rewrites it, in a filter that requires a creature, to
//! "neither the source nor the object it's attached to"
//! ([`crate::kw::other_than_attached::OTHER_THAN_ATTACHED`]). Abilities the Aura or
//! Equipment grants (their "other" is about the creature that has them) and targets
//! (support's "other target creatures" is other than the source) are left alone.

use crate::ability::*;
use crate::oracle::CompileContext;
use crate::types::CardType;

/// Rewrites the card's abilities (see the module docs).
pub(crate) fn apply(abilities: &mut [Ability], ctx: &CompileContext) {
    let tl = ctx.type_line;
    let attaches = tl
        .subtypes
        .iter()
        .any(|s| s.as_str() == "Aura" || s.as_str() == "Equipment");
    if !attaches || tl.card_types.contains(CardType::Creature) {
        return;
    }
    let (Ok(other), Ok(creature), Ok(not_attached)) = (
        serde_json::to_value(Filter::Other),
        serde_json::to_value(Filter::Type(CardType::Creature)),
        serde_json::to_value(Filter::Custom(
            crate::kw::other_than_attached::OTHER_THAN_ATTACHED.into(),
        )),
    ) else {
        return;
    };
    fn walk(
        v: &mut serde_json::Value,
        other: &serde_json::Value,
        creature: &serde_json::Value,
        to: &serde_json::Value,
    ) -> bool {
        let mut changed = false;
        if let Some(serde_json::Value::Array(parts)) = v.get_mut("And") {
            if parts.contains(creature) {
                for p in parts.iter_mut() {
                    if p == other {
                        *p = to.clone();
                        changed = true;
                    }
                }
            }
        }
        match v {
            serde_json::Value::Array(a) => {
                for x in a {
                    changed |= walk(x, other, creature, to);
                }
            }
            serde_json::Value::Object(m) => {
                // A granted ability (an `AbilityDef`) keeps its own "other".
                if m.contains_key("kind") && m.contains_key("text") {
                    return changed;
                }
                for (k, x) in m.iter_mut() {
                    if k == "targets" {
                        continue;
                    }
                    changed |= walk(x, other, creature, to);
                }
            }
            _ => {}
        }
        changed
    }
    for a in abilities.iter_mut() {
        if !matches!(a.kind, AbilityKind::Static(_) | AbilityKind::Triggered(_)) {
            continue;
        }
        let Ok(mut json) = serde_json::to_value(&a.kind) else {
            continue;
        };
        if !walk(&mut json, &other, &creature, &not_attached) {
            continue;
        }
        if let Ok(kind) = serde_json::from_value::<AbilityKind>(json) {
            *a = std::sync::Arc::new(AbilityDef {
                uid: a.uid,
                kind,
                text: a.text.clone(),
                link: a.link,
            });
        }
    }
}
