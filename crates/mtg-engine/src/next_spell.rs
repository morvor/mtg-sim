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
    let mut mods = g.fix_mods(mods, ctx);
    // "The next spell you cast this turn costs {X} less to cast, where X is ...": X is
    // determined now (CR 608.2h).
    crate::kw::spell_cost_grammar::lock_granted_cost_changes(g, &mut mods, ctx);
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

/// Whether the waiting effect lets the next matching spell be cast as though it had flash
/// ("The next creature spell you cast this turn can be cast as though it had flash.",
/// CR 601.3b; "... has flash"): it's still waiting this turn.
pub fn gives_flash(g: &Game, e: &NextSpellEffect) -> bool {
    let current = !matches!(e.expires, Duration::EndOfTurn | Duration::ThisTurn)
        || e.created_turn == g.turn.number;
    current
        && e.mods.iter().any(|m| {
            matches!(m, Modification::AddKeyword(k)
                if k.kind == crate::keywords::KeywordKind::Flash)
        })
}

/// Static abilities of the form "[filter] spells you cast have [ability]" create
/// one-shot effects that make spells gain the ability as they're cast (CR 610.5), rather
/// than applying continuously to spells on the stack ("[filter] spells you control have
/// [ability]", `Filter::SpellOnStack`, CR 611.3a).
pub fn is_cast_grant(s: &StaticAbility) -> bool {
    if s.is_cda {
        return false;
    }
    let StaticEffect::Continuous { affected, mods } = &s.effect else {
        return false;
    };
    affected.zone() == Some(ZoneKind::Stack)
        && !mentions(affected, &Filter::SpellOnStack)
        && !mentions_source(affected)
        && !mods.is_empty()
        && mods.iter().all(|m| {
            matches!(
                m,
                Modification::AddKeyword(_)
                    | Modification::AddKeywordOwnCost(..)
                    | Modification::AddAbility(_)
            )
        })
}

/// `Filter::Custom` conjunct of a waiting effect's filter: it applies to each matching
/// spell its controller casts while it lasts ("as you cast spells from your hand this
/// turn, they gain cascade"), rather than only to the next one. Matches every object.
pub const EACH_SPELL_THIS_TURN: &str = "next spell:each spell this turn";

/// `Filter::Custom` conjunct of a waiting effect's filter: the effect waits for the next
/// matching card its controller *plays* ("the next creature card you play this turn can be
/// played as though it had flash"), so a land card with that quality played as a land
/// (Dryad Arbor) uses it up and gets its timing too (CR 305.1, 702.8a). Matches every
/// object.
pub const CARD_YOU_PLAY: &str = "next spell:card you play";

/// Whether a waiting effect lets player `p` play the land card `card` as though it had
/// flash: one waiting for the next matching card `p` plays (not merely casts).
pub fn lets_land_be_played_with_flash(g: &Game, p: PlayerId, card: ObjectId) -> bool {
    g.next_spell_effects.iter().any(|e| {
        e.player == p
            && filter_has_custom(&e.filter, CARD_YOU_PLAY)
            && gives_flash(g, e)
            && g.matches(
                card,
                &crate::casting::as_spell_filter(&e.filter),
                &Ctx::new(e.source, e.player),
            )
    })
}

/// Called as player `p` plays the land card `card` (still where it was played from):
/// effects waiting for the next matching card `p` plays are used up (CR 305.1).
pub fn land_played(g: &mut Game, p: PlayerId, card: ObjectId) {
    let turn = g.turn.number;
    let pending = std::mem::take(&mut g.next_spell_effects);
    let keep = pending
        .into_iter()
        .filter(|e| {
            let current = !matches!(e.expires, Duration::EndOfTurn | Duration::ThisTurn)
                || e.created_turn == turn;
            !(current
                && e.player == p
                && filter_has_custom(&e.filter, CARD_YOU_PLAY)
                && g.matches(
                    card,
                    &crate::casting::as_spell_filter(&e.filter),
                    &Ctx::new(e.source, e.player),
                ))
        })
        .collect();
    g.next_spell_effects = keep;
}

pub(crate) fn filter_has_custom(f: &Filter, name: &str) -> bool {
    match f {
        Filter::Custom(n) => n == name,
        Filter::And(v) => v.iter().any(|x| filter_has_custom(x, name)),
        _ => false,
    }
}

/// Whether the filter has `part` among its conjuncts.
fn mentions(f: &Filter, part: &Filter) -> bool {
    match f {
        Filter::And(v) => v.iter().any(|x| mentions(x, part)),
        _ => std::mem::discriminant(f) == std::mem::discriminant(part),
    }
}

/// Whether a cast grant depends on the mana spent to cast the spell ("Each spell you cast
/// has split second if mana from an artifact was spent to cast it"): that's known only once
/// the total cost is paid, so the grant is applied then, as the spell becomes cast
/// (CR 601.2h-i), rather than as it's put on the stack.
fn depends_on_mana_spent(f: &Filter) -> bool {
    match f {
        Filter::Custom(n) => n.starts_with(crate::kw::cast_history::SPENT_SOME),
        Filter::And(v) => v.iter().any(depends_on_mana_spent),
        _ => false,
    }
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
            // "As you cast spells from your hand this turn, they gain cascade": every such
            // spell this turn, not only the next.
            if filter_has_custom(&e.filter, EACH_SPELL_THIS_TURN) {
                apply.push((e.source, e.player, e.mods.clone()));
                keep.push(e);
            } else {
                apply.push((e.source, e.player, e.mods));
            }
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
            if depends_on_mana_spent(affected) {
                continue;
            }
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
        // CR 400.7b: an ability granted to a permanent spell that functions on the
        // battlefield (offspring's triggered ability) continues to apply to the permanent
        // the spell becomes.
        let carry = mods.iter().any(functions_on_battlefield);
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
        if carry {
            g.carried_effects.push(id);
        }
    }
    g.dirty = true;
}

