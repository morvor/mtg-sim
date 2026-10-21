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
/// The objects chosen by "gain control of a [creature] of your choice ...", over all its
/// performances in the resolution.
const GAINED: Var = vars::USER + 41012;
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

/// The value grammar's atom "the number of Curses attached to that player" (Curse of
/// Surveillance): permanents attached to the player "that player" refers to.
pub fn attached_to_player_atom(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let r = s.strip_prefix("the number of ")?;
    let (noun, rest) = r.split_once(" attached to that player")?;
    if !(rest.is_empty() || rest.starts_with([' ', ',', '.'])) {
        return None;
    }
    if super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
        return None;
    }
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(noun)?;
    if !plural || !tail.trim().is_empty() {
        return None;
    }
    Some((
        Value::Count(Filter::and(vec![
            f,
            Filter::AttachedTo(Box::new(Sel::Players(b.it_player.clone()))),
        ])),
        rest.to_string(),
    ))
}

/// "any number of target players other than that player each draw ..." (Curse of
/// Surveillance): the targets can't include the player "that player" refers to.
fn target_players_other_than_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" target players other than that player each ")?;
    if super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
        || matches!(b.it_player, PlayerRef::You)
    {
        return None;
    }
    let that = b.it_player.clone();
    let rewritten = format!(
        "{} target players each {}",
        &l[..i],
        &l[i + " target players other than that player each ".len()..]
    );
    let first = b.targets.len();
    let e = parse_clause(&rewritten, b)?;
    if b.targets.len() != first + 1 {
        return None;
    }
    let spec = &mut b.targets[first];
    let TargetKind::Player(f) = &spec.what else {
        return None;
    };
    spec.what = TargetKind::Player(PlayerFilter::And(vec![
        f.clone(),
        PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(that)))),
    ]));
    Some(e)
}

inventory::submit! { EffectPattern { name: "choices 4: target players other than that player each", priority: 120, parse: target_players_other_than_that_player } }

/// A verb phrase in the third person singular: "draw a card, then discard a card at
/// random" → "draws a card, then discards a card at random".
fn third_person(pred: &str) -> String {
    pred.split(", then ")
        .map(|part| {
            let (verb, rest) = part.split_once(' ').unwrap_or((part, ""));
            let verb = match verb {
                "lose" => "loses".to_string(),
                v if v.ends_with('s') || v.ends_with("sh") => format!("{v}es"),
                v => format!("{v}s"),
            };
            if rest.is_empty() {
                verb
            } else {
                format!("{verb} {rest}")
            }
        })
        .collect::<Vec<_>>()
        .join(", then ")
}

/// "you and those players each draw a card, then discard a card at random" (Zurzoth, Chaos
/// Rider: the players the triggering attackers attack), "you and the controller of those
/// creatures each draw a card" (Nelly Borca, Impulsive Accuser): each of the players
/// performs the instructions, each instruction by all of them in APNAP order before the
/// next (CR 101.4, 608.2e).
fn you_and_players_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you and ")?;
    let (who, pred) = r.split_once(" each ")?;
    if !b.in_trigger {
        return None;
    }
    let others = match who {
        // The players the triggering creatures attack.
        "those players" => PlayerFilter::History(Box::new(Condition::SelNonEmpty(Sel::All(
            Filter::and(vec![
                Filter::In(Box::new(Sel::TriggerObjects)),
                Filter::AttackingPlayer(PlayerRel::Iterated),
            ]),
        )))),
        "the controller of those creatures" => PlayerFilter::Ref(Box::new(
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObjects)),
        )),
        _ => return None,
    };
    let e = parse_clause(&format!("each player {}", third_person(pred)), b)?;
    let group = PlayerRef::Each(PlayerFilter::Or(vec![PlayerFilter::You, others]));
    let json = serde_json::to_string(&e).ok()?;
    let each = serde_json::to_string(&PlayerRef::EachPlayer).ok()?;
    if !json.contains(&each) {
        return None;
    }
    let json = json.replace(&each, &serde_json::to_string(&group).ok()?);
    serde_json::from_str(&json).ok()
}

inventory::submit! { EffectPattern { name: "choices 4: you and [those players] each", priority: 120, parse: you_and_players_each } }

