//! Choices of words that players make for the same object, each player's own choice read
//! later (CR 607.2d; `choices.rs` keeps what each player chose):
//!
//! - "choose war or peace", "choose money, friends, or secrets", "choose 1, 2, or 3": one
//!   of the listed words (or numbers), with any player as the subject ("each player
//!   chooses war or peace": in APNAP order, each knowing the earlier choices, CR 101.4,
//!   101.4b; Archangel of Strife's rulings);
//! - "players who chose war" (in an object phrase: "creatures controlled by players who
//!   chose war"), "each opponent who chose silence", "for each player who chose money,
//!   [instruction with 'that player']": [`PlayerFilter::Chose`];
//! - conditions "you chose [word]", "each opponent chose [word]".

use super::{ConditionPattern, EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::*;
use smol_str::SmolStr;

/// Words that make "choose ... or ..." something other than a choice among words.
const NOT_WORDS: [&str; 22] = [
    "one", "two", "three", "four", "both", "more", "any", "all", "each", "up", "target", "a",
    "an", "the", "left", "right", "odd", "even", "x", "new", "that", "this",
];

/// One listed word: one or two lowercase words of letters (or a number).
fn word_ok(w: &str) -> bool {
    let parts: Vec<&str> = w.split(' ').collect();
    !w.is_empty()
        && parts.len() <= 2
        && !NOT_WORDS.contains(&parts[0])
        && parts.iter().all(|p| {
            !p.is_empty()
                && !p.starts_with('-')
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

/// "war or peace", "money, friends, or secrets": the listed words.
pub fn word_list(s: &str) -> Option<Vec<String>> {
    let s = end(s);
    let words: Vec<String> = if let Some((a, b)) = s.split_once(", or ") {
        let mut v: Vec<String> = a.split(", ").map(str::to_string).collect();
        v.push(b.to_string());
        v
    } else {
        let (a, b) = s.split_once(" or ")?;
        if a.contains(',') {
            return None;
        }
        vec![a.to_string(), b.to_string()]
    };
    (words.len() >= 2 && words.iter().all(|w| word_ok(w))).then_some(words)
}

/// The choice among `words`: numbers in a run ("1, 2, or 3") are a number choice;
/// words naming colors or types are stored as such too.
pub fn word_choice(words: Vec<String>) -> ChoiceKind {
    let nums: Option<Vec<i32>> = words.iter().map(|w| w.parse().ok()).collect();
    if let Some(n) = nums {
        let (min, max) = (n[0], n[n.len() - 1]);
        if n.iter().enumerate().all(|(i, x)| *x == min + i as i32) {
            return ChoiceKind::Number { min, max };
        }
    }
    // Creature and land types are capitalized: "secretly choose Human, Merfolk, or Goblin".
    let cap = |w: &str| {
        let mut c = w.chars();
        c.next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default()
    };
    let subtypes: Vec<String> = words.iter().map(|w| cap(w)).collect();
    if subtypes
        .iter()
        .all(|w| crate::types::is_creature_type(w) || crate::types::is_land_type(w))
    {
        return ChoiceKind::OneOf(subtypes);
    }
    let typed = words.iter().any(|w| {
        crate::types::Color::from_word(w).is_some()
            || crate::types::CardType::from_word(w).is_some()
    });
    if typed {
        ChoiceKind::OneOf(words)
    } else {
        ChoiceKind::Word(words)
    }
}

/// "choose war or peace".
fn choose_words(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let words = word_list(r)?;
    Some(Effect::Choose {
        who: PlayerRef::You,
        kind: word_choice(words),
    })
}

inventory::submit! { EffectPattern { name: "choose one of the listed words", priority: 120, parse: choose_words } }

/// "chose war": the word a player chose.
fn chose_word(s: &str) -> Option<(SmolStr, &str)> {
    let r = s.trim_start().strip_prefix("chose ")?;
    // One word, up to a space or punctuation.
    let i = r.find([' ', ',', '.']).unwrap_or(r.len());
    let (w, rest) = (&r[..i], &r[i..]);
    word_ok(w).then(|| (SmolStr::new(w), rest))
}

/// "you chose war", "each opponent chose silence", "they chose snitch".
fn chose_condition(c: &str) -> Option<Condition> {
    let c = end(c);
    for (p, who, all) in [
        ("you ", PlayerRef::You, false),
        ("each opponent ", PlayerRef::EachOpponent, true),
        ("each player ", PlayerRef::EachPlayer, true),
        ("that player ", PlayerRef::Iterated, false),
    ] {
        let Some(r) = c.strip_prefix(p) else {
            continue;
        };
        let (w, rest) = chose_word(r)?;
        if !rest.is_empty() {
            return None;
        }
        return Some(if all {
            Condition::Not(Box::new(Condition::PlayerMatches(
                who,
                PlayerFilter::Not(Box::new(PlayerFilter::Chose(w))),
            )))
        } else {
            Condition::PlayerMatches(who, PlayerFilter::Chose(w))
        });
    }
    None
}

inventory::submit! { ConditionPattern { name: "a player chose a word", priority: 120, parse: chose_condition } }

/// "controlled by players who chose war" after an object noun.
fn controlled_by_choosers<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("controlled by players who ")
        .or_else(|| s.strip_prefix("controlled by a player who "))?;
    let (w, rest) = chose_word(r)?;
    Some((
        Filter::ControllerMatches(Box::new(PlayerFilter::Chose(w))),
        rest,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "controlled by players who chose a word", priority: 120, parse: controlled_by_choosers } }

/// "for each player who chose money, [instruction]": the instruction once for each
/// such player, "that player" being that player.
fn for_each_player_who_chose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("for each player who ")?;
    let (w, rest) = chose_word(r)?;
    let rest = rest.strip_prefix(", ")?;
    let saved = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let targets = b.targets.len();
    let e = parse_effect_text(rest, b);
    b.it_player = saved;
    if b.targets.len() != targets {
        b.targets.truncate(targets);
        return None;
    }
    Some(Effect::ForEachPlayer {
        who: PlayerRef::Each(PlayerFilter::Chose(w)),
        effect: Box::new(e?),
    })
}

inventory::submit! { EffectPattern { name: "for each player who chose a word", priority: 120, parse: for_each_player_who_chose } }

/// Whether `e` only makes choices (for an "as ~ enters" ability, whose choices are made
/// as the permanent enters, CR 614.12a).
fn only_choices(e: &Effect) -> bool {
    match e {
        Effect::Choose { .. } => true,
        Effect::Custom(n) => n == crate::choices::SECRET_NEXT,
        Effect::Seq(v) => !v.is_empty() && v.iter().all(only_choices),
        Effect::ForEachPlayer { effect, .. } | Effect::AsPlayer { effect, .. } => {
            only_choices(effect)
        }
        _ => false,
    }
}

/// "As ~ enters, each player chooses war or peace.", "As ~ enters, secretly choose an
/// opponent.": choices made as the permanent enters (CR 614.12a), by any players, that
/// the etb grammar (`etb_choices.rs`) doesn't word.
fn as_enters_choices(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || block.contains('\n') {
        return None;
    }
    let lower = block
        .to_lowercase()
        .replace("enters the battlefield", "enters");
    let body = end(&lower).strip_prefix("as ~ enters, ")?;
    let mut b = Builder::new(ctx);
    let e = parse_effect_text(body, &mut b)?;
    if !b.targets.is_empty() || !only_choices(&e) {
        return None;
    }
    let effect = StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(e)),
        self_replacement: false,
        optional: false,
    });
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        block,
    )])
}

