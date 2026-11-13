//! Reflexive triggered abilities with an intervening "if" (CR 603.4, 603.12): "When you
//! do, if you control a red permanent other than Ajani, he deals damage ..." (Ajani,
//! Nacatl Avenger), "When you do, if there are four or more creature cards in your
//! graveyard, ..." (Sigil of Myrkul). The reflexive ability triggers only if the condition
//! is true as it would trigger, and checks it again as it resolves (its body's own "if").
//!
//! The compiled body is "if [condition], [effect]" ([`Effect::If`] with nothing
//! otherwise), and the engine reads a reflexive ability whose whole effect has that shape
//! as one with an intervening "if" ([`fails_to_trigger`], used by `Effect::Reflexive` in
//! `resolve.rs`). "When you do, [effect] if [condition]" compiles to the same body, but its
//! condition isn't an intervening "if" (it doesn't follow the trigger condition) and is
//! checked only as the ability resolves; such an ability isn't understood ([`misread`],
//! one of the compiler's checks of what it parsed), rather than read the other way.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;

/// The intervening "if" of a reflexive ability's body: its whole effect is "if
/// [condition], [effect]", with a condition that isn't about the ability's own targets
/// (those aren't chosen until it's put on the stack).
pub fn intervening_if(body: &Body) -> Option<&Condition> {
    match &body.effect {
        Effect::If {
            cond, otherwise, ..
        } if matches!(**otherwise, Effect::Noop) && !format!("{cond:?}").contains("Target(") => {
            Some(cond)
        }
        _ => None,
    }
}

/// Whether the reflexive ability `body` doesn't trigger, its intervening "if" being false
/// as it would trigger (CR 603.4), with `ctx` the ability that creates it.
pub fn fails_to_trigger(g: &Game, body: &Body, ctx: &Ctx) -> bool {
    intervening_if(body).is_some_and(|c| !g.eval_cond(c, ctx))
}

/// Whether `a` has a reflexive ability shaped as one with an intervening "if" although its
/// text has no "When ..., if ..." sentence after the first ("When you do, draw a card if
/// you control an artifact.").
pub fn misread(a: &AbilityDef) -> bool {
    fn has_shape(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(m) => {
                let here = m
                    .get("Reflexive")
                    .and_then(|r| r.get("body"))
                    .and_then(|b| serde_json::from_value::<Body>(b.clone()).ok())
                    .is_some_and(|b| intervening_if(&b).is_some());
                here || m.values().any(has_shape)
            }
            serde_json::Value::Array(x) => x.iter().any(has_shape),
            _ => false,
        }
    }
    if !serde_json::to_value(&a.kind).is_ok_and(|v| has_shape(&v)) {
        return false;
    }
    let text = a.text.to_lowercase();
    !text.split(". ").skip(1).any(|s| {
        let s = s.trim_start_matches(|c: char| !c.is_alphanumeric());
        (s.starts_with("when ") || s.starts_with("whenever ")) && s.contains(", if ")
    })
}
