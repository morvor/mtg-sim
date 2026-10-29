//! Referents of a granted ability that are fixed as the granting effect resolves (CR
//! 608.2h, 611.2c): "permanents your opponents control gain "When this permanent deals
//! damage to the player who cast Hellish Rebuke, ..."" (each grant refers to its own
//! caster, Hellish Rebuke rulings), "It gains "When this token deals combat damage to a
//! player, sacrifice it and return the exiled card to the battlefield tapped."" (the card
//! exiled by the resolution that created that token, Timothar, Baron of Bats rulings).
//!
//! The compiler writes such a referent as a variable in a reserved range
//! ([`CAPTURED_CASTER`], [`CAPTURED_EXILED`]); [`bind`] replaces it, as the effect that
//! grants the ability is created (`Game::fix_mods`), by the players or objects it stands
//! for then: the caster of the resolving spell (nobody, for a copy that wasn't cast), and
//! the objects the granting effect stored in that variable, followed to where they are now
//! (an exiled card is a new object in exile, CR 400.7).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// "the player who cast [the granting spell]".
pub const CAPTURED_CASTER: Var = vars::USER + 51_300;
/// "the exiled card": what the granting effect stored here before granting.
pub const CAPTURED_EXILED: Var = vars::USER + 51_301;
/// "you" of the granting effect ("Only you may activate this ability.", Martyrdom): its
/// controller as the effect is created.
pub const CAPTURED_YOU: Var = vars::USER + 51_302;

fn is_captured(v: Var) -> bool {
    v == CAPTURED_CASTER || v == CAPTURED_EXILED || v == CAPTURED_YOU
}

/// Whether the ability refers to a captured referent.
pub fn mentions(a: &Ability) -> bool {
    let s = serde_json::to_string(&a.kind).unwrap_or_default();
    [CAPTURED_CASTER, CAPTURED_EXILED, CAPTURED_YOU]
        .iter()
        .any(|v| s.contains(&format!("{{\"Var\":{v}}}")))
}

/// The caster of the spell `ctx` resolves (none for a copy that wasn't cast).
fn caster(g: &Game, ctx: &Ctx) -> Option<PlayerId> {
    let id = ctx.stack_obj?;
    let o = g.obj(id);
    let si = o.stack.as_deref()?;
    (si.cast.was_cast && matches!(si.kind, crate::object::StackKind::Spell))
        .then_some(o.base_controller)
}

fn substitute(v: &mut serde_json::Value, g: &Game, ctx: &Ctx) {
    use serde_json::Value as J;
    if let J::Object(m) = v {
        if m.len() == 1 {
            if let Some(J::Number(n)) = m.get("Var") {
                let var = n.as_u64().unwrap_or(0) as Var;
                if is_captured(var) {
                    // A player reference ("PlayerRef::Var") or an object selection
                    // ("Sel::Var"): both serialize the same way; which one it is follows
                    // from the variable.
                    if var == CAPTURED_YOU {
                        *v = serde_json::to_value(PlayerRef::Player(ctx.controller))
                            .unwrap_or(J::Null);
                    } else if var == CAPTURED_CASTER {
                        if let Some(p) = caster(g, ctx) {
                            *v = serde_json::to_value(PlayerRef::Player(p)).unwrap_or(J::Null);
                        }
                    } else {
                        let ids: Vec<ObjectId> = match ctx.vars.get(&var) {
                            Some(e) => e
                                .iter()
                                .filter_map(|x| x.object())
                                .map(|o| g.current(o))
                                .collect(),
                            None => last_linked_exile(g, ctx).into_iter().collect(),
                        };
                        *v =
                            serde_json::to_value(Sel::All(Filter::Objects(ids))).unwrap_or(J::Null);
                    }
                    return;
                }
            }
        }
        for x in m.values_mut() {
            substitute(x, g, ctx);
        }
    } else if let J::Array(a) = v {
        for x in a.iter_mut() {
            substitute(x, g, ctx);
        }
    }
}

/// The card the resolving ability most recently exiled with its source (CR 607.2a): the
/// last object linked to the source's ability that is in exile now. (Tokens the ability
/// created are linked too, but aren't in exile.)
fn last_linked_exile(g: &Game, ctx: &Ctx) -> Option<ObjectId> {
    let src = ctx.source?;
    g.obj(src)
        .linked
        .get(&ctx.link)?
        .iter()
        .rev()
        .map(|o| g.current(*o))
        .find(|o| g.is_live(*o) && g.obj(*o).zone == crate::object::Zone::Exile)
}

/// `a` with its captured referents bound for the effect being created in `ctx`.
pub fn bind(g: &Game, a: &Ability, ctx: &Ctx) -> Ability {
    let Ok(mut kind) = serde_json::to_value(&a.kind) else {
        return a.clone();
    };
    substitute(&mut kind, g, ctx);
    match serde_json::from_value::<AbilityKind>(kind) {
        Ok(k) => AbilityDef::with_link(k, a.text.clone(), a.link),
        Err(_) => a.clone(),
    }
}
