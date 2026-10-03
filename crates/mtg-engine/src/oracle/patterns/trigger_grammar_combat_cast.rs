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

use super::{ConditionPattern, FilterSuffixPattern, TriggerPattern};
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
    // "a player kicks a spell": casts a kicked spell (CR 702.33d).
    if let Some(x) = rest
        .strip_prefix("kicks ")
        .or_else(|| rest.strip_prefix("kick "))
    {
        let x = x.strip_prefix("a ").or_else(|| x.strip_prefix("an "))?;
        let (f, c) = super::triggers::parse_spell_phrase(&format!("kicked {x}"))?;
        let c0 = TriggerCond::CastSpell {
            who: subj.rel,
            filter: f,
        };
        let c0 = match c {
            Some(cond) => TriggerCond::Where {
                trigger: Box::new(c0),
                cond,
            },
            None => c0,
        };
        return Some((
            only_player(c0, &subj.only),
            Sel::TriggerSpell,
            PlayerRef::TriggerPlayer,
        ));
    }
    let t = cast_verb(rest)?;
    // "a player casts a card": a spell that's a card, not a copy (CR 707.12).
    if end(t) == "a card" {
        let c = TriggerCond::CastSpell {
            who: subj.rel,
            filter: Filter::Card,
        };
        return Some((
            only_player(c, &subj.only),
            Sel::TriggerSpell,
            PlayerRef::TriggerPlayer,
        ));
    }
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

/// Whether a trigger is on casting this very spell, with qualifiers ("when you cast ~
/// from your hand", "when you cast ~ while you control a creature"): it functions from the
/// stack (CR 113.6).
pub(crate) fn casts_this_spell(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::Where { trigger, .. } => casts_this_spell(trigger),
        TriggerCond::CastSpell { filter, .. } => match filter {
            Filter::Source => true,
            Filter::And(v) => v.iter().any(|f| matches!(f, Filter::Source)),
            _ => false,
        },
        _ => false,
    }
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
    for q in [
        " with the same name as a card in their graveyard",
        " that doesn't share a creature type with ",
    ] {
        if let Some(i) = x.find(q) {
            let (head, q) = (&x[..i], &x[i..]);
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
    // "that doesn't share a creature type with a creature you control or a creature card
    // in your graveyard" (CR 205.3).
    if let Some(x) = t.strip_prefix("that doesn't share a creature type with ") {
        let mut sels = Vec::new();
        for part in x.split(" or ") {
            let y = part.strip_prefix("a ").or_else(|| part.strip_prefix("an "))?;
            let (f, plural, tail) = parse_object_phrase(y)?;
            if plural || !end(tail).is_empty() {
                return None;
            }
            sels.push(Sel::All(f));
        }
        let sel = if sels.len() == 1 {
            sels.pop()?
        } else {
            Sel::Union(sels)
        };
        return filter(Filter::not(Filter::SharesCreatureType(Box::new(sel))), "");
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

/// "[creature] that's enchanted or equipped", "that's enchanted", "that's equipped"
/// (CR 303.4, 301.5), "[creature] with a mana ability" (CR 605.1a), "[creature] you own
/// but don't control" (CR 108.3).
fn creature_qualities<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for (p, f) in [
        (
            "that's enchanted or equipped",
            Filter::Or(vec![Filter::Enchanted, Filter::Equipped]),
        ),
        (
            "that are enchanted or equipped",
            Filter::Or(vec![Filter::Enchanted, Filter::Equipped]),
        ),
        (
            "that are enchanted by an aura you control",
            Filter::Custom(crate::attach::ENCHANTED_BY_YOUR_AURA.into()),
        ),
        ("that has been dealt damage this turn", Filter::DealtDamageThisTurn),
        ("that was dealt damage this turn", Filter::DealtDamageThisTurn),
        (
            "that was turned face up this turn",
            Filter::Custom(crate::kw::activated_ability_kind::TURNED_FACE_UP_THIS_TURN.into()),
        ),
        ("that's enchanted", Filter::Enchanted),
        ("that are enchanted", Filter::Enchanted),
        ("that's equipped", Filter::Equipped),
        ("that are equipped", Filter::Equipped),
        (
            "with a mana ability",
            Filter::Custom(crate::kw::activated_ability_kind::HAS_MANA_ABILITY.into()),
        ),
        (
            "with mana abilities",
            Filter::Custom(crate::kw::activated_ability_kind::HAS_MANA_ABILITY.into()),
        ),
        ("but don't control", Filter::not(Filter::ControlledBy(PlayerRel::You))),
    ] {
        if let Some(r) = t.strip_prefix(p).filter(|r| word_end(r)) {
            return Some((f, r));
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "that's enchanted or equipped, with a mana ability", priority: 100, parse: creature_qualities } }

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
    // "…, if it isn't a mana ability" (Harsh Mentor): a mana ability never stops being
    // one, so it's part of the trigger event (CR 603.4).
    let t = match t.strip_suffix(", if it isn't a mana ability") {
        Some(x) => {
            include_mana = false;
            x
        }
        None => t,
    };
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
    } else if let Some((x, name)) = [
        ("a ninjutsu ability", crate::kw::activated_ability_kind::NINJUTSU),
        ("a power-up ability", crate::kw::activated_ability_kind::POWER_UP),
    ]
    .into_iter()
    .find_map(|(p, n)| t.strip_prefix(p).map(|x| (x, n)))
    {
        include_mana = false;
        conds.push(Condition::Custom(name.into()));
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

// ---------------------------------------------------------------------------
// Attacking (CR 508.3)
// ---------------------------------------------------------------------------

/// "a player who controls eight or more lands", "a player who controls more lands than
/// you", "a player who has more life than you", "the monarch".
fn player_who(s: &str) -> Option<PlayerFilter> {
    if s == "the monarch" {
        return Some(PlayerFilter::Monarch);
    }
    let x = s.strip_prefix("a player who ")?;
    if x == "has more life than you" {
        return Some(PlayerFilter::Life(
            Cmp::Gt,
            Box::new(Value::LifeTotal(PlayerRef::You)),
        ));
    }
    let x = x.strip_prefix("controls ")?;
    // "more lands than you"
    if let Some(y) = x.strip_prefix("more ") {
        let y = y.strip_suffix(" than you")?;
        let (f, plural, tail) = parse_object_phrase(y)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        let yours = Value::Count(f.clone().you_control());
        return Some(PlayerFilter::Controls(
            Box::new(f),
            Cmp::Gt,
            Box::new(yours),
        ));
    }
    // "eight or more lands"
    let (n, y) = parse_number(x)?;
    let y = y.trim_start().strip_prefix("or more ")?;
    let (f, plural, tail) = parse_object_phrase(y)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    Some(PlayerFilter::Controls(Box::new(f), Cmp::Ge, Box::new(n)))
}

/// The attacked player is a player (not a planeswalker or battle) matching `f`.
fn attacked_player_is(f: PlayerFilter) -> Condition {
    Condition::And(vec![
        Condition::PlayerMatches(PlayerRef::TriggerPlayer, f),
        Condition::Not(Box::new(Condition::SelNonEmpty(Sel::TriggerOtherObject))),
    ])
}

/// "[subject] and at least [N] [other] [creatures] attack", "[subject] and another
/// [creature] attack": the subject attacks along with that many other such creatures (the
/// creatures attacking are those just declared, CR 508.1).
fn attacks_with_others(r: &str) -> Option<Parsed> {
    let x = r.strip_suffix(" attack")?;
    let (subj, others) = x.split_once(" and ")?;
    let (sf, it) = match subj {
        "~" => (Filter::Source, Sel::This),
        "equipped creature" | "enchanted creature" => {
            (Filter::AttachedToSource, Sel::TriggerObject)
        }
        _ => return None,
    };
    let (n, phrase) = if let Some(y) = others.strip_prefix("at least ") {
        let (n, y) = parse_number(y)?;
        (n.as_const()?, y.trim_start())
    } else if let Some(y) = others.strip_prefix("another ") {
        (1, y)
    } else {
        return None;
    };
    let phrase = phrase.strip_prefix("other ").unwrap_or(phrase);
    let (f, _, tail) = parse_object_phrase(phrase)?;
    if !end(tail).is_empty() {
        return None;
    }
    let count = Value::Count(Filter::and(vec![
        f,
        Filter::creature(),
        Filter::Attacking,
        Filter::not(sf.clone()),
    ]));
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Attacks(sf)),
            cond: Condition::Compare(count, Cmp::Ge, Value::c(n)),
        },
        it,
        PlayerRef::TriggerPlayer,
    ))
}

