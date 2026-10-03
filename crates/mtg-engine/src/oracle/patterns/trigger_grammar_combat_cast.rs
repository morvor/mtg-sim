//! Trigger grammar II: cast, play, activation, attack, block, damage and targeting trigger
//! conditions the compositional parser in `triggers.rs` doesn't cover.
//!
//! Casting (CR 601.2i) and playing (glossary "Play"):
//!
//! * `[player] cast(s) [spell] or [spell]` ("a noncreature spell or a Dragon spell", "a
//!   noncreature or Dragon spell"): either description.
//! * `[player] cast(s) [spell] or [other verb]` ("cast an instant or sorcery spell or
//!   activate an ability", "casts a spell other than … or copies a spell", "cast or cycle
//!   ~"): either event (CR 603.2c: one event triggers it once).
//! * `[player] cast(s) your/their [Nth] [spell] each turn`: the Nth such spell that player
//!   cast this turn; `your first spell during each of your turns`; passive forms `the
//!   [Nth] [spell] of a turn is cast`, `[a spell] is cast during your turn`.
//! * Spell qualifiers ([`spell_qualifier`], used by `triggers::parse_spell_phrase`): `that
//!   targets [only] …`, `from their hand`, `from anywhere other than exile`, `other than
//!   your first spell each turn` (CR 601.2i: it counts), `during combat`, `during your main
//!   phase`, `during their turn`, `that has flash`, `they don't own`, `with the same name
//!   as a card in their graveyard`, `with one or more [color] mana symbols in its mana
//!   cost`, `with a mana cost that contains {X}`.
//! * `[player] play(s) [a land phrase] [from exile]`, `play(s) a land … or cast(s) a spell
//!   …`.
//!
//! Players: `you`, `an opponent`, `a player`, `another player`, `the chosen player`,
//! `enchanted player`.

use super::{FilterSuffixPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::phrases::*;
use crate::keywords::KeywordKind;
use crate::types::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

/// A complete trigger condition (the text after "when"/"whenever").
fn reparse(s: &str) -> Option<Parsed> {
    crate::oracle::triggers::parse_trigger_condition(&format!("whenever {s}"))
}

fn word_end(r: &str) -> bool {
    r.is_empty() || r.starts_with([' ', ',', '.'])
}

/// The player a trigger's player subject names: a relation, and maybe a further
/// restriction ("enchanted player": a player, if it's the enchanted one).
struct PlayerSubject {
    rel: PlayerRel,
    only: Option<PlayerFilter>,
    /// The subject as the compositional parser reads it ("a player").
    word: &'static str,
}

fn player_subject(r: &str) -> Option<(PlayerSubject, &str)> {
    let enchanted = || {
        Some(PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(
            Box::new(Sel::AttachedTo),
        ))))
    };
    for (p, rel, only, word) in [
        ("you ", PlayerRel::You, None, "you"),
        ("an opponent ", PlayerRel::Opponent, None, "an opponent"),
        ("a player ", PlayerRel::Any, None, "a player"),
        ("another player ", PlayerRel::NotYou, None, "another player"),
        // The player chosen as the source entered (CR 607.2d).
        ("the chosen player ", PlayerRel::Chosen, None, "the chosen player"),
        ("enchanted player ", PlayerRel::Any, enchanted(), "a player"),
    ] {
        if let Some(x) = r.strip_prefix(p) {
            return Some((PlayerSubject { rel, only, word }, x));
        }
    }
    None
}

/// Restricts a player event to the subject's player.
fn only_player(c: TriggerCond, only: &Option<PlayerFilter>) -> TriggerCond {
    match only {
        None => c,
        Some(f) => super::trigger_grammar_events::where_events(
            c,
            Condition::PlayerMatches(PlayerRef::TriggerPlayer, f.clone()),
        ),
    }
}

/// Merges alternative trigger conditions (CR 603.2c: an event matching several of them
/// triggers the ability once): "it" and "that player" only where they agree.
fn any_of(parts: Vec<Parsed>) -> Parsed {
    let same_it = parts
        .windows(2)
        .all(|w| format!("{:?}", w[0].1) == format!("{:?}", w[1].1));
    let same_player = parts
        .windows(2)
        .all(|w| format!("{:?}", w[0].2) == format!("{:?}", w[1].2));
    let it = if same_it {
        parts[0].1.clone()
    } else {
        Sel::None
    };
    let player = if same_player {
        parts[0].2.clone()
    } else {
        PlayerRef::Iterated
    };
    let mut conds = Vec::new();
    for (c, _, _) in parts {
        match c {
            TriggerCond::AnyOf(v) => conds.extend(v),
            c => conds.push(c),
        }
    }
    (TriggerCond::AnyOf(conds), it, player)
}

