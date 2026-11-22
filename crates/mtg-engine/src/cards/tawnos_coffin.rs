//! Tawnos's Coffin: "{3}, {T}: Exile target creature and all Auras attached to it. Note the
//! number and kind of counters that were on that creature. When ~ leaves the battlefield or
//! becomes untapped, return that exiled card to the battlefield under its owner's control
//! tapped with the noted number and kind of counters on it. If you do, return the other
//! exiled cards to the battlefield under their owner's control attached to that permanent."
//!
//! The target, the exile of the creature and its Auras and the delayed triggered ability
//! ("When ~ leaves the battlefield or becomes untapped", CR 603.7) are the compiled ones.
//! Hand-written: the note of the creature's counters and Auras as it's exiled (CR 607.2e),
//! and the return: the card comes back tapped (it doesn't enter and then become tapped)
//! with the noted counters, under its owner's control, and only if it did come back, each
//! exiled Aura that can legally enchant it returns attached to it (CR 303.4f; one that
//! can't stays exiled). A token creature ceases to exist in exile (CR 704.5d, 111.8), so
//! nothing returns and its Auras stay exiled; if the Coffin leaves the battlefield before
//! the ability resolves, the delayed trigger never triggers (rulings).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{Entity, ObjectId};

const TEXT: &str = "{3}, {T}: Exile target creature and all Auras attached to it. Note the number and kind of counters that were on that creature. When ~ leaves the battlefield or becomes untapped, return that exiled card to the battlefield under its owner's control tapped with the noted number and kind of counters on it. If you do, return the other exiled cards to the battlefield under their owner's control attached to that permanent.";
const NOTE: &str = "card:Tawnos's Coffin:note the creature's counters and Auras";
const RETURN: &str = "card:Tawnos's Coffin:return the card with its counters, then its Auras";
/// Rows `[creature, number]` under `COUNTERS` + the counter kind: the noted counters of
/// the creature (its id as it was exiled).
const COUNTERS: &str = "card:Tawnos's Coffin:noted counters:";

/// The exiled creature (as it was on the battlefield).
const CARD: Var = vars::USER + 31240;
/// The Auras exiled with it (as they were on the battlefield).
const AURAS: Var = vars::USER + 31241;
/// One object being returned.
const RETURNING: Var = vars::USER + 31242;

inventory::submit! { ManualAbility {
    card: "Tawnos's Coffin",
    face: 0,
    text: TEXT,
    build: |ctx| {
        parse(
            ctx,
            "{3}, {T}: Exile target creature and all Auras attached to it. When ~ leaves the battlefield or becomes untapped, return that card to the battlefield under its owner's control tapped.",
        )
        .iter()
        .map(|a| {
            map_effect(a, TEXT, |e| match e {
                Effect::Seq(v) => {
                    let mut out = vec![Effect::Custom(NOTE.into())];
                    for e in v {
                        match e {
                            // The card is noted before it's exiled.
                            Effect::Store { .. } => {}
                            Effect::DelayedTrigger { trigger, mut body, once } => {
                                body.effect = Effect::Custom(RETURN.into());
                                out.push(Effect::DelayedTrigger { trigger, body, once });
                            }
                            e => out.push(e),
                        }
                    }
                    Effect::Seq(out)
                }
                e => e,
            })
        })
        .collect()
    },
    reason: "returns the exiled creature with its noted counters and then its exiled Auras attached to it: only this card",
} }

fn first_object(ctx: &Ctx, v: Var) -> Option<ObjectId> {
    ctx.vars.get(&v)?.iter().find_map(|e| e.object())
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            NOTE => {
                if g.dirty {
                    g.recompute();
                }
                let Some(creature) = g
                    .resolve_sel(&Sel::Target(0), ctx)
                    .into_iter()
                    .filter_map(|e| e.object())
                    .find(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Battlefield)
                else {
                    return true;
                };
                let counters: Vec<(String, u32)> = g
                    .obj(creature)
                    .counters
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .map(|(k, n)| (k.to_string(), *n))
                    .collect();
                for (k, n) in counters {
                    g.cards
                        .push(&format!("{COUNTERS}{k}"), vec![creature.0 as i64, n as i64]);
                }
                let auras: Vec<Entity> = g
                    .permanents()
                    .filter(|o| {
                        o.chars.has_subtype("Aura")
                            && o.attached_to == Some(Entity::Object(creature))
                    })
                    .map(|o| Entity::Object(o.id))
                    .collect();
                ctx.set_var(CARD, vec![Entity::Object(creature)]);
                ctx.set_var(AURAS, auras);
                true
            }
            RETURN => {
                let Some(creature) = first_object(ctx, CARD) else {
                    return true;
                };
                let card = g.current(creature);
                if !g.is_live(card) || g.obj(card).zone != Zone::Exile {
                    return true;
                }
                let prefix = COUNTERS;
                let with_counters: Vec<(crate::types::CounterKind, Value)> = g
                    .cards
                    .rows
                    .iter()
                    .filter_map(|(k, rows)| {
                        let kind = k.strip_prefix(prefix)?;
                        let n: i64 = rows
                            .iter()
                            .filter(|r| r[0] == creature.0 as i64)
                            .map(|r| r[1])
                            .sum();
                        (n > 0).then(|| (kind.into(), Value::Const(n as i32)))
                    })
                    .collect();
                ctx.set_var(RETURNING, vec![Entity::Object(card)]);
                let mut to = Destination::zone(ZoneKind::Battlefield);
                to.tapped = true;
                to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(RETURNING))));
                to.with_counters = with_counters;
                g.exec(
                    &Effect::Move {
                        what: Sel::Var(RETURNING),
                        to,
                    },
                    ctx,
                );
                let perm = g.current(card);
                if !g.is_live(perm) || g.obj(perm).zone != Zone::Battlefield {
                    return true;
                }
                // "If you do": the Auras exiled with it, attached to it.
                let auras: Vec<ObjectId> = ctx
                    .vars
                    .get(&AURAS)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.object())
                    .collect();
                for aura in auras {
                    if g.dirty {
                        g.recompute();
                    }
                    let a = g.current(aura);
                    if !g.is_live(a)
                        || g.obj(a).zone != Zone::Exile
                        || !g.is_live(perm)
                        || g.obj(perm).zone != Zone::Battlefield
                        || !crate::attach::can_attach(g, a, Entity::Object(perm))
                    {
                        continue;
                    }
                    ctx.set_var(RETURNING, vec![Entity::Object(a)]);
                    let mut to = Destination::zone(ZoneKind::Battlefield);
                    to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(RETURNING))));
                    to.attached_to = Some(Sel::Var(CARD_NOW));
                    ctx.set_var(CARD_NOW, vec![Entity::Object(perm)]);
                    g.exec(
                        &Effect::Move {
                            what: Sel::Var(RETURNING),
                            to,
                        },
                        ctx,
                    );
                }
                true
            }
            _ => false,
        }
    }
}

/// The returned permanent.
const CARD_NOW: Var = vars::USER + 31243;

inventory::submit! { KeywordRegistration(&Rules) }