inventory::submit! { super::AbilityPattern { name: "as ~ enters, players' choices", priority: 60, parse: as_enters_choices } }

/// Replaces each `from` in `v` with `to`, counting the replacements.
fn replace_json(
    v: serde_json::Value,
    from: &serde_json::Value,
    to: &serde_json::Value,
    n: &mut usize,
) -> serde_json::Value {
    use serde_json::Value as J;
    if v == *from {
        *n += 1;
        return to.clone();
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| replace_json(x, from, to, n)).collect()),
        J::Object(m) => J::Object(
            m.into_iter()
                .map(|(k, x)| (k, replace_json(x, from, to, n)))
                .collect(),
        ),
        x => x,
    }
}

/// "each opponent who chose silence" anywhere in an instruction ("~ deals 12 damage to
/// each opponent who chose silence", "each player who chose war draws a card"): the
/// instruction for "each opponent" / "each player", performed for those who chose it.
fn each_player_who_chose(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (phrase, who, filter) = [
        ("each opponent who ", PlayerRef::EachOpponent, PlayerFilter::Opponent),
        ("each player who ", PlayerRef::EachPlayer, PlayerFilter::Any),
    ]
    .into_iter()
    .find(|(p, _, _)| l.contains(&format!("{p}chose ")))?;
    let at = l.find(&format!("{phrase}chose "))?;
    let (w, rest) = chose_word(&l[at + phrase.len()..])?;
    let plain_subject = phrase.trim_end().trim_end_matches(" who");
    // The plain phrase mustn't appear elsewhere in the instruction.
    if l.matches(plain_subject).count() != 1 {
        return None;
    }
    let text = format!("{}{plain_subject}{rest}", &l[..at]);
    let targets = b.targets.len();
    let e = crate::oracle::effects::parse_clause(&text, b);
    let Some(e) = e else {
        b.targets.truncate(targets);
        return None;
    };
    let from = serde_json::to_value(&who).ok()?;
    let to = serde_json::to_value(PlayerRef::Each(PlayerFilter::And(vec![
        filter,
        PlayerFilter::Chose(w),
    ])))
    .ok()?;
    let mut n = 0;
    let v = replace_json(serde_json::to_value(&e).ok()?, &from, &to, &mut n);
    if n != 1 {
        b.targets.truncate(targets);
        return None;
    }
    serde_json::from_value(v).ok()
}

