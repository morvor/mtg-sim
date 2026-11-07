//! Copy effects of static abilities (CR 707.2, 613.2a): "Enchanted creature is a copy of
//! the chosen creature." (Metamorphic Alteration), "Equipped creature is a copy of the
//! last chosen card." ([`Modification::CopyOf`]).
//!
//! The effect applies in layer 1 with the copy effects of resolved spells and abilities,
//! in timestamp order (the static ability's timestamp is its object's, CR 613.7a). The
//! copiable values are locked in as the effect first begins to apply: they don't change
//! if the copied object's copiable values change later or it leaves the battlefield
//! (Metamorphic Alteration rulings). They're kept per source and copied object, so a new
//! choice ("the last chosen card") locks in the new object's values.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::{Characteristics, Zone};
use crate::types::*;
use std::sync::Arc;

/// One static copy effect that applies now.
pub(crate) struct StaticCopy {
    pub source: ObjectId,
    pub controller: PlayerId,
    pub timestamp: Timestamp,
    pub affected: Vec<ObjectId>,
    pub values: Arc<Characteristics>,
}

/// The id standing for a static copy effect of `src` when the abilities it copies are
/// linked to each other (CR 607.5a; see `layers::copied_ability`).
pub(crate) fn link_effect_id(src: ObjectId) -> u32 {
    1_000_000 + (src.0 % 1_000_000)
}

/// The object a static copy effect copies: the last object noted for the source's linked
/// abilities ("the chosen creature", "the last chosen card"; as it last existed if it has
/// left its zone), or what the selection selects.
fn copied(g: &Game, of: &Sel, ctx: &Ctx) -> Option<ObjectId> {
    match of {
        Sel::LinkedNoted => crate::linked_notes::noted_objects(g, ctx.source?, ctx.link)
            .into_iter()
            .last(),
        _ => g.eval_sel(of, ctx).into_iter().find_map(|e| e.object()),
    }
}

/// The static copy effects of the permanents in `live` that apply now, locking in the
/// copiable values of what each copies the first time it applies.
pub(crate) fn collect(g: &mut Game, live: &[ObjectId]) -> Vec<StaticCopy> {
    let mut out = Vec::new();
    for &id in live {
        let o = g.obj(id);
        if o.zone != Zone::Battlefield || o.phased_out {
            continue;
        }
        let mut found: Vec<(u16, Filter, Sel, Option<Condition>)> = Vec::new();
        for a in &o.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::Continuous { affected, mods } = &s.effect else {
                continue;
            };
            if !g.ability_functions(o, s.zone, s.is_cda) {
                continue;
            }
            for m in mods {
                if let Modification::CopyOf(of) = m {
                    found.push((
                        a.link,
                        affected.clone(),
                        (**of).clone(),
                        s.condition.clone(),
                    ));
                }
            }
        }
        for (link, affected, of, cond) in found {
            let mut ctx = Ctx::for_object(g, id);
            ctx.link = link;
            if cond.is_some_and(|c| !g.eval_cond(&c, &ctx)) {
                continue;
            }
            let Some(src) = copied(g, &of, &ctx) else {
                continue;
            };
            let values = match g.linked_notes.copy_values.get(&(id, src)) {
                Some(v) => v.clone(),
                None => {
                    let v = Arc::new(g.obj(src).copiable.clone());
                    g.linked_notes.copy_values.insert((id, src), v.clone());
                    v
                }
            };
            let affected = g.static_affected(id, &affected, live, &ctx);
            out.push(StaticCopy {
                source: id,
                controller: g.obj(id).controller,
                timestamp: g.obj(id).timestamp,
                affected,
                values,
            });
        }
    }
    out
}