// ---------------------------------------------------------------------------
// Casting and playing
// ---------------------------------------------------------------------------

/// "cast", "casts" at the start of `s`.
fn cast_verb(s: &str) -> Option<&str> {
    s.strip_prefix("casts ").or_else(|| s.strip_prefix("cast "))
}

/// Other verbs that can follow "cast … or": the second event of the trigger.
const SECOND_VERBS: [&str; 9] = [
    " or activate ",
    " or activates ",
    " or copy ",
    " or copies ",
    " or cycle ",
    " or cycles ",
    " or play ",
    " or plays ",
    " or cast ",
];

fn cast_events(r: &str) -> Option<Parsed> {
    let (subj, rest) = player_subject(r)?;
    // "you cast or cycle ~" (Warped Tusker): either event.
    if let Some(x) = rest.strip_prefix("cast or cycle ") {
        let a = reparse(&format!("{} cast {x}", subj.word))?;
        let b = reparse(&format!("{} cycle {x}", subj.word))?;
        return Some(any_of(vec![a, b]));
    }
    let t = cast_verb(rest)?;
    // "[cast …] or [another verb] …": either event.
    for v in SECOND_VERBS {
        if v.contains("cast ") {
            continue;
        }
        if let Some((a, b)) = t.split_once(v) {
            let first = reparse(&format!("{} cast {a}", subj.word))
                .or_else(|| cast_spell(&subj, a))?;
            let second = reparse(&format!("{} {}{b}", subj.word, v.trim_start_matches(" or ")))?;
            let (c, it, p) = any_of(vec![first, second]);
            return Some((only_player(c, &subj.only), it, p));
        }
    }
    if subj.only.is_some() {
        // "enchanted player casts …", "the chosen player casts …": the same event of "a
        // player", for that player only.
        let (c, it, p) = reparse(&format!("{} cast {t}", subj.word)).or_else(|| cast_spell(&subj, t))?;
        if !matches!(p, PlayerRef::TriggerPlayer) {
            return None;
        }
        return Some((only_player(c, &subj.only), it, p));
    }
    cast_spell(&subj, t)
}

/// What a player casts: a spell description and its qualifiers, alternatives, or an
/// ordinal.
fn cast_spell(subj: &PlayerSubject, t: &str) -> Option<Parsed> {
    let who = subj.rel;
    let t = end(t);
    let spell = |filter: Filter, cond: Option<Condition>| {
        let c = TriggerCond::CastSpell { who, filter };
        let c = match cond {
            Some(cond) => TriggerCond::Where {
                trigger: Box::new(c),
                cond,
            },
            None => c,
        };
        (c, Sel::TriggerSpell, PlayerRef::TriggerPlayer)
    };
    // "~ from your hand", "~ from anywhere other than exile", "~ while …".
    if let Some(x) = t.strip_prefix("~ ") {
        let (f, cond) = qualifiers_only(x, Filter::Source)?;
        let (c, _, p) = spell(f, cond);
        return Some((c, Sel::This, p));
    }
    // "your commander" (CR 903.3).
    if t == "your commander" {
        return Some(spell(
            Filter::and(vec![Filter::Commander, Filter::OwnedBy(PlayerRel::You)]),
            None,
        ));
    }
    // "your fourth noncreature spell each turn", "your first spell during each of your
    // turns".
    if let Some(x) = t.strip_prefix("your ").or_else(|| t.strip_prefix("their ")) {
        let (w, x) = split_word(x);
        let n = ordinal(w)?;
        if let Some(phrase) = x.strip_suffix(" each turn") {
            let (f, cond) = super::triggers::parse_spell_phrase(phrase)?;
            if cond.is_some() {
                return None;
            }
            let count = Value::SpellsCastThisTurn(PlayerRef::TriggerPlayer, spell_type(&f));
            let (c, it, p) = spell(
                f,
                Some(Condition::Compare(count, Cmp::Eq, Value::c(n as i32))),
            );
            return Some((c, it, p));
        }
        if x == "spell during each of your turns" && n == 1 {
            let c = TriggerCond::Where {
                trigger: Box::new(TriggerCond::NthSpellCast { who, n }),
                cond: Condition::YourTurn,
            };
            return Some((c, Sel::TriggerSpell, PlayerRef::TriggerPlayer));
        }
        return None;
    }
    // "a noncreature spell or a Dragon spell", "an Equipment spell or a spell that targets
    // a creature you control".
    for sep in [" or a ", " or an "] {
        let mut from = 0;
        while let Some(i) = t[from..].find(sep).map(|i| i + from) {
            let (a, b) = (&t[..i], &t[i + " or ".len()..]);
            if let (Some(x), Some(y)) = (one_spell(a), one_spell(b)) {
                return Some(spell(Filter::Or(vec![cond_free(x)?, cond_free(y)?]), None));
            }
            from = i + 1;
        }
    }
    // "a noncreature or Dragon spell": "a noncreature spell" or "a Dragon spell".
    if let Some(x2) = t
        .strip_suffix(" spell")
        .and_then(|x| x.strip_prefix("a ").or_else(|| x.strip_prefix("an ")))
    {
        if let Some((a, b)) = x2.split_once(" or ") {
            if !b.contains(" or ") {
                let fa = one_spell(&format!("a {a} spell")).and_then(cond_free);
                let fb = one_spell(&format!("a {b} spell")).and_then(cond_free);
                if let (Some(fa), Some(fb)) = (fa, fb) {
                    // Only where the object phrase can't read the list itself.
                    if parse_object_phrase(x2).is_none_or(|(_, _, tail)| !end(tail).is_empty()) {
                        return Some(spell(Filter::Or(vec![fa, fb]), None));
                    }
                }
            }
        }
    }
    // A plain description ("a spell from their hand") for a player the compositional
    // parser doesn't name.
    let (f, cond) = one_spell(t)?;
    Some(spell(f, cond))
}

