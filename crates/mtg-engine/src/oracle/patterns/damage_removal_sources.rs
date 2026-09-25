//! Prevention and redirection effects tied to "a source of your choice" (CR 609.7):
//!
//! - "The next time a [red] source of your choice would deal damage to you this turn,
//!   prevent that damage." (Circles of Protection)
//! - "The next time a source of your choice would deal damage to target creature this
//!   turn, prevent that damage."
//! - "The next time a source of your choice would deal damage to you this turn, that
//!   damage is dealt to ~ instead." (CR 614.9 redirection)
//! - "Prevent all damage a source of your choice would deal [to you] this turn."
//! - "Prevent all damage that would be dealt to you this turn by a source of your
//!   choice." / "If damage would be dealt to you this turn by a source of your choice,
//!   prevent that damage."
//! - Follow-up: "You gain life equal to the damage prevented this way." (CR 615.5)
//!
//! The source is chosen as the effect is created, i.e. on resolution (CR 609.7a); the
//! effect then applies only to that object and rechecks the stated characteristics when
//! the damage would be dealt (CR 609.7b, 615.9 via the source filter).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Variable holding the chosen source.
const CHOSEN_SOURCE: Var = vars::USER + 42;

/// "a [red|black or red|artifact] source of your choice [of the chosen color]". Returns
/// the filter the chosen source must match and the rest of the text.
fn chosen_source(s: &str) -> Option<(Filter, &str)> {
    let s = s.trim_start();
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let idx = s.find("source of your choice")?;
    let adj = s[..idx].trim();
    let mut rest = &s[idx + "source of your choice".len()..];
    let mut parts = Vec::new();
    if !adj.is_empty() {
        let mut alts = Vec::new();
        for w in adj.split(" or ") {
            let w = w.trim();
            if w.contains(' ') {
                return None;
            }
            // Colors ("red"), card types ("artifact", "land").
            let f = match adjective(w) {
                Some(f @ (Filter::Color(_) | Filter::Colorless | Filter::Multicolored)) => f,
                _ => match head_noun(w)? {
                    f @ Filter::Type(_) => f,
                    _ => return None,
                },
            };
            alts.push(f);
        }
        parts.push(if alts.len() == 1 {
            alts.pop()?
        } else {
            Filter::Or(alts)
        });
    }
    if let Some(r) = rest.strip_prefix(" of the chosen color") {
        parts.push(Filter::ChosenColor);
        rest = r;
    }
    Some((Filter::and(parts), rest))
}

/// Strips `p` from the start of `s` if a word boundary follows.
fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    match r.chars().next() {
        None | Some(' ' | ',' | '.') => Some(r),
        _ => None,
    }
}

/// Who or what the damage would be dealt to: (player filter, object filter).
type Recipient = (Option<PlayerFilter>, Option<Filter>);

/// "you", "target creature [you control]", "~". Returns the recipient and the rest.
fn recipient<'a>(s: &'a str, b: &mut Builder) -> Option<(Recipient, &'a str)> {
    let s = s.trim_start();
    if let Some(r) = word(s, "you") {
        return Some(((Some(PlayerFilter::You), None), r));
    }
    for p in ["~", "this creature"] {
        if let Some(r) = word(s, p) {
            return Some(((None, Some(Filter::In(Box::new(Sel::This)))), r));
        }
    }
    if s.starts_with("target ") {
        let (spec, r) = parse_target(s)?;
        // Only single object targets: the filter is locked onto the chosen object.
        if !matches!(spec.what, TargetKind::Object(_))
            || spec.min != 1
            || !matches!(spec.max, Value::Const(1))
        {
            return None;
        }
        let text = s[..s.len() - r.len()].trim().to_string();
        let slot = b.add_target(spec, &text);
        return Some(((None, Some(Filter::In(Box::new(Sel::Target(slot))))), r));
    }
    None
}

/// The effect: choose the source, then create the prevention/redirection effect.
fn chosen_source_effect(
    source: Filter,
    to: Recipient,
    action: ReplacementAction,
    uses: Option<u32>,
) -> Effect {
    Effect::seq(vec![
        Effect::ChooseSource {
            who: PlayerRef::You,
            filter: source.clone(),
            var: CHOSEN_SOURCE,
        },
        Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Damage {
                    source: Filter::and(vec![
                        Filter::In(Box::new(Sel::Var(CHOSEN_SOURCE))),
                        source,
                    ]),
                    to_players: to.0,
                    to_objects: to.1,
                    combat_only: false,
                },
                action,
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses,
        },
    ])
}

