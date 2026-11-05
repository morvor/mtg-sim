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

/// The link of an ability a token copy gets from its creating effect's exceptions that
/// refers to "the exiled card" ([`confine_to_created`]), and under which that token keeps
/// the cards the effect exiled ([`keep_created_exiled`]). Printed links are smaller, and
/// copied or acquired abilities get links of 0x4000 and more (`layers::copied_link`,
/// `layers::acquired_ability`).
pub const CREATED_EXILED_LINK: u16 = 0x3fbb;

/// The exceptions of an effect that creates token copies ("create a token that's a copy
/// of that creature, except ... it has "When this token leaves the battlefield, return
/// the exiled card to its owner's graveyard."", Hofri Ghostforge), before their other
/// captured referents are bound ([`bind`]).
/// The granted ability and the ability that created the token are linked across the two
/// objects (CR 607.1d, 607.2a), so "the exiled card" is the card exiled by the
/// resolution that created that token: the granted ability refers to the cards the token
/// keeps under [`CREATED_EXILED_LINK`] ([`keep_created_exiled`] records them as the
/// tokens are created), whether or not the creating object is still on the battlefield.
/// The exception is part of the token's copiable values (CR 707.9a), so another object
/// that becomes or enters as a copy of the token has the ability too, but with a link of
/// its own (`layers::copied_ability`) under which it keeps nothing: for it, "the exiled
/// card" is nothing (Hofri Ghostforge ruling: it "will not be linked to Hofri's ability
/// and will not return the card to the graveyard"), and a token Hofri makes of such a
/// copy returns only the card exiled as it was made.
pub fn confine_to_created(mods: &[Modification]) -> Vec<Modification> {
    use serde_json::Value as J;
    fn walk(v: &mut J, with: &J) {
        match v {
            J::Object(m) => {
                if m.len() == 1
                    && m.get("Var")
                        .and_then(|n| n.as_u64())
                        .is_some_and(|n| n as Var == CAPTURED_EXILED)
                {
                    *v = with.clone();
                    return;
                }
                m.values_mut().for_each(|x| walk(x, with));
            }
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, with)),
            _ => {}
        }
    }
    let Ok(with) = serde_json::to_value(&Sel::Linked) else {
        return mods.to_vec();
    };
    mods.iter()
        .map(|m| match m {
            Modification::AddAbility(a) if mentions(a) => {
                let Ok(mut kind) = serde_json::to_value(&a.kind) else {
                    return m.clone();
                };
                let before = kind.clone();
                walk(&mut kind, &with);
                if kind == before {
                    return m.clone();
                }
                match serde_json::from_value::<AbilityKind>(kind) {
                    Ok(k) => Modification::AddAbility(AbilityDef::with_link(
                        k,
                        a.text.clone(),
                        CREATED_EXILED_LINK,
                    )),
                    Err(_) => m.clone(),
                }
            }
            _ => m.clone(),
        })
        .collect()
}

/// Records, on the tokens `created` by the effect resolving in `ctx`, the cards that
/// effect exiled for a confined granted ability ([`confine_to_created`]): the objects it
/// stored in [`CAPTURED_EXILED`] (followed to where they are now, CR 400.7), or else the
/// card its source's ability exiled last. Only tokens with such an ability keep them.
pub fn keep_created_exiled(g: &mut Game, ctx: &Ctx, created: &[ObjectId]) {
    let keeps: Vec<ObjectId> = created
        .iter()
        .copied()
        .filter(|t| {
            g.obj(*t)
                .copiable
                .abilities
                .iter()
                .chain(g.obj(*t).chars.abilities.iter())
                .any(|a| a.link == CREATED_EXILED_LINK)
        })
        .collect();
    if keeps.is_empty() {
        return;
    }
    let exiled: Vec<ObjectId> = match ctx.vars.get(&CAPTURED_EXILED) {
        Some(e) => e
            .iter()
            .filter_map(|x| x.object())
            .map(|o| g.current(o))
            .collect(),
        None => last_linked_exile(g, ctx).into_iter().collect(),
    };
    for t in keeps {
        let linked = g.objects[t.0 as usize]
            .linked
            .entry(CREATED_EXILED_LINK)
            .or_default();
        for o in &exiled {
            if !linked.contains(o) {
                linked.push(*o);
            }
        }
    }
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