/// "[creature] attacks [a battle | a player who … | the monarch | one of your opponents
/// or a planeswalker an opponent controls | a player and isn't blocked]".
fn attacks_recipient(r: &str) -> Option<Parsed> {
    let (subj_s, rec) = r.split_once(" attacks ")?;
    let subj = super::triggers::parse_subject(subj_s)?;
    if subj.one_or_more {
        return None;
    }
    let f = subj.filter.clone();
    let it = if subj.self_only {
        Sel::This
    } else {
        Sel::TriggerObject
    };
    let tp = PlayerRef::TriggerPlayer;
    if rec == "a battle" {
        return Some((
            TriggerCond::AttacksRecipient {
                attacker: f,
                recipient: DamageRecipient::Object(Filter::Type(CardType::Battle)),
            },
            it,
            tp,
        ));
    }
    if rec == "one of your opponents or a planeswalker an opponent controls" {
        return Some((
            TriggerCond::AttacksRecipient {
                attacker: f,
                recipient: DamageRecipient::PlayerOrPlaneswalker(PlayerRel::Opponent),
            },
            it,
            tp,
        ));
    }
    if rec == "a player and isn't blocked" {
        // Declared unblocked while attacking a player (CR 509.1h).
        return Some((
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::AttacksUnblocked(f)),
                cond: Condition::SelMatches(
                    Sel::TriggerObject,
                    Filter::AttackingPlayer(PlayerRel::Any),
                ),
            },
            it,
            PlayerRef::DefendingPlayer,
        ));
    }
    let who = player_who(rec)?;
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Attacks(f)),
            cond: attacked_player_is(who),
        },
        it,
        tp,
    ))
}

/// "[N] or more [creatures]", "at least [N] [creatures]", "one or more [creatures]".
fn count_phrase(s: &str) -> Option<(u32, Filter, &str)> {
    let (n, rest) = if let Some(x) = s.strip_prefix("one or more ") {
        (1, x)
    } else if let Some(x) = s.strip_prefix("at least ") {
        let (n, x) = parse_number(x)?;
        (n.as_const()?, x.trim_start())
    } else {
        let (n, x) = parse_number(s)?;
        (n.as_const()?, x.trim_start().strip_prefix("or more ")?)
    };
    // "non-~ creatures": creatures other than this one.
    let (rest, not_self) = match rest.strip_prefix("non-~ ") {
        Some(x) => (x, true),
        None => (rest, false),
    };
    let (mut f, plural, tail) = parse_object_phrase(rest)?;
    if !plural && n != 1 {
        return None;
    }
    if not_self {
        f = Filter::and(vec![f, Filter::not(Filter::Source)]);
    }
    Some((n.max(1) as u32, f, tail))
}

/// Attack triggers about the attacking player: "you attack with exactly two creatures",
/// "you attack with at least two [creatures]", "you attack with your commander", "you
/// attack with a creature an opponent owns", "an opponent attacks with creatures", "an
/// opponent attacks [a planeswalker you control with one or more creatures | one or more
/// planeswalkers you control | you and/or one or more planeswalkers you control]".
fn player_attacks(r: &str) -> Option<Parsed> {
    let (subj, rest) = player_subject(r)?;
    if subj.only.is_some() {
        return None;
    }
    let who = subj.rel;
    let t = rest
        .strip_prefix("attacks ")
        .or_else(|| rest.strip_prefix("attack "))?;
    let objs = || Sel::TriggerObjects;
    let tp = PlayerRef::TriggerPlayer;
    if let Some(x) = t.strip_prefix("with ") {
        if x == "creatures" {
            return Some((TriggerCond::PlayerAttacks(who), objs(), tp));
        }
        if let Some(y) = x.strip_prefix("exactly ") {
            let (n, y) = parse_number(y)?;
            let n = n.as_const()?;
            if !matches!(end(y), "creatures" | "creature") {
                return None;
            }
            return Some((
                TriggerCond::Where {
                    trigger: Box::new(TriggerCond::PlayerAttacks(who)),
                    cond: Condition::Compare(Value::EventAmount, Cmp::Eq, Value::c(n)),
                },
                objs(),
                tp,
            ));
        }
        if x == "your commander" {
            return Some((
                TriggerCond::PlayerAttacksWith {
                    who,
                    filter: Filter::and(vec![Filter::Commander, Filter::OwnedBy(PlayerRel::You)]),
                    min: 1,
                },
                objs(),
                tp,
            ));
        }
        if x == "~ and/or your commander" {
            return Some((
                TriggerCond::PlayerAttacksWith {
                    who,
                    filter: Filter::Or(vec![
                        Filter::Source,
                        Filter::and(vec![Filter::Commander, Filter::OwnedBy(PlayerRel::You)]),
                    ]),
                    min: 1,
                },
                objs(),
                tp,
            ));
        }
        // "you attack with ~ and another legendary creature"
        if let Some(y) = x.strip_prefix("~ and ") {
            if who != PlayerRel::You {
                return None;
            }
            return attacks_with_others(&format!("~ and {y} attack"));
        }
        // "you attack with a creature an opponent owns": once for each such creature;
        // "that player" is its owner.
        if let Some(y) = x.strip_prefix("a ").or_else(|| x.strip_prefix("an ")) {
            let (f, plural, tail) = parse_object_phrase(y)?;
            if plural || !end(tail).is_empty() || who != PlayerRel::You {
                return None;
            }
            return Some((
                TriggerCond::Attacks(Filter::and(vec![
                    f,
                    Filter::creature(),
                    Filter::ControlledBy(PlayerRel::You),
                ])),
                Sel::TriggerObject,
                PlayerRef::OwnerOf(Box::new(Sel::TriggerObject)),
            ));
        }
        let (n, f, tail) = count_phrase(x)?;
        if !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::PlayerAttacksWith {
                who,
                filter: Filter::and(vec![f, Filter::creature()]),
                min: n,
            },
            objs(),
            tp,
        ));
    }
    // "you attack the player who has the initiative": once, if creatures you control attack
    // that player (CR 508.3e).
    if t == "the player who has the initiative" {
        return Some((
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::PlayerAttacked {
                    attacker: who,
                    defender: PlayerFilter::Any,
                    with: Filter::creature(),
                    min: 1,
                }),
                cond: Condition::Custom(
                    crate::kw::activated_ability_kind::EVENT_PLAYER_HAS_INITIATIVE.into(),
                ),
            },
            objs(),
            tp,
        ));
    }
    if who != PlayerRel::Opponent {
        return None;
    }
    // Attacks on you or your planeswalkers: "that player" is the attacking player.
    let pw = || {
        Filter::and(vec![
            Filter::Type(CardType::Planeswalker),
            Filter::ControlledBy(PlayerRel::You),
        ])
    };
    let attacker = PlayerRef::ActivePlayer;
    match t {
        "a planeswalker you control with one or more creatures" => Some((
            TriggerCond::IsAttacked(DamageRecipient::Object(pw())),
            Sel::None,
            attacker,
        )),
        "one or more planeswalkers you control" => Some((
            TriggerCond::Batched {
                trigger: Box::new(TriggerCond::IsAttacked(DamageRecipient::Object(pw()))),
                per: BatchPer::Batch,
            },
            Sel::None,
            attacker,
        )),
        "you and/or one or more planeswalkers you control" => Some((
            TriggerCond::Batched {
                trigger: Box::new(TriggerCond::IsAttacked(
                    DamageRecipient::PlayerOrPlaneswalker(PlayerRel::You),
                )),
                per: BatchPer::Batch,
            },
            Sel::None,
            attacker,
        )),
        _ => None,
    }
}

/// "[N] or more [creatures] [you control | your opponents control] attack [a player]",
/// "one or more [creatures] attack [one of your opponents or a planeswalker they control |
/// one or more players | you and aren't blocked]".
fn creatures_attack(r: &str) -> Option<Parsed> {
    let (subject, verb_rest) = r
        .split_once(" attack ")
        .or_else(|| r.strip_suffix(" attack").map(|s| (s, "")))?;
    let (n, f, tail) = count_phrase(subject)?;
    if !end(tail).is_empty() {
        return None;
    }
    let f = Filter::and(vec![f, Filter::creature()]);
    // Who controls them (the attacking player).
    let controller = controlled_by(&f);
    let objs = Sel::TriggerObjects;
    let tp = PlayerRef::TriggerPlayer;
    let batch = |c: TriggerCond| TriggerCond::Batched {
        trigger: Box::new(c),
        per: BatchPer::Batch,
    };
    match verb_rest {
        "" if n == 1 => None,
        "" => {
            let who = controller.unwrap_or(PlayerRel::Any);
            // Only the attacking player's creatures are declared attackers.
            Some((
                TriggerCond::PlayerAttacksWith { who, filter: f, min: n },
                objs,
                tp,
            ))
        }
        "a player" if n > 1 => Some((
            TriggerCond::PlayerAttacked {
                attacker: controller.unwrap_or(PlayerRel::Any),
                defender: PlayerFilter::Any,
                with: f,
                min: n,
            },
            objs,
            tp,
        )),
        "one or more players" if n == 1 => Some((
            batch(TriggerCond::Where {
                trigger: Box::new(TriggerCond::Attacks(f)),
                cond: attacked_player_is(PlayerFilter::Any),
            }),
            objs,
            PlayerRef::Iterated,
        )),
        "one of your opponents or a planeswalker they control" if n == 1 => Some((
            batch(TriggerCond::AttacksRecipient {
                attacker: f,
                recipient: DamageRecipient::PlayerOrPlaneswalker(PlayerRel::Opponent),
            }),
            objs,
            PlayerRef::Iterated,
        )),
        "you and aren't blocked" if n == 1 => Some((
            // Declared unblocked while attacking you (CR 509.1h): "that player" is the
            // attacking player.
            batch(TriggerCond::AttacksUnblocked(Filter::and(vec![
                f,
                Filter::AttackingPlayer(PlayerRel::You),
            ]))),
            objs,
            PlayerRef::ActivePlayer,
        )),
        _ => None,
    }
}

