//! Spells cast while an effect resolves (CR 608.2g; timing permissions based on card type
//! are ignored), by the player the text names:
//!
//! * "Otherwise, the player casts it without paying its mana cost if able." (Omen Machine,
//!   Wild Evocation): that player must cast it if they can (CR 101.3; Omen Machine ruling:
//!   if it can't be cast, it stays where it is);
//! * "Otherwise, they may cast it without paying its mana cost." (Audacious Swap), "that
//!   opponent may cast the revealed card without paying its mana cost" (Swindler's
//!   Scheme): that player may;
//! * "Whenever you discard a nonland card, you may cast it from your graveyard." (Oskar,
//!   Rubbish Reclaimer): the card the trigger is about, from the graveyard.

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use super::oracle_hardening_referents::{is_no_player_referent, is_no_referent};

fn p_player_casts_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (subject, r) = [
        "the player ",
        "that player ",
        "they ",
        "that opponent ",
    ]
    .iter()
    .find_map(|s| l.strip_prefix(s).map(|r| (*s, r)))?;
    let _ = subject;
    let (optional, r) = if let Some(r) = r.strip_prefix("may cast ") {
        (true, r)
    } else {
        (false, r.strip_prefix("casts ")?)
    };
    let r = [
        "it",
        "that card",
        "the revealed card",
        "the exiled card",
    ]
    .iter()
    .find_map(|w| r.strip_prefix(w))?;
    let rest = r.strip_prefix(" without paying its mana cost")?;
    match (optional, rest) {
        (true, "") | (false, " if able") => {}
        _ => return None,
    }
    if is_no_referent(&b.it) || is_no_player_referent(&b.it_player) {
        return None;
    }
    if matches!(b.it_player, PlayerRef::You) {
        return None;
    }
    Some(Effect::CastCard {
        who: b.it_player.clone(),
        what: b.it.clone(),
        free: true,
        optional,
    })
}

inventory::submit! { EffectPattern { name: "resolution casts: that player casts it without paying its mana cost [if able]", priority: 110, parse: p_player_casts_it } }

/// "If it's a land card, the player puts it onto the battlefield. Otherwise, the player
/// casts it without paying its mana cost if able.": the alternative to the condition. In
/// an ability whose instructions that player performs (`Effect::AsPlayer`), "you" is that
/// player ("Otherwise, cast it without paying its mana cost if able.").
fn f_otherwise_player_casts_it(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l.trim()).strip_prefix("otherwise, ") else {
        return false;
    };
    // The last instruction, inside the instructions a player performs.
    fn last_if(e: &mut Effect, as_player: bool) -> Option<(&mut Box<Effect>, bool)> {
        match e {
            Effect::AsPlayer { effect, .. } => last_if(effect, true),
            Effect::Seq(v) => v.last_mut().and_then(|x| last_if(x, as_player)),
            Effect::If { otherwise, .. } if matches!(**otherwise, Effect::Noop) => {
                Some((otherwise, as_player))
            }
            _ => None,
        }
    }
    let Some((otherwise, as_player)) = last_if(prev, false) else {
        return false;
    };
    // The player: "the player", "they" (the player the ability names; "you" when that
    // player performs the instructions), or no subject (you).
    let (subject, r) = match ["the player ", "that player ", "they ", "that opponent "]
        .iter()
        .find_map(|p| r.strip_prefix(p))
    {
        Some(x) => (true, x),
        None => (false, r),
    };
    let (optional, rest) = if let Some(x) = r
        .strip_prefix("may cast it without paying its mana cost")
        .or_else(|| r.strip_prefix("you may cast it without paying its mana cost"))
    {
        (true, x)
    } else if let Some(x) = r
        .strip_prefix("casts it without paying its mana cost")
        .filter(|_| subject)
        .or_else(|| {
            r.strip_prefix("cast it without paying its mana cost")
                .filter(|_| !subject)
        })
    {
        (false, x)
    } else {
        return false;
    };
    match (optional, rest) {
        (true, "") | (false, " if able") => {}
        _ => return false,
    }
    let who = if subject && !as_player {
        if is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
            return false;
        }
        b.it_player.clone()
    } else {
        PlayerRef::You
    };
    if is_no_referent(&b.it) {
        return false;
    }
    let e = Effect::CastCard {
        who,
        what: b.it.clone(),
        free: true,
        optional,
    };
    **otherwise = e;
    true
}

inventory::submit! { FollowupPattern { name: "resolution casts: otherwise, that player casts it", priority: 150, apply: f_otherwise_player_casts_it } }

/// "Whenever you discard a nonland card, you may cast it from your graveyard." (the card
/// in the graveyard, as the ability resolves).
fn p_cast_it_from_graveyard_now(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.in_trigger || matches!(b.it, Sel::This) || is_no_referent(&b.it) {
        return None;
    }
    let l = end(l.trim());
    if !matches!(
        l,
        "you may cast it from your graveyard" | "cast it from your graveyard"
    ) {
        return None;
    }
    Some(Effect::CastCard {
        who: PlayerRef::You,
        what: Sel::All(Filter::and(vec![
            Filter::In(Box::new(b.it.clone())),
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ])),
        free: false,
        optional: true,
    })
}

inventory::submit! { EffectPattern { name: "resolution casts: you may cast it from your graveyard (as the ability resolves)", priority: 121, parse: p_cast_it_from_graveyard_now } }
