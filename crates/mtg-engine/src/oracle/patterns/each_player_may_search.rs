//! "Each player may search their library for up to two basic land cards, put them onto
//! the battlefield, then shuffle." (Veteran Explorer), "Each player may search their
//! library for a card and put that card into their hand. Then each player who searched
//! their library this way shuffles." (Noble Benefactor).
//!
//! The players decide in APNAP order whether to search, each knowing the earlier choices
//! (CR 101.4, 101.4b); a player who can't search libraries can't choose to. Those who do
//! search at the same time, choosing their cards in APNAP order, then the found cards
//! move and those players shuffle, in APNAP order (CR 701.23i).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::scry_rules::{OPTED, OPT_IN};
use crate::search_rules::ITERATED_CAN_SEARCH;

/// Marks, in [`Builder::named`], that "each player may search their library ..." was
/// parsed: a later "each player who searched their library this way" refers to it. (The
/// leading control character keeps it from ever matching words of the text.)
const SEARCHED_NAME: &str = "\u{1}the players who chose to search";

fn each_player_may_search(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player may search their library for ")?;
    let saved = b.targets.len();
    let parsed = parse_clause(&format!("each player searches their library for {r}"), b);
    let Some(Effect::Search {
        who: PlayerRef::EachPlayer,
        whose,
        filter,
        count,
        to,
        reveal,
        shuffle,
    }) = parsed
    else {
        b.targets.truncate(saved);
        return None;
    };
    b.named
        .push((SEARCHED_NAME.to_string(), Sel::Var(OPTED)));
    Some(Effect::seq(vec![
        Effect::Store {
            var: OPTED,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::If {
                cond: Condition::Custom(ITERATED_CAN_SEARCH.into()),
                then: Box::new(Effect::May {
                    who: PlayerRef::Iterated,
                    effect: Box::new(Effect::Custom(OPT_IN.into())),
                }),
                otherwise: Box::new(Effect::Noop),
            }),
        },
        Effect::Search {
            who: PlayerRef::Var(OPTED),
            whose,
            filter,
            count,
            to,
            reveal,
            shuffle,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each player may search their library", priority: 150, parse: each_player_may_search } }

/// "Then each player who searched their library this way shuffles."
fn each_player_who_searched_shuffles(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.named.iter().any(|(p, _)| p == SEARCHED_NAME) {
        return None;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player who searched their library this way shuffles" {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Var(OPTED),
        effect: Box::new(Effect::Shuffle {
            who: PlayerRef::Iterated,
        }),
    })
}

inventory::submit! { EffectPattern { name: "each player who searched their library this way shuffles", priority: 150, parse: each_player_who_searched_shuffles } }
