//! "Each opponent may [effect]. For each opponent who does, [effect]." — the tempting
//! offer ability word (Tempt with Glory, Tempt with Vengeance, ...) and the like (Tempting
//! Contract).
//!
//! The opponents choose in APNAP order whether to do it, each knowing the choices made
//! before theirs (CR 101.4, 101.4b); then each opponent who accepted performs the effect
//! (as the player performing it, "you" in it is that opponent), and after that "for each
//! opponent who does" performs the next effect once per opponent who accepted.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;
use crate::scry_rules::{OPTED, OPT_IN};

/// The opponents who accepted the most recent "each opponent may [effect]".
pub const ACCEPTED: Var = vars::USER + 1911;

/// Marks, in [`Builder::named`], that an earlier sentence of the text was "each opponent
/// may [effect]": the later "for each opponent who does" and "each player who searched a
/// library this way" refer to it, and mean nothing without it. (The leading control
/// character keeps it from ever matching words of the text.)
const ACCEPTED_NAME: &str = "\u{1}the opponents who accepted";

/// Marks that "for each opponent who searches a library this way, search your library …"
/// was parsed: you search too (Tempt with Discovery).
const YOU_SEARCH_NAME: &str = "\u{1}you searched a library";

fn after_each_opponent_may(b: &Builder) -> bool {
    b.named.iter().any(|(p, _)| p == ACCEPTED_NAME)
}

/// The effect an opponent performs, worded for that opponent as "you": "put a +1/+1
/// counter on each creature they control" is "put a +1/+1 counter on each creature you
/// control" performed by the opponent.
fn as_performer(text: &str) -> String {
    let mut t = format!(" {text} ");
    for (from, to) in [
        (" they control ", " you control "),
        (" they own ", " you own "),
        (" their ", " your "),
    ] {
        t = t.replace(from, to);
    }
    t.trim().to_string()
}

/// "each opponent may [effect]" (also after "then").
fn each_opponent_may(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("then ").unwrap_or(l);
    let r = r.strip_prefix("each opponent may ")?;
    let text = as_performer(r);
    let saved_targets = b.targets.len();
    let Some(effect) = parse_clause(&text, b) else {
        b.targets.truncate(saved_targets);
        return None;
    };
    b.named
        .push((ACCEPTED_NAME.to_string(), Sel::Var(ACCEPTED)));
    Some(Effect::seq(vec![
        // CR 101.4: they decide in APNAP order...
        Effect::Store {
            var: OPTED,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::EachOpponent,
            effect: Box::new(Effect::May {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::Custom(OPT_IN.into())),
            }),
        },
        Effect::Store {
            var: ACCEPTED,
            sel: Sel::Var(OPTED),
        },
        // ...then those who accepted do it.
        Effect::ForEachPlayer {
            who: PlayerRef::Var(ACCEPTED),
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(effect),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each opponent may [effect]", priority: 150, parse: each_opponent_may } }

/// "for each opponent who does, [effect]" (after "each opponent may [effect]"); "for each
/// opponent who searches a library this way, [effect]" (Tempt with Discovery).
fn for_each_opponent_who_does(l: &str, b: &mut Builder) -> Option<Effect> {
    if !after_each_opponent_may(b) {
        return None;
    }
    let l = end(l);
    let (r, searches) = if let Some(r) = l.strip_prefix("for each opponent who does, ") {
        (r, false)
    } else {
        (
            l.strip_prefix("for each opponent who searches a library this way, ")?,
            true,
        )
    };
    let saved_targets = b.targets.len();
    let Some(effect) = parse_clause(r, b) else {
        b.targets.truncate(saved_targets);
        return None;
    };
    if searches {
        if !matches!(&effect, Effect::Search { who: PlayerRef::You, .. }) {
            b.targets.truncate(saved_targets);
            return None;
        }
        b.named
            .push((YOU_SEARCH_NAME.to_string(), Sel::Var(ACCEPTED)));
    }
    Some(Effect::Repeat {
        times: Value::CountSel(Box::new(Sel::Var(ACCEPTED))),
        effect: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "for each opponent who does, [effect]", priority: 150, parse: for_each_opponent_who_does } }

/// "then each player who searched a library this way shuffles" (Tempt with Discovery):
/// you, and each opponent who accepted to search. Only after "for each opponent who
/// searches a library this way, search your library …", which makes "you" one of the
/// players who searched.
fn each_player_who_searched_shuffles(l: &str, b: &mut Builder) -> Option<Effect> {
    if !b.named.iter().any(|(p, _)| p == YOU_SEARCH_NAME) {
        return None;
    }
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    if l != "each player who searched a library this way shuffles" {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Shuffle {
            who: PlayerRef::You,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Var(ACCEPTED),
            effect: Box::new(Effect::Shuffle {
                who: PlayerRef::Iterated,
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "each player who searched a library this way shuffles", priority: 150, parse: each_player_who_searched_shuffles } }

#[cfg(test)]
mod tests {
    use crate::card::Layout;
    use crate::types::TypeLine;
    use crate::oracle::{compile, CompileContext};

    /// Whether a sorcery with oracle text `text` compiles completely.
    fn compiles(text: &str) -> bool {
        let tl = TypeLine::parse("Sorcery");
        let ctx = CompileContext {
            card_name: "Test Offer",
            full_name: "Test Offer",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        compile(text, &ctx).unsupported.is_empty()
    }

    #[test]
    fn the_follow_ups_need_an_earlier_each_opponent_may() {
        assert!(compiles(
            "Draw a card. Each opponent may draw a card. For each opponent who does, you gain 2 life."
        ));
        // Without "each opponent may", no opponent "does" anything.
        assert!(!compiles("Draw a card. For each opponent who does, you gain 2 life."));
        assert!(!compiles(
            "Each player may draw a card. For each opponent who does, you gain 2 life."
        ));
        // "Each player who searched a library this way" includes you only after you
        // searched for each opponent who did.
        assert!(compiles(
            "Each opponent may search their library for a land card and put it onto the battlefield. For each opponent who searches a library this way, search your library for a land card and put it onto the battlefield. Then each player who searched a library this way shuffles."
        ));
        assert!(!compiles(
            "Each opponent may search their library for a land card and put it onto the battlefield. Then each player who searched a library this way shuffles."
        ));
    }
}
