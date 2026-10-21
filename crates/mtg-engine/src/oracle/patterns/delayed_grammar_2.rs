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

// ---------------------------------------------------------------------------------------
// "Until your next turn, whenever either of those creatures ...", "... this combat"
// ---------------------------------------------------------------------------------------

/// How long a delayed triggered ability with a stated duration lasts (CR 603.7b).
#[derive(Clone, Copy, PartialEq)]
enum Lasts {
    ThisTurn,
    UntilYourNextTurn,
    ThisCombat,
}

/// Rewraps the duration of the delayed triggered abilities `e` creates ("this turn", as
/// the grammar read them) as `lasts`; false if there's none.
fn rewrap(e: &mut Effect, lasts: Lasts) -> bool {
    match e {
        Effect::DelayedTrigger {
            trigger,
            once: false,
            ..
        } => {
            let TriggerCond::ThisTurn(inner) = std::mem::replace(trigger, TriggerCond::PlayerLoses)
            else {
                return false;
            };
            *trigger = match lasts {
                Lasts::ThisTurn => TriggerCond::ThisTurn(inner),
                Lasts::UntilYourNextTurn => TriggerCond::UntilYourNextTurn(inner),
                Lasts::ThisCombat => TriggerCond::ThisCombat(inner),
            };
            true
        }
        Effect::Seq(v) => {
            let mut any = false;
            for x in v.iter_mut() {
                if matches!(x, Effect::DelayedTrigger { .. } | Effect::Seq(_)) {
                    if !rewrap(x, lasts) {
                        return false;
                    }
                    any = true;
                }
            }
            any
        }
        _ => false,
    }
}

/// "he" (a character's card) is ~.
fn normalize_subject(cond: &str) -> String {
    let mut c = cond.to_string();
    if let Some(r) = c.strip_prefix("he ").or_else(|| c.strip_prefix("she ")) {
        c = format!("~ {r}");
    }
    c
}

/// "Until end of turn, whenever that creature deals damage, ...", "Until your next turn,
/// whenever either of those creatures deals combat damage, you draw a card.", "Whenever
/// that creature deals combat damage to a player this combat, ...": the referent and
/// general delayed trigger grammar, with the stated duration.
pub(crate) fn delayed_with_duration(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (lasts, r) = if let Some(r) = l.strip_prefix("until end of turn, whenever ") {
        (Lasts::ThisTurn, r)
    } else if let Some(r) = l.strip_prefix("until your next turn, whenever ") {
        (Lasts::UntilYourNextTurn, r)
    } else {
        let r = l.strip_prefix("whenever ")?;
        if !r.contains(" this combat, ") {
            return None;
        }
        (Lasts::ThisCombat, r)
    };
    let (cond, eff) = match lasts {
        Lasts::ThisCombat => r.split_once(" this combat, ")?,
        _ => r.split_once(", ")?,
    };
    if cond.contains(" this turn") || cond.contains(" this combat") {
        return None;
    }
    let cond = normalize_subject(cond);
    let mut eff = eff.to_string();
    if cond.starts_with("~ ") {
        eff = eff.replacen("he deals ", "~ deals ", 1).replacen("she deals ", "~ deals ", 1);
    }
    let text = format!("whenever {cond} this turn, {eff}");
    let n0 = b.targets.len();
    // With ~ as its subject ("whenever he deals combat damage to an opponent, he deals
    // that much damage to each other opponent"), the trigger is about the delayed
    // ability's source (CR 603.7d) and its effect about the delayed ability's own event:
    // the general grammar's reading.
    let general_first = cond.starts_with("~ ");
    let first = if general_first {
        super::triggers_delayed::this_turn_trigger(&text, b)
    } else {
        super::delayed_grammar::referent_trigger(&text, b)
    };
    let mut e = match first {
        Some(e) => e,
        None => {
            b.targets.truncate(n0);
            if general_first {
                super::delayed_grammar::referent_trigger(&text, b)?
            } else {
                super::triggers_delayed::this_turn_trigger(&text, b)?
            }
        }
    };
    if !rewrap(&mut e, lasts) {
        b.targets.truncate(n0);
        return None;
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "delayed grammar 2: until your next turn / this combat, whenever [referent] ...", priority: 95, parse: delayed_with_duration } }

// ---------------------------------------------------------------------------------------
// "Return ~ to the battlefield ... when that token dies."
// ---------------------------------------------------------------------------------------