/// The controller relation a creature description names ("you control", "your opponents
/// control").
fn controlled_by(f: &Filter) -> Option<PlayerRel> {
    match f {
        Filter::ControlledBy(r) => Some(*r),
        Filter::And(v) => v.iter().find_map(controlled_by),
        _ => None,
    }
}

/// "a creature you control attacks or enters attacking": either event.
fn attacks_or_enters_attacking(r: &str) -> Option<Parsed> {
    let s = r.strip_suffix(" attacks or enters attacking")?;
    let subj = super::triggers::parse_subject(s)?;
    if subj.one_or_more || subj.self_only {
        return None;
    }
    Some((
        TriggerCond::AnyOf(vec![
            TriggerCond::Attacks(subj.filter.clone()),
            // Put onto the battlefield attacking (CR 508.4).
            TriggerCond::EntersBattlefield(Filter::and(vec![subj.filter, Filter::Attacking])),
        ]),
        Sel::TriggerObject,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

// ---------------------------------------------------------------------------
// Blocking (CR 509.3)
// ---------------------------------------------------------------------------

/// "a [creature]" / "one or more [creatures]" after a block verb.
fn block_object(s: &str) -> Option<(Filter, bool, &str)> {
    if let Some(x) = s.strip_prefix("one or more ") {
        let (f, plural, tail) = parse_object_phrase(x)?;
        return plural.then_some((f, true, tail));
    }
    let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(x)?;
    (!plural).then_some((f, false, tail))
}

/// A creature with lesser power than the other creature of a block: the blocked attacker
/// (event object) vs. the blocker (other), or the reverse.
fn lesser_power(of: Sel, than: Sel) -> Condition {
    Condition::Compare(
        Value::PowerOf(Box::new(of)),
        Cmp::Lt,
        Value::PowerOf(Box::new(than)),
    )
}

fn block_events(r: &str) -> Option<Parsed> {
    // "[creature] blocks or becomes blocked by ~" (Mirror Shield): "that creature" is the
    // other creature.
    if let Some(s) = r.strip_suffix(" blocks or becomes blocked by ~") {
        let subj = super::triggers::parse_subject(s)?;
        if subj.one_or_more || subj.self_only {
            return None;
        }
        return Some((
            TriggerCond::AnyOf(vec![
                TriggerCond::BlocksCreature {
                    blocker: subj.filter.clone(),
                    attacker: Filter::Source,
                },
                TriggerCond::BlockedByCreature {
                    attacker: subj.filter,
                    blocker: Filter::Source,
                },
            ]),
            Sel::TriggerOtherObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject)),
        ));
    }
    if let Some((s, o)) = r.split_once(" blocks or becomes blocked by ") {
        let (g, many, tail) = block_object(o)?;
        if !end(tail).is_empty() {
            return None;
        }
        let g = Filter::and(vec![g, Filter::creature()]);
        if s == "~" && many {
            // Once per combat (CR 509.3a, 509.3c), if any of those creatures matches.
            return Some((
                TriggerCond::Where {
                    trigger: Box::new(TriggerCond::BlocksOrBecomesBlocked(Filter::Source)),
                    cond: Condition::Exists(Filter::and(vec![
                        g,
                        Filter::Or(vec![Filter::BlockingSource, Filter::BlockedBySource]),
                    ])),
                },
                Sel::This,
                PlayerRef::You,
            ));
        }
        let f = match s {
            "enchanted creature" | "equipped creature" => Filter::AttachedToSource,
            _ => return None,
        };
        if many {
            return None;
        }
        // Once for each creature it blocks or is blocked by (CR 509.3b, 509.3d): "that
        // creature" is that one.
        return Some((
            TriggerCond::AnyOf(vec![
                TriggerCond::BlocksCreature {
                    blocker: f.clone(),
                    attacker: g.clone(),
                },
                TriggerCond::BlockedByCreature {
                    attacker: f,
                    blocker: g,
                },
            ]),
            Sel::TriggerObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        ));
    }
    // "a creature blocks a creature with lesser power" (No Quarter).
    if let Some(s) = r.strip_suffix(" blocks a creature with lesser power") {
        let subj = super::triggers::parse_subject(s)?;
        if subj.one_or_more || subj.self_only {
            return None;
        }
        return Some((
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::BlocksCreature {
                    blocker: subj.filter,
                    attacker: Filter::creature(),
                }),
                cond: lesser_power(Sel::TriggerObject, Sel::TriggerOtherObject),
            },
            Sel::None,
            PlayerRef::Iterated,
        ));
    }
    if let Some(s) = r.strip_suffix(" becomes blocked by a creature with lesser power") {
        let subj = super::triggers::parse_subject(s)?;
        if subj.one_or_more || subj.self_only {
            return None;
        }
        return Some((
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::BlockedByCreature {
                    attacker: subj.filter,
                    blocker: Filter::creature(),
                }),
                cond: lesser_power(Sel::TriggerObject, Sel::TriggerOtherObject),
            },
            Sel::None,
            PlayerRef::Iterated,
        ));
    }
    // "a creature attacking one of your opponents becomes blocked by two or more
    // creatures".
    if let Some((s, n)) = r.split_once(" becomes blocked by ") {
        let n = match n {
            "two or more creatures" => 2,
            "three or more creatures" => 3,
            _ => return None,
        };
        let (s, attacking) = match s.strip_suffix(" attacking one of your opponents") {
            Some(x) => (x, Some(Filter::AttackingPlayer(PlayerRel::Opponent))),
            None => (s, None),
        };
        let subj = super::triggers::parse_subject(s)?;
        if subj.one_or_more || subj.self_only {
            return None;
        }
        let mut parts = vec![subj.filter];
        parts.extend(attacking);
        let attacker = Filter::and(parts);
        return Some((
            TriggerCond::BlockedByN { attacker, n },
            Sel::TriggerObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        ));
    }
    if let Some((s, o)) = r.split_once(" blocks ") {
        if s == "~" {
            // "~ blocks one or more black creatures", "~ blocks two or more creatures":
            // once per combat (CR 509.3a), counting the creatures it blocks.
            let (n, g, tail) = count_phrase(o)?;
            if !end(tail).is_empty() {
                return None;
            }
            let blocked = Value::Count(Filter::and(vec![g, Filter::BlockedBySource]));
            return Some((
                TriggerCond::Where {
                    trigger: Box::new(TriggerCond::Blocks(Filter::Source)),
                    cond: Condition::Compare(blocked, Cmp::Ge, Value::c(n as i32)),
                },
                Sel::This,
                PlayerRef::You,
            ));
        }
        // "a creature blocks a black or red creature": once for each such block
        // (CR 509.3b); "the blocking creature" / "the attacking creature" name them.
        let subj = super::triggers::parse_subject(s)?;
        if subj.one_or_more || subj.self_only {
            return None;
        }
        let (g, many, tail) = block_object(o)?;
        if many || !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::BlocksCreature {
                blocker: subj.filter,
                attacker: Filter::and(vec![g, Filter::creature()]),
            },
            Sel::None,
            PlayerRef::Iterated,
        ));
    }
    // "one or more creatures block": once per declaration of blockers.
    if let Some(s) = r.strip_suffix(" block") {
        let (n, f, tail) = count_phrase(s)?;
        if n != 1 || !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::Batched {
                trigger: Box::new(TriggerCond::Blocks(Filter::and(vec![f, Filter::creature()]))),
                per: BatchPer::Batch,
            },
            Sel::None,
            PlayerRef::Iterated,
        ));
    }
    None
}

// ---------------------------------------------------------------------------
// Damage (CR 120)
// ---------------------------------------------------------------------------

