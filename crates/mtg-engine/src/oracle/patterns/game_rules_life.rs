//! Life totals compared with and set to the starting life total (CR 119.1, 119.5): "your
//! life total becomes equal to your starting life total", "half your starting life total,
//! rounded up", "up to one target player's life total becomes half their starting life
//! total, rounded down", "if your life total is less than your starting life total", "as
//! long as your life total is at least 10 greater than your starting life total".

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_simple, player_ref, Builder};
use crate::oracle::phrases::*;

/// "becomes equal to [value]": the life total becomes that value.
fn becomes_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, value) = l.split_once(" life total becomes equal to ")?;
    parse_simple(&format!("{head} life total becomes {value}"), b)
}

inventory::submit! { EffectPattern { name: "game rules: life total becomes equal to [value]", priority: 50, parse: becomes_equal_to } }

/// "it becomes equal to your starting life total" after "if your life total is less than
/// your starting life total" (Resolute Archangel): "it" is your life total.
fn it_becomes_starting_life(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "it becomes equal to your starting life total").then_some(Effect::SetLife {
        who: PlayerRef::You,
        n: Value::StartingLife,
    })
}

inventory::submit! { EffectPattern { name: "game rules: it (your life total) becomes your starting life total", priority: 50, parse: it_becomes_starting_life } }

/// "[player]'s life total becomes half their starting life total, rounded down": the
/// player's own starting life total (CR 119.1).
fn half_their_starting_life(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, r) = l.split_once("'s life total becomes half their starting life total, rounded ")?;
    let up = match r {
        "up" => true,
        "down" => false,
        _ => return None,
    };
    let who = if subject == "up to one target player" {
        let mut spec = TargetSpec::player(PlayerFilter::Any, "up to one target player");
        spec.min = Value::c(0);
        PlayerRef::Target(b.add_target(spec, "up to one target player"))
    } else {
        let (who, rest) = player_ref(subject, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        who
    };
    Some(Effect::AsPlayer {
        who,
        effect: Box::new(Effect::SetLife {
            who: PlayerRef::You,
            n: Value::Div(Box::new(Value::StartingLife), 2, up),
        }),
    })
}

inventory::submit! { EffectPattern { name: "game rules: life total becomes half their starting life total", priority: 50, parse: half_their_starting_life } }

/// "your life total is less than your starting life total", "... is greater than or
/// equal to ...", "... is at least 10 greater than ...".
fn life_vs_starting(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("your life total is ")?
        .strip_suffix(" your starting life total")?;
    let life = Value::LifeTotal(PlayerRef::You);
    let start = Value::StartingLife;
    Some(match r {
        "less than" => Condition::Compare(life, Cmp::Lt, start),
        "less than or equal to" => Condition::Compare(life, Cmp::Le, start),
        "greater than" => Condition::Compare(life, Cmp::Gt, start),
        "greater than or equal to" | "at least" => Condition::Compare(life, Cmp::Ge, start),
        _ => {
            let r = r.strip_prefix("at least ")?;
            let (n, rest) = parse_number(r)?;
            if end(rest) != "greater than" {
                return None;
            }
            Condition::Compare(life, Cmp::Ge, Value::Sum(vec![start, n]))
        }
    })
}

inventory::submit! { ConditionPattern { name: "game rules: your life total compared with your starting life total", priority: 100, parse: life_vs_starting } }