/// The card types of a spell description, for counting earlier spells of the turn (which
/// may have left the stack since): "noncreature spell" → noncreature.
fn spell_type(f: &Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.iter().map(spell_type).collect()),
        other => other.clone(),
    }
}

/// "a [spell phrase]" / "another [spell phrase]" with its qualifiers.
fn one_spell(s: &str) -> Option<(Filter, Option<Condition>)> {
    if let Some(x) = s.strip_prefix("another ") {
        // Other than this object, if it's a spell (a commander's eminence ability).
        let (f, c) = one_spell(&format!("a {x}"))?;
        return Some((Filter::and(vec![Filter::Other, f]), c));
    }
    let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    // Qualifiers the object phrase would misread ("with the same name as a card" is a
    // name comparison of its own).
    for q in [" with the same name as a card in their graveyard"] {
        if let Some(head) = x.strip_suffix(q) {
            let (f, c) = super::triggers::parse_spell_phrase(head)?;
            let (fs, c2, r) = spell_qualifier(q.trim_start(), &f)?;
            if !r.is_empty() {
                return None;
            }
            let mut parts = vec![f];
            parts.extend(fs);
            let cond = match (c, c2) {
                (Some(a), Some(b)) => Some(Condition::And(vec![a, b])),
                (a, b) => a.or(b),
            };
            return Some((Filter::and(parts), cond));
        }
    }
    super::triggers::parse_spell_phrase(x)
}

fn cond_free((f, c): (Filter, Option<Condition>)) -> Option<Filter> {
    c.is_none().then_some(f)
}

/// Qualifiers after a spell named directly ("~ from your hand").
fn qualifiers_only(x: &str, base: Filter) -> Option<(Filter, Option<Condition>)> {
    let mut parts = vec![base];
    let mut cond: Option<Condition> = None;
    let mut rest = x;
    loop {
        let t = rest.trim_start();
        if t.is_empty() {
            break;
        }
        let (f, c, r) = spell_qualifier(t, &Filter::and(parts.clone()))?;
        parts.extend(f);
        if let Some(c) = c {
            cond = Some(match cond {
                Some(p) => Condition::And(vec![p, c]),
                None => c,
            });
        }
        rest = r;
    }
    Some((Filter::and(parts), cond))
}

fn ordinal(w: &str) -> Option<u32> {
    Some(match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    })
}

/// The number of spells matching `f` cast this turn by the casting player, at least 2:
/// "other than your first spell each turn".
fn not_first(f: Filter) -> Condition {
    Condition::Compare(
        Value::SpellsCastThisTurn(PlayerRef::TriggerPlayer, f),
        Cmp::Ge,
        Value::c(2),
    )
}

/// What a spell's "that targets [only] …" names: a player, an object, or "[player] or
/// [object]".
fn targeted(s: &str) -> Option<(Option<Filter>, Option<PlayerFilter>)> {
    if let Some(x) = super::r115_targets::targeted_thing(s) {
        return Some(x);
    }
    // "an opponent or a creature an opponent controls"
    let (p, o) = s.split_once(" or ")?;
    let (None, Some(players)) = super::r115_targets::targeted_thing(p)? else {
        return None;
    };
    let (Some(objects), None) = super::r115_targets::targeted_thing(o)? else {
        return None;
    };
    Some((Some(objects), Some(players)))
}

