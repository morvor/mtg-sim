//! Special actions (CR 116) that aren't tied to a keyword: turning a face-down permanent
//! face up (CR 116.2b), actions effects allow later (CR 116.2c), ignoring a static
//! ability's effect (CR 116.2d), discarding a card any time you could cast an instant
//! (CR 116.2e), and turning a face-down conspiracy face up (CR 116.2j). Keyword special
//! actions (suspend, foretell, plot, companion) live in `kw/`.
//!
//! Special actions don't use the stack (CR 116.1); the player who takes one receives
//! priority afterward (CR 116.3), and the choice of how to pay hybrid or Phyrexian mana
//! symbols in its cost is made immediately before paying (CR 118.13c).

use crate::ability::*;
use crate::casting::Illegal;
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::mana::SpendContext;
use crate::object::*;
use crate::types::*;

/// A special action an effect allows a player to take later (CR 116.2c).
#[derive(Clone, Debug)]
pub struct SpecialOffer {
    pub id: u32,
    pub def: SpecialActionDef,
    /// The context of the effect that allowed it ("that permanent or player").
    pub ctx: Ctx,
    pub duration: Duration,
    pub repeatable: bool,
    pub created_turn: u32,
}

/// A card a keyword special action put somewhere, with the turn it happened: a foretold
/// card (CR 702.143a), a plotted card (CR 702.170a).
#[derive(Clone, Debug)]
pub struct KeywordMark {
    pub obj: ObjectId,
    pub kind: KeywordKind,
    pub turn: u32,
}

/// Special-action state kept by the game.
#[derive(Clone, Debug, Default)]
pub struct SpecialState {
    pub offers: Vec<SpecialOffer>,
    pub next_offer: u32,
    pub marks: Vec<KeywordMark>,
    /// Each player's chosen companion (CR 103.2b), and whether they've put it into their
    /// hand (CR 702.139a).
    pub companions: Vec<(PlayerId, ObjectId, bool)>,
    /// How many spells are being cast / abilities activated right now, the cards drawn
    /// meanwhile (kept face down until then, CR 121.8), and the "as you draw it" choices
    /// waiting until then: (player, card, nth card drawn this turn).
    pub casting: u32,
    pub drawn_while_casting: Vec<ObjectId>,
    pub deferred_draws: Vec<(PlayerId, ObjectId, u32)>,
    /// Cards a player may spend mana of any type to cast (CR 118.14): (player, card,
    /// duration, source, turn created).
    pub any_type_mana: Vec<(PlayerId, ObjectId, Duration, Option<ObjectId>, u32)>,
    /// (source, player, turn): the player ignores the source's static effects until end
    /// of that turn (CR 116.2d).
    pub ignoring: Vec<(ObjectId, PlayerId, u32)>,
}

/// Marks `obj` as foretold/plotted/... this turn.
pub fn mark(g: &mut Game, obj: ObjectId, kind: KeywordKind) {
    let turn = g.turn.number;
    g.special.marks.push(KeywordMark { obj, kind, turn });
}

/// The turn `obj` was marked with `kind`, if it was (and is still the same object).
pub fn marked(g: &Game, obj: ObjectId, kind: KeywordKind) -> Option<u32> {
    g.special
        .marks
        .iter()
        .rev()
        .find(|m| m.obj == obj && m.kind == kind && g.is_live(obj))
        .map(|m| m.turn)
}

/// Whether `p` is ignoring the effects of `source` this turn (CR 116.2d).
pub fn ignores(g: &Game, source: ObjectId, p: PlayerId) -> bool {
    g.special
        .ignoring
        .iter()
        .any(|(s, q, turn)| *s == source && *q == p && *turn == g.turn.number)
}

fn offer_active(g: &Game, o: &SpecialOffer) -> bool {
    match o.duration {
        Duration::EndOfTurn => o.created_turn == g.turn.number,
        _ => !g.effect_expired(&o.duration, o.ctx.source, o.ctx.controller),
    }
}