/// Once the spell's total cost is paid (CR 601.2h-i): static abilities whose grants depend
/// on the mana spent to cast it make it gain their abilities (CR 610.5).
pub fn grant_after_payment(g: &mut Game, spell: ObjectId) {
    g.recompute();
    let mut apply: Vec<(ObjectId, PlayerId, Vec<Modification>)> = Vec::new();
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
            if !depends_on_mana_spent(affected) {
                continue;
            }
            let ctx = Ctx::new(Some(id), o.controller);
            if s.condition.as_ref().is_some_and(|c| !g.eval_cond(c, &ctx)) {
                continue;
            }
            if g.matches(spell, affected, &ctx) {
                apply.push((id, o.controller, mods.clone()));
            }
        }
    }
    if apply.is_empty() {
        return;
    }
    let turn = g.turn.number;
    for (source, controller, mods) in apply {
        let id = g.new_effect_id();
        let ts = g.new_timestamp();
        let carry = mods.iter().any(functions_on_battlefield);
        g.effects.push(ContinuousEffect {
            id,
            source: Some(source),
            controller,
            timestamp: ts,
            duration: Duration::Permanent,
            affected: Affected::Objects(vec![spell]),
            mods,
            layer1: None,
            created_turn: turn,
        });
        if carry {
            g.carried_effects.push(id);
        }
    }
    g.dirty = true;
}

/// As the spell becomes cast (CR 601.2i): an ability a static ability of an object made
/// the spell gain as it was put on the stack ("Spells you cast with mana value 6 or
/// greater have cascade") is lost if that static ability no longer applies to it — e.g.
/// its source was sacrificed or changed control while the spell's costs were paid
/// (Imoti, Celebrant of Bounty and Party Thrasher rulings). Abilities from effects
/// waiting for "the next spell you cast" (CR 611.2f) aren't affected.
pub fn recheck_static_cast_grants(g: &mut Game, spell: ObjectId) {
    g.recompute();
    let mut drop: Vec<u32> = Vec::new();
    for e in &g.effects {
        let (Some(src), Affected::Objects(v)) = (e.source, &e.affected) else {
            continue;
        };
        if v.as_slice() != [spell] || e.layer1.is_some() {
            continue;
        }
        let mods = format!("{:?}", e.mods);
        let o = g.obj(src);
        // The static ability (of the source as it last existed) that granted this.
        let Some(s) = o.chars.abilities.iter().find_map(|a| match &a.kind {
            AbilityKind::Static(s)
                if is_cast_grant(s)
                    && matches!(&s.effect, StaticEffect::Continuous { mods: m, .. }
                        if format!("{m:?}") == mods) =>
            {
                Some(s)
            }
            _ => None,
        }) else {
            continue;
        };
        let StaticEffect::Continuous { affected, .. } = &s.effect else {
            continue;
        };
        let ctx = Ctx::new(Some(src), o.controller);
        let applies = g.is_live(src)
            && g.ability_functions(o, s.zone, s.is_cda)
            && !s.condition.as_ref().is_some_and(|c| !g.eval_cond(c, &ctx))
            && g.matches(spell, affected, &ctx);
        if !applies {
            drop.push(e.id);
        }
    }
    if !drop.is_empty() {
        g.effects.retain(|e| !drop.contains(&e.id));
        g.carried_effects.retain(|id| !drop.contains(id));
        g.dirty = true;
        g.recompute();
    }
}

/// Whether a granted keyword stands for an ability that functions on the battlefield
/// (CR 400.7b).
fn functions_on_battlefield(m: &Modification) -> bool {
    let Modification::AddKeyword(k) = m else {
        return false;
    };
    crate::kw::derived(k).iter().any(|a| match &a.kind {
        AbilityKind::Triggered(t) => t.zone == FunctionZone::Battlefield,
        AbilityKind::Static(s) => s.zone == FunctionZone::Battlefield,
        _ => false,
    })
}

/// Keywords the static abilities of objects `p` controls would make `card` gain as `p`
/// casts it (CR 610.5: "Assassin spells you cast have freerunning {B}{B}"). Those that
/// offer a way to cast it (an alternative cost, CR 601.2b) must be known before it's cast.
pub fn cast_grant_keywords(g: &Game, p: PlayerId, card: ObjectId) -> Vec<crate::keywords::Keyword> {
    let mut out = Vec::new();
    for id in g.live_objects() {
        let o = g.obj(id);
        if o.controller != p {
            continue;
        }
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
            if !g.matches(card, &crate::casting::as_spell_filter(affected), &ctx) {
                continue;
            }
            for m in mods {
                match m {
                    Modification::AddKeyword(k) => out.push(k.clone()),
                    // "The emerge cost is equal to its mana cost."
                    Modification::AddKeywordOwnCost(k, reduce) => {
                        out.push(crate::layers::own_cost_keyword(
                            k,
                            *reduce,
                            &g.characteristics_to_cast(card),
                        ))
                    }
                    _ => {}
                }
            }
        }
    }
    out
}
