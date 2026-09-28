//! Text-changing effects (CR 612): "Change the text of target spell or permanent by
//! replacing all instances of one color word with another." (Sleight of Mind), "... one
//! basic land type with another" (Magical Hack), "... one color word with another or one
//! basic land type with another [until end of turn]" (Mind Bend, Crystal Spray), "... one
//! creature type with another. The new creature type can't be Wall." (Artificial
//! Evolution). The words are chosen as the effect resolves (see `text_change`).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_target};
use crate::text_change::TextWords;
use smol_str::SmolStr;

fn change_text(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("change the text of ")?;
    let (what, words) = r.split_once(" by replacing all instances of ")?;
    let (words, duration) = match words.strip_suffix(" until end of turn") {
        Some(w) => (w, Duration::EndOfTurn),
        None => (words, Duration::Permanent),
    };
    let words = match words {
        "one color word with another" => TextWords::Color,
        "one basic land type with another" => TextWords::BasicLandType,
        "one color word with another or one basic land type with another" => {
            TextWords::ColorOrBasicLandType
        }
        "one creature type with another" => TextWords::CreatureType,
        _ => return None,
    };
    let (spec, tail) = parse_target(what)?;
    if !end(tail).is_empty() {
        return None;
    }
    let slot = b.add_target(spec, what);
    Some(Effect::ChangeText {
        what: Sel::Target(slot),
        words,
        exclude: vec![],
        duration,
    })
}

inventory::submit! { EffectPattern { name: "r612 change the text of", priority: 100, parse: change_text } }

/// "The new creature type can't be Wall." after changing creature types.
fn new_word_cant_be(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = s.to_lowercase();
    let Some(word) = end(&l)
        .strip_prefix("the new creature type can't be ")
        .filter(|w| !w.contains(' '))
    else {
        return false;
    };
    let Effect::ChangeText {
        words: TextWords::CreatureType,
        exclude,
        ..
    } = prev
    else {
        return false;
    };
    let mut w = word.to_string();
    if let Some(first) = w.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    exclude.push(SmolStr::new(w));
    true
}

inventory::submit! { FollowupPattern { name: "r612 the new creature type can't be", priority: 100, apply: new_word_cant_be } }