/// Ends "until end of turn" special actions and ignoring (CR 514.2).
pub fn end_of_turn(g: &mut Game) {
    let turn = g.turn.number;
    g.special
        .offers
        .retain(|o| !(matches!(o.duration, Duration::EndOfTurn) && o.created_turn == turn));
    g.special.ignoring.retain(|(_, _, t)| *t != turn);
}

/// Adds a special action offer (CR 116.2c).
pub fn offer(g: &mut Game, def: SpecialActionDef, ctx: &Ctx, duration: Duration, repeatable: bool) {
    let id = g.special.next_offer;
    g.special.next_offer += 1;
    let created_turn = g.turn.number;
    g.special.offers.push(SpecialOffer {
        id,
        def,
        ctx: ctx.clone(),
        duration,
        repeatable,
        created_turn,
    });
}

/// The special-action statics of objects, with the zones they function from.
fn static_actions(g: &Game) -> Vec<(ObjectId, u64, SpecialActionDef)> {
    let mut out = Vec::new();
    for id in g.live_objects() {
        let o = g.obj(id);
        for a in &o.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::SpecialAction(def) = &s.effect else {
                continue;
            };
            if g.ability_functions(o, s.zone, s.is_cda) {
                out.push((id, a.uid, def.clone()));
            }
        }
    }
    out
}

/// The cost of turning a face-down permanent face up (CR 116.2b): its morph or disguise
/// cost (CR 702.37e, 702.168d), or the mana cost of a manifested or cloaked
/// creature card (CR 701.40b, 701.58b).
pub fn turn_face_up_cost(g: &Game, id: ObjectId) -> Option<Cost> {
    let o = g.obj(id);
    if !o.face_down || o.zone != Zone::Battlefield {
        return None;
    }
    let card = o.card.as_ref()?;
    let face = card.characteristics(FaceState::Front);
    for a in &face.abilities {
        if let AbilityKind::Keyword(k) = &a.kind {
            if matches!(k.kind, KeywordKind::Morph | KeywordKind::Disguise) {
                return Some(k.cost.clone().unwrap_or_default());
            }
        }
    }
    let kind = o.choices.text.as_deref().unwrap_or("");
    if matches!(kind, "Manifest" | "Cloak") && face.card_types.contains(CardType::Creature) {
        return face.mana_cost.clone().map(Cost::mana);
    }
    None
}

/// The special actions (other than keyword ones and playing lands) `p` could take now.
pub fn available(g: &Game, p: PlayerId) -> Vec<Action> {
    let mut out = Vec::new();
    if !g.has_priority(p) {
        return out;
    }
    // CR 116.2b: face-down permanents the player controls that can be turned face up.
    for id in g.battlefield.iter().copied() {
        let o = g.obj(id);
        if o.controller == p && o.face_down {
            if let Some(cost) = turn_face_up_cost(g, id) {
                if g.can_pay_cost(p, &cost, Some(id), &Ctx::new(Some(id), p)) {
                    out.push(Action::Special(SpecialAction::TurnFaceUp { obj: id }));
                }
            }
        }
    }
    // CR 116.2j: face-down conspiracies in the command zone the player owns.
    for id in g.command.iter().copied() {
        let o = g.obj(id);
        if o.owner == p && o.face_down && o.base.is(CardType::Conspiracy) {
            out.push(Action::Special(SpecialAction::TurnFaceUp { obj: id }));
        }
    }
    // CR 116.2d, 116.2e: special actions granted by static abilities.
    for (src, uid, def) in static_actions(g) {
        let ctl = g.obj(src).controller;
        let ctx = Ctx::new(Some(src), ctl);
        if !g.player_filter_matches(&def.who, p, &ctx) {
            continue;
        }
        if matches!(def.action, SpecialActionEffect::IgnoreSourceEffects) && ignores(g, src, p) {
            continue;
        }
        if g.can_pay_cost(p, &def.cost, Some(src), &Ctx::new(Some(src), p)) {
            out.push(Action::Special(SpecialAction::Static {
                source: src,
                ability: uid,
            }));
        }
    }
    // CR 116.2c: special actions effects allow.
    for o in &g.special.offers {
        if !offer_active(g, o) {
            continue;
        }
        if !g.player_filter_matches(&o.def.who, p, &o.ctx) {
            continue;
        }
        if g.can_pay_cost(p, &o.def.cost, o.ctx.source, &o.ctx) {
            out.push(Action::Special(SpecialAction::Offer { id: o.id }));
        }
    }
    out
}