/// A source of damage: "~", "a [adjectives] source [you control | an opponent controls]
/// [other than ~]", "a red instant or sorcery spell you control", or any object subject.
fn damage_source(s: &str) -> Option<super::triggers::Subject> {
    use super::triggers::Subject;
    if let Some(x) = s.strip_prefix("a ").or_else(|| s.strip_prefix("an ")) {
        if let Some(i) = x.find("source") {
            let (adjs, rest) = (&x[..i], &x[i + "source".len()..]);
            let mut parts = Vec::new();
            for w in adjs.split_whitespace() {
                parts.push(adjective(w)?);
            }
            let mut rest = rest.trim_start();
            loop {
                if rest.is_empty() {
                    break;
                }
                if let Some(r) = rest.strip_prefix("you control") {
                    parts.push(Filter::ControlledBy(PlayerRel::You));
                    rest = r.trim_start();
                } else if let Some(r) = rest.strip_prefix("an opponent controls") {
                    parts.push(Filter::ControlledBy(PlayerRel::Opponent));
                    rest = r.trim_start();
                } else if let Some(r) = rest.strip_prefix("other than ~") {
                    parts.push(Filter::not(Filter::Source));
                    rest = r.trim_start();
                } else if let Some(r) = rest.strip_prefix("of the chosen color") {
                    parts.push(Filter::ChosenColor);
                    rest = r.trim_start();
                } else {
                    return None;
                }
            }
            return Some(Subject {
                filter: Filter::and(parts),
                self_only: false,
                one_or_more: false,
            });
        }
    }
    super::triggers::parse_subject(s)
}

/// A recipient of damage, maybe one of several ("you or a permanent you control"), each
/// with an optional condition on the event; `true` if it names "one or more" recipients
/// (the damage dealt to them at once is one batch).
type Recipient = (DamageRecipient, Option<Condition>);

fn damage_recipients(s: &str) -> Option<(Vec<Recipient>, bool)> {
    let one = |s: &str| -> Option<Recipient> {
        let s = s.trim();
        let pl = |rel| Some((DamageRecipient::Player(rel), None));
        match s {
            "you" => return pl(PlayerRel::You),
            "an opponent" | "opponent" | "one of your opponents" => return pl(PlayerRel::Opponent),
            "a player" => return pl(PlayerRel::Any),
            "another player" => return pl(PlayerRel::NotYou),
            "battle" | "a battle" => {
                return Some((
                    DamageRecipient::Object(Filter::Type(CardType::Battle)),
                    None,
                ))
            }
            "enchanted player" => {
                return Some((
                    DamageRecipient::Player(PlayerRel::Any),
                    Some(Condition::PlayerMatches(
                        PlayerRef::TriggerPlayer,
                        PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(
                            Sel::AttachedTo,
                        )))),
                    )),
                ))
            }
            "its owner" => {
                return Some((
                    DamageRecipient::Player(PlayerRel::Any),
                    Some(Condition::PlayerMatches(
                        PlayerRef::TriggerPlayer,
                        PlayerFilter::Ref(Box::new(PlayerRef::OwnerOf(Box::new(
                            Sel::TriggerOtherObject,
                        )))),
                    )),
                ))
            }
            "enchanted planeswalker" | "enchanted creature" | "equipped creature" => {
                return Some((DamageRecipient::Object(Filter::AttachedToSource), None))
            }
            _ => {}
        }
        if let Some(pf) = player_who(s) {
            return Some((
                DamageRecipient::Player(PlayerRel::Any),
                Some(Condition::PlayerMatches(PlayerRef::TriggerPlayer, pf)),
            ));
        }
        let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(x)?;
        if plural || !end(tail).is_empty() || matches!(f, Filter::Any) {
            return None;
        }
        Some((DamageRecipient::Object(f), None))
    };
    let s = s.trim();
    // "one or more [creatures | blocking creatures | of your opponents | players]",
    // "your opponents", "one or more permanents and/or players".
    let plural_players = [
        ("your opponents", PlayerRel::Opponent),
        ("one or more of your opponents", PlayerRel::Opponent),
        ("one or more opponents", PlayerRel::Opponent),
        ("one or more players", PlayerRel::Any),
    ];
    for (p, rel) in plural_players {
        if s == p {
            return Some((vec![(DamageRecipient::Player(rel), None)], true));
        }
    }
    if s == "one or more permanents and/or players" {
        return Some((vec![(DamageRecipient::Any, None)], true));
    }
    if let Some(x) = s.strip_prefix("one or more ") {
        let (f, plural, tail) = parse_object_phrase(x)?;
        if !plural || !end(tail).is_empty() {
            return None;
        }
        return Some((vec![(DamageRecipient::Object(f), None)], true));
    }
    if let Some(r) = one(s) {
        return Some((vec![r], false));
    }
    // "you or a permanent you control", "a creature or opponent", "an opponent or battle".
    let (a, b) = s.split_once(" or ")?;
    let b = match (a.starts_with("a ") || a.starts_with("an "), b) {
        // "a creature or opponent": the article covers both.
        (true, b) if !b.starts_with("a ") && !b.starts_with("an ") && b != "you" => {
            if one(&format!("a {b}")).is_some() && b != "opponent" && b != "battle" {
                format!("a {b}")
            } else {
                b.to_string()
            }
        }
        _ => b.to_string(),
    };
    Some((vec![one(a)?, one(&b)?], false))
}

/// "[source] deals [combat | noncombat] damage [to [recipients]] [during your turn]".
fn deals_damage(r: &str) -> Option<Parsed> {
    let (subj_s, rest) = [" deals ", " deal "]
        .iter()
        .find_map(|v| r.split_once(v))?;
    let subj = damage_source(subj_s)?;
    let (kind, rest) = if let Some(x) = rest.strip_prefix("combat damage") {
        (Some(true), x)
    } else if let Some(x) = rest.strip_prefix("noncombat damage") {
        (Some(false), x)
    } else {
        (None, rest.strip_prefix("damage")?)
    };
    let mut rest = rest.trim_start();
    let mut conds: Vec<Condition> = Vec::new();
    let mut first_time = false;
    for (q, your_turn) in [(" during your turn", true), (" for the first time each turn", false)] {
        let matched = if let Some(x) = rest.strip_suffix(q) {
            rest = x;
            true
        } else if rest == q.trim_start() {
            rest = "";
            true
        } else {
            false
        };
        if matched {
            if your_turn {
                conds.push(Condition::YourTurn);
            } else {
                first_time = true;
            }
        }
    }
    // "to a creature equal to that creature's toughness": the amount dealt (Taii Wakeen).
    if let Some(x) = rest.strip_suffix(" equal to that creature's toughness") {
        rest = x;
        conds.push(Condition::Compare(
            Value::EventAmount,
            Cmp::Eq,
            Value::ToughnessOf(Box::new(Sel::TriggerObject)),
        ));
    }
    let (recipients, many) = if rest.is_empty() {
        (vec![(DamageRecipient::Any, None)], false)
    } else {
        damage_recipients(rest.strip_prefix("to ")?)?
    };
    // Only forms the compositional parser doesn't read: several recipients, a player
    // condition, a qualifier, or "one or more".
    let combat_only = kind == Some(true);
    let mk = |to: DamageRecipient, c: Option<Condition>| {
        let mut d = TriggerCond::DealsDamage {
            source: subj.filter.clone(),
            to,
            combat_only,
        };
        if kind == Some(false) {
            d = TriggerCond::Noncombat(Box::new(d));
        }
        let mut cs: Vec<Condition> = conds.clone();
        cs.extend(c);
        match cs.len() {
            0 => d,
            1 => TriggerCond::Where {
                trigger: Box::new(d),
                cond: cs.pop().unwrap_or(Condition::Always),
            },
            _ => TriggerCond::Where {
                trigger: Box::new(d),
                cond: Condition::And(cs),
            },
        }
    };
    let to_object = recipients
        .iter()
        .all(|(r, _)| matches!(r, DamageRecipient::Object(_)));
    let n = recipients.len();
    let mut alts: Vec<TriggerCond> = recipients.into_iter().map(|(r, c)| mk(r, c)).collect();
    let cond = if n == 1 {
        alts.pop()?
    } else {
        TriggerCond::AnyOf(alts)
    };
    let source_it = if subj.self_only {
        Sel::This
    } else {
        Sel::TriggerOtherObject
    };
    let (cond, it, player) = if subj.one_or_more {
        // "one or more Pirates you control deal damage to your opponents": once per batch
        // of damage, however many sources and players.
        if !many && !matches!(rest, "a player" | "an opponent" | "one of your opponents") {
            return None;
        }
        (
            TriggerCond::Batched {
                trigger: Box::new(cond),
                per: if many { BatchPer::Batch } else { BatchPer::Player },
            },
            Sel::TriggerObjects,
            if many {
                PlayerRef::Iterated
            } else {
                PlayerRef::TriggerPlayer
            },
        )
    } else if many || matches!(rest, "") {
        // A source dealing damage to several recipients at once: once per source.
        (
            TriggerCond::Batched {
                trigger: Box::new(cond),
                per: BatchPer::Other,
            },
            source_it,
            PlayerRef::Iterated,
        )
    } else if n > 1 {
        // Either recipient: only the source is common to both events.
        (cond, source_it, PlayerRef::Iterated)
    } else if to_object {
        if subj.self_only {
            (
                cond,
                Sel::TriggerObject,
                PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
            )
        } else {
            (cond, Sel::None, PlayerRef::Iterated)
        }
    } else {
        (cond, source_it, PlayerRef::TriggerPlayer)
    };
    let cond = if first_time {
        super::trigger_grammar_events::first_time(cond)
    } else {
        cond
    };
    Some((cond, it, player))
}

