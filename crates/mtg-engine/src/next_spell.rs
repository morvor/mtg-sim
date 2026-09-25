//! Effects that apply to spells as they're cast (CR 601.2a): continuous effects that
//! modify "the next spell you cast" (CR 611.2f), and static abilities that make spells
//! gain abilities as they're cast (CR 610.5).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::*;
use crate::types::*;

/// A continuous effect waiting for the next matching spell its controller casts
/// (CR 611.2f).
#[derive(Clone, Debug)]
pub struct NextSpellEffect {
    pub id: u32,
    pub player: PlayerId,
    pub source: Option<ObjectId>,
    pub filter: Filter,
    pub mods: Vec<Modification>,
    /// How long it waits for that spell.
    pub expires: Duration,
    pub created_turn: u32,
}

/// Creates a "the next [filter] spell you cast [this turn] has ..." effect. It doesn't
/// apply to anything yet (CR 611.2f).
pub fn exec_next_spell(
    g: &mut Game,
    filter: &Filter,
    mods: &[Modification],
    expires: &Duration,
    ctx: &Ctx,
) {
    let id = g.new_effect_id();
    let mods = g.fix_mods(mods, ctx);
    g.next_spell_effects.push(NextSpellEffect {
        id,
        player: ctx.controller,
        source: ctx.source,
        filter: filter.clone(),
        mods,
        expires: expires.clone(),
        created_turn: g.turn.number,
    });
}

/// Static abilities of the form "[filter] spells you cast have [ability]" create
/// one-shot effects that make spells gain the ability as they're cast (CR 610.5), rather
/// than applying continuously to spells on the stack.
pub fn is_cast_grant(s: &StaticAbility) -> bool {
    if s.is_cda {
        return false;
    }
    let StaticEffect::Continuous { affected, mods } = &s.effect else {
        return false;
    };
    affected.zone() == Some(ZoneKind::Stack)
        && !mentions_source(affected)
        && !mods.is_empty()
        && mods
            .iter()
            .all(|m| matches!(m, Modification::AddKeyword(_) | Modification::AddAbility(_)))
}

fn mentions_source(f: &Filter) -> bool {
    match f {
        Filter::Source => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(mentions_source),
        Filter::Not(x) => mentions_source(x),
        _ => false,
    }
}

/// Called as a spell is put on the stack while being cast (CR 601.2a): effects waiting
/// for the next spell begin to apply to it (CR 611.2f), and static abilities make it
/// gain abilities (CR 610.5).
pub fn spell_put_on_stack(g: &mut Game, spell: ObjectId, p: PlayerId) {
    g.recompute();
    let turn = g.turn.number;
    // Waiting effects that expired ("this turn") are gone.
    g.next_spell_effects.retain(|e| {
        !(matches!(e.expires, Duration::EndOfTurn | Duration::ThisTurn) && e.created_turn != turn)
    });
    let mut apply: Vec<(Option<ObjectId>, PlayerId, Vec<Modification>)> = Vec::new();
    let pending = std::mem::take(&mut g.next_spell_effects);
    let mut keep = Vec::new();
    for e in pending {
        let ctx = Ctx::new(e.source, e.player);
        if e.player == p && g.matches(spell, &e.filter, &ctx) {
            apply.push((e.source, e.player, e.mods));
        } else {
            keep.push(e);
        }
    }
    g.next_spell_effects = keep;
    // CR 611.3d: abilities granted to spells cast with a permission, until end of game.
    let from = g.obj(spell).stack.as_ref().and_then(|s| s.cast.from);
    let mut carried: Vec<(Option<ObjectId>, PlayerId, Vec<Modification>)> = Vec::new();
    for id in g.live_objects() {
        let o = g.obj(id);
        for a in &o.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::CastGrant { zone, what, mods } = &s.effect else {
                continue;
            };
            if !g.ability_functions(o, s.zone, s.is_cda) || from != Some(*zone) {
                continue;
            }
            let ctx = Ctx::new(Some(id), o.controller);
            if o.controller == p && g.matches(spell, what, &ctx) {
                carried.push((Some(id), o.controller, mods.clone()));
            }
        }
    }
    for (source, controller, mods) in carried {
        let id = g.new_effect_id();
        let ts = g.new_timestamp();
        g.effects.push(ContinuousEffect {
            id,
            source,
            controller,
            timestamp: ts,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![spell]),
            mods,
            layer1: None,
            created_turn: turn,
        });
        g.carried_effects.push(id);
    }
    // CR 610.5: "spells you cast have ..." abilities of objects.
    for id in g.live_objects() {
        let o = g.obj(id);
        for a in &o.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            if !is_cast_grant(s) || !g.ability_functions(o, s.zone, s.is_cda) {
                continue;
            }
            let StaticEffect::Continuous { affected, mods } = &s.effect else {
                continue;
            };
            let ctx = Ctx::new(Some(id), o.controller);
            if s.condition.as_ref().is_some_and(|c| !g.eval_cond(c, &ctx)) {
                continue;
            }
            if g.matches(spell, affected, &ctx) {
                apply.push((Some(id), o.controller, mods.clone()));
            }
        }
    }
    for (source, controller, mods) in apply {
        let id = g.new_effect_id();
        let ts = g.new_timestamp();
        g.effects.push(ContinuousEffect {
            id,
            source,
            controller,
            timestamp: ts,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![spell]),
            mods,
            layer1: None,
            created_turn: turn,
        });
    }
    g.dirty = true;
}
