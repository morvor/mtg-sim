//! Having or gaining the activated abilities of other objects (CR 113.6, 602.5c):
//!
//! - "~ has all activated abilities of all creatures your opponents control." (Drana and
//!   Linvala), "... of all lands on the battlefield" (Manascape Refractor), "... of all
//!   legendary creatures you control" (Robaran Mercenaries), "... of the exiled card"
//!   (Territory Forge), "~ has each activated ability of the exiled cards used to craft
//!   it" (Locus of Enlightenment): a static ability; the abilities change as those objects
//!   do.
//! - "~ gains all activated abilities of target creature until end of turn" (Quicksilver
//!   Elemental), "each Horror you control gains all activated abilities of target artifact
//!   an opponent controls until end of turn" (Grell Philosopher): which abilities is
//!   determined as the effect begins (CR 611.2c).
//!
//! Only activated abilities are gained, each as acquired from its object: an ability that
//! names its object names the one that now has it. (Cards in graveyards are left to
//! `hand_graveyard_grammar.rs`.)

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "all creatures your opponents control", "the exiled card", "the exiled cards used to
/// craft it".
fn sources(s: &str) -> Option<Filter> {
    match s {
        "the exiled card" => {
            return Some(Filter::And(vec![
                Filter::In(Box::new(Sel::Linked)),
                Filter::InZone(ZoneKind::Exile),
            ]))
        }
        "the exiled cards used to craft it" | "the exiled cards used to craft ~" => {
            return Some(crate::kw::craft::used_to_craft_filter())
        }
        _ => {}
    }
    let p = s.strip_prefix("all ")?;
    let (f, plural, rest) = super::statics::object_phrase(p)?;
    if !plural || !end(rest).is_empty() {
        return None;
    }
    Some(f)
}

fn has_activated_abilities_of(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("~ has ")?;
    let r = r
        .strip_prefix("all activated abilities of ")
        .or_else(|| r.strip_prefix("each activated ability of "))?;
    let from = sources(r)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::AddActivatedAbilitiesOf(from)],
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "grants: ~ has all activated abilities of ...", priority: 970, parse: has_activated_abilities_of } }

fn gains_activated_abilities_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_suffix(" until end of turn")?;
    let (subject, of) = r.split_once(" all activated abilities of ")?;
    let what = match subject {
        "~ gains" => Sel::This,
        s => {
            let p = s
                .strip_suffix(" gains")
                .or_else(|| s.strip_suffix(" gain"))?;
            let p = p.strip_prefix("each ").or_else(|| p.strip_prefix("all ")).unwrap_or(p);
            let (f, _, rest) = super::statics::object_phrase(p)?;
            if !end(rest).is_empty() {
                return None;
            }
            Sel::All(f)
        }
    };
    let (spec, rest) = crate::oracle::phrases::parse_target(of)?;
    if !end(rest).is_empty() || !matches!(spec.min, Value::Const(1)) {
        return None;
    }
    let slot = b.add_target(spec, of);
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddActivatedAbilitiesOf(Filter::In(Box::new(
            Sel::Target(slot),
        )))],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "grants: gains all activated abilities of target ...", priority: 100, parse: gains_activated_abilities_of } }

/// "You may spend blue mana as though it were mana of any color to pay the activation
/// costs of ~'s abilities." (Quicksilver Elemental), "You may spend mana as though it were
/// mana of any color to pay the activation costs of ~'s abilities." (Manascape
/// Refractor): any of its activated abilities, including those it gained (CR 609.4b).
fn spend_for_own_abilities(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may spend ")?;
    let (kind, r) = r.split_once(" as though it were mana of any color to pay the activation costs of ~'s abilities")?;
    if !r.is_empty() {
        return None;
    }
    let types = match kind {
        "mana" => vec![],
        "white mana" => vec![crate::mana::ManaType::W],
        "blue mana" => vec![crate::mana::ManaType::U],
        "black mana" => vec![crate::mana::ManaType::B],
        "red mana" => vec![crate::mana::ManaType::R],
        "green mana" => vec![crate::mana::ManaType::G],
        "colorless mana" => vec![crate::mana::ManaType::C],
        _ => return None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::SpendAsAnyColor {
            applies_to: CostTarget::Abilities(Filter::Source),
            types,
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "grants: spend mana as though any color for ~'s abilities", priority: 100, parse: spend_for_own_abilities } }