/// "you're dealt combat damage", "combat damage is dealt to you [or a planeswalker you
/// control]", "your opponents are dealt combat damage", "one or more of your opponents
/// are dealt combat damage [during your turn]", "one or more opponents are dealt
/// noncombat damage", "an opponent is dealt combat damage by one or more creatures you
/// control", "an opponent is dealt damage by [source] or by [source]".
fn dealt_damage(r: &str) -> Option<Parsed> {
    let (r, your_turn) = match r.strip_suffix(" during your turn") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let finish = |c: TriggerCond, per: BatchPer, player: PlayerRef| {
        let c = if your_turn {
            super::trigger_grammar_events::where_events(c, Condition::YourTurn)
        } else {
            c
        };
        Some((
            TriggerCond::Batched {
                trigger: Box::new(c),
                per,
            },
            Sel::None,
            player,
        ))
    };
    let pdd = |who, combat_only| TriggerCond::PlayerDealtDamage { who, combat_only };
    // "combat damage is dealt to you [or a planeswalker you control]"
    if let Some(x) = r.strip_prefix("combat damage is dealt to ") {
        return match x {
            "you" => finish(pdd(PlayerRel::You, true), BatchPer::Batch, PlayerRef::ActivePlayer),
            "you or a planeswalker you control" => finish(
                TriggerCond::AnyOf(vec![
                    pdd(PlayerRel::You, true),
                    TriggerCond::IsDealtDamage {
                        filter: Filter::and(vec![
                            Filter::Type(CardType::Planeswalker),
                            Filter::ControlledBy(PlayerRel::You),
                        ]),
                        combat_only: true,
                    },
                ]),
                BatchPer::Batch,
                PlayerRef::ActivePlayer,
            ),
            _ => None,
        };
    }
    for (p, who, per) in [
        ("you're dealt", PlayerRel::You, BatchPer::Batch),
        ("you are dealt", PlayerRel::You, BatchPer::Batch),
        ("your opponents are dealt", PlayerRel::Opponent, BatchPer::Batch),
        ("one or more of your opponents are dealt", PlayerRel::Opponent, BatchPer::Batch),
        ("one or more opponents are dealt", PlayerRel::Opponent, BatchPer::Batch),
        ("an opponent is dealt", PlayerRel::Opponent, BatchPer::Player),
    ] {
        let Some(x) = r.strip_prefix(p) else {
            continue;
        };
        let x = x.trim_start();
        let (kind, x) = if let Some(y) = x.strip_prefix("combat damage") {
            (Some(true), y)
        } else if let Some(y) = x.strip_prefix("noncombat damage") {
            (Some(false), y)
        } else {
            (None, x.strip_prefix("damage")?)
        };
        let base = pdd(who, kind == Some(true));
        let base = if kind == Some(false) {
            TriggerCond::Noncombat(Box::new(base))
        } else {
            base
        };
        let x = x.trim_start();
        let player = if per == BatchPer::Player {
            PlayerRef::TriggerPlayer
        } else if who == PlayerRel::You {
            // "the attacking player": the active player.
            PlayerRef::ActivePlayer
        } else {
            PlayerRef::Iterated
        };
        if x.is_empty() {
            // (Plain "you're dealt damage" is the compositional parser's.)
            if kind.is_none() && who == PlayerRel::You {
                return None;
            }
            return finish(base, per, player);
        }
        // "by one or more creatures you control", "by a red instant or sorcery spell you
        // control or by a red planeswalker you control".
        let by = x.strip_prefix("by ")?;
        let mut sources = Vec::new();
        for part in by.split(" or by ") {
            let part = part.strip_prefix("one or more ").map_or_else(
                || part.to_string(),
                |p| {
                    // "one or more creatures you control": any of them.
                    format!("a {}", p)
                },
            );
            let s = damage_source(&part).or_else(|| {
                let x = part.strip_prefix("a ")?;
                let (f, _, tail) = parse_object_phrase(x)?;
                end(tail).is_empty().then_some(super::triggers::Subject {
                    filter: f,
                    self_only: false,
                    one_or_more: false,
                })
            })?;
            sources.push(s.filter);
        }
        let source = if sources.len() == 1 {
            sources.pop()?
        } else {
            Filter::Or(sources)
        };
        let c = TriggerCond::Where {
            trigger: Box::new(base),
            cond: Condition::SelMatches(Sel::TriggerOtherObject, source),
        };
        return finish(c, per, player);
    }
    // "one or more creatures your opponents control are dealt excess noncombat damage"
    if let Some(x) = r.strip_suffix(" are dealt excess noncombat damage") {
        let (n, f, tail) = count_phrase(x)?;
        if n != 1 || !end(tail).is_empty() {
            return None;
        }
        return Some((
            TriggerCond::Batched {
                trigger: Box::new(TriggerCond::DealtExcessDamage {
                    filter: f,
                    noncombat_only: true,
                }),
                per: BatchPer::Batch,
            },
            Sel::None,
            PlayerRef::Iterated,
        ));
    }
    None
}

// ---------------------------------------------------------------------------
// Becoming the target (CR 115.10, 603.3d)
// ---------------------------------------------------------------------------

/// What targets: "a spell or ability", "a spell", "an ability", "an activated ability",
/// "an instant or sorcery spell", "an Aura spell", "an ability that targets only it",
/// "a backup ability", then "[an opponent | you] controls".
fn targeting_what(s: &str) -> Option<(PlayerRel, Option<Filter>)> {
    let (s, by) = if let Some(x) = s.strip_suffix(" an opponent controls") {
        (x, PlayerRel::Opponent)
    } else if let Some(x) = s.strip_suffix(" you control") {
        (x, PlayerRel::You)
    } else {
        (s, PlayerRel::Any)
    };
    let not_spell = || Filter::not(Filter::Spell);
    let f = match s {
        "a spell or ability" => None,
        "a spell" => Some(Filter::Spell),
        "an ability" => Some(not_spell()),
        "an activated ability" => Some(Filter::Custom(
            crate::kw::activated_ability_kind::ACTIVATED_ABILITY.into(),
        )),
        "a backup ability" => Some(Filter::Custom(
            crate::kw::activated_ability_kind::BACKUP_ABILITY.into(),
        )),
        "an ability that targets only it" => Some(Filter::and(vec![
            not_spell(),
            Filter::StackTargets(Box::new(TargetsFilter::Only {
                objects: Some(Filter::Source),
                players: None,
            })),
        ])),
        _ => {
            let x = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
            let (f, c) = super::triggers::parse_spell_phrase(x)?;
            if c.is_some() {
                return None;
            }
            Some(f)
        }
    };
    Some((by, f))
}

/// "[subject] become(s) the target of [what]": objects, players ("you", "you or a
/// permanent you control", "a player or permanent"), and "you and/or at least one
/// permanent you control" / "one or more creatures you control" (once per spell or
/// ability).
fn becomes_target(r: &str) -> Option<Parsed> {
    let (subj, what) = r
        .split_once(" becomes the target of ")
        .or_else(|| r.split_once(" become the target of "))?;
    let (by, kind) = targeting_what(what)?;
    let only_kind = |c: TriggerCond| match &kind {
        None => c,
        Some(f) => TriggerCond::Where {
            trigger: Box::new(c),
            cond: Condition::SelMatches(Sel::TriggerSpell, f.clone()),
        },
    };
    let player_target = |who: PlayerRel| {
        TriggerCond::Custom(crate::kw::activated_ability_kind::player_targeted(who, by).into())
    };
    let perm = |other: bool| {
        let mut v = vec![Filter::Permanent, Filter::ControlledBy(PlayerRel::You)];
        if other {
            v.push(Filter::Other);
        }
        TriggerCond::BecomesTarget {
            filter: Filter::and(v),
            by,
        }
    };
    let tp = PlayerRef::TriggerPlayer;
    let c = match subj {
        "you" => (only_kind(player_target(PlayerRel::You)), Sel::TriggerSpell, tp),
        "you or a permanent you control" | "you or another permanent you control" => (
            only_kind(TriggerCond::AnyOf(vec![
                player_target(PlayerRel::You),
                perm(subj.contains("another")),
            ])),
            Sel::TriggerSpell,
            tp,
        ),
        "you and/or at least one permanent you control" => (
            TriggerCond::Batched {
                trigger: Box::new(only_kind(TriggerCond::AnyOf(vec![
                    player_target(PlayerRel::You),
                    perm(false),
                ]))),
                per: BatchPer::Batch,
            },
            Sel::TriggerSpell,
            tp,
        ),
        "a player or permanent" => (
            only_kind(TriggerCond::AnyOf(vec![
                player_target(PlayerRel::Any),
                TriggerCond::BecomesTarget {
                    filter: Filter::Permanent,
                    by,
                },
            ])),
            Sel::TriggerSpell,
            tp,
        ),
        _ => {
            let s = super::triggers::parse_subject(subj)?;
            let c = only_kind(TriggerCond::BecomesTarget {
                filter: s.filter,
                by,
            });
            if s.one_or_more {
                (
                    TriggerCond::Batched {
                        trigger: Box::new(c),
                        per: BatchPer::Batch,
                    },
                    Sel::TriggerSpell,
                    tp,
                )
            } else if s.self_only {
                (c, Sel::This, tp)
            } else {
                (c, Sel::TriggerObject, tp)
            }
        }
    };
    Some(c)
}