/// "[instruction] when [referent] dies / leaves the battlefield" (Tatsumasa, the Dragon's
/// Fang; Portcullis): the delayed triggered ability "When [referent] dies, [instruction]"
/// (CR 603.7), created as the ability resolves.
fn trailing_when(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, tail) = l.rsplit_once(" when ")?;
    let (subject, event) = [" dies", " leaves the battlefield"]
        .into_iter()
        .find_map(|e| tail.strip_suffix(e).map(|s| (s, e)))?;
    if !matches!(subject, "that token" | "~" | "that creature" | "it") || head.contains(',') {
        return None;
    }
    super::delayed_grammar::referent_trigger(&format!("when {subject}{event}, {head}"), b)
}

inventory::submit! { EffectPattern { name: "delayed grammar 2: [instruction] when that token dies", priority: 990, parse: trailing_when } }

// ---------------------------------------------------------------------------------------
// "At the beginning of the next end step, if you haven't cast it, return it ..."
// ---------------------------------------------------------------------------------------

/// The object the delayed instruction acts on (a captured variable).
fn instruction_object(e: &Effect) -> Option<Sel> {
    match e {
        Effect::Move { what, .. } => match what {
            Sel::Var(_) => Some(what.clone()),
            _ => None,
        },
        Effect::Seq(v) => v.first().and_then(instruction_object),
        _ => None,
    }
}

/// Adds `cond` as an intervening "if" clause (CR 603.4) to the delayed ability `e` ends
/// with: checked as its event occurs and again as it resolves.
fn with_intervening_if(e: &mut Effect, cond_of: &dyn Fn(&Sel) -> Condition) -> bool {
    match e {
        Effect::Seq(v) => v.last_mut().is_some_and(|x| with_intervening_if(x, cond_of)),
        Effect::DelayedTrigger {
            trigger,
            body,
            once: true,
        } => {
            let Some(obj @ Sel::Var(_)) = instruction_object(&body.effect) else {
                return false;
            };
            let cond = cond_of(&obj);
            let t = std::mem::replace(trigger, TriggerCond::PlayerLoses);
            *trigger = TriggerCond::Where {
                trigger: Box::new(t),
                cond: cond.clone(),
            };
            let eff = std::mem::take(&mut body.effect);
            body.effect = Effect::If {
                cond,
                then: Box::new(eff),
                otherwise: Box::new(Effect::Noop),
            };
            true
        }
        _ => false,
    }
}

/// "At the beginning of the next end step, if you haven't cast the card, return it to its
/// owner's hand.", "..., if any of those cards remain exiled, return them to ...", "...,
/// if you haven't played it, put it into its owner's graveyard.": the cards were exiled
/// with a permission to play them; casting or playing one moves it out of exile, after
/// which it's a new object (CR 400.7). So "if you haven't cast it" is "if it remains
/// exiled" (see the interpretation `delayed-if-not-played-means-still-exiled`).
fn delayed_if_still_exiled(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("at the beginning of ")?;
    let (delay, rest) = r.split_once(", if ")?;
    let (cond, instruction) = [
        "any of those cards remain exiled, ",
        "you haven't cast the card, ",
        "you haven't cast it, ",
        "you haven't played it, ",
        "you haven't played the card, ",
    ]
    .into_iter()
    .find_map(|c| rest.strip_prefix(c).map(|i| (c, i)))?;
    // "it" for a single card, "them" for several.
    let plural = cond.starts_with("any of those cards");
    if plural != (instruction.contains(" them ") || instruction.ends_with(" them")) {
        return None;
    }
    let mut e =
        super::delayed_grammar::delayed_instruction(&format!("at the beginning of {delay}, {instruction}"), b)?;
    let still_exiled = |obj: &Sel| match obj {
        Sel::Var(v) => Condition::Custom(
            format!("{}{v}", crate::kw::delayed_conditions::STILL_EXILED).into(),
        ),
        _ => Condition::Never,
    };
    with_intervening_if(&mut e, &still_exiled).then_some(e)
}

inventory::submit! { EffectPattern { name: "delayed grammar 2: at the beginning of the next end step, if you haven't cast it, ...", priority: 985, parse: delayed_if_still_exiled } }

/// "At the beginning of your next upkeep, put any of those cards you didn't play into your
/// graveyard." (Three Wishes): a card played from exile left it and is a new object
/// (CR 400.7), which the delayed ability doesn't affect (CR 603.7c): those still exiled.
fn delayed_cards_you_didnt_play(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.starts_with("at the beginning of ") {
        return None;
    }
    let rewritten = l.replacen(
        "put any of those cards you didn't play into ",
        "put those cards into ",
        1,
    );
    if rewritten == l {
        return None;
    }
    super::delayed_grammar::delayed_instruction(&rewritten, b)
}

inventory::submit! { EffectPattern { name: "delayed grammar 2: put any of those cards you didn't play into your graveyard", priority: 985, parse: delayed_cards_you_didnt_play } }
