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
use crate::oracle::phrases::{end, parse_number};

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
pub(super) fn third_person(pred: &str) -> String {
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

/// One alternative of a choice over objects in several zones, as "you" (the player
/// choosing, who owns or controls them) words it: "a permanent you control", "cards from
/// their hand", "a card from your hand or graveyard" (two alternatives).
fn zone_alternatives(s: &str) -> Option<Vec<Filter>> {
    let s = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .unwrap_or(s);
    let you = || PlayerRel::You;
    for p in [
        "permanent you control",
        "permanents you control",
        "permanent they control",
        "permanents they control",
    ] {
        if s == p {
            return Some(vec![Filter::and(vec![
                Filter::Permanent,
                Filter::ControlledBy(you()),
            ])]);
        }
    }
    let r = s
        .strip_prefix("card from ")
        .or_else(|| s.strip_prefix("cards from "))?;
    let r = r
        .strip_prefix("your ")
        .or_else(|| r.strip_prefix("their "))?;
    let zones: Vec<ZoneKind> = match r {
        "hand" => vec![ZoneKind::Hand],
        "graveyard" => vec![ZoneKind::Graveyard],
        "hand or graveyard" => vec![ZoneKind::Hand, ZoneKind::Graveyard],
        _ => return None,
    };
    Some(
        zones
            .into_iter()
            .map(|z| {
                Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(z),
                    Filter::OwnedBy(you()),
                ])
            })
            .collect(),
    )
}

/// "a card from their hand or a permanent they control", "X permanents they control and/or
/// cards from their hand", "a permanent you control or a card from your hand or
/// graveyard": the objects the player may choose among, in several zones.
fn mixed_objects(s: &str) -> Option<Filter> {
    let parts: Vec<&str> = if s.contains(" and/or ") {
        s.split(" and/or ").collect()
    } else {
        let i = s.find(" or a ").or_else(|| s.find(" or an "))?;
        vec![&s[..i], &s[i + " or ".len()..]]
    };
    let mut alts = vec![];
    for p in parts {
        alts.extend(zone_alternatives(p.trim())?);
    }
    (alts.len() >= 2).then_some(Filter::Or(alts))
}

/// "Each opponent exiles a card from their hand or a permanent they control." (Nicol Bolas,
/// Dragon-God), "each player exiles X permanents they control and/or cards from their
/// hand" (Descent into Madness), "for each 1 life you lost, exile a permanent you control or
/// a card from your hand or graveyard" (Lich's Mastery): each player chooses that many of
/// the objects in any mix of the zones, in APNAP order, then they're exiled at once (CR
/// 101.4, 608.2e); a player with fewer exiles all they can.
fn exile_mixed(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (group, r) = if let Some(r) = l.strip_prefix("each opponent exiles ") {
        (Some(PlayerRef::EachOpponent), r)
    } else if let Some(r) = l.strip_prefix("each player exiles ") {
        (Some(PlayerRef::EachPlayer), r)
    } else if let Some(r) = l.strip_prefix("exile ") {
        (None, r)
    } else if let Some(r) = l.strip_prefix("for each 1 life you lost, exile ") {
        if !b.in_trigger {
            return None;
        }
        (None, r)
    } else {
        return None;
    };
    let per_life = l.starts_with("for each 1 life you lost, ");
    let (count, phrase) = if per_life {
        (Value::EventAmount, r.to_string())
    } else if let Some(x) = r.strip_prefix("x ") {
        if !super::value_grammar::x_defined() {
            return None;
        }
        (Value::X, x.to_string())
    } else if r.starts_with("a ") || r.starts_with("an ") {
        (Value::c(1), r.to_string())
    } else {
        let (n, x) = parse_number(r)?;
        n.as_const()?;
        (n, x.trim_start().to_string())
    };
    let filter = mixed_objects(&phrase)?;
    let exile = Effect::Exile {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count,
            up_to: false,
            store: None,
        },
        face_down: false,
        link: false,
    };
    Some(match group {
        None => exile,
        Some(who) => Effect::ForEachPlayer {
            who,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(exile),
            }),
        },
    })
}

inventory::submit! { EffectPattern { name: "choices 4: exile [objects] in several zones", priority: 120, parse: exile_mixed } }

