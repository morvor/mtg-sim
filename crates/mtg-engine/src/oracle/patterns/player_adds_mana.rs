//! "Target player adds [mana]" (Jetfire, Ingenious Scientist: "Target player adds that
//! much {C}."; Radiant Lotus: "Target player adds three mana of the chosen color for each
//! artifact sacrificed this way."): mana added to another player's mana pool (CR 106.4).
//! An ability with a target isn't a mana ability (CR 605.1a), so it uses the stack.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_sentence, player_ref, Builder};

fn player_adds(l: &str, b: &mut Builder) -> Option<Effect> {
    if !l.starts_with("target player adds ") {
        return None;
    }
    let before = b.targets.len();
    let parsed = (|| {
        let (who, rest) = player_ref(l, b)?;
        let r = rest.trim().strip_prefix("adds ")?;
        let mut e = parse_sentence(&format!("add {r}"), b)?;
        // The mana instruction alone (with a choice of color before it).
        let add = match &mut e {
            Effect::AddMana { who: w, .. } => w,
            Effect::Seq(v) => match v.as_mut_slice() {
                [Effect::Choose { .. }, Effect::AddMana { who: w, .. }] => w,
                _ => return None,
            },
            _ => return None,
        };
        if !matches!(add, PlayerRef::You) {
            return None;
        }
        *add = who;
        Some(e)
    })();
    if parsed.is_none() {
        b.targets.truncate(before);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "target player adds [mana]", priority: 50, parse: player_adds } }

/// "add three mana of the chosen color for each artifact sacrificed this way": that much
/// mana of the color chosen earlier in the effect.
fn chosen_color_for_each(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("add ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let fe = r.strip_prefix("mana of the chosen color for each ")?;
    let each = super::statics::parse_for_each(fe, None)?;
    let amount = match n.as_const() {
        Some(1) => each,
        _ => Value::Mul(Box::new(n), Box::new(each)),
    };
    Some(Effect::AddMana {
        who: PlayerRef::You,
        mana: ManaProduction::ChosenColor(amount),
        restriction: None,
    })
}

inventory::submit! { EffectPattern { name: "add N mana of the chosen color for each", priority: 50, parse: chosen_color_for_each } }
