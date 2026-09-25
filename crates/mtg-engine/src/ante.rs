//! Playing for ante (CR 407), an optional variation enabled by `GameConfig::ante`.
//!
//! * Before any cards are drawn, each player antes a random card from their deck; the
//!   winner of the game becomes the owner of every card in the ante zone (CR 407.2).
//! * Cards with "Remove this card from your deck before playing if you're not playing for
//!   ante" are the only ones that add or remove cards from the ante zone or change a card's
//!   owner; without ante they can't be in decks or sideboards, nor be brought into the
//!   game from outside it (CR 407.3).
//! * Only an object's owner can ante it (CR 407.4).

use crate::ability::*;
use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::{Game, GameResult};
use crate::object::*;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::*;
use std::sync::Arc;

/// `StaticEffect::Custom` name of "Remove this card from your deck before playing if
/// you're not playing for ante."
pub const ANTE_CARD: &str = "ante:card";
/// `Effect::Custom`: the acting player (the iterated player of "each player", else the
/// controller) antes the top card of their library.
pub const ANTE_TOP: &str = "ante:top card of library";
/// `Effect::Custom`: the controller becomes the owner of the objects in `vars::IT`
/// ("You own target card in the ante").
pub const GAIN_OWNERSHIP: &str = "ante:gain ownership";
/// `Effect::Custom`: the objects in `vars::IT` (cards in the ante) are exchanged with the
/// top card of the controller's library ("Exchange that card with the top card of your
/// library").
pub const EXCHANGE_WITH_TOP: &str = "ante:exchange with top card";

/// Whether a card has the ante reminder "Remove this card from your deck before playing
/// if you're not playing for ante." (CR 407.3).
pub fn is_ante_card(card: &CardDef) -> bool {
    card.faces.iter().any(|f| {
        f.chars.abilities.iter().any(|a| {
            matches!(&a.kind, AbilityKind::Static(s)
                if matches!(&s.effect, StaticEffect::Custom(n) if n.as_str() == ANTE_CARD))
        })
    })
}

fn object_is_ante_card(g: &Game, id: ObjectId) -> bool {
    g.obj(id).card.as_deref().is_some_and(is_ante_card)
}

/// CR 407.3: when not playing for ante, ante cards can't be in a deck or sideboard.
pub fn check_deck(
    deck: &[Arc<CardDef>],
    sideboard: &[Arc<CardDef>],
    playing_for_ante: bool,
) -> Vec<DeckProblem> {
    if playing_for_ante {
        return vec![];
    }
    let mut out: Vec<DeckProblem> = Vec::new();
    for c in deck.iter().chain(sideboard) {
        if is_ante_card(c) {
            let p = DeckProblem::AnteCard {
                name: c.name.to_string(),
            };
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

/// Moves the rules of ante forbid; the object stays where it is.
pub fn move_forbidden(g: &Game, mv: &MoveEv) -> bool {
    let o = g.obj(mv.obj);
    // CR 407.3: without ante, these cards can't be brought into the game from outside.
    if matches!(o.zone, Zone::Outside(_))
        && !matches!(mv.to, Zone::Outside(_))
        && !g.config.ante
        && object_is_ante_card(g, mv.obj)
    {
        return true;
    }
    let into = mv.to == Zone::Ante && o.zone != Zone::Ante;
    let out_of = o.zone == Zone::Ante && !matches!(mv.to, Zone::Ante | Zone::Nowhere);
    if into || out_of {
        // CR 407.3: only ante cards add cards to or remove cards from the ante zone.
        if !mv.source.is_some_and(|s| object_is_ante_card(g, s)) {
            return true;
        }
        // CR 407.4: only an object's owner can ante it.
        if into && mv.by != Some(o.owner) {
            return true;
        }
    }
    false
}

/// CR 407.2: after the starting player is determined and before any cards are drawn,
/// each player puts a random card from their deck into the ante zone.
pub fn ante_at_start(g: &mut Game) {
    use rand::Rng;
    if !g.config.ante {
        return;
    }
    for p in g.apnap() {
        let n = g.player(p).library.len();
        if n == 0 {
            continue;
        }
        let i = g.rng.gen_range(0..n);
        let card = g.players[p.idx()].library.remove(i);
        let new = g.create_incarnation(card, Zone::Ante);
        g.ante.push(new);
        g.log(|g| format!("{p} antes {}", g.describe(new)));
    }
    g.dirty = true;
}

/// CR 407.2: at the end of the game, the winner becomes the owner of all the cards in
/// the ante zone.
pub fn game_ended(g: &mut Game) {
    if !g.config.ante {
        return;
    }
    let Some(GameResult::Win(winners)) = &g.result else {
        return;
    };
    let Some(&winner) = winners.first() else {
        return;
    };
    for id in g.ante.clone() {
        g.objects[id.0 as usize].owner = winner;
    }
    g.log(|_| format!("{winner} becomes the owner of the cards in the ante"));
}

/// `p` antes `obj`: puts it into the ante zone from whichever zone it's in (CR 407.4).
/// The rules of ante may forbid the move (CR 407.3, 407.4). Returns the new object.
pub fn ante(
    g: &mut Game,
    p: PlayerId,
    obj: ObjectId,
    source: Option<ObjectId>,
) -> Option<ObjectId> {
    g.move_object_ev(MoveEv {
        obj,
        to: Zone::Ante,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo::default(),
        source,
    })
}

fn it_objects(ctx: &Ctx) -> Vec<ObjectId> {
    ctx.vars
        .get(&vars::IT)
        .map(|v| v.iter().filter_map(|e| e.object()).collect())
        .unwrap_or_default()
}

/// Performs an ante-related `Effect::Custom`, if `name` is one.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    match name {
        ANTE_TOP => {
            let p = ctx.iter_player.unwrap_or(ctx.controller);
            if let Some(top) = g.library_top(p) {
                let new = ante(g, p, top, ctx.source);
                ctx.set_var(vars::IT, new.into_iter().map(Entity::Object).collect());
            }
            true
        }
        GAIN_OWNERSHIP => {
            // CR 407.3: only ante cards change a card's owner.
            if !ctx.source.is_some_and(|s| object_is_ante_card(g, s)) {
                return true;
            }
            let p = ctx.controller;
            for o in it_objects(ctx) {
                let o = g.current(o);
                if g.is_live(o) {
                    g.objects[o.0 as usize].owner = p;
                    g.log(|g| format!("{p} becomes the owner of {}", g.describe(o)));
                }
            }
            g.dirty = true;
            true
        }
        EXCHANGE_WITH_TOP => {
            let p = ctx.controller;
            let cards: Vec<ObjectId> = it_objects(ctx)
                .into_iter()
                .map(|o| g.current(o))
                .filter(|o| g.is_live(*o) && g.obj(*o).zone == Zone::Ante)
                .collect();
            let Some(card) = cards.first().copied() else {
                return true;
            };
            let Some(top) = g.library_top(p) else {
                return true;
            };
            let base = MoveEv {
                obj: card,
                to: Zone::Library(p),
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(p),
                etb: EtbInfo::default(),
                source: ctx.source,
            };
            let to_ante = MoveEv {
                obj: top,
                to: Zone::Ante,
                ..base.clone()
            };
            // An exchange of zones happens only if both objects can move (CR 701.12a).
            if g.move_forbidden(&base) || g.move_forbidden(&to_ante) {
                return true;
            }
            g.move_objects(vec![to_ante, base]);
            true
        }
        _ => false,
    }
}
