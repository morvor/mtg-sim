//! The characteristics a card not yet on the stack would have as the spell it's proposed
//! as (CR 601.3e): "the spell's characteristics are determined as if it were on the stack,
//! including effects that would apply to it there".
//!
//! [`Game::option_characteristics`] gives the printed characteristics of the face, half,
//! Adventure or face-down spell it would be cast as. When a continuous effect that applies
//! to spells is in play (Celestial Dawn's "spells you control are white", Encroaching
//! Mycosynth's "permanent spells you control are artifacts"), those may differ from the
//! spell's: [`spell_characteristics`] then computes them on a copy of the game with the
//! card put on the stack the way it would be cast (CR 601.2a), all layers applied.

use crate::ability::*;
use crate::casting::CastOption;
use crate::game::{Affected, Game};
use crate::object::*;
use crate::types::*;

/// The characteristics `card` would have as a spell `p` casts with `opt` (CR 601.3e).
pub fn spell_characteristics(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    opt: &CastOption,
) -> Characteristics {
    let printed = g.option_characteristics(card, opt);
    if g.obj(card).zone == Zone::Stack
        || !(spell_effects_in_play(g) || own_effects_on_the_stack(&printed))
    {
        return printed;
    }
    let mut h = g.clone();
    // CR 601.2a, as `cast_inner` does: the card moves to the stack (a new object).
    let from = h.obj(card).zone;
    if let Some(list) = h.zone_list_mut(from) {
        list.retain(|x| *x != card);
    }
    let id = h.create_incarnation(card, Zone::Stack);
    {
        let o = &mut h.objects[id.0 as usize];
        o.zone = Zone::Stack;
        o.controller = p;
        o.base_controller = p;
        o.face = opt.face;
        if let CastMethod::FaceDown(_) = opt.method {
            o.face_down = true;
        }
        if let Some(d) = o.card.clone() {
            o.base = d.characteristics(opt.face);
        }
        let mut cast = CastInfo {
            method: opt.method.clone(),
            from: from.kind(),
            was_cast: true,
            turn: g.turn.number,
            ..Default::default()
        };
        if let Some(t) = opt.tag {
            cast.paid.push(t.into());
        }
        o.stack = Some(Box::new(StackInfo {
            kind: StackKind::Spell,
            chosen: vec![],
            x: None,
            cast,
            event: None,
            source_lki: None,
            chosen_values: Default::default(),
        }));
    }
    h.stack.push(id);
    // Effects waiting for the next spell apply to it as it's put on the stack (CR 601.2a).
    crate::next_spell::spell_put_on_stack(&mut h, id, p);
    h.recompute();
    h.obj(id).chars.clone()
}

/// Whether a continuous effect that may apply to spells is in play: a static ability of a
/// permanent (or an object in the command zone, or a spell on the stack) whose affected
/// objects include spells or that grants abilities to spells cast with a permission
/// (CR 611.3d), an effect of a resolved spell or ability that applies to spells, or an
/// effect waiting for the next spell cast. Otherwise a spell has its printed
/// characteristics.
pub fn spell_effects_in_play(g: &Game) -> bool {
    !g.next_spell_effects.is_empty()
        || g.effects
            .iter()
            .any(|e| matches!(&e.affected, Affected::Filter(f) if mentions_spells(f)))
        || g.battlefield
            .iter()
            .chain(g.command.iter())
            .chain(g.stack.iter())
            .any(|id| {
                g.obj(*id).chars.abilities.iter().any(|a| {
                    matches!(&a.kind, AbilityKind::Static(s)
                    if match &s.effect {
                        StaticEffect::Continuous { affected, .. } => mentions_spells(affected),
                        StaticEffect::CastGrant { .. } => true,
                        _ => false,
                    })
                })
            })
}

/// Whether the spell's own characteristics include a non-characteristic-defining static
/// ability with a continuous effect that functions on the stack ("as long as this spell is
/// on the stack", "this spell ... wherever it is"): one its printed characteristics don't
/// show applied.
fn own_effects_on_the_stack(c: &Characteristics) -> bool {
    c.abilities.iter().any(|a| {
        matches!(&a.kind, AbilityKind::Static(s)
            if !s.is_cda
                && matches!(s.zone, FunctionZone::Stack | FunctionZone::Anywhere | FunctionZone::AnywhereExcept(_))
                && matches!(s.effect, StaticEffect::Continuous { .. }))
    })
}

/// Whether a filter of affected objects can match spells.
fn mentions_spells(f: &Filter) -> bool {
    match f {
        Filter::Spell
        | Filter::SpellOnStack
        | Filter::InZone(ZoneKind::Stack)
        | Filter::CastBy(_)
        | Filter::CastFrom(_)
        | Filter::CastWithCost(_)
        | Filter::Targets(_)
        | Filter::StackTargets(_) => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_spells),
        _ => false,
    }
}
