//! Target players described by a comparison with you ("choose target opponent who has
//! more life than you do as you activate this ability", the Keepers): the requirement is
//! a [`PlayerFilter`]; "as you activate this ability" makes it apply only as the target is
//! chosen, not as the ability resolves (`PlayerFilter::AsChosen`; the Keepers' rulings:
//! "It is only necessary that the condition be true as you activate the ability").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "who has more life than you [do]", "who has at least two more cards in hand than you
/// [do]", "who controls more creatures than you [do]": a comparison of the player with
/// you.
pub fn compared_with_you(s: &str) -> Option<PlayerFilter> {
    compared(s, "you", &PlayerRef::You)
}

/// [`compared_with_you`] with another player: "who controls more creatures than they do"
/// (`than` is the word naming them, `who` the player).
pub fn compared(s: &str, than: &str, who: &PlayerRef) -> Option<PlayerFilter> {
    let s = s.trim();
    let s = s.strip_suffix(" do").unwrap_or(s);
    let r = s.strip_suffix(&format!(" than {than}"))?;
    if let Some(r) = r.strip_prefix("who has ") {
        // "at least two more", "more"
        let (cmp, extra, r) = if let Some(x) = r.strip_prefix("at least ") {
            let (n, x) = parse_number(x)?;
            let x = x.trim_start().strip_prefix("more ")?;
            (Cmp::Ge, Some(n), x)
        } else {
            (Cmp::Gt, None, r.strip_prefix("more ")?)
        };
        let theirs_vs = |v: Value| match extra.clone() {
            Some(n) => Value::Sum(vec![v, n]),
            None => v,
        };
        return match r {
            "life" => Some(PlayerFilter::Life(
                cmp,
                Box::new(theirs_vs(Value::LifeTotal(who.clone()))),
            )),
            "cards in hand" | "cards in their hand" => Some(PlayerFilter::HandSize(
                cmp,
                Box::new(theirs_vs(Value::HandSize(who.clone()))),
            )),
            _ => None,
        };
    }
    let noun = r.strip_prefix("who controls more ")?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let control = match who {
        PlayerRef::You => Filter::ControlledBy(PlayerRel::You),
        w => Filter::ControlledByPlayer(Box::new(w.clone())),
    };
    Some(PlayerFilter::Controls(
        Box::new(f.clone()),
        Cmp::Gt,
        Box::new(Value::Count(Filter::and(vec![f, control]))),
    ))
}

/// "Choose target opponent who has more life than you do as you activate this ability."
fn choose_target_player_who(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (spec, tail) = parse_target(r)?;
    let TargetKind::Player(base) = &spec.what else {
        return None;
    };
    if spec.fixed_min() != Some(1) || !matches!(spec.max, Value::Const(1)) {
        return None;
    }
    let tail = tail.trim();
    let (pred, as_chosen) = match tail.strip_suffix(" as you activate this ability") {
        Some(p) => (p, true),
        None => (tail, false),
    };
    let f = compared_with_you(pred)?;
    let f = if as_chosen {
        PlayerFilter::AsChosen(Box::new(f))
    } else {
        f
    };
    let what = match base {
        PlayerFilter::Any => f,
        other => PlayerFilter::And(vec![other.clone(), f]),
    };
    let text = r.trim().to_string();
    let spec = TargetSpec {
        what: TargetKind::Player(what),
        ..spec
    };
    let slot = b.add_target(spec, &text);
    b.it_player = PlayerRef::Target(slot);
    b.named.push((
        "the chosen player".into(),
        Sel::Players(PlayerRef::Target(slot)),
    ));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose target player who [compared with you]", priority: 86, parse: choose_target_player_who } }

/// The player chosen by "choose another player" and the like.
pub const CHOSEN_PLAYER: Var = vars::USER + 4433;

/// "choose another player", "choose an opponent who controls more creatures than you",
/// "choose a player with the most life or tied for most life": the controller chooses one
/// of the players described as the effect happens (CR 608.2d); "that player" and "the
/// chosen player" refer to them.
fn choose_a_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (base, rest) = if let Some(x) = r.strip_prefix("another player") {
        (PlayerFilter::NotYou, x)
    } else if let Some(x) = r.strip_prefix("an opponent") {
        (PlayerFilter::Opponent, x)
    } else if let Some(x) = r.strip_prefix("a player") {
        (PlayerFilter::Any, x)
    } else {
        return None;
    };
    let rest = rest.trim();
    let quality = match rest {
        "" => None,
        "with the most life or tied for most life" => Some(PlayerFilter::Life(
            Cmp::Ge,
            Box::new(Value::OverPlayers(
                AggOp::Max,
                PlayerFilter::Any,
                Box::new(Value::LifeTotal(PlayerRef::Iterated)),
            )),
        )),
        _ => Some(compared_with_you(rest)?),
    };
    // "choose an opponent" alone is the core's (the ETB-style choice).
    if quality.is_none() && !matches!(base, PlayerFilter::NotYou) {
        return None;
    }
    let filter = match quality {
        None => base,
        Some(q) if matches!(base, PlayerFilter::Any) => q,
        Some(q) => PlayerFilter::And(vec![base, q]),
    };
    let e = crate::kw::choice_grammar::choose_player_effect(CHOSEN_PLAYER, filter)?;
    b.it_player = PlayerRef::Var(CHOSEN_PLAYER);
    b.named.push((
        "the chosen player".into(),
        Sel::Players(PlayerRef::Var(CHOSEN_PLAYER)),
    ));
    Some(e)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose a player [described]", priority: 87, parse: choose_a_player } }

/// "At the beginning of each player's upkeep, that player chooses target player who
/// controls more creatures than they do and is their opponent. The first player may ..."
/// (the Oaths): a target chosen by "that player" (CR 601.2c), compared with them; "the
/// first player" is the chooser, "the second player" the target.
fn that_player_chooses_target_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("that player chooses target player who ")?;
    let pred = r.strip_suffix(" and is their opponent")?;
    let chooser = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&chooser)
        || matches!(chooser, PlayerRef::You)
    {
        return None;
    }
    let f = compared(&format!("who {pred}"), "they", &chooser)?;
    let text = "target player".to_string();
    let mut spec = TargetSpec::player(
        PlayerFilter::And(vec![PlayerFilter::OpponentOf(Box::new(chooser.clone())), f]),
        &text,
    );
    spec.chosen_by = Some(chooser.clone());
    let slot = b.add_target(spec, &text);
    b.it_player = chooser.clone();
    b.named.push(("the first player".into(), Sel::Players(chooser)));
    b.named.push((
        "the second player".into(),
        Sel::Players(PlayerRef::Target(slot)),
    ));
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: that player chooses target player who ... and is their opponent", priority: 86, parse: that_player_chooses_target_player } }
