//! Referents in the "instead" sentences of the Courts (and others like them): "each of
//! those players mills ten cards" (the players an earlier instruction's target slot
//! named), "return that permanent card to the battlefield" (the card targeted), and
//! "Draw another card if [condition]." (Imoen, Mystic Trickster).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::end;

/// "each of those players mills ten cards": each player of the last target slot that
/// can name several players performs the instruction.
fn each_of_those_players(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each of those players ")?;
    let (verb, rest) = r.split_once(' ').unwrap_or((r, ""));
    let base = verb
        .strip_suffix("es")
        .filter(|v| v.ends_with("ss") || v.ends_with("sh"))
        .or_else(|| verb.strip_suffix('s'))?;
    let slot = b.targets.iter().rposition(|t| {
        matches!(t.what, TargetKind::Player(_)) && !matches!(t.max, Value::Const(1))
    })?;
    let effect = parse_simple(&format!("you {base} {rest}").trim().to_string(), b)?;
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Target(slot as u8),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(effect),
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: each of those players [does something]", priority: 100, parse: each_of_those_players } }

/// "return that permanent card to the battlefield instead": the card the ability
/// targeted ("target permanent card"), which "it" names.
fn that_kind_card(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find("that permanent card")?;
    if !matches!(b.it, Sel::Target(_)) {
        return None;
    }
    let reworded = format!("{}it{}", &l[..i], &l[i + "that permanent card".len()..]);
    parse_simple(&reworded, b)
}

inventory::submit! { EffectPattern { name: "game rules: that permanent card (the target)", priority: 100, parse: that_kind_card } }

/// "Draw another card if you've completed a dungeon.": a conditional instruction.
fn draw_another_if(l: &str, b: &mut Builder) -> Option<Effect> {
    let c = end(l).strip_prefix("draw another card if ")?;
    let cond = crate::oracle::statics::parse_condition(c, b.ctx)?;
    Some(Effect::If {
        cond,
        then: Box::new(Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "game rules: draw another card if [condition]", priority: 100, parse: draw_another_if } }
