//! What happens when a regeneration shield is used (CR 701.19a): "Regenerate ~. When it
//! regenerates this way, put a -1/-1 counter on it." (Matopi Golem: a triggered ability
//! that triggers when the shield replaces a destruction), "Regenerate target creature.
//! You gain control of that creature if it regenerates this way." (Debt of Loyalty: part
//! of the regeneration itself — the ruling: "instead it regenerates and you gain control
//! of it"). The shield's rider is `Effect::RegenerateWith::then`; the regenerated permanent is
//! "it" in it.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};

/// The last instruction of the effect (in execution order), if it's a regeneration
/// without a rider.
fn last_regenerate(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => v.last_mut().and_then(last_regenerate),
        Effect::Regenerate { .. } => Some(e),
        _ => None,
    }
}

/// The rider's instructions, with "it" and "that creature" the regenerated permanent.
fn rider_body(r: &str, b: &Builder) -> Option<Body> {
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let r = r.replace("that creature", "it");
    let effect = parse_effect_text(&r, &mut sub)?;
    Some(Body {
        targets: sub.targets,
        effect,
        modal: None,
    })
}

fn regenerates_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (r, triggered) = if let Some(r) = l.strip_prefix("when it regenerates this way, ") {
        (r.to_string(), true)
    } else if let Some(r) = l.strip_suffix(" if it regenerates this way") {
        (r.to_string(), false)
    } else {
        return false;
    };
    let Some(body) = rider_body(&r, b) else {
        return false;
    };
    // The regeneration itself has no targets of its own to choose then.
    if !triggered && !body.targets.is_empty() {
        return false;
    }
    let Some(slot) = last_regenerate(prev) else {
        return false;
    };
    let Effect::Regenerate { what } = slot else {
        return false;
    };
    let then = Box::new(if triggered {
        Effect::Reflexive {
            body: Box::new(body),
        }
    } else {
        body.effect
    });
    *slot = Effect::RegenerateWith {
        what: what.clone(),
        then,
    };
    true
}

inventory::submit! { FollowupPattern { name: "replacement riders: when it regenerates this way", priority: 60, apply: regenerates_this_way } }

// ---------------------------------------------------------------------------
// Damage prevented this way (CR 615.5)
// ---------------------------------------------------------------------------

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

/// "Whenever damage from a black or red source is prevented this way this turn, you gain
/// that much life." (Samite Ministration), "When damage is prevented this way, ~ deals
/// that much damage to that source's controller and you draw that many cards." (New Way
/// Forward): a triggered ability each time the prevention effect prevents damage, "that
/// much" being the damage prevented and "that source" its source.
fn prevented_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (quality, r) = if let Some(r) = l.strip_prefix("when damage is prevented this way, ") {
        (None, r)
    } else if let Some(r) = l.strip_prefix("whenever damage from ") {
        let Some((src, r)) = r.split_once(" source is prevented this way this turn, ") else {
            return false;
        };
        let Some(f) = source_quality(src) else {
            return false;
        };
        (Some(f), r)
    } else {
        return false;
    };
    let Some(effect) =
        super::replacement_grammar::instructions(r, b.ctx, Sel::TriggerObject)
    else {
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

inventory::submit! { FollowupPattern { name: "replacement riders: when damage is prevented this way", priority: 60, apply: prevented_this_way } }

/// "black or red", "black", "artifact": the quality of a source.
fn source_quality(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")).unwrap_or(s);
    let mut alts = Vec::new();
    for w in s.split(" or ") {
        alts.push(match crate::types::Color::from_word(w.trim()) {
            Some(c) => Filter::Color(c),
            None => return None,
        });
    }
    Some(if alts.len() == 1 {
        alts.pop()?
    } else {
        Filter::Or(alts)
    })
}

/// "an opponent of your choice gains control of it" (Khârn the Betrayer): the opponent is
/// chosen as it happens.
fn opponent_of_your_choice_gains_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = crate::oracle::phrases::end(l)
        .strip_prefix("an opponent of your choice gains control of ")?;
    let (what, rest) = crate::oracle::effects::object_ref(r, b)?;
    if !crate::oracle::phrases::end(&rest).is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        Effect::GainControl {
            what,
            who: PlayerRef::ChosenOpponent,
            duration: Duration::Permanent,
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "replacement riders: an opponent of your choice gains control of it", priority: 100, parse: opponent_of_your_choice_gains_control } }
