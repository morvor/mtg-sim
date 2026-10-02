//! Values chosen as a spell or ability resolves (CR 608.2d), named inline with "of your
//! choice" and locked into the effects that use them (CR 608.2h):
//!
//! - "Creatures of the creature type of your choice get +0/+4 until end of turn.", "~
//!   becomes the creature type of your choice in addition to its other types until end of
//!   turn.", "Target land you control becomes the basic land type of your choice in
//!   addition to its other types.", "~ gets +1/+1 and becomes the color of your choice
//!   until end of turn.", "Until end of turn, ~ becomes the color of your choice and gains
//!   hexproof from that color.", "you gain protection from the color of your choice":
//!   the choice is made first, then the instruction with "the chosen [quality]";
//! - "[X] gains protection from colorless or from the color of your choice": colorless or
//!   one of the five colors (CR 105.2c, 702.16a);
//! - "[X] become(s) the chosen color / the chosen type / the chosen basic land type [in
//!   addition to its other types]" (CR 105.3, 205.1a, 305.7).

use super::EffectPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::{duration_suffix, keyword_mods, object_ref, parse_pt_mod, Builder};
use crate::oracle::phrases::*;
use crate::types::Color;

/// The "of your choice" qualities: the phrase, what is chosen, and what the sentence says
/// instead once it's chosen.
const QUALITIES: [(&str, &str); 4] = [
    ("the creature type of your choice", "the chosen type"),
    ("the color of your choice", "the chosen color"),
    ("the basic land type of your choice", "the chosen basic land type"),
    ("the card type of your choice", "the chosen card type"),
];

fn kind_of(phrase: &str) -> ChoiceKind {
    match phrase {
        "the creature type of your choice" => ChoiceKind::CreatureType,
        "the color of your choice" => ChoiceKind::Color,
        "the basic land type of your choice" => ChoiceKind::BasicLandType,
        _ => ChoiceKind::CardType,
    }
}

/// A sentence naming a quality "of your choice": the choice, then the sentence about "the
/// chosen [quality]".
fn of_your_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (phrase, chosen) = QUALITIES.iter().find(|(p, _)| l.contains(p))?;
    if l.matches(phrase).count() != 1 || l.contains(" or from the color of your choice") {
        return None;
    }
    let text = l.replacen(phrase, chosen, 1);
    let e = crate::oracle::effects::parse_sentence(&text, b)?;
    Some(Effect::seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: kind_of(phrase),
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "choice grammar: [quality] of your choice", priority: 40, parse: of_your_choice } }

/// "[X] gains protection from colorless or from the color of your choice until end of
/// turn": colorless or a color is chosen as it resolves.
fn protection_colorless_or_color(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (body, duration) = match l.strip_suffix(" until end of turn") {
        Some(x) => (x, Duration::EndOfTurn),
        None => return None,
    };
    let subj = body
        .strip_suffix(" gains protection from colorless or from the color of your choice")
        .or_else(|| body.strip_suffix(" gain protection from colorless or from the color of your choice"))?;
    let (what, rest) = object_ref(subj, b)?;
    if !end(&rest).is_empty() || matches!(what, Sel::None) {
        return None;
    }
    let mut options = vec!["colorless".to_string()];
    options.extend(Color::ALL.iter().map(|c| c.word().to_string()));
    let protection = |f: Filter| {
        let mut kw = Keyword::new(KeywordKind::Protection);
        kw.filter = Some(f);
        Effect::Modify {
            what: what.clone(),
            mods: vec![Modification::AddKeyword(kw)],
            duration: duration.clone(),
        }
    };
    Some(Effect::Seq(vec![
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::OneOf(options),
        },
        Effect::If {
            cond: Condition::ChosenWord("colorless".into()),
            then: Box::new(protection(Filter::Colorless)),
            otherwise: Box::new(protection(Filter::ChosenColor)),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choice grammar: protection from colorless or from the color of your choice", priority: 60, parse: protection_colorless_or_color } }

/// What "becomes [quality]" does: "the chosen color" (CR 105.3), "the chosen type" (a
/// creature type: it replaces the others unless "in addition", CR 205.1a), "the chosen
/// basic land type" (CR 305.7).
fn becomes_quality(q: &str) -> Option<Vec<Modification>> {
    let (q, in_addition) = match q
        .strip_suffix(" in addition to its other types")
        .or_else(|| q.strip_suffix(" in addition to their other types"))
    {
        Some(x) => (x, true),
        None => (q, false),
    };
    Some(match (q, in_addition) {
        ("the chosen color", false) => vec![Modification::SetChosenColor],
        ("the chosen type" | "that type" | "the chosen creature type", false) => vec![
            Modification::RemoveAllCreatureTypes,
            Modification::AddChosenType,
        ],
        ("the chosen type" | "that type" | "the chosen creature type", true) => {
            vec![Modification::AddChosenType]
        }
        ("the chosen basic land type", false) => vec![Modification::SetChosenBasicLandType],
        ("the chosen basic land type", true) => vec![Modification::AddChosenType],
        _ => return None,
    })
}

/// "[X] becomes [the chosen quality] [until end of turn]", "[X] gets +1/+1 and becomes the
/// chosen color until end of turn", "until end of turn, [X] becomes the chosen color and
/// gains hexproof from that color".
fn becomes_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (lead, l) = match l.strip_prefix("until end of turn, ") {
        Some(r) => (Some(Duration::EndOfTurn), r),
        None => (None, l),
    };
    let (dur, body) = duration_suffix(l);
    let duration = match (lead, dur) {
        (Some(d), Duration::Permanent) => d,
        (None, d) => d,
        _ => return None,
    };
    let saved = (b.targets.len(), b.it.clone());
    let (what, rest) = object_ref(body, b)?;
    let r = rest.trim_start();
    let mut mods = Vec::new();
    // "gets +1/+1 and becomes ..."
    let r = match r.strip_prefix("gets ").or_else(|| r.strip_prefix("get ")) {
        Some(x) => {
            let (p, t, tail) = parse_pt_mod(x)?;
            mods.push(Modification::ModifyPT(p, t));
            tail.trim_start().strip_prefix("and ")?
        }
        None => r,
    };
    let r = r
        .strip_prefix("becomes ")
        .or_else(|| r.strip_prefix("become "))?;
    // "... and gains hexproof from that color"
    let (q, gains) = match r.split_once(" and gains ") {
        Some((q, g)) => (q, Some(g.replace("that color", "the chosen color"))),
        None => (r, None),
    };
    let Some(m) = becomes_quality(q) else {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    };
    mods.extend(m);
    if let Some(g) = gains {
        mods.extend(keyword_mods(&g)?);
    }
    // A spell's color can change as well as a permanent's; its types only matter as a
    // permanent's (the subject must be an object).
    if matches!(what, Sel::None) {
        return None;
    }
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "choice grammar: becomes the chosen color/type", priority: 940, parse: becomes_chosen } }