/// A qualifier on a spell being cast, at the start of `t`: a filter on the spell and/or a
/// condition on the moment it's cast (checked with the cast event), and the rest of the
/// text. `so_far` is the spell description before it.
pub(crate) fn spell_qualifier<'a>(
    t: &'a str,
    so_far: &Filter,
) -> Option<(Vec<Filter>, Option<Condition>, &'a str)> {
    let filter = |f: Filter, r: &'a str| Some((vec![f], None, r));
    let cond = |c: Condition, r: &'a str| Some((vec![], Some(c), r));
    // "that targets only ~", "that targets you or a creature you control" (CR 115.9b-c).
    for (p, only) in [("that targets only ", true), ("that targets ", false)] {
        let Some(x) = t.strip_prefix(p) else {
            continue;
        };
        // The rest of the trigger condition is the target description (qualifiers after
        // it aren't separable from it).
        let (objects, players) = targeted(x)?;
        // "that targets ~" / "that targets a creature" stay `Filter::Targets` (heroic's
        // "it" is ~): only targets with players or "only" are read here.
        if !only && players.is_none() {
            return None;
        }
        let tf = if only {
            TargetsFilter::Only { objects, players }
        } else {
            TargetsFilter::Targets { objects, players }
        };
        return filter(Filter::StackTargets(Box::new(tf)), "");
    }
    if let Some(r) = t.strip_prefix("from their hand").filter(|r| word_end(r)) {
        return filter(Filter::CastFrom(ZoneKind::Hand), r);
    }
    if let Some(r) = t
        .strip_prefix("from anywhere other than exile")
        .filter(|r| word_end(r))
    {
        return filter(Filter::not(Filter::CastFrom(ZoneKind::Exile)), r);
    }
    if let Some(r) = t.strip_prefix("from your hand").filter(|r| word_end(r)) {
        return filter(Filter::CastFrom(ZoneKind::Hand), r);
    }
    if let Some(r) = t.strip_prefix("during combat").filter(|r| word_end(r)) {
        return cond(Condition::Phase(PhaseCond::Combat), r);
    }
    if let Some(r) = t
        .strip_prefix("during your main phase")
        .filter(|r| word_end(r))
    {
        return cond(
            Condition::And(vec![
                Condition::YourTurn,
                Condition::Phase(PhaseCond::MainPhase),
            ]),
            r,
        );
    }
    if let Some(r) = t.strip_prefix("during their turn").filter(|r| word_end(r)) {
        return cond(
            Condition::PlayerMatches(PlayerRef::TriggerPlayer, PlayerFilter::Active),
            r,
        );
    }
    // "other than your first spell each turn", "other than the first spell they cast
    // each turn", "other than your first spell that turn", "other than the first instant
    // spell that player casts each turn".
    if let Some(x) = t.strip_prefix("other than ") {
        let x = x
            .strip_prefix("your first ")
            .or_else(|| x.strip_prefix("the first "))?;
        let (phrase, r) = [
            " that player casts each turn",
            " they cast each turn",
            " each turn",
            " that turn",
        ]
        .iter()
        .find_map(|s| {
            let i = x.find(s)?;
            Some((&x[..i], &x[i + s.len()..]))
        })?;
        let (f, c) = super::triggers::parse_spell_phrase(phrase)?;
        if c.is_some() || !word_end(r) {
            return None;
        }
        // The spell is one of them: only a spell of that kind can be other than the first.
        let _ = so_far;
        return cond(not_first(spell_type(&f)), r);
    }
    if let Some(r) = t
        .strip_prefix("that has flash")
        .or_else(|| t.strip_prefix("that have flash"))
        .filter(|r| word_end(r))
    {
        return filter(Filter::HasKeyword(KeywordKind::Flash), r);
    }
    if let Some(r) = t
        .strip_prefix("they don't own")
        .or_else(|| t.strip_prefix("that player doesn't own"))
        .filter(|r| word_end(r))
    {
        return cond(
            Condition::Not(Box::new(Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::OwnedByPlayer(Box::new(PlayerRef::TriggerPlayer)),
            ))),
            r,
        );
    }
    if let Some(r) = t
        .strip_prefix("with the same name as a card in their graveyard")
        .filter(|r| word_end(r))
    {
        let cards = Filter::and(vec![
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedByPlayer(Box::new(PlayerRef::TriggerPlayer)),
        ]);
        return cond(
            Condition::SelMatches(
                Sel::TriggerSpell,
                Filter::SameNameAs(Box::new(Sel::All(cards))),
            ),
            r,
        );
    }
    if let Some(r) = t
        .strip_prefix("with a mana cost that contains {x}")
        .filter(|r| word_end(r))
    {
        return filter(Filter::HasX, r);
    }
    // "with one or more blue mana symbols in its mana cost" (CR 107.4e: hybrid symbols of
    // the color count).
    if let Some(x) = t.strip_prefix("with one or more ") {
        let (w, x) = split_word(x);
        let color = Color::from_word(w)?;
        let r = x
            .trim_start()
            .strip_prefix("mana symbols in its mana cost")
            .filter(|r| word_end(r))?;
        let symbols = Value::Aggregate(
            AggOp::Sum,
            Stat::ManaSymbols(color),
            Box::new(Sel::Var(vars::TESTED)),
        );
        return filter(
            Filter::ValueCmp(Box::new(symbols), Cmp::Ge, Box::new(Value::c(1))),
            r,
        );
    }
    None
}