inventory::submit! { EffectPattern { name: "each player who chose a word", priority: 120, parse: each_player_who_chose } }

/// "if each opponent chose silence, ~ deals 4 damage to each of them": "them" is those
/// players.
fn if_each_chose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let group = ["each opponent", "each player"]
        .into_iter()
        .find(|g| c.starts_with(&format!("{g} chose ")))?;
    if !rest.contains("each of them") {
        return None;
    }
    let cond = chose_condition(c)?;
    let rest = rest.replace("each of them", group);
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(&rest, b) else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "if each opponent chose a word, ... each of them", priority: 120, parse: if_each_chose } }

/// Whether `c` is only about what players chose.
fn about_choices(c: &Condition) -> bool {
    match c {
        Condition::PlayerMatches(_, f) => player_filter_about_choices(f),
        Condition::Not(x) => about_choices(x),
        Condition::And(v) | Condition::Or(v) => !v.is_empty() && v.iter().all(about_choices),
        _ => false,
    }
}

fn player_filter_about_choices(f: &PlayerFilter) -> bool {
    match f {
        PlayerFilter::Chose(_) => true,
        PlayerFilter::Not(x) => player_filter_about_choices(x),
        _ => false,
    }
}

/// For a run of conditional instructions about what players chose ("If each opponent
/// chose silence, .... If each opponent chose snitch, .... Otherwise, ..."): the key
/// grouping them, so that "otherwise" is the alternative to all of them
/// (`conditional_followups::group_condition_run`).
pub(crate) fn choice_run_key(c: &Condition) -> Option<String> {
    about_choices(c).then(|| "player choices".to_string())
}

// ---------------------------------------------------------------------------
// "choose two players": the chosen players
// ---------------------------------------------------------------------------

/// "choose two players".
fn choose_players(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(n @ 2..=4) = n else {
        return None;
    };
    (r.trim() == "players").then_some(Effect::Choose {
        who: PlayerRef::You,
        kind: ChoiceKind::Players(n as u8),
    })
}