/// "prevent that damage" / "that damage is dealt to ~ instead" / "that damage is dealt
/// to target creature you control instead" (the new recipient is locked in as the
/// effect is created).
fn action(s: &str, b: &mut Builder) -> Option<ReplacementAction> {
    let s = s.trim();
    match s {
        "prevent that damage" => return Some(ReplacementAction::Prevent),
        "that damage is dealt to ~ instead" | "that damage is dealt to this creature instead" => {
            return Some(ReplacementAction::Redirect(Sel::This))
        }
        _ => {}
    }
    let t = s
        .strip_prefix("that damage is dealt to ")?
        .strip_suffix(" instead")?;
    let (spec, rest) = parse_target(t)?;
    if !rest.trim().is_empty()
        || spec.min != 1
        || !matches!(spec.max, Value::Const(1))
        || !matches!(spec.what, TargetKind::Object(_))
    {
        return None;
    }
    let slot = b.add_target(spec, t);
    Some(ReplacementAction::Redirect(Sel::Target(slot)))
}

/// "the next time [source] would deal damage to [recipient] this turn, [action]".
fn p_next_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("the next time ")?;
    let (source, r) = chosen_source(r)?;
    let r = r.trim_start().strip_prefix("would deal damage to ")?;
    let saved = b.targets.len();
    let parsed = (|| {
        let (to, r) = recipient(r, b)?;
        let r = r.trim_start().strip_prefix("this turn, ")?;
        let act = action(r, b)?;
        // "The next time": one application (CR 615.7 for the prevention shield; a
        // single damage event from the chosen source to that recipient).
        Some(chosen_source_effect(source, to, act, Some(1)))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "damage_removal: next time a source of your choice", priority: 45, parse: p_next_time } }

/// "prevent all damage a source of your choice would deal [to recipient] this turn",
/// "prevent all damage that would be dealt to [recipient] this turn by a source of your
/// choice", "if damage would be dealt to you this turn by a source of your choice,
/// prevent that damage".
fn p_prevent_all_from_source(l: &str, b: &mut Builder) -> Option<Effect> {
    let saved = b.targets.len();
    let parsed = (|| {
        if let Some(r) = l.strip_prefix("prevent all damage ") {
            if let Some(r) = r.strip_prefix("that would be dealt to ") {
                // "... to you this turn by a source of your choice"
                let (to, r) = recipient(r, b)?;
                let r = r.trim_start().strip_prefix("this turn by ")?;
                let (source, rest) = chosen_source(r)?;
                if !rest.trim().is_empty() {
                    return None;
                }
                return Some(chosen_source_effect(
                    source,
                    to,
                    ReplacementAction::Prevent,
                    None,
                ));
            }
            let (source, r) = chosen_source(r)?;
            let r = r.trim_start().strip_prefix("would deal")?;
            let (to, r) = match r.trim_start().strip_prefix("to ") {
                Some(x) => recipient(x, b)?,
                // Damage to anything.
                None => ((Some(PlayerFilter::Any), Some(Filter::Any)), r),
            };
            if r.trim() != "this turn" {
                return None;
            }
            return Some(chosen_source_effect(
                source,
                to,
                ReplacementAction::Prevent,
                None,
            ));
        }
        let r = l.strip_prefix("if damage would be dealt to ")?;
        let (to, r) = recipient(r, b)?;
        let r = r.trim_start().strip_prefix("this turn by ")?;
        let (source, r) = chosen_source(r)?;
        if r.trim() != ", prevent that damage" {
            return None;
        }
        Some(chosen_source_effect(
            source,
            to,
            ReplacementAction::Prevent,
            None,
        ))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "damage_removal: prevent all damage from a source of your choice", priority: 45, parse: p_prevent_all_from_source } }

/// The last prevention replacement created by an effect.
fn last_prevention(e: &mut Effect) -> Option<&mut ReplacementAction> {
    match e {
        Effect::Seq(v) => v.iter_mut().rev().find_map(last_prevention),
        Effect::AddReplacement { def, .. }
            if matches!(def.event, ReplacementEvent::Damage { .. })
                && matches!(def.action, ReplacementAction::Prevent) =>
        {
            Some(&mut def.action)
        }
        _ => None,
    }
}

/// "You gain life equal to the damage prevented this way." (CR 615.5: the rest of the
/// prevention effect happens immediately after the damage is prevented).
fn f_gain_prevented(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "you gain life equal to the damage prevented this way" {
        return false;
    }
    let Some(a) = last_prevention(prev) else {
        return false;
    };
    *a = ReplacementAction::PreventAndThen(
        None,
        Box::new(Effect::GainLife {
            who: PlayerRef::You,
            n: Value::EventAmount,
        }),
    );
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: gain life equal to damage prevented", priority: 50, apply: f_gain_prevented } }
