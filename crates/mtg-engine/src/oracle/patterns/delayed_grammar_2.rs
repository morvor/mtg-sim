//! Delayed triggered abilities, more grammar (CR 603.7):
//!
//! * Pay-to-stop delayed triggers (CR 116.2c): "that player loses 2 life at the beginning
//!   of their next draw step unless they pay {2} before that step" (Glass Asp, Nafs Asp,
//!   Sabertooth Cobra, Quenchable Fire). The delayed triggered ability is created, and the
//!   player may pay as a special action, once, any time they have priority until it
//!   triggers; paying stops it ([`SpecialActionEffect::StopDelayedTrigger`]).

use super::EffectPattern;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::oracle_hardening_referents as refs;
use crate::oracle::phrases::end;

/// The delayed ability last in `e` (as the delayed grammar builds it).
fn ends_with_delayed(e: &Effect) -> bool {
    match e {
        Effect::DelayedTrigger { once: true, .. } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_with_delayed),
        _ => false,
    }
}

/// "[delayed instruction] unless they pay {N} before that step".
fn delayed_unless_pays_before(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, tail) = l.rsplit_once(" unless ")?;
    let (payer, rest) = [
        "they pay ",
        "that player pays ",
        "that player or that planeswalker's controller pays ",
    ]
    .into_iter()
    .find_map(|p| tail.strip_prefix(p).map(|r| (p, r)))?;
    let (cost_s, before) = rest.split_once(" before ")?;
    if !matches!(before, "that step" | "that draw step" | "that upkeep") {
        return None;
    }
    if !cost_s.starts_with('{') || cost_s.contains(' ') {
        return None;
    }
    let cost = Cost::mana(ManaCost::parse(&cost_s.to_uppercase())?);
    // "The player gets another poison counter": one more. "It deals an additional 3
    // damage to that player": the spell or ability's source deals 3 more.
    let mut head = head.replace(
        "the player gets another poison counter",
        "that player gets a poison counter",
    );
    if let Some(r) = head.strip_prefix("it deals an additional ") {
        head = format!("~ deals {r}");
    }
    let n0 = b.targets.len();
    let e = super::delayed_grammar::delayed_instruction(&head, b)?;
    if !ends_with_delayed(&e) || b.targets.len() != n0 {
        return None;
    }
    let who = if payer.starts_with("that player or that planeswalker's") {
        // The target the delayed instruction is about: a player, or a planeswalker whose
        // controller pays.
        let slot = b
            .targets
            .iter()
            .rposition(|t| t.text.contains("player or planeswalker"))? as u8;
        PlayerFilter::Or(vec![
            PlayerFilter::Ref(Box::new(PlayerRef::Target(slot))),
            PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::Target(slot))))),
        ])
    } else {
        // "they": the player the delayed instruction affects.
        if refs::is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
            return None;
        }
        PlayerFilter::Ref(Box::new(b.it_player.clone()))
    };
    Some(Effect::seq(vec![
        e,
        Effect::OfferSpecialAction {
            def: Box::new(SpecialActionDef {
                who,
                cost,
                action: SpecialActionEffect::StopDelayedTrigger,
                mana_timing: false,
            }),
            duration: Duration::Permanent,
            repeatable: false,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "delayed grammar 2: unless they pay before that step", priority: 980, parse: delayed_unless_pays_before } }
