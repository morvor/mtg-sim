//! Replacement grammar IV (cards3-costs-mana-replacements).
//!
//! - "The next time you would lose the game this turn, instead [effect]." (Stunning
//!   Reversal): a one-use replacement effect on the game-loss event (CR 104.3, 614.1a),
//!   for the rest of the turn.
//! - "the creature an opponent controls" after "target creature an opponent controls" in
//!   the same ability ("Target creature you control fights target creature an opponent
//!   controls. If the creature an opponent controls would die this turn, exile it
//!   instead."): that target, named again by its own words ("the opponent" for "an
//!   opponent", as The Last Agni Kai words it).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// "the [words of a target]" naming a target chosen earlier in the ability: the words
/// after "target" in its text, ending in who controls it (so "the creature" alone, which
/// usually means something else, isn't read this way). The target must be the only one
/// with those words.
pub(crate) fn target_named_again(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let r = s.strip_prefix("the ")?;
    let mut found: Option<(usize, usize)> = None;
    for (i, t) in b.targets.iter().enumerate() {
        let Some(words) = t.text.strip_prefix("target ") else {
            continue;
        };
        // A controller qualifier tells the targets apart ("creature you control",
        // "creature an opponent controls").
        if words.contains("target") || !(words.ends_with(" control") || words.ends_with(" controls")) {
            continue;
        }
        let alt = words.replace(" an opponent ", " the opponent ");
        for w in [words, alt.as_str()] {
            let Some(rest) = r.strip_prefix(w) else {
                continue;
            };
            if !(rest.is_empty() || rest.starts_with([' ', ',', '.', '\''])) {
                continue;
            }
            if found.is_some_and(|(j, _)| j != i) {
                return None;
            }
            found = Some((i, w.len()));
        }
    }
    let (i, len) = found?;
    let sel = Sel::Target(i as u8);
    b.it = sel.clone();
    Some((sel, r[len..].to_string()))
}

/// "the next time you would lose the game this turn, instead [effect]".
fn next_loss_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the next time you would lose the game this turn, instead ")?;
    let saved = b.targets.len();
    let e = parse_effect_text(r, b);
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::LoseGame(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(e?)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "replacements 4: the next time you would lose the game this turn, instead [effect]", priority: 70, parse: next_loss_instead } }
