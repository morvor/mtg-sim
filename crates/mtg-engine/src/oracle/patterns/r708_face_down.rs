//! Oracle patterns for face-down permanents (CR 708):
//!
//! - "Turn [permanents] face down." (CR 708.2a; nothing happens to one that's already
//!   face down, CR 708.2b)
//! - "As ~ is turned face up, [effect on it]." (CR 708.11): applied while it's being
//!   turned face up.

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::facedown::{REVEALED_CREATURE_CARD, REVEAL_IT};
use crate::oracle::effects::{object_ref, parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

fn p_turn_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("turn ")?;
    let r = r.strip_suffix(" face down")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::TurnFaceDown { what })
}

inventory::submit! { EffectPattern { name: "r708 turn face down", priority: 0, parse: p_turn_face_down } }

/// "Reveal target face-down permanent." (CR 708.12)
fn p_reveal_face_down(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "reveal target face-down permanent" {
        return None;
    }
    let slot = b.add_target(
        TargetSpec::object(
            Filter::And(vec![Filter::Permanent, Filter::FaceDown]),
            "target face-down permanent",
        ),
        "target face-down permanent",
    );
    Some(Effect::Seq(vec![
        Effect::Store {
            var: vars::IT,
            sel: Sel::Target(slot),
        },
        Effect::Custom(REVEAL_IT.into()),
    ]))
}

inventory::submit! { EffectPattern { name: "r708 reveal a face-down permanent", priority: 0, parse: p_reveal_face_down } }

fn reveals(e: &Effect) -> bool {
    match e {
        Effect::Custom(n) => n == REVEAL_IT,
        Effect::Seq(v) => v.iter().any(reveals),
        _ => false,
    }
}

/// "If it's a creature card, you may turn it face up." after revealing a face-down
/// permanent: the revealed card's own characteristics are used, ignoring continuous
/// effects (CR 708.12).
fn f_revealed_creature(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if l != "if it's a creature card, you may turn it face up" || !reveals(prev) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond: Condition::Custom(REVEALED_CREATURE_CARD.into()),
            then: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::TurnFaceUp {
                    what: Sel::Var(vars::IT),
                }),
            }),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "r708 revealed creature card", priority: 0, apply: f_revealed_creature } }

/// "As ~ is turned face up, put five +1/+1 counters on it."
fn s_as_turned_face_up(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as ~ is turned face up, ")?;
    let mut b = Builder::new(ctx);
    b.it = Sel::This;
    let e = parse_clause(r, &mut b)?;
    if !b.targets.is_empty() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::TurnedFaceUp,
                action: ReplacementAction::AsEnters(Box::new(e)),
                self_replacement: true,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "r708 as turned face up", priority: 0, parse: s_as_turned_face_up } }
