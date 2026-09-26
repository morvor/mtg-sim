//! Drawing cards (CR 121): limits on the number of cards drawn (CR 121.2b, 121.3), the
//! order in which several players draw (CR 121.2c, 121.2d), and replacement effects that
//! refer to the number of cards drawn (CR 121.2a, 616.1g).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Characteristics;
use crate::object::EventInfo;
use crate::replacement::ReplKey;
use crate::types::*;

/// How many more cards `p` may draw this turn because of effects that say a player can't
/// draw more than N cards each turn (or can't draw cards at all). `None`: no limit.
pub fn draws_left(g: &Game, p: PlayerId) -> Option<u32> {
    let drawn = g.history.cards_drawn.get(&p).copied().unwrap_or(0);
    let limit = |r: &Restriction, ctx: &Ctx| match r {
        Restriction::MaxDrawsPerTurn(pf, n) if g.player_filter_matches(pf, p, ctx) => Some(*n),
        _ => None,
    };
    g.statics
        .restrictions
        .iter()
        .filter_map(|(s, c, r)| limit(r, &Ctx::new(Some(*s), *c)))
        .chain(
            g.rule_effects
                .iter()
                .filter_map(|e| limit(&e.restriction, &Ctx::new(e.source, e.controller))),
        )
        .min()
        .map(|n| n.saturating_sub(drawn))
}

/// The order in which players perform their draws when several players are instructed to
/// draw: the active player first, then each other player in turn order (CR 121.2c). With
/// shared team turns, each player on the active team draws first (CR 121.2d, 805.6a).
pub fn draw_order(g: &Game, mut players: Vec<PlayerId>) -> Vec<PlayerId> {
    let apnap = g.apnap();
    let active_team = g.player(g.turn.active).team;
    let shared = crate::combat::shared_team_turns(g);
    players.sort_by_key(|p| {
        let team_rank = if shared && g.player(*p).team != active_team {
            1
        } else {
            0
        };
        (
            team_rank,
            apnap.iter().position(|x| x == p).unwrap_or(usize::MAX),
        )
    });
    players.dedup();
    players
}

/// Whether an effect instructs a player to draw cards.
pub fn draws_cards(e: &Effect) -> bool {
    match e {
        Effect::Draw { .. } => true,
        Effect::Seq(v) => v.iter().any(draws_cards),
        Effect::If {
            then, otherwise, ..
        } => draws_cards(then) || draws_cards(otherwise),
        Effect::May { effect, .. } | Effect::Repeat { effect, .. } => draws_cards(effect),
        Effect::ForEach { effect, .. } | Effect::ForEachPlayer { effect, .. } => {
            draws_cards(effect)
        }
        _ => false,
    }
}

/// Whether a player may choose to have `e` performed (a "you may draw" choice, or a cost
/// that includes drawing cards): not if it would have a player draw more cards than an
/// effect lets them draw — including any card at all for a player who can't draw cards
/// (CR 121.2b, 121.3, 121.3a). A player with no cards in their library may still choose
/// to draw (CR 121.3).
pub fn can_choose(g: &Game, e: &Effect, ctx: &Ctx) -> bool {
    match e {
        Effect::Draw { who, n } => {
            let n = g.eval_value(n, ctx).max(0) as u32;
            n == 0
                || g.eval_players(who, ctx)
                    .into_iter()
                    .all(|p| draws_left(g, p).is_none_or(|left| n <= left))
        }
        Effect::Seq(v) => v.iter().all(|x| can_choose(g, x, ctx)),
        // Keyword actions a player is unable to perform (e.g. CR 701.68b).
        Effect::KeywordAction { .. } | Effect::KeywordActionEx(_) => {
            crate::kwa::can_choose(g, e, ctx).unwrap_or(true)
        }
        _ => true,
    }
}

