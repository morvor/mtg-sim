//! Scry and surveil (CR 701.22, 701.25):
//!
//! * "each player scries 1", "each player may scry 1", "each player surveils 2", "each
//!   player may surveil 2": the players look at the same time, decide in APNAP order, and
//!   the cards move at the same time (CR 701.22c).
//! * "You may look at an additional two cards each time you surveil." (CR 701.25b).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::scry_rules::{OPTED, OPT_IN, SURVEIL_EXTRA};
use smol_str::SmolStr;

fn each_player_scries(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("each player ") {
        (PlayerRef::EachPlayer, r)
    } else if let Some(r) = l.strip_prefix("each opponent ") {
        (PlayerRef::EachOpponent, r)
    } else {
        return None;
    };
    let (may, r) = match r.strip_prefix("may ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    let (scry, r) = if let Some(r) = r
        .strip_prefix(if may { "scry " } else { "scries " })
    {
        (true, r)
    } else if let Some(r) = r.strip_prefix(if may { "surveil " } else { "surveils " }) {
        (false, r)
    } else {
        return None;
    };
    let (n, t) = parse_number(r)?;
    if !end(t).is_empty() {
        return None;
    }
    let look = |who: PlayerRef| {
        if scry {
            Effect::Scry { who, n: n.clone() }
        } else {
            Effect::Surveil { who, n: n.clone() }
        }
    };
    if !may {
        return Some(look(who));
    }
    // Each player chooses in APNAP order whether to take part (CR 101.4); then those who
    // do scry at the same time.
    Some(Effect::seq(vec![
        Effect::Store {
            var: OPTED,
            sel: Sel::None,
        },
        Effect::ForEachPlayer {
            who,
            effect: Box::new(Effect::May {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::Custom(OPT_IN.into())),
            }),
        },
        look(PlayerRef::Var(OPTED)),
    ]))
}

inventory::submit! { EffectPattern { name: "a701 each player scries", priority: 100, parse: each_player_scries } }

/// "you may look at an additional two cards each time you surveil".
fn surveil_extra(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may look at an additional ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    if !matches!(
        r.trim(),
        "cards each time you surveil" | "card each time you surveil"
    ) {
        return None;
    }
    let name = SmolStr::new(format!("{SURVEIL_EXTRA}{n}"));
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "a701 surveil additional cards", priority: 100, parse: surveil_extra } }