/// "[object] that's exactly two colors" / "that isn't exactly two colors".
fn exactly_two_colors<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let pairs = || {
        Filter::Or(
            ColorSet::color_pairs()
                .into_iter()
                .map(Filter::ExactColors)
                .collect(),
        )
    };
    for (p, yes) in [
        ("that's exactly two colors", true),
        ("that are exactly two colors", true),
        ("that isn't exactly two colors", false),
        ("that aren't exactly two colors", false),
    ] {
        if let Some(r) = t.strip_prefix(p).filter(|r| word_end(r)) {
            return Some((if yes { pairs() } else { Filter::not(pairs()) }, r));
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "that's exactly two colors", priority: 100, parse: exactly_two_colors } }

/// Passive cast triggers: "an instant or sorcery spell is cast during your turn", "the
/// first noncreature spell of a turn is cast", "the fourth spell of a turn is cast".
fn spell_is_cast(r: &str) -> Option<Parsed> {
    let x = r.strip_suffix(" is cast").or_else(|| {
        r.strip_suffix(" is cast during your turn")
            .filter(|_| r.starts_with("a ") || r.starts_with("an "))
    })?;
    let your_turn = r.ends_with(" during your turn");
    if let Some(y) = x.strip_prefix("the ") {
        let (w, y) = split_word(y);
        let n = ordinal(w)?;
        let phrase = y.trim_start().strip_suffix(" of a turn")?;
        let (f, c) = super::triggers::parse_spell_phrase(phrase)?;
        if c.is_some() {
            return None;
        }
        let count = Value::SpellsCastThisTurn(PlayerRef::EachPlayer, spell_type(&f));
        return Some((
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::CastSpell {
                    who: PlayerRel::Any,
                    filter: f,
                }),
                cond: Condition::Compare(count, Cmp::Eq, Value::c(n as i32)),
            },
            Sel::TriggerSpell,
            PlayerRef::TriggerPlayer,
        ));
    }
    let (f, c) = one_spell(x)?;
    let mut conds: Vec<Condition> = c.into_iter().collect();
    if your_turn {
        conds.push(Condition::YourTurn);
    }
    let base = TriggerCond::CastSpell {
        who: PlayerRel::Any,
        filter: f,
    };
    let c = if conds.is_empty() {
        base
    } else {
        TriggerCond::Where {
            trigger: Box::new(base),
            cond: if conds.len() == 1 {
                conds.pop()?
            } else {
                Condition::And(conds)
            },
        }
    };
    Some((c, Sel::TriggerSpell, PlayerRef::TriggerPlayer))
}

/// "[player] play(s) [a land phrase] [from exile]" (CR 305.1).
fn play_land(subj: &PlayerSubject, t: &str) -> Option<Parsed> {
    let x = t.strip_prefix("a ").or_else(|| t.strip_prefix("an "))?;
    let (mut f, plural, tail) = parse_object_phrase(x)?;
    if plural {
        return None;
    }
    let tail = end(tail);
    let tail = if let Some(r) = tail.strip_prefix("from exile") {
        f = Filter::and(vec![
            f,
            Filter::Custom(crate::kw::played_from_zone::came_from(ZoneKind::Exile).into()),
        ]);
        r
    } else {
        tail
    };
    if !end(tail).is_empty() || !is_land_filter(&f) {
        return None;
    }
    let c = TriggerCond::LandPlayed { who: subj.rel, filter: f };
    Some((
        only_player(c, &subj.only),
        Sel::TriggerObject,
        PlayerRef::TriggerPlayer,
    ))
}

/// A description of lands ("a nonbasic land", "an Island", "a legendary land").
fn is_land_filter(f: &Filter) -> bool {
    match f {
        Filter::Type(CardType::Land) => true,
        Filter::Subtype(s) => crate::types::is_land_type(s.as_str()),
        Filter::And(v) => v.iter().any(is_land_filter),
        _ => false,
    }
}

