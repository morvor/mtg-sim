//! Damage results VI (cards5-mana-replacements): the turn's damage history of a creature
//! the ability names (CR 120, 614.1a, 701.19c).
//!
//! - "If the creature deals damage to a creature this turn, the creature dealt damage
//!   can't be regenerated this turn." (Runesword, about its target): a restriction until
//!   end of turn on each creature that creature has dealt damage to this turn, in effect
//!   from the moment the damage is dealt (not a trigger: lethal damage destroys the
//!   creature before a trigger could resolve, CR 704.5g).
//! - "If a creature dealt damage by the targeted creature would die this turn, exile that
//!   creature instead." (Runesword): a replacement effect until end of turn on the
//!   creatures that creature has dealt damage to this turn (Torch the Tower's form with
//!   the source as "~").
//!
//! The creature is locked in as the effect is created (`prevention::lock_damage_history`);
//! damage it dealt earlier in the turn counts, as "this turn" says (interpretation
//! `runesword-damage-this-turn`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "the creature", "the targeted creature", "that creature": the ability's only target, a
/// creature (Runesword's "target attacking creature").
fn the_creature(s: &str, b: &Builder) -> Option<Sel> {
    if !matches!(s, "the creature" | "the targeted creature" | "that creature") {
        return None;
    }
    let creatures: Vec<usize> = b
        .targets
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            matches!(&t.what, TargetKind::Object(f) if {
                let j = serde_json::to_string(f).unwrap_or_default();
                j.contains("\"Creature\"")
            })
        })
        .map(|(i, _)| i)
        .collect();
    match (creatures.as_slice(), b.targets.len()) {
        ([i], 1) => Some(Sel::Target(*i as u8)),
        _ => None,
    }
}

/// Creatures the selected creature has dealt damage to this turn.
fn damaged_by(sel: Sel) -> Filter {
    Filter::and(vec![
        Filter::creature(),
        Filter::DealtDamageThisTurnBy(Box::new(sel)),
    ])
}

/// "if the creature deals damage to a creature this turn, the creature dealt damage can't
/// be regenerated this turn".
fn dealt_damage_cant_regenerate(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (who, rest) = r.split_once(" deals damage to a creature this turn, ")?;
    if rest != "the creature dealt damage can't be regenerated this turn" {
        return None;
    }
    let sel = the_creature(who, b)?;
    Some(Effect::AddRestriction {
        restriction: Restriction::CantBeRegenerated(damaged_by(sel)),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "prevention 6: if [creature] deals damage to a creature this turn, the creature dealt damage can't be regenerated", priority: 100, parse: dealt_damage_cant_regenerate } }

/// "if a creature dealt damage by the targeted creature would die this turn, exile that
/// creature instead".
fn dealt_damage_exile_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if a creature dealt damage by ")?;
    let (who, rest) = r.split_once(" would die this turn, ")?;
    if !matches!(rest, "exile that creature instead" | "exile it instead") {
        return None;
    }
    let sel = the_creature(who, b)?;
    // "Dies" means put into a graveyard from the battlefield (CR 700.4).
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: damaged_by(sel),
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "prevention 6: if a creature dealt damage by [creature] would die this turn, exile it instead", priority: 100, parse: dealt_damage_exile_instead } }

/// The prevention effect the instructions so far create, last.
fn last_prevention(e: &mut Effect) -> Option<&mut ReplacementDef> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_prevention),
        Effect::AddReplacement { def, .. }
            if matches!(def.action, ReplacementAction::Prevent)
                && matches!(def.event, ReplacementEvent::Damage { .. }) =>
        {
            Some(def)
        }
        _ => None,
    }
}

/// "a creature", "a black or red source", "an artifact source": what the source of the
/// damage prevented is.
fn source_quality(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let s = s.strip_suffix(" source").unwrap_or(s);
    let mut alts = Vec::new();
    for w in s.split(" or ") {
        let w = w.trim();
        let f = match crate::types::Color::from_word(w) {
            Some(c) => Filter::Color(c),
            None => {
                let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(w)?;
                if plural || !end(tail).is_empty() || matches!(f, Filter::Any) {
                    return None;
                }
                f
            }
        };
        alts.push(f);
    }
    Some(if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    })
}

/// "Whenever damage from a creature is prevented this way, each commander creature you
/// control deals damage equal to its power to that creature." after "Prevent all damage
/// that would be dealt to you this turn by sources your opponents control." (Judgment of
/// Alexander): a triggered ability the spell creates (CR 603.7, 615.5) that triggers each
/// time the shield prevents damage from such a source, once for each source of damage
/// prevented at the same time, with "that creature" (or "that source") being the source.
fn whenever_prevented_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("whenever damage ") else {
        return false;
    };
    let (quality, r) = match r.strip_prefix("from ") {
        Some(x) => {
            let Some((q, r)) = x.split_once(" is prevented this way") else {
                return false;
            };
            let Some(f) = source_quality(q) else {
                return false;
            };
            (Some(f), r)
        }
        None => {
            let Some(r) = r.strip_prefix("is prevented this way") else {
                return false;
            };
            (None, r)
        }
    };
    let r = r.strip_prefix(" this turn").unwrap_or(r);
    let Some(body) = r.strip_prefix(", ") else {
        return false;
    };
    // The source is "that creature" (or "that source") in the instructions.
    let named = b.named.len();
    for p in ["that creature", "that source", "the source"] {
        b.named.push((p.into(), Sel::TriggerOtherObject));
    }
    let effect =
        super::replacement_grammar::instructions(body, b.ctx, Sel::TriggerOtherObject);
    b.named.truncate(named);
    let Some(effect) = effect else {
        return false;
    };
    let Some(def) = last_prevention(prev) else {
        return false;
    };
    let reflexive = Effect::Reflexive {
        body: Box::new(Body {
            targets: vec![],
            effect,
            modal: None,
        }),
    };
    let then = match quality {
        Some(f) => Effect::If {
            cond: Condition::SelMatches(Sel::TriggerOtherObject, f),
            then: Box::new(reflexive),
            otherwise: Box::new(Effect::Noop),
        },
        None => reflexive,
    };
    def.action = ReplacementAction::PreventAndThen(None, Box::new(then));
    true
}

inventory::submit! { super::FollowupPattern { name: "prevention 6: whenever damage from [a source] is prevented this way, [effect]", priority: 61, apply: whenever_prevented_this_way } }