/// Pays the cost of a special action (CR 116.1). The choice of how to pay symbols that
/// can be paid in more than one way is made immediately before paying (CR 118.13c).
/// Nothing is paid if the whole cost can't be.
pub fn pay(g: &mut Game, p: PlayerId, cost: &Cost, src: Option<ObjectId>, ctx: &Ctx) -> bool {
    let snapshot = g.clone();
    let mut cost = cost.clone();
    crate::cost_rules::choose_payment_ways(g, p, src, &mut cost);
    let spend = SpendContext {
        source: src,
        ..Default::default()
    };
    match g.pay_total_cost(p, &cost, src, &spend, ctx) {
        Ok(_) => true,
        Err(_) => {
            let agents = g.agents.clone();
            *g = snapshot;
            g.agents = agents;
            false
        }
    }
}

/// Takes a special action handled here. Returns None if it isn't one of these.
pub fn perform(g: &mut Game, p: PlayerId, sa: &SpecialAction) -> Option<Result<(), Illegal>> {
    let bad = |s: &str| Some(Err(Illegal(s.into())));
    match sa {
        SpecialAction::TurnFaceUp { obj } => {
            let obj = *obj;
            if !g.is_live(obj) {
                return bad("no such object");
            }
            let o = g.obj(obj);
            if o.zone == Zone::Command {
                // CR 116.2j: a face-down conspiracy its owner turns face up.
                if o.owner != p || !o.face_down || !o.base.is(CardType::Conspiracy) {
                    return bad("can't turn that face up");
                }
                crate::variants::turn_face_up_in_command(g, obj);
                return Some(Ok(()));
            }
            if o.controller != p {
                return bad("you don't control that permanent");
            }
            let Some(cost) = turn_face_up_cost(g, obj) else {
                return bad("can't turn that face up");
            };
            if !pay(g, p, &cost, Some(obj), &Ctx::new(Some(obj), p)) {
                return bad("can't pay the cost");
            }
            crate::facedown::turn_face_up(g, obj, true);
            Some(Ok(()))
        }
        SpecialAction::Static { source, ability } => {
            let (source, ability) = (*source, *ability);
            let Some((_, _, def)) = static_actions(g)
                .into_iter()
                .find(|(s, u, _)| *s == source && *u == ability)
            else {
                return bad("no such special action");
            };
            let ctl = g.obj(source).controller;
            if !g.player_filter_matches(&def.who, p, &Ctx::new(Some(source), ctl)) {
                return bad("you can't take that special action");
            }
            let mut ctx = Ctx::new(Some(source), p);
            if !pay(g, p, &def.cost, Some(source), &ctx) {
                return bad("can't pay the cost");
            }
            match &def.action {
                SpecialActionEffect::IgnoreSourceEffects => {
                    let turn = g.turn.number;
                    g.special.ignoring.push((source, p, turn));
                    g.dirty = true;
                    g.recompute();
                }
                SpecialActionEffect::Effect(e) => g.exec(e, &mut ctx),
            }
            Some(Ok(()))
        }
        SpecialAction::Offer { id } => {
            let Some(o) = g
                .special
                .offers
                .iter()
                .find(|o| o.id == *id && offer_active(g, o))
                .cloned()
            else {
                return bad("no such special action");
            };
            if !g.player_filter_matches(&o.def.who, p, &o.ctx) {
                return bad("you can't take that special action");
            }
            let mut ctx = o.ctx.clone();
            if !pay(g, p, &o.def.cost, o.ctx.source, &ctx) {
                return bad("can't pay the cost");
            }
            if !o.repeatable {
                g.special.offers.retain(|x| x.id != o.id);
            }
            if let SpecialActionEffect::Effect(e) = &o.def.action {
                ctx.controller = p;
                g.exec(e, &mut ctx);
            }
            Some(Ok(()))
        }
        _ => None,
    }
}