/// "Then each player sacrifices all other creatures they control that don't share a
/// creature type with the chosen creature they control." (Winnowing): after "for each
/// player, you choose a creature that player controls", each player keeps the creature
/// chosen for them and those sharing a creature type with it.
fn sacrifices_others_not_sharing_with_chosen(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let head = l.strip_suffix(
        " sacrifices all other creatures they control that don't share a creature type with the chosen creature they control",
    )?;
    if head != "each player" {
        return None;
    }
    let chosen = super::iteration_grammar::CHOSEN;
    let e = parse_clause(
        "each player sacrifices all creatures they control not chosen this way",
        b,
    )?;
    let json = serde_json::to_string(&e).ok()?;
    let not_chosen =
        serde_json::to_string(&Filter::not(Filter::In(Box::new(Sel::Var(chosen))))).ok()?;
    if json.matches(&not_chosen).count() != 1 {
        return None;
    }
    let both = Filter::and(vec![
        Filter::not(Filter::In(Box::new(Sel::Var(chosen)))),
        Filter::not(Filter::SharesCreatureType(Box::new(Sel::Matching(
            Box::new(Sel::Var(chosen)),
            Filter::ControlledBy(PlayerRel::You),
        )))),
    ]);
    serde_json::from_str(&json.replace(&not_chosen, &serde_json::to_string(&both).ok()?)).ok()
}

inventory::submit! { EffectPattern { name: "choices 4: each player sacrifices all other creatures that don't share a creature type with the chosen creature", priority: 120, parse: sacrifices_others_not_sharing_with_chosen } }

/// "two target players each reveal the top card of their library" (Parker Luck): two
/// different target players (CR 115.3), each revealing.
fn two_target_players_each_reveal(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("two target players each ")?;
    if r != "reveal the top card of their library" {
        return None;
    }
    let first = b.targets.len();
    let e0 = parse_clause("target player reveals the top card of their library", b)?;
    let e1 = parse_clause("target player reveals the top card of their library", b)?;
    if b.targets.len() != first + 2 {
        return None;
    }
    b.targets[first + 1].distinct_from.push(first as u8);
    Some(Effect::seq(vec![e0, e1]))
}

inventory::submit! { EffectPattern { name: "choices 4: two target players each reveal the top card of their library", priority: 120, parse: two_target_players_each_reveal } }

/// The cards the two players of a pair revealed ("the card revealed by the other player").
const REVEALED_FIRST: Var = vars::USER + 41013;
const REVEALED_SECOND: Var = vars::USER + 41014;

