//! Oracle patterns for permanents becoming copies of other objects (CR 707.2, 613.2a):
//! "[objects] become(s) a copy of [object][ until end of turn]" (True Polymorph:
//! "Target artifact or creature becomes a copy of another target artifact or creature.").
//! The affected permanents get the other object's copiable values in layer 1 for the
//! duration; everything else about them (designations, counters, status) is unchanged.
//! Exceptions ("..., except those creatures aren't legendary", CR 707.9) are parsed like
//! a token copy's (see `tokens_copies_copy::copy_exceptions`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, object_ref, Builder};
use crate::oracle::phrases::end;

/// The permanents that become copies and also gain an ability.
const BECAME_COPIES: Var = vars::USER + 5105;

/// "Each land you control of that type becomes a copy of target creature you control
/// until end of turn and gains haste until end of turn." (March from Velis Vel): the
/// permanents are those that became copies, though they may no longer be lands.
fn becomes_copy_and_gains(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, gains) = l.split_once(" until end of turn and gains ")?;
    if !gains.ends_with(" until end of turn") || head.contains(", except ") {
        return None;
    }
    let copy = becomes_copy(&format!("{head} until end of turn"), b)?;
    let Effect::BecomeCopy { what, of, duration } = copy else {
        return None;
    };
    let saved = b.it.clone();
    b.it = Sel::Var(BECAME_COPIES);
    let grant = crate::oracle::effects::parse_clause(&format!("it gains {gains}"), b);
    b.it = saved;
    let grant = grant?;
    Some(Effect::seq(vec![
        Effect::Store {
            var: BECAME_COPIES,
            sel: what,
        },
        Effect::BecomeCopy {
            what: Sel::Var(BECAME_COPIES),
            of,
            duration,
        },
        grant,
    ]))
}

inventory::submit! { EffectPattern { name: "r707 becomes a copy of ... and gains", priority: 81, parse: becomes_copy_and_gains } }

fn becomes_copy(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" a copy of ") && !l.contains(" copies of ") {
        return None;
    }
    // "..., except those creatures aren't legendary" (Echoing Equation).
    let (l, exceptions) = match l.split_once(", except ") {
        Some((head, except)) => {
            // Quoted abilities ("it has this ability and \"...\"", Aurora Shifter).
            let (masked, quotes) = super::statics::mask_quotes(except)?;
            (
                head,
                super::tokens_copies_copy::copy_exceptions(&masked, &quotes, b.ctx)?,
            )
        }
        None => (l, vec![]),
    };
    let (duration, l) = if let Some(r) = l.strip_prefix("until end of turn, ") {
        (Duration::EndOfTurn, r)
    } else if let Some(r) = l.strip_prefix("until your next turn, ") {
        // "until your next turn, ~ becomes a copy of ..." (Absorbing Man).
        (Duration::UntilYourNextTurn, r)
    } else {
        duration_suffix(l)
    };
    if !l.contains(" a copy of ") && !l.contains(" copies of ") {
        return None;
    }
    // "that creature" after the verb refers to what an earlier sentence mentioned
    // (Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature ..."), not to the subject of this one.
    let before = b.it.clone();
    let (what, rest) = object_ref(l, b)?;
    let subject = b.it.clone();
    let rest = rest.trim_start();
    // "Shards you control become copies of it" (Niko, Light of Hope).
    let r = [
        "becomes a copy of ",
        "become a copy of ",
        "each become a copy of ",
        "become copies of ",
        "becomes copies of ",
    ]
    .iter()
    .find_map(|p| rest.strip_prefix(p))?;
    // "becomes a copy of a second target artifact you control" (Shuri, Wakandan
    // Inventor).
    let second;
    let r = match r.strip_prefix("a second target ") {
        Some(x) => {
            second = format!("target {x}");
            second.as_str()
        }
        None => r,
    };
    // "~ becomes a copy of a creature card exiled with it": "it" is the subject, ~.
    b.it = if matches!(what, Sel::This) && rest.contains(" exiled with it") {
        Sel::This
    } else {
        before
    };
    let n = b.targets.len();
    let parsed = super::tokens_copies_copy::exiled_with_source(r, b)
        .or_else(|| object_ref(r, b).filter(|(_, t)| end(t).is_empty()))
        .or_else(|| {
            // "the exiled card", "the sacrificed creature", "another creature you control"
            // (one chosen as the effect happens): the objects a token copy can copy.
            b.targets.truncate(n);
            // "up to one target artifact, non-Aura enchantment, or land" (Absorbing Man).
            if let Some((spec, tail)) = super::basic_effects_targets::target_alternatives(r)
                .filter(|(_, t)| end(t).is_empty())
            {
                let text = spec.text.clone();
                let slot = b.add_target(spec, &text);
                return Some((Sel::Target(slot), tail.to_string()));
            }
            super::tokens_copies_copy::copied_object(r, b)
        });
    b.it = subject;
    let (of, tail) = parsed?;
    if !end(&tail).is_empty() {
        return None;
    }
    // "Each other creature you control becomes a copy of it": other than the object
    // copied (the chosen or targeted creature), not only other than the source; but
    // "creatures you control other than ~" names the source (Deceiver of Form).
    let names_source = l.split_once(" become").is_some_and(|(s, _)| s.contains("other than ~"));
    let what = match what {
        Sel::All(f) if !names_source => Sel::All(other_than(f, &of)),
        w => w,
    };
    Some(if exceptions.is_empty() {
        Effect::BecomeCopy { what, of, duration }
    } else {
        Effect::BecomeCopyExcept {
            what,
            of,
            duration,
            exceptions,
        }
    })
}

/// Replaces "other" in the description of the objects becoming copies with "other than
/// the copied object".
fn other_than(f: Filter, of: &Sel) -> Filter {
    match f {
        Filter::Other => Filter::not(Filter::In(Box::new(of.clone()))),
        Filter::And(v) => Filter::And(v.into_iter().map(|x| other_than(x, of)).collect()),
        x => x,
    }
}

inventory::submit! { EffectPattern { name: "r707 becomes a copy of", priority: 80, parse: becomes_copy } }

/// "As this artifact enters, you may have it become a copy of any creature on the
/// battlefield until end of turn, except it has haste." (Cursed Mirror): a copy effect
/// that begins as it enters (CR 614.1c, 707.2) and ends at end of turn.
fn as_enters_become_copy_until(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() {
        return None;
    }
    let lower = block.to_lowercase();
    let r = end(&lower).strip_prefix("as ~ enters, you may have it become a copy of ")?;
    let r = r
        .strip_prefix("any ")
        .map_or(r.to_string(), |x| format!("a {x}"));
    if !r.contains(" until end of turn") {
        return None;
    }
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let e = becomes_copy(&format!("~ becomes a copy of {r}"), &mut b)?;
    if !b.targets.is_empty()
        || !matches!(
            &e,
            Effect::BecomeCopy {
                what: Sel::This,
                duration: Duration::EndOfTurn,
                ..
            } | Effect::BecomeCopyExcept {
                what: Sel::This,
                duration: Duration::EndOfTurn,
                ..
            }
        )
    {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::AsEnters(Box::new(Effect::May {
                    who: PlayerRef::You,
                    effect: Box::new(e),
                })),
                self_replacement: true,
                optional: false,
            },
        ))),
        block,
    )])
}

inventory::submit! { super::AbilityPattern { name: "r707 as ~ enters, it becomes a copy until end of turn", priority: 60, parse: as_enters_become_copy_until } }
