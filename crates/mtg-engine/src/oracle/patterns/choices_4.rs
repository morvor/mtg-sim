//! Choice grammar IV: player phrases compared across players (CR 101.4, 608.2h).
//!
//! - "Each player with fewer cards in hand than the player with the most cards in hand
//!   draws cards equal to the difference." (Tales of the Ancestors), "each player who
//!   controls fewer lands than the player who controls the most lands searches their
//!   library for a number of basic land cards less than or equal to the difference, ..."
//!   (Scholarship Sponsor): the greatest number is determined once, before any player
//!   acts (CR 608.2h), and each player's difference is their own.
//! - "the player with the lowest life total gains control of ~. If two or more players
//!   are tied for lowest life total, you choose one of them, and that player gains control
//!   of ~." (Loxodon Peacekeeper).

use super::{EffectPattern, FilterSuffixPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

/// The greatest number among players, determined before any of them acts.
const MOST: Var = vars::USER + 41010;
/// The player chosen among those tied for a superlative.
const TIED_CHOSEN: Var = vars::USER + 41011;

/// A number each player has: "cards in hand", "lands" (that they control).
#[derive(Clone, Copy)]
enum Stat {
    Hand,
    Lands,
}

impl Stat {
    fn of(self, who: PlayerRef) -> Value {
        match self {
            Stat::Hand => Value::HandSize(who),
            Stat::Lands => {
                let rel = match who {
                    PlayerRef::You => PlayerRel::You,
                    _ => PlayerRel::Iterated,
                };
                Value::Count(Filter::and(vec![
                    Filter::Type(crate::types::CardType::Land),
                    Filter::ControlledBy(rel),
                ]))
            }
        }
    }
}

/// "each player with fewer cards in hand than the player with the most cards in hand
/// [predicate]", "each player who controls fewer lands than the player who controls the
/// most lands [predicate]": the stat and the predicate.
fn fewer_than_most(l: &str) -> Option<(Stat, &str)> {
    for (head, stat) in [
        (
            "each player with fewer cards in hand than the player with the most cards in hand ",
            Stat::Hand,
        ),
        (
            "each player who controls fewer lands than the player who controls the most lands ",
            Stat::Lands,
        ),
    ] {
        if let Some(r) = l.strip_prefix(head) {
            return Some((stat, r));
        }
    }
    None
}

/// The predicate with "the difference" as x: "draws cards equal to the difference",
/// "searches their library for a number of basic land cards less than or equal to the
/// difference, ...".
fn with_x(pred: &str) -> Option<String> {
    if let Some(r) = pred.strip_prefix("draws cards equal to the difference") {
        return Some(format!("draws x cards{r}"));
    }
    let (a, r) = pred.split_once(" a number of ")?;
    let (noun, rest) = r.split_once(" less than or equal to the difference")?;
    Some(format!("{a} up to x {noun}{rest}"))
}

fn each_fewer_than_most(l: &str, b: &mut Builder) -> Option<Effect> {
    let (stat, pred) = fewer_than_most(end(l))?;
    let clause = format!("that player {}", with_x(pred)?);
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let e = super::value_grammar::with_x_defined(true, || parse_clause(&clause, b));
    b.it_player = saved;
    let e = e?;
    let d = format!("{e:?}");
    if d.contains("EachPlayer") || d.contains("EachOpponent") {
        return None;
    }
    let diff = Value::Diff(
        Box::new(Value::Var(MOST)),
        Box::new(stat.of(PlayerRef::Iterated)),
    );
    let e = super::r107_numbers::substitute_x(&e, &diff)?;
    let most = Value::OverPlayers(
        AggOp::Max,
        PlayerFilter::Any,
        Box::new(stat.of(PlayerRef::Iterated)),
    );
    let fewer = match stat {
        Stat::Hand => PlayerFilter::HandSize(Cmp::Lt, Box::new(Value::Var(MOST))),
        Stat::Lands => PlayerFilter::Controls(
            Box::new(Filter::Type(crate::types::CardType::Land)),
            Cmp::Lt,
            Box::new(Value::Var(MOST)),
        ),
    };
    Some(Effect::seq(vec![
        Effect::StoreValue {
            var: MOST,
            value: most,
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Each(fewer),
            effect: Box::new(e),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: each player with fewer [stat] than the player with the most", priority: 120, parse: each_fewer_than_most } }

/// The players tied for the lowest life total.
fn lowest_life() -> PlayerFilter {
    PlayerFilter::Life(
        Cmp::Eq,
        Box::new(Value::OverPlayers(
            AggOp::Min,
            PlayerFilter::Any,
            Box::new(Value::LifeTotal(PlayerRef::Iterated)),
        )),
    )
}

const TIE_SENTENCE: &str = "if two or more players are tied for lowest life total, you choose one of them, and that player ";

/// "the player with the lowest life total gains control of ~", when the text goes on to
/// say who breaks a tie ("If two or more players are tied for lowest life total, you
/// choose one of them, and that player gains control of ~."): you choose among the
/// players tied for the lowest life total (one player: that player), and that player
/// performs the instruction.
fn lowest_life_player_does(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the player with the lowest life total ")?;
    let raw = crate::oracle::raw_text().to_lowercase();
    let (verb, tail) = r.split_once(' ')?;
    // The tie sentence repeats the instruction (with the card's name as printed).
    let rest = raw.split(TIE_SENTENCE).nth(1)?;
    if !rest.starts_with(&format!("{verb} ")) {
        return None;
    }
    let verb = match verb {
        "gains" => "gain",
        _ => return None,
    };
    let e = parse_clause(&format!("{verb} {tail}"), b)?;
    let choose = crate::kw::choice_grammar::choose_player_effect(TIED_CHOSEN, lowest_life())?;
    Some(Effect::seq(vec![
        choose,
        Effect::AsPlayer {
            who: PlayerRef::Var(TIED_CHOSEN),
            effect: Box::new(e),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: the player with the lowest life total [does]", priority: 120, parse: lowest_life_player_does } }

/// The tie-breaking sentence after [`lowest_life_player_does`]: already part of it.
fn tie_sentence(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    if !l.starts_with(TIE_SENTENCE) {
        return false;
    }
    let d = format!("{prev:?}");
    d.contains(crate::kw::choice_grammar::CHOOSE_PLAYER)
        && d.contains(&format!("AsPlayer {{ who: Var({TIED_CHOSEN})"))
}

inventory::submit! { FollowupPattern { name: "choices 4: if two or more players are tied for lowest life total", priority: 120, apply: tie_sentence } }

/// "with mana value of that quality", "with power of the chosen quality" after "choose odd
/// or even" (CR 202.3, 208): the quality chosen by the instruction's own ability (CR
/// 607.2d).
fn quality_of<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    for (p, f) in [
        (
            "with mana value of that quality",
            Filter::ManaValueOfChosenQuality,
        ),
        (
            "with power of the chosen quality",
            Filter::Custom(crate::kw::choices_4::POWER_OF_CHOSEN_QUALITY.into()),
        ),
        (
            "with power of that quality",
            Filter::Custom(crate::kw::choices_4::POWER_OF_CHOSEN_QUALITY.into()),
        ),
    ] {
        if let Some(r) = t.strip_prefix(p) {
            if r.is_empty() || r.starts_with([' ', ',', '.']) {
                return Some((f, r));
            }
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "choices 4: with mana value or power of the chosen quality", priority: 95, parse: quality_of } }

/// "with power, toughness, or mana value 4" (The Fantastic Four): any of them is that
/// number (CR 208.1, 202.3); a spell with no power or toughness has neither.
fn any_stat_is<'a>(t: &'a str, _so_far: &Filter) -> Option<(Filter, &'a str)> {
    let r = t.strip_prefix("with power, toughness, or mana value ")?;
    let (n, rest) = crate::oracle::phrases::parse_number(r)?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    n.as_const()?;
    let v = || Box::new(n.clone());
    Some((
        Filter::Or(vec![
            Filter::Power(Cmp::Eq, v()),
            Filter::Toughness(Cmp::Eq, v()),
            Filter::ManaValue(Cmp::Eq, v()),
        ]),
        rest,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "choices 4: with power, toughness, or mana value N", priority: 95, parse: any_stat_is } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn players_compared() {
        for (ty, text) in [
            ("Enchantment", "At the beginning of your upkeep, the player with the lowest life total gains control of ~. If two or more players are tied for lowest life total, you choose one of them, and that player gains control of ~."),
            ("Creature", "Whenever you attack, choose odd or even. Creatures with mana value of that quality can't block this turn."),
            ("Sorcery", "You may put a +1/+1 counter on a creature. Then choose odd or even. Return each creature with power of the chosen quality to its owner's hand."),
            ("Legendary Creature", "When ~ enter and whenever you cast a spell with power, toughness, or mana value 4, choose one that hasn't been chosen this turn.\n• Create a 0/4 colorless Wall creature token with defender.\n• ~ deal 3 damage to each opponent.\n• Put two +1/+1 counters on target creature.\n• Draw a card."),
            ("Sorcery", "Each player with fewer cards in hand than the player with the most cards in hand draws cards equal to the difference."),
            ("Creature", "When ~ enters, each player who controls fewer lands than the player who controls the most lands searches their library for a number of basic land cards less than or equal to the difference, puts those cards onto the battlefield tapped, then shuffles."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