/// "~ enters or becomes the target of an Aura spell", "~ enters or enchanted creature
/// becomes the target of an Aura spell": either event.
fn either_condition(r: &str) -> Option<Parsed> {
    for (i, _) in r.match_indices(" or ") {
        let (a, b) = (&r[..i], &r[i + " or ".len()..]);
        let b_full = if b.starts_with("becomes ") || b.starts_with("attacks") {
            // The same subject: "~ enters or becomes …".
            let subj = a.split_once(' ')?.0;
            if subj != "~" {
                continue;
            }
            format!("{subj} {b}")
        } else if ["enchanted ", "equipped ", "~ ", "you "]
            .iter()
            .any(|p| b.starts_with(p))
        {
            b.to_string()
        } else {
            continue;
        };
        let (Some(x), Some(y)) = (reparse(a), reparse(&b_full)) else {
            continue;
        };
        return Some(any_of(vec![x, y]));
    }
    None
}

/// "When ~ enters and whenever one or more Assassins you control deal combat damage to a
/// player": each condition (CR 603.2c); the batched one triggers once per batch.
fn and_whenever_batched(r: &str) -> Option<Parsed> {
    let (a, b) = r.split_once(" and whenever ")?;
    let x = reparse(a)?;
    let y = reparse(b)?;
    if !super::trigger_grammar_events::is_batched(&y.0)
        || super::trigger_grammar_events::is_batched(&x.0)
    {
        return None;
    }
    Some(any_of(vec![x, y]))
}

// ---------------------------------------------------------------------------
// Referents named by the trigger event
// ---------------------------------------------------------------------------

thread_local! {
    static NAMED: std::cell::RefCell<Option<Vec<(String, Sel)>>> = const { std::cell::RefCell::new(None) };
}

/// Sets the phrases the trigger condition being compiled names for its body (cleared
/// when the returned guard is dropped, or taken by the body's parser).
pub(crate) fn name_referents(trigger: &TriggerCond) -> NamedGuard {
    let v = named_referents(trigger);
    NAMED.with(|n| *n.borrow_mut() = Some(v));
    NamedGuard
}

pub(crate) struct NamedGuard;

impl Drop for NamedGuard {
    fn drop(&mut self) {
        NAMED.with(|n| *n.borrow_mut() = None);
    }
}

/// The phrases named by the trigger whose body is being parsed (only the body itself:
/// abilities nested in it get none).
pub(crate) fn take_named() -> Vec<(String, Sel)> {
    NAMED.with(|n| n.borrow_mut().take()).unwrap_or_default()
}

fn mentions(f: &Filter, t: CardType) -> bool {
    match f {
        Filter::Type(x) => *x == t,
        Filter::And(v) => v.iter().any(|g| mentions(g, t)),
        Filter::Or(v) => !v.is_empty() && v.iter().all(|g| mentions(g, t)),
        _ => false,
    }
}

fn subtypes(f: &Filter) -> Vec<String> {
    match f {
        Filter::Subtype(s) => vec![s.to_lowercase()],
        Filter::And(v) => v.iter().flat_map(subtypes).collect(),
        _ => vec![],
    }
}

/// Phrases a trigger event names: in "Whenever a Sliver deals combat damage to a creature,
/// destroy that creature", "that creature" is the creature dealt damage; in "Whenever a
/// creature blocks a black or red creature, the blocking creature gets +1/+1", the
/// blocker; in "Whenever an Archer you control deals damage to a creature, that Archer
/// deals that much damage to that creature's controller", the source.
fn named_referents(trigger: &TriggerCond) -> Vec<(String, Sel)> {
    let mut out: Vec<(String, Sel)> = Vec::new();
    let mut push = |p: &str, s: Sel| {
        if !out.iter().any(|(q, _)| q == p) {
            out.push((p.to_string(), s));
        }
    };
    match trigger {
        TriggerCond::Where { trigger, .. }
        | TriggerCond::FirstTimeEachTurn(trigger)
        | TriggerCond::Noncombat(trigger) => return named_referents(trigger),
        // Once per source dealing damage ("Whenever a red creature or spell deals damage"):
        // that source is one object. (A batch of several sources names none.)
        TriggerCond::Batched {
            trigger,
            per: BatchPer::Other,
        } => return named_referents(trigger),
        TriggerCond::DealsDamage { source, to, .. } => {
            let source_self = matches!(source, Filter::Source);
            if !source_self {
                push("that source", Sel::TriggerOtherObject);
                // "a red creature or spell deals damage, ... that creature's or spell's
                // controller" (Justice).
                if let Filter::And(v) | Filter::Or(v) = source {
                    if v.iter().any(|g| matches!(g, Filter::Or(w) if w.len() == 2
                        && mentions(&w[0], CardType::Creature)
                        && matches!(&w[1], Filter::Spell)))
                    {
                        push("that creature's or spell", Sel::TriggerOtherObject);
                    }
                }
                for s in subtypes(source) {
                    push(&format!("that {s}"), Sel::TriggerOtherObject);
                }
            }
            match to {
                DamageRecipient::Object(f) if !matches!(f, Filter::Source) => {
                    for (t, w) in [
                        (CardType::Creature, "that creature"),
                        (CardType::Planeswalker, "that planeswalker"),
                        (CardType::Battle, "that battle"),
                    ] {
                        if mentions(f, t) {
                            push(w, Sel::TriggerObject);
                        }
                    }
                    push("that permanent", Sel::TriggerObject);
                    // "Whenever a creature deals damage to enchanted planeswalker, destroy
                    // that creature": the source, when the recipient isn't a creature.
                    if !mentions(f, CardType::Creature) && mentions(source, CardType::Creature) {
                        push("that creature", Sel::TriggerOtherObject);
                    }
                }
                // Damage dealt to ~: the source is the only other object.
                DamageRecipient::Object(_) if !source_self => {
                    if mentions(source, CardType::Creature) {
                        push("that creature", Sel::TriggerOtherObject);
                    }
                    if matches!(source, Filter::Spell)
                        || matches!(source, Filter::And(v) if v.iter().any(|g| matches!(g, Filter::Spell)))
                    {
                        push("that spell", Sel::TriggerOtherObject);
                    }
                }
                _ => {}
            }
        }
        // Several recipients: only the source is common to every event.
        TriggerCond::AnyOf(v)
            if !v.is_empty()
                && v.iter().all(|c| {
                    matches!(
                        strip(c),
                        TriggerCond::DealsDamage { source, .. } if !matches!(source, Filter::Source)
                    )
                }) =>
        {
            if let TriggerCond::DealsDamage { source, .. } = strip(&v[0]) {
                push("that source", Sel::TriggerOtherObject);
                if mentions(source, CardType::Creature) {
                    push("that creature", Sel::TriggerOtherObject);
                }
            }
        }
        // Event object = the blocked attacker, other = the blocker (CR 509.3b).
        TriggerCond::BlocksCreature { .. } => {
            push("the blocking creature", Sel::TriggerOtherObject);
            push("that blocking creature", Sel::TriggerOtherObject);
            push("the attacking creature", Sel::TriggerObject);
            push("that attacking creature", Sel::TriggerObject);
        }
        // Event object = the blocker, other = the attacker (CR 509.3d).
        TriggerCond::BlockedByCreature { .. } => {
            push("the blocking creature", Sel::TriggerObject);
            push("that blocking creature", Sel::TriggerObject);
            push("the attacking creature", Sel::TriggerOtherObject);
            push("that attacking creature", Sel::TriggerOtherObject);
        }
        // The spell or ability that targeted it (CR 115.10).
        TriggerCond::BecomesTarget { .. } => {
            push("that ability", Sel::TriggerSpell);
        }
        TriggerCond::BlockedByN { .. } => {
            push("the attacking creature", Sel::TriggerObject);
            push("that attacking creature", Sel::TriggerObject);
        }
        _ => {}
    }
    out
}

/// A trigger condition without its event qualifiers.
fn strip(c: &TriggerCond) -> &TriggerCond {
    match c {
        TriggerCond::Where { trigger, .. }
        | TriggerCond::FirstTimeEachTurn(trigger)
        | TriggerCond::Noncombat(trigger) => strip(trigger),
        c => c,
    }
}

/// "[permanent] enters from anywhere other than your hand": a permanent whose previous
/// object (CR 400.7) wasn't in a hand (The Lost and the Damned).
fn enters_not_from_hand(r: &str) -> Option<Parsed> {
    let s = r.strip_suffix(" enters from anywhere other than your hand")?;
    let subj = super::triggers::parse_subject(s)?;
    if subj.one_or_more || subj.self_only {
        return None;
    }
    let f = Filter::and(vec![
        subj.filter,
        Filter::not(Filter::Custom(
            crate::kw::played_from_zone::came_from(ZoneKind::Hand).into(),
        )),
    ]);
    Some((
        TriggerCond::EntersBattlefield(f),
        Sel::TriggerObject,
        PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
    ))
}

