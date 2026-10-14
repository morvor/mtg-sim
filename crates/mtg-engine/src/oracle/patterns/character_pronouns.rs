//! A legendary character's pronoun as the subject of an instruction: "At the beginning of
//! your end step, if Kaito entered this turn, he phases out." (Kaito Shizuki) — "he" or
//! "she" is the card itself. Also a card named for several characters takes a plural
//! verb: "Tokka & Rahzar, Terrible Twos deal 3 damage to that player."

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::types::Supertype;

fn character_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = if let Some(r) = ["he ", "she "].iter().find_map(|p| l.strip_prefix(p)) {
        // Only a legendary card is a character with a pronoun of its own.
        if !b.ctx.type_line.supertypes.contains(Supertype::Legendary) {
            return None;
        }
        r
    } else {
        // "~ deal 3 damage": a plural name.
        let r = l.strip_prefix("~ ")?;
        let (verb, rest) = r.split_once(' ')?;
        if !matches!(verb, "deal" | "get" | "gain" | "phase" | "fight") {
            return None;
        }
        return parse_clause(&format!("~ {verb}s {rest}"), b);
    };
    parse_clause(&format!("~ {r}"), b)
}

inventory::submit! { EffectPattern { name: "a legendary character's pronoun as subject", priority: 950, parse: character_subject } }
