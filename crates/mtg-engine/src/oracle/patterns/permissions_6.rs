//! Permissions grammar VI (cards6-history-permissions): permissions over the top card of
//! another player's library, given by a resolved effect for the rest of the turn (CR
//! 611.2a).
//!
//! "At the beginning of your upkeep, choose target opponent. Until end of turn, that player
//! can't cast spells, you may look at the top card of their library any time, you may play
//! the top card of their library, and you may spend mana as though it were mana of any
//! color to cast spells this way." (Xanathar, Guild Kingpin)
//!
//! * "you may look at the top card of [their] library any time": the player may look at
//!   that card whenever they want (CR 401.5), except that a new top card can't be looked
//!   at while a spell is being cast, an ability activated or a land played
//!   ([`PlayerModification::LookAtTopCardOf`], `zones::update_revealed_tops`).
//! * "you may play the top card of [their] library", "you may cast spells from the top of
//!   [their] library": the permission over your own library's top card ("You may play the
//!   top card of your library.", The Lunar Whale) for that player's library
//!   (`PlayTerms::any_owner` with the owner named). The usual timing and land-play rules
//!   apply (CR 305.1, 305.2, 601.3); a land played this way is your land play, and the
//!   card is still owned by that player (CR 108.3).
//! * "you may spend mana as though it were mana of any color to cast spells this way":
//!   for spells cast with the permissions of the same list (CR 609.4b).
//!
//! The player is the one the effect means as it begins (CR 608.2h): "their" is the
//! player the text named before ("that player", a target), and "target opponent's"
//! makes them a target of the ability.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;

/// Whose library a phrase names: a player, or each opponent.
#[derive(Clone)]
enum Owner {
    Player(PlayerRef),
    EachOpponent,
}

/// "their library", "that player's library", "target opponent's library", "each
/// opponent's library" (not "your library", which the permission grammar reads).
fn library_owner(s: &str, b: &mut Builder) -> Option<Owner> {
    let whose = s.strip_suffix(" library")?;
    match whose {
        "their" | "that player's" | "his or her" => {
            let p = b.it_player.clone();
            if matches!(p, PlayerRef::You)
                || super::oracle_hardening_referents::is_no_player_referent(&p)
                || !matches!(
                    p,
                    PlayerRef::Target(_)
                        | PlayerRef::Var(_)
                        | PlayerRef::TriggerPlayer
                        | PlayerRef::ChosenOpponent
                        | PlayerRef::DefendingPlayer
                )
            {
                return None;
            }
            Some(Owner::Player(p))
        }
        "each opponent's" => Some(Owner::EachOpponent),
        w => {
            let t = w.strip_suffix("'s")?;
            if !t.starts_with("target ") {
                return None;
            }
            let saved = b.targets.len();
            let (p, rest) = player_ref(t, b)?;
            if !rest.trim().is_empty() || !matches!(p, PlayerRef::Target(_)) {
                b.targets.truncate(saved);
                return None;
            }
            Some(Owner::Player(p))
        }
    }
}

/// "you may look at the top card of [their] library any time".
fn look_part(part: &str, b: &mut Builder) -> Option<PlayerModification> {
    let r = part
        .strip_prefix("you may look at the top card of ")?
        .strip_suffix(" any time")?;
    match library_owner(r, b)? {
        Owner::Player(p) => Some(PlayerModification::LookAtTopCardOf(p)),
        // Several players' libraries: not read.
        Owner::EachOpponent => None,
    }
}

/// "you may play the top card of [their] library", "you may cast spells from the top of
/// [their] library", "you may play lands and cast spells from the top of [their]
/// library".
fn play_part(part: &str, b: &mut Builder) -> Option<PlayPermission> {
    let r = part.strip_prefix("you may ")?;
    let (lands, spells, lib) = if let Some(x) = r.strip_prefix("play the top card of ") {
        (true, true, x)
    } else if let Some(x) = r.strip_prefix("play lands and cast spells from the top of ") {
        (true, true, x)
    } else if let Some(x) = r.strip_prefix("cast spells from the top of ") {
        (false, true, x)
    } else if let Some(x) = r.strip_prefix("cast the top card of ") {
        (false, true, x)
    } else {
        return None;
    };
    let owned = match library_owner(lib, b)? {
        Owner::Player(p) => Filter::OwnedByPlayer(Box::new(p)),
        Owner::EachOpponent => Filter::OwnedBy(PlayerRel::Opponent),
    };
    Some(PlayPermission {
        who: PlayerRel::You,
        zone: ZoneKind::Library,
        top_only: true,
        what: Filter::and(vec![Filter::Card, owned]),
        lands,
        spells,
        cost: None,
        flash: false,
        terms: PlayTerms {
            any_owner: true,
            ..Default::default()
        },
    })
}

/// Splits "a, b, and c" / "a and b" / "a, b" into its parts.
fn list_parts(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (head, last) = match s.rsplit_once(", and ") {
        Some((h, l)) => (h, Some(l)),
        None => (s, None),
    };
    for p in head.split(", ") {
        out.push(p.to_string());
    }
    if let Some(l) = last {
        out.push(l.to_string());
    }
    // "you may look at ... any time and you may play ...".
    let mut split = Vec::new();
    for p in out {
        let mut pieces = p.split(" and you may ");
        if let Some(first) = pieces.next() {
            split.push(first.to_string());
        }
        split.extend(pieces.map(|x| format!("you may {x}")));
    }
    split
}

fn until_eot_library_permissions(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l.trim()).strip_prefix("until end of turn, ")?;
    if !r.contains(" library") || r.contains("your library") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let parsed = (|| {
        let grant = |effect| Effect::AddPlayerEffect {
            who: PlayerRef::You,
            effect,
            duration: Duration::EndOfTurn,
        };
        let mut out: Vec<Effect> = Vec::new();
        let mut plays: Vec<usize> = Vec::new();
        let mut library = false;
        for part in list_parts(r) {
            let part = part.trim();
            if let Some(m) = look_part(part, b) {
                library = true;
                out.push(grant(m));
            } else if let Some(pp) = play_part(part, b) {
                library = true;
                plays.push(out.len());
                out.push(grant(PlayerModification::PlayPermission(pp)));
            } else if matches!(
                part,
                "you may spend mana as though it were mana of any color to cast spells this way"
                    | "you may spend mana as though it were mana of any color to cast those spells"
            ) {
                // The rider of the permissions before it in the list.
                if plays.is_empty() {
                    return None;
                }
                for &i in &plays {
                    if let Effect::AddPlayerEffect {
                        effect: PlayerModification::PlayPermission(pp),
                        ..
                    } = &mut out[i]
                    {
                        if pp.spells {
                            pp.terms.spend_as_any_color = true;
                        }
                    }
                }
            } else {
                // "that player can't cast spells": an instruction of its own, for the same
                // duration.
                if part.contains(" library") {
                    return None;
                }
                out.push(parse_clause(&format!("until end of turn, {part}"), b)?);
            }
        }
        library.then(|| Effect::seq(out))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    parsed
}

inventory::submit! { EffectPattern { name: "permissions 6: until end of turn, you may look at / play the top card of their library", priority: 100, parse: until_eot_library_permissions } }

#[cfg(test)]
mod tests {
    use super::list_parts;

    #[test]
    fn lists() {
        assert_eq!(
            list_parts("a can't b, you may c, you may d, and you may e"),
            vec!["a can't b", "you may c", "you may d", "you may e"]
        );
        assert_eq!(
            list_parts("you may look at x any time and you may play y"),
            vec!["you may look at x any time", "you may play y"]
        );
    }
}