/// CR 121.2a, 616.1g: an instruction to draw several cards can be modified by replacement
/// effects that refer to the number of cards drawn ("If an opponent would draw two or
/// more cards, ..."); this happens before any of the individual draws. Returns the cards
/// drawn by the instruction if such an effect replaced it.
pub fn replace_multiple_draws(g: &mut Game, p: PlayerId, n: u32) -> Option<Vec<ObjectId>> {
    if n < 2 {
        return None;
    }
    if g.dirty {
        g.recompute();
    }
    let applied: Vec<ReplKey> = g.repl_context.last().cloned().unwrap_or_default();
    let mut cands: Vec<(ReplKey, Option<ObjectId>, PlayerId, ReplacementDef)> = Vec::new();
    for (s, c, _, a, d) in &g.statics.replacements {
        cands.push((ReplKey::Static(*s, a.uid), Some(*s), *c, d.clone()));
    }
    for inst in &g.replacements {
        if inst.uses != Some(0) {
            cands.push((
                ReplKey::Instance(inst.id),
                inst.source,
                inst.controller,
                inst.def.clone(),
            ));
        }
    }
    cands.retain(|(key, s, c, d)| match &d.event {
        ReplacementEvent::DrawCards { who, min } => {
            !applied.contains(key)
                && n >= *min
                && g.player_filter_matches(who, p, &Ctx::new(*s, *c))
        }
        _ => false,
    });
    if cands.is_empty() {
        return None;
    }
    // The affected player chooses which one applies (CR 616.1).
    let i = if cands.len() == 1 {
        0
    } else {
        let options = cands
            .iter()
            .map(|(_, s, _, _)| s.map(|s| g.describe(s)).unwrap_or_default())
            .collect();
        g.ask_option(p, None, "Choose a replacement effect", options)
    };
    let (key, source, controller, def) = cands.swap_remove(i);
    if def.optional && !g.ask_yes_no(controller, source, "Apply replacement effect?", true) {
        return None;
    }
    if let ReplKey::Instance(id) = key {
        if let Some(inst) = g.replacements.iter_mut().find(|r| r.id == id) {
            if let Some(u) = inst.uses.as_mut() {
                *u = u.saturating_sub(1);
            }
        }
    }
    match def.action {
        ReplacementAction::Prevent => Some(vec![]),
        ReplacementAction::Instead(effect) => {
            let mut ctx = Ctx::new(source, controller);
            ctx.event = Some(EventInfo {
                player: Some(p),
                amount: n as i32,
                ..Default::default()
            });
            let mut ctx_applied = applied;
            ctx_applied.push(key);
            g.repl_context.push(ctx_applied);
            g.exec(&effect, &mut ctx);
            g.repl_context.pop();
            Some(vec![])
        }
        _ => None,
    }
}

/// A card was drawn (CR 121.1). While a spell is being cast or an ability activated, the
/// card is kept face down until that's done: it has no characteristics, and effects that
/// let the player reveal it as it's drawn wait until then (CR 121.8).
pub fn card_drawn(g: &mut Game, p: PlayerId, card: ObjectId, nth: u32) {
    if g.special.casting > 0 {
        g.special.drawn_while_casting.push(card);
        g.special.deferred_draws.push((p, card, nth));
        return;
    }
    crate::kw::after_draw(g, p, card, nth);
}

/// A spell became cast or an ability became activated (CR 601.2i, 602.2e): cards drawn
/// meanwhile are no longer hidden, and deferred "as you draw it" reveals happen now
/// (CR 121.8).
pub fn finish_casting(g: &mut Game) {
    g.special.casting = g.special.casting.saturating_sub(1);
    if g.special.casting > 0 {
        return;
    }
    g.special.drawn_while_casting.clear();
    for (p, card, nth) in std::mem::take(&mut g.special.deferred_draws) {
        if g.is_live(card) {
            crate::kw::after_draw(g, p, card, nth);
        }
    }
    // CR 401.5: a new top card of a library is revealed now.
    if g.dirty {
        g.recompute();
    }
    crate::zones::update_revealed_tops(g);
}

/// A view in which one object has no characteristics.
struct NoCharacteristics(ObjectId);

fn empty_characteristics() -> &'static Characteristics {
    static EMPTY: std::sync::OnceLock<Characteristics> = std::sync::OnceLock::new();
    EMPTY.get_or_init(Characteristics::default)
}

impl crate::eval::View for NoCharacteristics {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics {
        if id == self.0 {
            empty_characteristics()
        } else {
            &g.obj(id).chars
        }
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        g.obj(id).controller
    }
}

/// Whether `card` may be used to pay a cost that needs a card matching `filter`: a card
/// drawn while the spell is being cast is considered to have no characteristics, so it
/// can't pay a cost that requires specific characteristics (CR 121.8).
pub fn usable_for_cost(g: &Game, card: ObjectId, filter: &Filter, ctx: &Ctx) -> bool {
    if !g.special.drawn_while_casting.contains(&card) {
        return g.matches(card, filter, ctx);
    }
    g.matches_view(&NoCharacteristics(card), card, filter, ctx)
}