fn play_events(r: &str) -> Option<Parsed> {
    let (subj, rest) = player_subject(r)?;
    let t = rest
        .strip_prefix("plays ")
        .or_else(|| rest.strip_prefix("play "))?;
    // "play a land from exile or cast a spell from exile", "play a legendary land or cast
    // a legendary spell": either event; "it" is the land or the spell.
    for sep in [" or cast ", " or casts "] {
        if let Some((a, b)) = t.split_once(sep) {
            let land = play_land(&subj, a)?;
            let (cast, _, _) = cast_spell(&subj, b).or_else(|| {
                if subj.only.is_some() {
                    return None;
                }
                reparse(&format!("{} cast {b}", subj.word))
            })?;
            let cast = only_player(cast, &subj.only);
            return Some((
                TriggerCond::AnyOf(vec![land.0, cast]),
                Sel::TriggerObject,
                PlayerRef::TriggerPlayer,
            ));
        }
    }
    play_land(&subj, t)
}

// ---------------------------------------------------------------------------
// Activating abilities (CR 602.2)
// ---------------------------------------------------------------------------

/// "[player] activate(s) [an ability | a loyalty ability | a ninjutsu ability] [of
/// [object]] [with/without {T} in its activation cost] [that isn't a mana ability]";
/// "[player] activate(s) [an artifact's ability] …". "That ability" is the ability
/// activated, "that player" its controller.
fn activate_events(r: &str) -> Option<Parsed> {
    let (subj, rest) = player_subject(r)?;
    let t = rest
        .strip_prefix("activates ")
        .or_else(|| rest.strip_prefix("activate "))?;
    let mut include_mana = true;
    let mut conds: Vec<Condition> = Vec::new();
    let mut source = Filter::Any;
    // The kind of ability.
    let mut t = if let Some(x) = t.strip_prefix("an ability") {
        x
    } else if let Some(x) = t.strip_prefix("a loyalty ability") {
        // CR 606.3: loyalty abilities are never mana abilities.
        include_mana = false;
        conds.push(Condition::SelMatches(
            Sel::TriggerSpell,
            Filter::Custom(crate::stack_ability_filters::LOYALTY_ABILITY.into()),
        ));
        x
    } else if let Some(x) = t.strip_prefix("a ninjutsu ability") {
        include_mana = false;
        conds.push(Condition::Custom(
            crate::kw::activated_ability_kind::NINJUTSU.into(),
        ));
        x
    } else if let Some((obj, x)) = t
        .strip_prefix("an ")
        .or_else(|| t.strip_prefix("a "))
        .and_then(|x| x.split_once("'s ability"))
    {
        // "an artifact's ability": an ability of an artifact.
        let (f, plural, tail) = parse_object_phrase(obj)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        source = Filter::and(vec![Filter::Permanent, f]);
        x
    } else {
        return None;
    };
    loop {
        let x = t.trim_start();
        if x.is_empty() {
            break;
        }
        if let Some(y) = x.strip_prefix("that isn't a mana ability") {
            include_mana = false;
            t = y;
            continue;
        }
        if let Some(y) = x.strip_prefix("with {t} in its activation cost") {
            conds.push(Condition::Custom(
                crate::kw::activated_ability_kind::TAP_IN_COST.into(),
            ));
            t = y;
            continue;
        }
        if let Some(y) = x.strip_prefix("without {t} in its activation cost") {
            conds.push(Condition::Not(Box::new(Condition::Custom(
                crate::kw::activated_ability_kind::TAP_IN_COST.into(),
            ))));
            t = y;
            continue;
        }
        if let Some(y) = x.strip_prefix("of ") {
            if !matches!(source, Filter::Any) {
                return None;
            }
            let (f, y) = ability_source(y, &mut conds)?;
            source = f;
            t = y;
            continue;
        }
        return None;
    }
    let mut c = TriggerCond::AbilityActivated {
        who: subj.rel,
        source,
        include_mana,
    };
    if !conds.is_empty() {
        c = TriggerCond::Where {
            trigger: Box::new(c),
            cond: if conds.len() == 1 {
                conds.pop()?
            } else {
                Condition::And(conds)
            },
        };
    }
    Some((
        only_player(c, &subj.only),
        Sel::TriggerSpell,
        PlayerRef::TriggerPlayer,
    ))
}

