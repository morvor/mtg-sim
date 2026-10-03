//! "If you would draw a card, draw two cards instead." (Thought Reflection; "Max speed —
//! If you would draw a card, draw two cards instead." on Vnwxt, Verbose Host) and "If you
//! would draw a card while you have no cards in hand, draw two cards instead." (Phial of
//! Galadriel): a replacement effect for each card draw (CR 121.6, 614.1a). The draws it
//! produces aren't replaced by the same effect again (CR 614.5), but another such effect
//! applies to each of them (so two of them make a draw into four).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::{end, parse_card_count};
use crate::oracle::CompileContext;

fn draw_more_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if you would draw a card")?;
    let (condition, r) = match r.strip_prefix(" while you have no cards in hand") {
        Some(r) => (
            Some(Condition::Compare(
                Value::HandSize(PlayerRef::You),
                Cmp::Eq,
                Value::c(0),
            )),
            r,
        ),
        None => (None, r),
    };
    let r = r.strip_prefix(", ")?;
    let count = r
        .strip_prefix("instead draw ")
        .or_else(|| r.strip_prefix("draw ").and_then(|x| x.strip_suffix(" instead")))?;
    let (n, tail) = parse_card_count(count)?;
    if !end(tail).is_empty() {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::Draw(PlayerFilter::You),
        action: ReplacementAction::Instead(Box::new(Effect::Draw {
            who: PlayerRef::You,
            n,
        })),
        self_replacement: false,
        optional: false,
    }));
    s.condition = condition;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "if you would draw a card, draw two cards instead", priority: 100, parse: draw_more_instead } }
