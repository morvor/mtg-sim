//! "As this creature enters, choose another creature you control." and the abilities
//! linked to it that refer to "the chosen creature" (CR 607.2d, 614.12): the choice is
//! made as the permanent enters (not a triggered ability; nobody can respond) and noted
//! for the permanent's abilities ([`Effect::NoteLinked`], `linked_notes.rs`). "The chosen
//! creature" is that object for as long as it stays on the battlefield (CR 400.7: a new
//! object if it leaves; no new creature is chosen).
//!
//! - "The chosen creature gets +3/+3 and has flying." (a static ability: it applies
//!   whoever controls the creature);
//! - "Sacrifice ~: The chosen creature gains indestructible until end of turn.";
//! - "When ~ leaves the battlefield, sacrifice the chosen creature."

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// The object chosen as the permanent entered: "another creature you control".
fn etb_object_choice(text: &str) -> Option<Filter> {
    let l = text.to_lowercase();
    let r = end(&l).strip_prefix("as ~ enters, choose ")?;
    let r = match r.strip_prefix("another ") {
        Some(_) => r,
        None => {
            let (n, r) = parse_number(r)?;
            if n.as_const() != Some(1) {
                return None;
            }
            r
        }
    };
    let (f, false, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    // A creature (not a card, a player or a word).
    if !matches!(&f, Filter::And(v) if v.iter().any(|x| matches!(x, Filter::Type(crate::types::CardType::Creature))))
        && !matches!(f, Filter::Type(crate::types::CardType::Creature))
    {
        return None;
    }
    Some(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]))
}

/// Whether the card has such a choice (in its raw text, normalized like blocks are).
fn card_chooses_object() -> bool {
    let raw = crate::oracle::raw_text();
    let name = crate::oracle::card_name();
    raw.lines().any(|line| {
        let mut t = line.to_lowercase();
        if !name.is_empty() {
            t = t.replace(&name.to_lowercase(), "~");
        }
        for p in ["this creature", "this artifact", "this enchantment", "this permanent"] {
            t = t.replace(p, "~");
        }
        etb_object_choice(&t).is_some()
    })
}

fn as_enters_choose_object(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let filter = etb_object_choice(block)?;
    let effect = StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(Effect::NoteLinked {
            what: Sel::Choose {
                chooser: PlayerRef::You,
                filter,
                count: Value::c(1),
                up_to: false,
                store: None,
            },
            replace: true,
        })),
        self_replacement: false,
        optional: false,
    });
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "choice grammar: as ~ enters, choose another creature you control", priority: 48, parse: as_enters_choose_object } }

/// "The chosen creature gets +3/+3 and has flying.": a static ability for the chosen
/// object (read as "enchanted creature ..." reads, with the chosen object in its place).
fn chosen_creature_static(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase();
    let rest = lower.strip_prefix("the chosen creature ")?;
    if !ctx.is_permanent() || !card_chooses_object() {
        return None;
    }
    let text = format!("Enchanted creature {rest}");
    let v = crate::oracle::statics::parse_static(&text, ctx)?;
    let mut out = Vec::new();
    for a in v {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        let StaticEffect::Continuous { affected, mods } = &s.effect else {
            return None;
        };
        if !matches!(affected, Filter::AttachedToSource) || s.condition.is_some() {
            return None;
        }
        let mut s2 = s.clone();
        s2.effect = StaticEffect::Continuous {
            affected: Filter::In(Box::new(Sel::LinkedNoted)),
            mods: mods.clone(),
        };
        out.push(AbilityDef::new(AbilityKind::Static(s2), block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "choice grammar: the chosen creature [static]", priority: 48, parse: chosen_creature_static } }

/// "The chosen creature gains indestructible until end of turn.", "sacrifice the chosen
/// creature": the creature chosen as the permanent entered.
fn the_chosen_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    const P: &str = "the chosen creature";
    if !l.contains(P) || b.named.iter().any(|(n, _)| n == P) || !card_chooses_object() {
        return None;
    }
    // "sacrifice the chosen creature" (only if you still control it, CR 701.21a).
    if end(l) == "sacrifice the chosen creature" {
        return Some(Effect::SacrificeObjects {
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(Sel::LinkedNoted)),
                Filter::ControlledBy(PlayerRel::You),
            ])),
        });
    }
    b.named.push((P.into(), Sel::LinkedNoted));
    let e = parse_sentence(l, b);
    b.named.retain(|(n, _)| n != P);
    e
}

inventory::submit! { EffectPattern { name: "choice grammar: the chosen creature (chosen as ~ entered)", priority: 30, parse: the_chosen_creature } }