inventory::submit! { EffectPattern { name: "choose two players", priority: 120, parse: choose_players } }

/// "damage is dealt to one of the chosen players" (Sower of Discord): "that player" is the
/// player dealt damage.
fn damage_dealt_to_chosen(t: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let t = end(t);
    let t = t.strip_prefix("whenever ").unwrap_or(t);
    if t != "damage is dealt to one of the chosen players" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerDealtDamage {
                who: PlayerRel::Any,
                combat_only: false,
            }),
            cond: Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::ChosenPlayers),
        },
        Sel::None,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { super::TriggerPattern { name: "damage is dealt to one of the chosen players", priority: 60, parse: damage_dealt_to_chosen } }

/// "the other chosen player": the chosen players other than the one the triggering event
/// is about.
fn other_chosen_player() -> PlayerRef {
    PlayerRef::Each(PlayerFilter::And(vec![
        PlayerFilter::ChosenPlayers,
        PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(
            PlayerRef::TriggerPlayer,
        )))),
    ]))
}

/// "the other chosen player also loses that much life".
fn other_chosen_loses(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the other chosen player ")?;
    let r = r.strip_prefix("also ").unwrap_or(r);
    let pred = super::player_subjects::as_you(r)?;
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(&format!("you {pred}"), b) else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::ForEachPlayer {
        who: other_chosen_player(),
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    })
}

inventory::submit! { EffectPattern { name: "the other chosen player [instruction]", priority: 120, parse: other_chosen_loses } }

/// "If a source controlled by one of the chosen players would deal damage to the other
/// chosen player or a permanent that player controls, that source deals double that
/// damage to that player or permanent instead." (Bitter Feud)
fn chosen_players_double_damage(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if end(l)
        != "if a source controlled by one of the chosen players would deal damage to the other chosen player or a permanent that player controls, that source deals double that damage to that player or permanent instead"
    {
        return None;
    }
    let chosen = PlayerFilter::ChosenPlayers;
    let source_controller = PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject));
    // The source's controller is one of the chosen players and the recipient (a player,
    // or a permanent's controller) is the other one.
    let other_than = |r: PlayerRef| {
        PlayerFilter::And(vec![
            chosen.clone(),
            PlayerFilter::Not(Box::new(PlayerFilter::Ref(Box::new(r)))),
        ])
    };
    let to_player = Condition::And(vec![
        Condition::PlayerMatches(PlayerRef::TriggerPlayer, chosen.clone()),
        Condition::PlayerMatches(source_controller.clone(), other_than(PlayerRef::TriggerPlayer)),
    ]);
    let recipient_controller = PlayerRef::ControllerOf(Box::new(Sel::TriggerObject));
    let to_permanent = Condition::And(vec![
        Condition::SelNonEmpty(Sel::TriggerObject),
        Condition::PlayerMatches(recipient_controller.clone(), chosen.clone()),
        Condition::PlayerMatches(source_controller, other_than(recipient_controller)),
    ]);
    let event = ReplacementEvent::Where {
        event: Box::new(ReplacementEvent::Damage {
            source: Filter::Any,
            to_players: Some(PlayerFilter::Any),
            to_objects: Some(Filter::Permanent),
            combat_only: false,
        }),
        cond: Condition::Or(vec![to_player, to_permanent]),
    };
    let effect = StaticEffect::Replacement(ReplacementDef {
        event,
        action: ReplacementAction::Multiply(2),
        self_replacement: false,
        optional: false,
    });
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "Bitter Feud: double damage between the chosen players", priority: 60, parse: chosen_players_double_damage } }

// ---------------------------------------------------------------------------
// Several chosen names
// ---------------------------------------------------------------------------