/// The source is on the battlefield: "… while ~ is on the battlefield" on a condition of an
/// ability that also functions from other zones.
fn source_on_battlefield() -> Condition {
    Condition::SelMatches(Sel::This, Filter::Permanent)
}

/// Whether a trigger alternative only triggers while its source is on the battlefield
/// ("cycle another card while ~ is on the battlefield"), so the ability may function from
/// anywhere for its other alternatives (CR 113.6).
pub(crate) fn requires_source_on_battlefield(t: &TriggerCond) -> bool {
    matches!(t, TriggerCond::Where { cond, .. }
        if format!("{cond:?}") == format!("{:?}", source_on_battlefield()))
}

/// "you cycle ~ or cycle another card while ~ is on the battlefield" (Astral Drift,
/// CR 702.29c-d).
fn cycle_this_or_another(r: &str) -> Option<Parsed> {
    if r != "you cycle ~ or cycle another card while ~ is on the battlefield" {
        return None;
    }
    Some((
        TriggerCond::AnyOf(vec![
            TriggerCond::Cycled {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::Cycled {
                    who: PlayerRel::You,
                    filter: Filter::Other,
                }),
                cond: source_on_battlefield(),
            },
        ]),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

/// "~ attacks while you have the most life or are tied for most life" (Preacher of the
/// Schism).
fn attacks_while_most_life(r: &str) -> Option<Parsed> {
    let s = r.strip_suffix(" attacks while you have the most life or are tied for most life")?;
    let subj = super::triggers::parse_subject(s)?;
    if subj.one_or_more {
        return None;
    }
    let most = Value::OverPlayers(
        AggOp::Max,
        PlayerFilter::Any,
        Box::new(Value::LifeTotal(PlayerRef::Iterated)),
    );
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::Attacks(subj.filter)),
            cond: Condition::Compare(Value::LifeTotal(PlayerRef::You), Cmp::Ge, most),
        },
        if subj.self_only {
            Sel::This
        } else {
            Sel::TriggerObject
        },
        PlayerRef::TriggerPlayer,
    ))
}

/// "At the beginning of each combat this turn, [effect]" in a spell (Full Throttle): a
/// delayed triggered ability that lasts until end of turn (CR 603.7b).
fn each_combat_this_turn(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("at the beginning of each combat this turn, ")?;
    let body = crate::oracle::effects::parse_trigger_body(
        r,
        b.ctx,
        super::oracle_hardening_referents::no_referent(),
        PlayerRef::ActivePlayer,
    )?;
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(TriggerCond::BeginningOf {
            step: TriggerStep::BeginningOfCombat,
            whose: PlayerRel::Any,
        })),
        body: Box::new(body),
        once: false,
    })
}

inventory::submit! { super::EffectPattern { name: "at the beginning of each combat this turn, …", priority: 100, parse: each_combat_this_turn } }

/// "counter that ability" in an ability that triggers on activating one (Imprison): the
/// ability activated (CR 701.6b).
fn counter_that_ability(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    if end(l) != "counter that ability" || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    Some(Effect::CounterSpell {
        what: Sel::TriggerSpell,
    })
}

inventory::submit! { super::EffectPattern { name: "counter that ability (the one activated)", priority: 100, parse: counter_that_ability } }

/// "At the end of the first combat phase on your turn" (CR 511.2: as the end of combat
/// step begins).
fn end_of_first_combat(r: &str) -> Option<Parsed> {
    let whose = match r {
        "at the end of the first combat phase on your turn" => PlayerRel::You,
        "at the end of the first combat phase of each turn" => PlayerRel::Any,
        _ => return None,
    };
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::BeginningOf {
                step: TriggerStep::EndOfCombat,
                whose,
            }),
            cond: Condition::Custom(crate::kw::activated_ability_kind::FIRST_COMBAT_PHASE.into()),
        },
        Sel::This,
        PlayerRef::ActivePlayer,
    ))
}

/// "all non-Wall creatures you control attack" (Mob Mentality): you attack, and every such
/// creature is attacking.
fn all_attack(r: &str) -> Option<Parsed> {
    let s = r.strip_prefix("all ")?.strip_suffix(" attack")?;
    let (f, plural, tail) = parse_object_phrase(s)?;
    if !plural || !end(tail).is_empty() || controlled_by(&f) != Some(PlayerRel::You) {
        return None;
    }
    let idle = Filter::and(vec![f, Filter::creature(), Filter::not(Filter::Attacking)]);
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAttacks(PlayerRel::You)),
            cond: Condition::Not(Box::new(Condition::Exists(idle))),
        },
        Sel::TriggerObjects,
        PlayerRef::TriggerPlayer,
    ))
}

/// "you attack a player or planeswalker with one or more creatures with power 1 or less"
/// (Rigo, Streetwise Mentor): once for each player or planeswalker attacked (CR 508.3b)
/// by such a creature of yours.
fn attack_player_or_planeswalker_with(r: &str) -> Option<Parsed> {
    let x = r.strip_prefix("you attack a player or planeswalker with one or more ")?;
    let (f, plural, tail) = parse_object_phrase(x)?;
    if !plural || !end(tail).is_empty() {
        return None;
    }
    let with = Value::Count(Filter::and(vec![
        f,
        Filter::creature(),
        Filter::In(Box::new(Sel::TriggerObjects)),
    ]));
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::IsAttacked(DamageRecipient::PlayerOrPlaneswalker(
                PlayerRel::Any,
            ))),
            // You're the attacking (active) player.
            cond: Condition::And(vec![
                Condition::YourTurn,
                Condition::Compare(with, Cmp::Ge, Value::c(1)),
            ]),
        },
        Sel::TriggerObjects,
        PlayerRef::TriggerPlayer,
    ))
}

/// "if two or more of those creatures are attacking you and/or planeswalkers you control"
/// after "Whenever an opponent attacks with creatures" (Mangara, the Diplomat): the
/// creatures that attacker declared (CR 506.2).
fn those_attacking_you(c: &str) -> Option<Condition> {
    let x = c.strip_suffix(" of those creatures are attacking you and/or planeswalkers you control")?;
    let (n, tail) = parse_number(x.strip_suffix(" or more").unwrap_or(x))?;
    if !tail.trim().is_empty() || !x.ends_with(" or more") {
        return None;
    }
    let those = Filter::and(vec![
        Filter::In(Box::new(Sel::TriggerObjects)),
        Filter::Custom(crate::kw::activated_ability_kind::ATTACKING_YOU_OR_YOUR_PLANESWALKER.into()),
    ]);
    Some(Condition::Compare(Value::Count(those), Cmp::Ge, n))
}

inventory::submit! { ConditionPattern { name: "N or more of those creatures are attacking you and/or planeswalkers you control", priority: 100, parse: those_attacking_you } }

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
        .or_else(|| attacks_with_others(r))
        .or_else(|| attacks_recipient(r))
        .or_else(|| player_attacks(r))
        .or_else(|| creatures_attack(r))
        .or_else(|| attacks_or_enters_attacking(r))
        .or_else(|| block_events(r))
        .or_else(|| deals_damage(r))
        .or_else(|| dealt_damage(r))
        .or_else(|| becomes_target(r))
        .or_else(|| and_whenever_batched(r))
        .or_else(|| either_condition(r))
        .or_else(|| enters_not_from_hand(r))
        .or_else(|| cycle_this_or_another(r))
        .or_else(|| attacks_while_most_life(r))
        .or_else(|| end_of_first_combat(r))
        .or_else(|| all_attack(r))
        .or_else(|| attack_player_or_planeswalker_with(r))
        .or_else(|| during_combat(r))
}

inventory::submit! { TriggerPattern { name: "trigger grammar II: cast, play, combat, damage, targets", priority: 150, parse } }