/// The source of an activated ability: "enchanted creature", "~", "a creature or land",
/// "an artifact they control", "a card in your graveyard", up to a qualifier of the
/// ability that follows ("with {T} in …", "that isn't a mana ability").
fn ability_source<'a>(s: &'a str, conds: &mut Vec<Condition>) -> Option<(Filter, &'a str)> {
    for p in [
        "enchanted creature",
        "enchanted artifact",
        "enchanted planeswalker",
        "enchanted permanent",
        "equipped creature",
    ] {
        if let Some(r) = s.strip_prefix(p).filter(|r| word_end(r)) {
            return Some((Filter::AttachedToSource, r));
        }
    }
    if let Some(r) = s.strip_prefix('~').filter(|r| word_end(r)) {
        return Some((Filter::Source, r));
    }
    let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    // The object phrase ends where a qualifier of the ability begins.
    let cut = ["with {t} in", "without {t} in", "that isn't a mana ability"]
        .iter()
        .filter_map(|q| x.find(q))
        .min()
        .unwrap_or(x.len());
    let (phrase, rest) = (x[..cut].trim_end(), &x[cut..]);
    // "an artifact they control": controlled by the player activating it.
    let (phrase, theirs) = match phrase.strip_suffix(" they control") {
        Some(p) => (p, true),
        None => (phrase, false),
    };
    let (f, plural, tail) = parse_object_phrase(phrase)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    if theirs {
        conds.push(Condition::SelMatches(
            Sel::TriggerObject,
            Filter::ControlledByPlayer(Box::new(PlayerRef::TriggerPlayer)),
        ));
    }
    // "of a creature" is of a permanent (CR 109.2); "of a card in your graveyard" names
    // its zone.
    let f = if f.zone().is_some() {
        f
    } else {
        Filter::and(vec![Filter::Permanent, f])
    };
    Some((f, rest))
}

/// "[an artifact] becomes tapped or [a player] activates [an artifact's ability] …"
/// (Haunting Wind): either event; "it" is the artifact.
fn tapped_or_activates(r: &str) -> Option<Parsed> {
    let (a, b) = r.split_once(" becomes tapped or ")?;
    let tapped = reparse(&format!("{a} becomes tapped"))?;
    let (act, _, _) = activate_events(b)?;
    Some((
        TriggerCond::AnyOf(vec![tapped.0, act]),
        Sel::TriggerObject,
        PlayerRef::Iterated,
    ))
}

/// "When ~ dies during combat" and other events qualified by a combat timing.
fn during_combat(r: &str) -> Option<Parsed> {
    let head = r.strip_suffix(" during combat")?;
    let (c, it, p) = reparse(head)?;
    if matches!(c, TriggerCond::ThisTurn(_) | TriggerCond::UntilYourNextTurn(_)) {
        return None;
    }
    Some((
        super::trigger_grammar_events::where_events(c, Condition::Phase(PhaseCond::Combat)),
        it,
        p,
    ))
}

fn parse(r: &str) -> Option<Parsed> {
    let r = end(r);
    cast_events(r)
        .or_else(|| spell_is_cast(r))
        .or_else(|| play_events(r))
        .or_else(|| activate_events(r))
        .or_else(|| tapped_or_activates(r))
        .or_else(|| during_combat(r))
}