/// "you and an opponent each choose a card name other than a basic land card name": you
/// choose, then an opponent you choose does (CR 101.4).
fn you_and_an_opponent_each_choose(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you and an opponent each choose ")?;
    let targets = b.targets.len();
    let Some(mine @ Effect::Choose { .. }) =
        crate::oracle::effects::parse_clause(&format!("choose {r}"), b)
    else {
        b.targets.truncate(targets);
        return None;
    };
    Some(Effect::seq(vec![
        mine.clone(),
        Effect::Choose {
            who: PlayerRef::You,
            kind: ChoiceKind::Opponent,
        },
        Effect::AsPlayer {
            who: PlayerRef::ChosenOpponent,
            effect: Box::new(mine),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "you and an opponent each choose", priority: 120, parse: you_and_an_opponent_each_choose } }

/// "with a name chosen for ~", "with the chosen names" after an object noun.
fn with_chosen_names<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s
        .strip_prefix("with a name chosen for ~")
        .or_else(|| s.strip_prefix("with the chosen names"))?;
    (r.is_empty() || r.starts_with([' ', ',', '.']))
        .then(|| (Filter::Custom(crate::choices::CHOSEN_NAMES.into()), r))
}

inventory::submit! { FilterSuffixPattern { name: "with a name chosen for ~", priority: 120, parse: with_chosen_names } }

/// "Spells with the chosen names can't be cast and lands with the chosen names can't be
/// played." (Null Chamber; the plural of Conjurer's Ban's).
fn chosen_names_cant_be_cast(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if end(l)
        != "spells with the chosen names can't be cast and lands with the chosen names can't be played"
    {
        return None;
    }
    let names = || Filter::Custom(crate::choices::CHOSEN_NAMES.into());
    let st = |r: Restriction| {
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
            text,
        )
    };
    Some(vec![
        st(Restriction::CantCast {
            who: PlayerFilter::Any,
            what: Filter::and(vec![Filter::Card, names()]),
        }),
        st(Restriction::CantPlayLandCards {
            who: PlayerFilter::Any,
            what: Filter::and(vec![Filter::Type(crate::types::CardType::Land), names()]),
        }),
    ])
}

inventory::submit! { super::StaticPattern { name: "spells with the chosen names can't be cast", priority: 60, parse: chosen_names_cant_be_cast } }

/// "exile each permanent unless it's colorless or it's only the color its controller
/// chose" (Selective Obliteration): the permanents that are neither.
fn each_unless_its(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, cond) = l.split_once(" unless it's ")?;
    let mut quals = Vec::new();
    for q in cond.split(" or it's ") {
        quals.push(match q {
            "colorless" => Filter::Colorless,
            "only the color its controller chose" => {
                Filter::Custom(crate::choices::ONLY_COLOR_ITS_CONTROLLER_CHOSE.into())
            }
            _ => return None,
        });
    }
    let noun = head
        .strip_prefix("exile each ")
        .or_else(|| head.strip_prefix("destroy each "))?;
    let (f, false, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let what = Sel::All(Filter::and(vec![
        f,
        Filter::InZone(ZoneKind::Battlefield),
        Filter::not(Filter::Or(quals)),
    ]));
    let _ = b;
    Some(if head.starts_with("exile") {
        Effect::Exile {
            what,
            face_down: false,
            link: false,
        }
    } else {
        Effect::Destroy {
            what,
            no_regen: false,
        }
    })
}

inventory::submit! { EffectPattern { name: "exile each permanent unless it's [qualities]", priority: 120, parse: each_unless_its } }


/// Whether `e` chooses among card type words with a qualified one ("artifact, creature,
/// land, or non-Aura enchantment").
fn chooses_type_words(e: &Effect) -> bool {
    match e {
        Effect::Choose {
            kind: ChoiceKind::OneOf(w),
            ..
        } => w.iter().any(|x| x.starts_with("non-")),
        Effect::AsPlayer { effect, .. } => chooses_type_words(effect),
        Effect::Seq(v) => v.last().is_some_and(chooses_type_words),
        _ => false,
    }
}

/// Replaces "the chosen type" in `v` with the chosen card type word.
fn type_word_filters(v: serde_json::Value, n: &mut usize) -> serde_json::Value {
    use serde_json::Value as J;
    if v == J::String("ChosenType".into()) {
        *n += 1;
        return serde_json::to_value(Filter::Custom(
            crate::choices::OF_THE_CHOSEN_TYPE_WORD.into(),
        ))
        .unwrap_or(v);
    }
    match v {
        J::Array(a) => J::Array(a.into_iter().map(|x| type_word_filters(x, n)).collect()),
        J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, type_word_filters(x, n))).collect()),
        x => x,
    }
}