fn not_players(f: &PlayerFilter, ps: &[PlayerId]) -> PlayerFilter {
    let mut v = vec![f.clone()];
    v.extend(
        ps.iter()
            .map(|p| PlayerFilter::Not(Box::new(PlayerFilter::Is(*p)))),
    );
    PlayerFilter::And(v)
}

fn not_objects(f: &Filter, objs: &[ObjectId]) -> Filter {
    if objs.is_empty() {
        return f.clone();
    }
    Filter::And(vec![
        f.clone(),
        Filter::Not(Box::new(Filter::Objects(objs.to_vec()))),
    ])
}

/// A restriction that doesn't apply to the players in `ps` or to objects they control
/// (`objs`) (CR 116.2d).
fn excluding(r: &Restriction, ps: &[PlayerId], objs: &[ObjectId]) -> Restriction {
    use Restriction as R;
    match r {
        R::CantAttack(f) => R::CantAttack(not_objects(f, objs)),
        R::CantBlock(f) => R::CantBlock(not_objects(f, objs)),
        R::CantAttackOrBlock(f) => R::CantAttackOrBlock(not_objects(f, objs)),
        R::DoesntUntap(f) => R::DoesntUntap(not_objects(f, objs)),
        R::CantCast { who, what } => R::CantCast {
            who: not_players(who, ps),
            what: what.clone(),
        },
        R::CantActivate {
            who,
            sources,
            include_mana,
        } => R::CantActivate {
            who: not_players(who, ps),
            sources: sources.clone(),
            include_mana: *include_mana,
        },
        R::CantGainLife(f) => R::CantGainLife(not_players(f, ps)),
        R::CantLoseLife(f) => R::CantLoseLife(not_players(f, ps)),
        R::CantSearch(f) => R::CantSearch(not_players(f, ps)),
        R::SorcerySpeedOnly(f) => R::SorcerySpeedOnly(not_players(f, ps)),
        R::CantPlayLands(f) => R::CantPlayLands(not_players(f, ps)),
        R::MaxDrawsPerTurn(f, n) => R::MaxDrawsPerTurn(not_players(f, ps), *n),
        R::MaxSpellsPerTurn(f, n) => R::MaxSpellsPerTurn(not_players(f, ps), *n),
        other => other.clone(),
    }
}

/// CR 116.2d: restrictions from sources whose effects some players are ignoring don't
/// apply to those players or to the objects they control. Called as static abilities
/// are collected.
pub fn apply_ignoring(g: &Game, restrictions: &mut [(ObjectId, PlayerId, Restriction)]) {
    if g.special.ignoring.is_empty() {
        return;
    }
    for (src, _, r) in restrictions.iter_mut() {
        let ps: Vec<PlayerId> = g
            .special
            .ignoring
            .iter()
            .filter(|(s, _, t)| *s == *src && *t == g.turn.number)
            .map(|(_, p, _)| *p)
            .collect();
        if ps.is_empty() {
            continue;
        }
        let objs: Vec<ObjectId> = g
            .battlefield
            .iter()
            .copied()
            .filter(|o| ps.contains(&g.obj(*o).controller))
            .collect();
        *r = excluding(r, &ps, &objs);
    }
}