/// The two players whose top cards `prev` revealed, one after the other.
fn revealing_pair(prev: &Effect) -> Option<(PlayerRef, PlayerRef)> {
    let Effect::Seq(v) = prev else {
        return None;
    };
    let players: Vec<PlayerRef> = v
        .iter()
        .filter_map(|e| match e {
            Effect::Dig {
                who,
                n: Value::Const(1),
                reveal: true,
                ..
            } => Some(who.clone()),
            Effect::AsPlayer { who, effect } => match &**effect {
                Effect::Dig {
                    who: PlayerRef::You,
                    n: Value::Const(1),
                    reveal: true,
                    ..
                } => Some(who.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();
    match players.as_slice() {
        [a, b] if v.len() == 2 => Some((a.clone(), b.clone())),
        _ => None,
    }
}

/// "You each lose life equal to the mana value of the card revealed by the other player."
/// (Keen Duelist), "They each lose life equal to ..." (Parker Luck), after two players each
/// revealed the top card of their library; then "You each put the card you revealed into
/// your hand." / "Then they each put the card they revealed into their hand."
fn pair_reveal_followups(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let lose = matches!(
        l,
        "you each lose life equal to the mana value of the card revealed by the other player"
            | "they each lose life equal to the mana value of the card revealed by the other player"
    );
    let put = matches!(
        l,
        "you each put the card you revealed into your hand"
            | "they each put the card they revealed into their hand"
    );
    if !lose && !put {
        return false;
    }
    let mv = |v: Var| Value::ManaValueOf(Box::new(Sel::Var(v)));
    let to_hand = |v: Var| Effect::Move {
        what: Sel::Var(v),
        to: Destination::zone(ZoneKind::Hand),
    };
    // After the reveals: what each revealed, then the life loss.
    if lose {
        let Some((a, b2)) = revealing_pair(prev) else {
            return false;
        };
        let old = std::mem::take(prev);
        *prev = Effect::seq(vec![
            old,
            Effect::Store {
                var: REVEALED_FIRST,
                sel: Sel::TopOfLibrary(a.clone(), Value::c(1)),
            },
            Effect::Store {
                var: REVEALED_SECOND,
                sel: Sel::TopOfLibrary(b2.clone(), Value::c(1)),
            },
            Effect::LoseLife {
                who: a,
                n: mv(REVEALED_SECOND),
            },
            Effect::LoseLife {
                who: b2,
                n: mv(REVEALED_FIRST),
            },
        ]);
        return true;
    }
    if !format!("{prev:?}").contains(&format!("var: {REVEALED_SECOND}")) {
        return false;
    }
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, to_hand(REVEALED_FIRST), to_hand(REVEALED_SECOND)]);
    true
}

inventory::submit! { FollowupPattern { name: "choices 4: each lose life equal to the card revealed by the other player", priority: 120, apply: pair_reveal_followups } }

/// "Cast this spell only during an opponent's turn. Tap target creature that player
/// controls. That creature deals damage equal to its power to the player." (Delirium):
/// "that player" and "the player" are the opponent whose turn it is, the active player.
fn that_player_whose_turn_it_is(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("cast this spell only during an opponent's turn") {
        return None;
    }
    if l.contains("that player")
        && super::oracle_hardening_referents::is_no_player_referent(&b.it_player)
    {
        let e = parse_clause(&l.replace("that player", "the active player"), b)?;
        b.it_player = PlayerRef::ActivePlayer;
        return Some(e);
    }
    let h = l.strip_suffix(" to the player")?;
    if !matches!(b.it_player, PlayerRef::ActivePlayer) {
        return None;
    }
    parse_clause(&format!("{h} to that player"), b)
}

inventory::submit! { EffectPattern { name: "choices 4: that player during an opponent's turn (the active player)", priority: 120, parse: that_player_whose_turn_it_is } }

/// "Then choose another attacking creature with lesser power." (Kamiz, Obscura Oculus):
/// another than the object "it" refers to, with lesser power than it has as the choice is
/// made.
fn choose_another_with_lesser_power(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let r = l.strip_prefix("choose another ")?;
    let noun = r.strip_suffix(" with lesser power")?;
    let it = b.it.clone();
    if !matches!(it, Sel::Target(_)) {
        return None;
    }
    let article = if noun.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    let mut e = parse_clause(&format!("choose {article} {noun}"), b)?;
    let store = match &mut e {
        Effect::Seq(v) => v.first_mut()?,
        e => e,
    };
    let Effect::Store {
        sel: Sel::Choose { filter, .. },
        ..
    } = store
    else {
        return None;
    };
    let f = std::mem::replace(filter, Filter::Any);
    *filter = Filter::and(vec![
        f,
        Filter::not(Filter::In(Box::new(it.clone()))),
        Filter::Power(Cmp::Lt, Box::new(Value::PowerOf(Box::new(it)))),
    ]);
    Some(e)
}

inventory::submit! { EffectPattern { name: "choices 4: choose another [object] with lesser power", priority: 120, parse: choose_another_with_lesser_power } }

/// "who has [at least N] fewer [cards] in their graveyard than [player]" (with the
/// "than [player]" already removed): a player compared with `who` by the number of those
/// cards in each graveyard.
pub(super) fn fewer_cards_in_graveyard(r: &str, who: &PlayerRef) -> Option<PlayerFilter> {
    let r = r.strip_prefix("who has ")?;
    let (n, r) = match r.strip_prefix("at least ") {
        Some(x) => {
            let (n, x) = parse_number(x)?;
            n.as_const()?;
            (Some(n), x.trim_start())
        }
        None => (None, r),
    };
    let noun = r
        .strip_prefix("fewer ")?
        .strip_suffix(" in their graveyard")?;
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let mine = Value::CardsInGraveyard(PlayerRef::Iterated, f.clone());
    let theirs = Value::CardsInGraveyard(who.clone(), f);
    let cond = match n {
        // At least N fewer: mine + N <= theirs.
        Some(n) => Condition::Compare(Value::Sum(vec![mine, n]), Cmp::Le, theirs),
        None => Condition::Compare(mine, Cmp::Lt, theirs),
    };
    Some(PlayerFilter::History(Box::new(cond)))
}

/// "If the first player does, ..." (Oath of Druids): "the first player" is the player who
/// chose the target ("that player chooses target player ..."), the one given the optional
/// instruction; as "if they do".
fn if_the_first_player_does(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if the first player does, ") else {
        return false;
    };
    if !b.named.iter().any(|(n, _)| n == "the first player") {
        return false;
    }
    crate::oracle_ext::apply_followup_ext(&format!("if they do, {r}"), prev, b)
}

inventory::submit! { FollowupPattern { name: "choices 4: if the first player does", priority: 10, apply: if_the_first_player_does } }

/// The creature a vow counter's restriction is about.
const VOWED: Var = vars::USER + 41015;

/// "Each player puts a vow counter on a creature they control and sacrifices the rest.
/// Each of those creatures can't attack you or planeswalkers you control for as long as it
/// has a vow counter on it." (Promise of Loyalty): each player chooses a creature to keep
/// (CR 101.4), it gets the counter, and the rest are sacrificed at the same time.
fn puts_counter_on_one_and_sacrifices_the_rest(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("each player puts a ")?;
    let (kind, r) = r.split_once(" counter on a ")?;
    let noun = r.strip_suffix(" they control and sacrifices the rest")?;
    if kind.contains(' ') {
        return None;
    }
    let keep = parse_clause(
        &format!("each player chooses a {noun} they control and sacrifices the rest"),
        b,
    )?;
    if !matches!(keep, Effect::KeepAndSacrificeRest { .. }) {
        return None;
    }
    let kept = Sel::Var(crate::apnap::KEPT);
    b.it = kept.clone();
    b.named.push(("those creatures".into(), kept.clone()));
    Some(Effect::seq(vec![
        keep,
        Effect::AddCounters {
            what: kept,
            kind: kind.into(),
            n: Value::c(1),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: each player puts a counter on a creature and sacrifices the rest", priority: 120, parse: puts_counter_on_one_and_sacrifices_the_rest } }

/// "Each of those creatures can't attack you or planeswalkers you control for as long as it
/// has a vow counter on it." after [`puts_counter_on_one_and_sacrifices_the_rest`]: for
/// each of them, a restriction lasting while it has that counter.
fn each_cant_attack_while_it_has_counter(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("each of those creatures can't attack you or planeswalkers you control for as long as it has a ") else {
        return false;
    };
    let Some(kind) = r.strip_suffix(" counter on it") else {
        return false;
    };
    if !format!("{prev:?}").contains(&format!("Var({})", crate::apnap::KEPT)) {
        return false;
    }
    let restriction = Effect::AddRestriction {
        restriction: Restriction::CantAttackPlayer {
            attackers: Filter::In(Box::new(Sel::Var(VOWED))),
            defender: PlayerFilter::You,
            planeswalkers: true,
            battles: false,
        },
        duration: Duration::WhileCondition(Condition::SelMatches(
            Sel::Var(VOWED),
            Filter::HasCounter(Some(kind.into())),
        )),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::ForEach {
            sel: Sel::Var(crate::apnap::KEPT),
            var: VOWED,
            effect: Box::new(restriction),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "choices 4: each of those creatures can't attack you while it has a counter", priority: 120, apply: each_cant_attack_while_it_has_counter } }

/// The two players exchanging their creatures, and the creatures they exchanged.
const PAIR: Var = vars::USER + 41016;
const EXCHANGED: Var = vars::USER + 41017;

/// "You and that opponent each gain control of all creatures the other controls until end
/// of turn." (Reins of Power): both sets of creatures are determined first, then control
/// of all of them changes at the same time (CR 608.2e); "those creatures"
/// later are all of them.
fn you_and_that_opponent_exchange_creatures(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("you and that opponent each gain control of all ")?;
    let (noun, dur) = r.split_once(" the other controls")?;
    let other = b.it_player.clone();
    if super::oracle_hardening_referents::is_no_player_referent(&other)
        || matches!(other, PlayerRef::You)
    {
        return None;
    }
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let probe = format!("x{dur}");
    let (duration, rest) = crate::oracle::effects::duration_suffix(&probe);
    if rest.trim() != "x" {
        return None;
    }
    let pair = PlayerFilter::Ref(Box::new(PlayerRef::Var(PAIR)));
    let e = Effect::seq(vec![
        Effect::Store {
            var: PAIR,
            sel: Sel::Union(vec![Sel::Players(PlayerRef::You), Sel::Players(other)]),
        },
        Effect::Store {
            var: EXCHANGED,
            sel: Sel::All(Filter::and(vec![
                f.clone(),
                Filter::ControllerMatches(Box::new(pair.clone())),
            ])),
        },
        Effect::ForEachPlayer {
            who: PlayerRef::Var(PAIR),
            effect: Box::new(Effect::GainControl {
                what: Sel::All(Filter::and(vec![
                    f,
                    Filter::In(Box::new(Sel::Var(EXCHANGED))),
                    Filter::ControllerMatches(Box::new(PlayerFilter::And(vec![
                        pair,
                        PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
                            PlayerRef::Iterated,
                        )))),
                    ]))),
                ])),
                who: PlayerRef::Iterated,
                duration,
            }),
        },
    ]);
    b.it = Sel::Var(EXCHANGED);
    b.named
        .push(("those creatures".into(), Sel::Var(EXCHANGED)));
    Some(e)
}

inventory::submit! { EffectPattern { name: "choices 4: you and that opponent each gain control of all creatures the other controls", priority: 120, parse: you_and_that_opponent_exchange_creatures } }

/// The player chosen by "another player of your choice".
const OF_YOUR_CHOICE: Var = vars::USER + 41018;

/// "another player of your choice may gain control of that permanent" (Stardew Valley):
/// you choose another player as the instruction is performed, and that player may gain
/// control of it.
fn another_player_of_your_choice_may(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("another player of your choice may ")?;
    let r = r.strip_prefix("gain control of ")?;
    let (what, rest) = crate::oracle::effects::object_ref(r, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let choose =
        crate::kw::choice_grammar::choose_player_effect(OF_YOUR_CHOICE, PlayerFilter::NotYou)?;
    Some(Effect::seq(vec![
        choose,
        Effect::AsPlayer {
            who: PlayerRef::Var(OF_YOUR_CHOICE),
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::GainControl {
                    what,
                    who: PlayerRef::You,
                    duration: Duration::Permanent,
                }),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: another player of your choice may gain control of it", priority: 120, parse: another_player_of_your_choice_may } }

/// The opponent and the creature of "an opponent may gain control of a creature you
/// control of their choice", and the player who controlled it.
const TAKER: Var = vars::USER + 41019;
const TAKEN: Var = vars::USER + 41029;
const GIVER: Var = vars::USER + 41035;

/// "an opponent may gain control of a creature you control of their choice for as long as
/// ~ remains on the battlefield" (Infernal Denizen): the opponent who decides (chosen by
/// you, CR 801.5a, see `Game::deciding_opponent`) chooses one of your creatures and may
/// gain control of it for that long.
fn an_opponent_may_take_one_of_their_choice(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("an opponent may gain control of a ")?;
    let (noun, dur) = r.split_once(" you control of their choice")?;
    let duration = match dur {
        " for as long as ~ remains on the battlefield" => Duration::WhileSourceOnBattlefield,
        " until end of turn" => Duration::EndOfTurn,
        "" => Duration::Permanent,
        _ => return None,
    };
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::Store {
            var: GIVER,
            sel: Sel::Players(PlayerRef::You),
        },
        crate::kw::choice_grammar::deciding_opponent_effect(TAKER)?,
        Effect::AsPlayer {
            who: PlayerRef::Var(TAKER),
            effect: Box::new(Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::seq(vec![
                    Effect::Store {
                        var: TAKEN,
                        sel: Sel::Choose {
                            chooser: PlayerRef::You,
                            filter: Filter::and(vec![
                                f,
                                Filter::InZone(ZoneKind::Battlefield),
                                Filter::ControllerMatches(Box::new(PlayerFilter::Ref(Box::new(
                                    PlayerRef::Var(GIVER),
                                )))),
                            ]),
                            count: Value::c(1),
                            up_to: false,
                            store: None,
                        },
                    },
                    Effect::GainControl {
                        what: Sel::Var(TAKEN),
                        who: PlayerRef::You,
                        duration,
                    },
                ])),
            }),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: an opponent may gain control of a creature you control of their choice", priority: 120, parse: an_opponent_may_take_one_of_their_choice } }

/// The card chosen by another player for a discard.
const DISCARD_CHOSEN: Var = vars::USER + 41036;

/// "Reveal your hand and discard a card of target opponent's choice." / "Then that player
/// reveals their hand and discards a card of your choice." (Rakdos Augermage): the player
/// reveals their hand, the other player chooses a card from it, and the first discards it.
fn reveal_and_discard_of_choice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (who, r) = if let Some(r) = l.strip_prefix("reveal your hand and discard a card of ") {
        (PlayerRef::You, r)
    } else if let Some(r) = l.strip_prefix("that player reveals their hand and discards a card of ")
    {
        let p = b.it_player.clone();
        if super::oracle_hardening_referents::is_no_player_referent(&p)
            || matches!(p, PlayerRef::You)
        {
            return None;
        }
        (p, r)
    } else {
        return None;
    };
    let chooser = match r {
        "your choice" => PlayerRef::You,
        "target opponent's choice" => {
            let text = "target opponent";
            let slot = b.add_target(TargetSpec::player(PlayerFilter::Opponent, text), text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    if format!("{who:?}") == format!("{chooser:?}") {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::RevealHand { who: who.clone() },
        Effect::Store {
            var: DISCARD_CHOSEN,
            sel: Sel::Choose {
                chooser,
                filter: Filter::and(vec![
                    Filter::Card,
                    Filter::InZone(ZoneKind::Hand),
                    Filter::OwnedByPlayer(Box::new(who.clone())),
                ]),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Discard {
            who,
            n: Value::c(1),
            random: false,
            filter: Filter::In(Box::new(Sel::Var(DISCARD_CHOSEN))),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "choices 4: reveal your hand and discard a card of [player]'s choice", priority: 120, parse: reveal_and_discard_of_choice } }

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
            ("Legendary Planeswalker — Bolas", "+1: You draw a card. Each opponent exiles a card from their hand or a permanent they control."),
            ("Enchantment", "Whenever you lose life, for each 1 life you lost, exile a permanent you control or a card from your hand or graveyard."),
            ("Enchantment", "At the beginning of your upkeep, put a despair counter on ~, then each player exiles X permanents they control and/or cards from their hand, where X is the number of despair counters on ~."),
            ("Sorcery", "Convoke\nFor each player, you choose a creature that player controls. Then each player sacrifices all other creatures they control that don't share a creature type with the chosen creature they control."),
            ("Creature", "At the beginning of your upkeep, you and target opponent each reveal the top card of your library. You each lose life equal to the mana value of the card revealed by the other player. You each put the card you revealed into your hand."),
            ("Enchantment", "At the beginning of your end step, two target players each reveal the top card of their library. They each lose life equal to the mana value of the card revealed by the other player. Then they each put the card they revealed into their hand."),
            ("Instant", "Cast this spell only during an opponent's turn.\nTap target creature that player controls. That creature deals damage equal to its power to the player. Prevent all combat damage that would be dealt to and dealt by the creature this turn."),
            ("Legendary Creature", "Whenever you attack, target attacking creature can't be blocked this turn. It connives. Then choose another attacking creature with lesser power. That creature gains double strike until end of turn."),
            ("Creature", "{B}, {T}: Choose target opponent who has at least two fewer creature cards in their graveyard than you do as you activate this ability. Destroy target nonblack creature that player controls."),
            ("Enchantment", "At the beginning of each player's upkeep, that player chooses target player whose graveyard has fewer creature cards in it than their graveyard does and is their opponent. The first player may return a creature card from their graveyard to their hand."),
            ("Enchantment", "At the beginning of each player's upkeep, that player chooses target player who controls more creatures than they do and is their opponent. The first player may reveal cards from the top of their library until they reveal a creature card. If the first player does, that player puts that card onto the battlefield and all other cards revealed this way into their graveyard."),
            ("Sorcery", "Each player puts a vow counter on a creature they control and sacrifices the rest. Each of those creatures can't attack you or planeswalkers you control for as long as it has a vow counter on it."),
            ("Instant", "Untap all creatures you control and all creatures target opponent controls. You and that opponent each gain control of all creatures the other controls until end of turn. Those creatures gain haste until end of turn."),
            ("Land", "{3}, {T}: Choose target permanent you control. Draw a card, then another player of your choice may gain control of that permanent. Activate only as a sorcery."),
            ("Creature", "At the beginning of your upkeep, sacrifice two Swamps. If you can't, tap ~, and an opponent may gain control of a creature you control of their choice for as long as ~ remains on the battlefield.\n{T}: Gain control of target creature for as long as ~ remains on the battlefield."),
            ("Creature", "First strike\n{T}: Reveal your hand and discard a card of target opponent's choice. Then that player reveals their hand and discards a card of your choice. Activate only as a sorcery."),
            ("Sorcery", "Each player with fewer cards in hand than the player with the most cards in hand draws cards equal to the difference."),
            ("Creature", "When ~ enters, each player who controls fewer lands than the player who controls the most lands searches their library for a number of basic land cards less than or equal to the difference, puts those cards onto the battlefield tapped, then shuffles."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }
}
