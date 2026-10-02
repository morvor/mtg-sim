//! "Create an X/X black Demon creature token with flying." where the text defines X — in
//! a spell's mana cost, a "where X is ..." clause, or an earlier sentence ("You may pay X
//! life, where X is ... If you do, create an X/X ...") (CR 107.3, 107.3c): the token's
//! power and toughness are that number, determined as it's created
//! ([`Effect::CreateTokenWithPT`]).
//!
//! Also "where X is the difference between those players' life totals" after "Two target
//! players exchange life totals." (Profane Transfusion).

use super::r107_numbers::where_x_is_value;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// Marks, in [`Builder::named`], that the text being parsed has defined X (CR 107.3c):
/// a "where X is ..." clause being parsed, or an earlier sentence that set it. (The
/// leading control character keeps it from ever matching words of the text.)
pub(crate) const X_DEFINED: &str = "\u{1}x is defined";

/// Whether "X" in the text being parsed has a value: an instant's or sorcery's X (its
/// mana cost's, CR 107.3a) or one the text defines.
pub(crate) fn x_defined(b: &Builder) -> bool {
    (b.ctx.is_spell() && !b.in_trigger)
        || b.named.iter().any(|(n, _)| n == X_DEFINED)
        || super::value_grammar::x_defined()
}

/// "create an X/X [token description]".
fn create_x_x(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("you ").unwrap_or(l);
    let rest = l
        .strip_prefix("create an x/x ")
        .or_else(|| l.strip_prefix("create a x/x "))?;
    if !x_defined(b) || rest.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    let saved = b.targets.len();
    match parse_clause(&format!("create a 0/0 {rest}"), b) {
        Some(Effect::CreateToken {
            spec,
            count,
            controller,
            tapped,
            attacking,
        }) if b.targets.len() == saved => Some(Effect::CreateTokenWithPT {
            spec,
            power: Value::X,
            toughness: Value::X,
            count,
            controller,
            tapped,
            attacking,
        }),
        _ => {
            b.targets.truncate(saved);
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "tokens: create an X/X token", priority: 60, parse: create_x_x } }

/// "[clause], where X is the difference between those players' life totals": the two
/// players are the last two player targets.
fn where_x_is_life_difference(l: &str, b: &mut Builder) -> Option<Effect> {
    let clause = end(l)
        .strip_suffix(", where x is the difference between those players' life totals")?;
    let players: Vec<u8> = b
        .targets
        .iter()
        .enumerate()
        .filter(|(_, t)| matches!(t.what, TargetKind::Player(_)))
        .map(|(i, _)| i as u8)
        .collect();
    let [.., i, j] = players[..] else {
        return None;
    };
    let life = |slot| Box::new(Value::LifeTotal(PlayerRef::Target(slot)));
    let v = Value::Max(
        Box::new(Value::Diff(life(i), life(j))),
        Box::new(Value::Diff(life(j), life(i))),
    );
    let it = b.it.clone();
    where_x_is_value(clause, v, b, it)
}

inventory::submit! { EffectPattern { name: "tokens: where X is the difference between those players' life totals", priority: 60, parse: where_x_is_life_difference } }