/// "... chooses artifact, creature, land, or non-Aura enchantment. All nontoken
/// permanents of that type phase out." (Teferi's Realm): "that type" is the chosen word's
/// type, "non-Aura enchantment" being enchantments that aren't Auras.
fn of_that_type_word(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !l.contains("of that type") || !chooses_type_words(prev) {
        return false;
    }
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(l, b) else {
        b.targets.truncate(targets);
        return false;
    };
    let mut n = 0;
    let Ok(v) = serde_json::to_value(&e) else {
        return false;
    };
    let v = type_word_filters(v, &mut n);
    let Ok(e) = serde_json::from_value::<Effect>(v) else {
        return false;
    };
    if n == 0 {
        b.targets.truncate(targets);
        return false;
    }
    let first = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![first, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "of that type (a chosen card type word)", priority: 20, apply: of_that_type_word } }

/// "Then each player reveals the top card of their library. If the card a player revealed
/// has the name they chose, that player puts it into their hand. If it doesn't, that
/// player puts it on the bottom of their library." (Conundrum Sphinx): each player
/// reveals their top card and moves it by their own chosen name.
fn revealed_has_the_name_they_chose(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("if the card a player revealed has the name they chose, that player ") else {
        return false;
    };
    const REVEAL: &str = "parley:each player reveals the top card of their library";
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(last) = last else {
        return false;
    };
    if !matches!(last, Effect::Custom(n) if n == REVEAL) {
        return false;
    }
    let Some(pred) = super::player_subjects::as_you(rest) else {
        return false;
    };
    let text = format!(
        "reveal the top card of your library. if it has the chosen name, {pred}"
    );
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_effect_text(&text, b) else {
        b.targets.truncate(targets);
        return false;
    };
    let mut n = 0;
    let from = serde_json::Value::String("ChosenName".into());
    let to = serde_json::to_value(Filter::Custom(crate::choices::NAME_THEY_CHOSE.into()))
        .unwrap_or(serde_json::Value::Null);
    let Ok(v) = serde_json::to_value(&e) else {
        return false;
    };
    let v = replace_json(v, &from, &to, &mut n);
    let Ok(e) = serde_json::from_value::<Effect>(v) else {
        return false;
    };
    if n != 1 {
        b.targets.truncate(targets);
        return false;
    }
    *last = Effect::ForEachPlayer {
        who: PlayerRef::EachPlayer,
        effect: Box::new(Effect::AsPlayer {
            who: PlayerRef::Iterated,
            effect: Box::new(e),
        }),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "if the card a player revealed has the name they chose", priority: 20, apply: revealed_has_the_name_they_chose } }

/// "If it doesn't, that player puts it on the bottom of their library." after the one
/// above: the other case, for each player.
fn if_it_doesnt_that_player(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("if it doesn't, that player ") else {
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::ForEachPlayer { effect, .. }) = last else {
        return false;
    };
    let Effect::AsPlayer { effect: inner, .. } = effect.as_mut() else {
        return false;
    };
    let iff = match inner.as_mut() {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::If { cond, otherwise, .. }) = iff else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop)
        || !format!("{cond:?}").contains(crate::choices::NAME_THEY_CHOSE)
    {
        return false;
    }
    let Some(pred) = super::player_subjects::as_you(rest) else {
        return false;
    };
    let targets = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_clause(&pred, b) else {
        b.targets.truncate(targets);
        return false;
    };
    **otherwise = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "if it doesn't, that player (per player)", priority: 20, apply: if_it_doesnt_that_player } }