/// "gain control of a creature of your choice that player controls until end of turn"
/// (Midnight Crusader Shuttle), "gain control of a creature that player controls until end
/// of turn" (Seize the Spotlight): you choose one of the objects as the instruction is
/// performed (not a target), then gain control of it.
fn gain_control_of_a_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("you gain control of ")
        .or_else(|| l.strip_prefix("gain control of "))?;
    let (article, r) = r.split_once(' ')?;
    if !matches!(article, "a" | "an") {
        return None;
    }
    let (noun, r) = r.split_once(' ')?;
    let r = r.strip_prefix("of your choice ").unwrap_or(r);
    let (player, dur) = r.split_once(" controls")?;
    if player.contains(" of ") || player.is_empty() {
        return None;
    }
    let choose = parse_clause(&format!("choose {article} {noun} {player} controls"), b)?;
    if !format!("{choose:?}").contains("Choose {") {
        return None;
    }
    let gain = parse_clause(&format!("gain control of it{dur}"), b)?;
    if !matches!(gain, Effect::GainControl { .. }) {
        return None;
    }
    // Done for each of several players ("for each player who chose fame, ..."), "those
    // creatures" later are all the creatures chosen this way.
    let Sel::Var(chosen) = b.it else {
        return None;
    };
    b.it = Sel::Var(GAINED);
    for (_, sel) in b.named.iter_mut() {
        if matches!(sel, Sel::Var(v) if *v == chosen) {
            *sel = Sel::Var(GAINED);
        }
    }
    Some(Effect::seq(vec![
        choose,
        gain,
        Effect::Store {
            var: GAINED,
            sel: Sel::Union(vec![Sel::Var(GAINED), Sel::Var(chosen)]),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: gain control of a [creature] of your choice [player] controls", priority: 120, parse: gain_control_of_a_chosen } }

/// "creatures controlled by that player or by that planeswalker's controller can't block
/// this turn" (Concussive Bolt), after "target player or planeswalker": the target player,
/// or the target planeswalker's controller.
fn controlled_by_that_player_or_controller(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let pred = [
        "creatures controlled by that player or by that planeswalker's controller ",
        "creatures that player or that planeswalker's controller controls ",
    ]
    .iter()
    .find_map(|p| l.strip_prefix(p))?;
    let k = b
        .targets
        .iter()
        .rposition(|t| matches!(t.what, TargetKind::ObjectOrPlayer(..)))?;
    let e = parse_clause(&format!("creatures you control {pred}"), b)?;
    let json = serde_json::to_string(&e).ok()?;
    let you = r#"{"ControlledBy":"You"}"#;
    if json.matches(you).count() != 1 {
        return None;
    }
    let theirs = serde_json::to_string(&Filter::ControlledBy(PlayerRel::TargetOrController(
        k as u8,
    )))
    .ok()?;
    serde_json::from_str(&json.replace(you, &theirs)).ok()
}

inventory::submit! { EffectPattern { name: "choices 4: creatures controlled by that player or by that planeswalker's controller", priority: 120, parse: controlled_by_that_player_or_controller } }

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
            ("Enchantment — Aura Curse", "Enchant player\nAt the beginning of enchanted player's upkeep, any number of target players other than that player each draw cards equal to the number of Curses attached to that player."),
            ("Legendary Creature", "Whenever one or more Devils you control attack one or more players, you and those players each draw a card, then discard a card at random."),
            ("Legendary Creature", "Whenever one or more creatures an opponent controls deal combat damage to one or more of your opponents, you and the controller of those creatures each draw a card."),
            ("Sorcery", "Each opponent chooses fame or fortune. For each player who chose fame, gain control of a creature that player controls until end of turn. Untap those creatures and they gain haste until end of turn. For each player who chose fortune, you draw a card and create a Treasure token."),
            ("Sorcery", "Each player with fewer cards in hand than the player with the most cards in hand draws cards equal to the difference."),
            ("Creature", "When ~ enters, each player who controls fewer lands than the player who controls the most lands searches their library for a number of basic land cards less than or equal to the difference, puts those cards onto the battlefield tapped, then shuffles."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