inventory::submit! { TriggerPattern { name: "trigger grammar II: cast, play, combat, damage, targets", priority: 150, parse } }

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
    }

    /// Every condition parses (all failures reported at once).
    fn all(v: &[&str]) {
        let failed: Vec<&&str> = v
            .iter()
            .filter(|s| crate::oracle::triggers::parse_trigger_condition(s).is_none())
            .collect();
        assert!(failed.is_empty(), "failed to parse: {failed:#?}");
    }

    #[test]
    fn cast_conditions() {
        all(&[
            "when you cast or cycle ~",
            "whenever a player casts an instant or sorcery spell from their hand",
            "when you cast ~ from anywhere other than exile",
            "when you cast ~ from your hand",
            "when you cast your commander",
            "whenever a player casts a spell they don't own",
            "whenever a player casts an instant or sorcery spell that targets only ~",
            "whenever an opponent casts a spell that targets you or a creature you control",
            "whenever you cast a spell that targets an opponent or a creature an opponent controls",
            "whenever an opponent casts an instant spell other than the first instant spell that player casts each turn",
            "whenever an instant or sorcery spell is cast during your turn",
            "whenever the first noncreature spell of a turn is cast",
            "whenever the fourth spell of a turn is cast",
            "whenever the chosen player casts a spell",
            "whenever enchanted player casts a spell other than the first spell they cast each turn or copies a spell",
            "whenever you cast a noncreature or dragon spell",
            "whenever you cast a noncreature spell or a dragon spell",
            "whenever you cast an equipment spell or a spell that targets a creature you control",
            "whenever you cast a noncreature spell with one or more blue mana symbols in its mana cost",
            "whenever you cast a permanent spell with a mana cost that contains {x}",
            "whenever you cast a spell during combat",
            "whenever you cast a spell during your turn other than your first spell that turn",
            "whenever you cast a spell other than your first spell each turn",
            "whenever you cast a spell that's exactly two colors",
            "whenever you cast an eldrazi creature spell with mana value 7 or greater",
            "whenever you cast an instant or sorcery spell or activate an ability",
            "whenever you cast an instant spell during your main phase",
            "whenever you cast another spell that has flash",
            "whenever you cast your first human creature spell each turn",
            "whenever you cast your first spell during each of your turns",
            "whenever you cast your fourth noncreature spell each turn",
            "whenever you play a land from exile or cast a spell from exile",
            "whenever you play a legendary land or cast a legendary spell",
            "whenever you play an island",
            "when an opponent plays a nonbasic land",
            "whenever an opponent casts a creature or planeswalker spell with the same name as a card in their graveyard",
            "when ~ dies during combat",
        ]);
    }

    #[test]
    fn activation_conditions() {
        all(&[
            "whenever a player activates an ability that isn't a mana ability",
            "whenever an opponent activates a loyalty ability",
            "whenever you activate a loyalty ability",
            "whenever you activate a loyalty ability of enchanted planeswalker",
            "whenever an opponent activates an ability of a creature or land that isn't a mana ability",
            "whenever an opponent activates an ability of a permanent that isn't a mana ability",
            "whenever an opponent activates an ability of an artifact they control",
            "whenever you activate a ninjutsu ability",
            "whenever a player activates an ability of enchanted creature with {t} in its activation cost that isn't a mana ability",
            "whenever an artifact becomes tapped or a player activates an artifact's ability without {t} in its activation cost",
            "whenever an artifact an opponent controls becomes tapped or an opponent activates an artifact's ability without {t} in its activation cost",
            "whenever enchanted artifact becomes tapped or a player activates an ability of enchanted artifact without {t} in its activation cost",
            "whenever an opponent casts a spell or activates an ability",
            "whenever you cast a spell from your graveyard or activate an ability of a card in your graveyard",
        ]);
    }
}

#[cfg(test)]
mod probe {
    /// Development aid: `PROBE_CONDS=<file>` parses each line as a trigger condition
    /// ("obj …" / "spell …" lines as object or spell phrases); `PROBE_CARDS=<file with card
    /// names>` reports, for each unsupported text of the cards, whether its trigger
    /// condition parses. Run with `cargo test -p mtg-engine --lib probe_ -- --nocapture`.
    #[test]
    fn probe_conditions() {
        let Ok(list) = std::env::var("PROBE_CONDS") else {
            return;
        };
        for l in std::fs::read_to_string(list).unwrap().lines() {
            let l = l.trim().to_lowercase();
            if l.is_empty() {
                continue;
            }
            if let Some(x) = l.strip_prefix("obj ") {
                println!("PROBE OBJ {x}\n   {:?}", crate::oracle::phrases::parse_object_phrase(x));
                continue;
            }
            if let Some(x) = l.strip_prefix("spell ") {
                println!("PROBE SPELL {x}\n   {:?}", super::super::triggers::parse_spell_phrase(x));
                continue;
            }
            match crate::oracle::triggers::parse_trigger_condition(&l) {
                Some(p) => println!("PROBE OK {l}\n   {p:?}"),
                None => println!("PROBE FAIL {l}"),
            }
        }
    }

    #[test]
    fn probe_cards() {
        let Ok(list) = std::env::var("PROBE_CARDS") else {
            return;
        };
        for name in std::fs::read_to_string(list).unwrap().lines() {
            let def = crate::card::card(name.trim());
            for u in def.unsupported_text() {
                // Quoted abilities: probe the quoted text.
                let inner: Vec<&str> = u.split('"').collect();
                let texts: Vec<&str> = if inner.len() >= 3 {
                    inner.iter().skip(1).step_by(2).copied().collect()
                } else {
                    vec![u]
                };
                for t in texts {
                    let t = t.split_once(" — ").map_or(t, |(_, b)| b);
                    let l = t.to_lowercase();
                    let mut ok = None;
                    for (i, _) in l.match_indices(',') {
                        if crate::oracle::triggers::parse_trigger_condition(&l[..i]).is_some() {
                            ok = Some(i);
                            break;
                        }
                    }
                    match ok {
                        Some(i) => println!("PROBE BODY {name} || {} || {}", &t[..i], &t[i + 1..]),
                        None => println!("PROBE TRIG {name} || {t}"),
                    }
                }
            }
        }
    }
}