#[cfg(test)]
mod tests {
    use super::*;

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
            "whenever a player kicks a spell",
            "whenever a player casts a card",
            "whenever a land you control enters from anywhere other than your hand or you cast a spell from anywhere other than your hand",
            "whenever you cycle ~ or cycle another card while ~ is on the battlefield",
            "whenever you cast a creature spell that doesn't share a creature type with a creature you control or a creature card in your graveyard",
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
    fn combat_conditions() {
        all(&[
            "whenever ~ and at least one other creature token attack",
            "whenever ~ and at least one human attack",
            "whenever ~ and at least two zombies attack",
            "whenever equipped creature and at least one other creature attack",
            "whenever you attack with ~ and another legendary creature",
            "whenever you attack with ~ and/or your commander",
            "whenever ~ attacks a battle",
            "whenever ~ attacks a player who controls eight or more lands",
            "whenever ~ attacks a player who has more life than you",
            "whenever equipped creature attacks the monarch",
            "whenever ~ attacks a player and isn't blocked",
            "whenever a creature attacks one of your opponents or a planeswalker an opponent controls",
            "when you attack with exactly two creatures",
            "whenever you attack with at least two creatures that have first strike",
            "whenever you attack with two or more non-~ creatures",
            "whenever you attack with your commander",
            "whenever you attack with a creature an opponent owns",
            "whenever an opponent attacks with creatures",
            "whenever an opponent attacks a planeswalker you control with one or more creatures",
            "whenever an opponent attacks one or more planeswalkers you control",
            "whenever an opponent attacks you and/or one or more planeswalkers you control",
            "whenever two or more creatures attack",
            "whenever two or more creatures your opponents control attack",
            "whenever two or more creatures you control attack a player",
            "whenever three or more creatures you control with flying attack",
            "whenever one or more devils you control attack one or more players",
            "whenever one or more creatures an opponent controls attack you and aren't blocked",
            "whenever one or more creatures attack one of your opponents or a planeswalker they control",
            "whenever a creature you control attacks or enters attacking",
            "whenever a creature with deathtouch blocks or becomes blocked by ~",
            "whenever enchanted creature blocks or becomes blocked by a non-wall creature",
            "whenever equipped creature blocks or becomes blocked by a vampire",
            "whenever ~ blocks or becomes blocked by one or more black creatures",
            "whenever ~ blocks or becomes blocked by one or more blue and/or black creatures",
            "whenever a creature blocks a creature with lesser power",
            "whenever a creature becomes blocked by a creature with lesser power",
            "whenever a creature attacking one of your opponents becomes blocked by two or more creatures",
            "whenever a creature blocks a black or red creature",
            "whenever ~ blocks one or more black creatures",
            "whenever ~ blocks two or more creatures",
            "whenever one or more creatures block",
            "at the end of the first combat phase on your turn",
            "whenever all non-wall creatures you control attack",
            "whenever you attack a player or planeswalker with one or more creatures with power 1 or less",
            "whenever you attack the player who has the initiative",
            "whenever ~ attacks while you have the most life or are tied for most life",
            "whenever a creature you control that was turned face up this turn deals combat damage to a player",
            "whenever one or more creatures that are enchanted by an aura you control attack",
            "whenever ~ blocks or becomes blocked by a creature that has been dealt damage this turn",
        ]);
    }

    #[test]
    fn damage_conditions() {
        all(&[
            "whenever a red source you control deals damage to one or more permanents and/or players",
            "whenever a noncreature source you control deals damage",
            "whenever a source of the chosen color deals damage to you",
            "whenever a source you control deals damage to another player",
            "whenever a source you control other than ~ deals damage to an opponent",
            "whenever a source an opponent controls deals damage to you or a permanent you control",
            "whenever a source you control deals noncombat damage to one or more of your opponents during your turn",
            "whenever a creature deals combat damage to enchanted player",
            "whenever a creature deals combat damage to its owner",
            "whenever a creature deals damage to enchanted planeswalker",
            "when ~ deals combat damage to a player who controls more lands than you",
            "whenever one or more pirates you control deal damage to your opponents",
            "whenever one or more zombies you control deal combat damage to one or more of your opponents",
            "whenever one or more creatures you control deal combat damage to one or more players",
            "whenever an instant or sorcery spell you control deals damage to an opponent or battle",
            "whenever ~ deals combat damage to one or more blocking creatures",
            "whenever ~ deals damage to a creature or opponent",
            "whenever ~ deals damage to one or more creatures",
            "when you're dealt combat damage",
            "whenever combat damage is dealt to you",
            "whenever combat damage is dealt to you or a planeswalker you control",
            "whenever your opponents are dealt combat damage",
            "when one or more of your opponents are dealt combat damage during your turn",
            "whenever one or more opponents are dealt noncombat damage",
            "whenever an opponent is dealt combat damage by one or more creatures you control",
            "whenever an opponent is dealt damage by a red instant or sorcery spell you control or by a red planeswalker you control",
            "whenever one or more creatures your opponents control are dealt excess noncombat damage",
            "when ~ enters and whenever one or more assassins you control deal combat damage to a player",
        ]);
    }

    #[test]
    fn target_conditions() {
        all(&[
            "whenever you become the target of a spell or ability an opponent controls",
            "whenever you become the target of a spell",
            "whenever you or a permanent you control becomes the target of a spell or ability an opponent controls",
            "whenever you or another permanent you control becomes the target of a spell or ability an opponent controls",
            "whenever you and/or at least one permanent you control becomes the target of a spell or ability an opponent controls",
            "whenever a player or permanent becomes the target of an ability you control",
            "whenever a wolf or werewolf you control becomes the target of an instant or sorcery spell",
            "whenever one or more creatures you control become the target of an activated ability",
            "whenever ~ becomes the target of an aura spell",
            "whenever ~ becomes the target of an ability that targets only it",
            "whenever ~ enters or becomes the target of an aura spell",
            "whenever ~ enters or enchanted creature becomes the target of an aura spell",
            "whenever a creature you control becomes the target of a backup ability",
        ]);
    }

    fn cond(s: &str) -> TriggerCond {
        crate::oracle::triggers::parse_trigger_condition(s)
            .unwrap_or_else(|| panic!("failed to parse {s:?}"))
            .0
    }

    #[test]
    fn batching_follows_the_wording() {
        // "you or a permanent you control": once for each of them targeted (Unsettled
        // Mariner's ruling); "you and/or at least one permanent you control": once per spell
        // or ability (Leyline of Combustion's ruling).
        assert!(matches!(
            cond("whenever you or a permanent you control becomes the target of a spell or ability an opponent controls"),
            TriggerCond::AnyOf(_)
        ));
        assert!(matches!(
            cond("whenever you and/or at least one permanent you control becomes the target of a spell or ability an opponent controls"),
            TriggerCond::Batched { per: BatchPer::Batch, .. }
        ));
        // Several sources at once: one batch; one source dealing damage to several
        // recipients: once per source.
        assert!(matches!(
            cond("whenever one or more pirates you control deal damage to your opponents"),
            TriggerCond::Batched { per: BatchPer::Batch, .. }
        ));
        assert!(matches!(
            cond("whenever ~ deals damage to one or more creatures"),
            TriggerCond::Batched { per: BatchPer::Other, .. }
        ));
        // "a creature or opponent": each damage event.
        assert!(matches!(
            cond("whenever ~ deals damage to a creature or opponent"),
            TriggerCond::AnyOf(_)
        ));
    }

    #[test]
    fn named_referents_follow_the_event() {
        let names = |s: &str| -> Vec<(String, String)> {
            named_referents(&cond(s))
                .into_iter()
                .map(|(p, s)| (p, format!("{s:?}")))
                .collect()
        };
        let has = |v: &[(String, String)], p: &str, s: &str| v.iter().any(|(a, b)| a == p && b == s);
        let v = names("whenever a sliver deals combat damage to a creature");
        assert!(has(&v, "that creature", "TriggerObject"));
        assert!(has(&v, "that sliver", "TriggerOtherObject"));
        let v = names("whenever a creature deals damage to enchanted planeswalker");
        assert!(has(&v, "that creature", "TriggerOtherObject"));
        let v = names("whenever a creature blocks a black or red creature");
        assert!(has(&v, "the blocking creature", "TriggerOtherObject"));
        let v = names("whenever a creature becomes blocked by a creature with lesser power");
        assert!(has(&v, "the blocking creature", "TriggerObject"));
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
            "whenever you activate a power-up ability",
            "whenever an opponent activates an ability of an artifact, creature, or land on the battlefield, if it isn't a mana ability",
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
            let raw = l.trim();
            if let Some(x) = raw.strip_prefix("text ") {
                let tl = crate::types::TypeLine::parse("Creature — Human");
                let ctx = crate::oracle::CompileContext {
                    card_name: "Probe",
                    full_name: "Probe",
                    type_line: &tl,
                    layout: crate::card::Layout::Normal,
                    face_index: 0,
                    keywords: &[],
                    power: None,
                    toughness: None,
                };
                let r = crate::oracle::parse_ability(x, &ctx);
                println!("PROBE TEXT {x}\n   {}", if r.is_some() { "OK" } else { "FAIL" });
                continue;
            }
            let l = raw.to_lowercase();
            if l.is_empty() {
                continue;
            }
            if let Some(x) = l.strip_prefix("text ") {
                // A whole ability of a creature card named "Probe".
                let tl = crate::types::TypeLine::parse("Creature — Human");
                let ctx = crate::oracle::CompileContext {
                    card_name: "Probe",
                    full_name: "Probe",
                    type_line: &tl,
                    layout: crate::card::Layout::Normal,
                    face_index: 0,
                    keywords: &[],
                    power: None,
                    toughness: None,
                };
                let r = crate::oracle::parse_ability(x, &ctx);
                println!("PROBE TEXT {x}\n   {}", if r.is_some() { "OK" } else { "FAIL" });
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
