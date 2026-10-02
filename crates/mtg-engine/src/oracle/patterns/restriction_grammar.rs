//! The grammar of restrictions and requirements: SUBJECT + "can't"/"must"/"can only" +
//! ACTION (+ DURATION), as static abilities and as effects of resolving spells and
//! abilities (CR 508.1c–d, 509.1b–c, 611.2c).
//!
//! * [`object_predicate`]: predicates about objects that the core predicate parser
//!   (`statics::restriction_predicate`) falls back to — "can't attack alone", "can only
//!   attack alone" (CR 506.5), "can block an additional seven creatures each combat",
//!   "can't be blocked except by six or more creatures", "must be blocked by two or more
//!   creatures if able", "must be blocked by a Dalek if able", "can't block or be blocked
//!   by non-Spirit creatures", "can't attack, block, or transform", "can't be the target
//!   of black or red spells your opponents control", "can't become untapped", "can't
//!   phase in", "can't be turned face up", "can't be equipped", "can't be enchanted by
//!   other Auras", "can't become suspected".
//! * Static lines: "No more than two creatures can attack you each combat", "All Walls
//!   able to block ~ do so", "[players] can't block with [creatures]", "[cards] in
//!   graveyards can't enter the battlefield", "Players can't get counters", "Counters
//!   can't be put on artifacts, creatures, enchantments, or lands".
//! * Effects: "[players] can't gain life this turn", "[spells] can't be countered this
//!   turn", "The next creature spell you cast this turn can't be countered", "Target
//!   spell can't be countered", "[objects] can't be the target of ... this turn", and
//!   combat restrictions with leading durations ("Until your next turn, ...").

use super::{EffectPattern, FilterSuffixPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{duration_suffix, Builder};
use crate::oracle::patterns::statics::{
    filter_mentions, restriction_predicate, targeting_sources, union_nouns, whole_object_phrase,
};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;
use crate::types::*;

/// A count: "one", "seven", "ninety-nine".
fn count(s: &str) -> Option<(u32, &str)> {
    if let Some(r) = s.strip_prefix("ninety-nine") {
        return Some((99, r.trim_start()));
    }
    let (v, r) = parse_number(s)?;
    let n = v.as_const()?;
    (n >= 0).then_some((n as u32, r.trim_start()))
}

/// "creature" / "creatures".
fn creatures_word(s: &str) -> bool {
    matches!(s.trim(), "creature" | "creatures")
}

/// What "can't [be] X" prohibits for objects ([`Restriction::CantBe`]).
fn object_action(x: &str) -> Option<ObjectAction> {
    Some(match x {
        "become untapped" => ObjectAction::Untapped,
        "phase in" => ObjectAction::PhasedIn,
        "be turned face up" => ObjectAction::TurnedFaceUp,
        "be equipped" => ObjectAction::Equipped,
        "be enchanted by other auras" => ObjectAction::EnchantedByOtherAuras,
        "become suspected" => ObjectAction::Suspected,
        _ => return None,
    })
}

/// "can't be the target(s) of [X] your opponents control (or [Y] your opponents
/// control)": spells and abilities the restriction's controller's opponents control
/// whose sources have the quality (CR 115.4).
fn opponents_targeting(x: &str) -> Option<TargetRestriction> {
    let x = x.strip_suffix(" your opponents control")?;
    if x == "abilities" {
        return Some(TargetRestriction::OpponentsSources(Filter::not(
            Filter::Spell,
        )));
    }
    // "nongreen spells your opponents control or abilities from nongreen sources your
    // opponents control".
    let x = x.replace(" your opponents control or ", " or ");
    let f = targeting_sources(&x)?;
    Some(TargetRestriction::OpponentsSources(f))
}

/// Predicates about objects that `restriction_predicate` doesn't know itself.
pub(crate) fn object_predicate(p: &str, f: &Filter) -> Option<Vec<Restriction>> {
    let fc = f.clone();
    let p = end(p.trim());
    match p {
        "can't attack alone" => return Some(vec![Restriction::CantAttackAlone(fc)]),
        "can't block alone" => return Some(vec![Restriction::CantBlockAlone(fc)]),
        "can't attack or block alone" => {
            return Some(vec![
                Restriction::CantAttackAlone(fc.clone()),
                Restriction::CantBlockAlone(fc),
            ])
        }
        // CR 506.5.
        "can only attack alone" => return Some(vec![Restriction::AttackOnlyAlone(fc)]),
        "can't transform" => return Some(vec![Restriction::CantTransform(fc)]),
        "must be blocked each combat if able" => {
            return Some(vec![Restriction::MustBeBlocked(fc)])
        }
        "can block any number of creatures" | "can block any number of creatures each combat" => {
            return Some(vec![Restriction::ExtraBlocks {
                blocker: fc,
                n: None,
            }])
        }
        _ => {}
    }
    // "can block an additional [N] creature(s) [each combat]".
    if let Some(r) = p.strip_prefix("can block an additional ") {
        let r = r.strip_suffix(" each combat").unwrap_or(r);
        let (n, noun) = if creatures_word(r) {
            (1, r)
        } else {
            count(r)?
        };
        if !creatures_word(noun) {
            return None;
        }
        return Some(vec![Restriction::ExtraBlocks {
            blocker: fc,
            n: Some(n),
        }]);
    }
    // "can't be blocked except by N or more creatures" (CR 509.1b).
    if let Some(r) = p.strip_prefix("can't be blocked except by ") {
        let (n, rest) = count(r)?;
        let noun = rest.strip_prefix("or more ")?;
        if !creatures_word(noun) || n < 2 {
            return None;
        }
        return Some(vec![Restriction::MinBlockers { attacker: fc, n }]);
    }
    // Blocking requirements on the attacker (CR 509.1c).
    if let Some(r) = p
        .strip_prefix("must be blocked by ")
        .and_then(|r| r.strip_suffix(" if able"))
    {
        if let Some(r) = r.strip_prefix("exactly ") {
            let (n, noun) = count(r)?;
            if !creatures_word(noun) {
                return None;
            }
            return Some(vec![Restriction::BlockerCountRequirement {
                attacker: fc,
                min: n,
                max: Some(n),
            }]);
        }
        if let Some((n, rest)) = count(r) {
            if let Some(noun) = rest.strip_prefix("or more ") {
                if !creatures_word(noun) {
                    return None;
                }
                return Some(vec![Restriction::BlockerCountRequirement {
                    attacker: fc,
                    min: n,
                    max: None,
                }]);
            }
        }
        // "must be blocked by a Dalek if able".
        let noun = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
        let (b, plural) = whole_object_phrase(noun)?;
        if plural {
            return None;
        }
        return Some(vec![Restriction::MustBeBlockedBy {
            attacker: fc,
            blocker: b,
        }]);
    }
    // "can't block or be blocked by non-Spirit creatures": both ways.
    if let Some(r) = p.strip_prefix("can't block or be blocked by ") {
        let (other, plural) = whole_object_phrase(&union_nouns(r))?;
        if !plural {
            return None;
        }
        return Some(vec![
            Restriction::CantBeBlockedBy {
                attacker: other.clone(),
                blocker: fc.clone(),
            },
            Restriction::CantBeBlockedBy {
                attacker: fc,
                blocker: other,
            },
        ]);
    }
    // "can't be the target of abilities your opponents control".
    if let Some(x) = p
        .strip_prefix("can't be the target of ")
        .or_else(|| p.strip_prefix("can't be the targets of "))
    {
        return Some(vec![Restriction::CantBeTargeted {
            what: fc,
            by: opponents_targeting(x)?,
        }]);
    }
    // "can't attack or block, and its activated abilities can't be activated".
    for sep in [", and ", " and "] {
        for poss in ["its activated abilities ", "their activated abilities "] {
            let key = format!("{sep}{poss}");
            if let Some(i) = p.find(&key) {
                let mut a = restriction_predicate(&p[..i], f)?;
                a.extend(restriction_predicate(&p[i + sep.len()..], f)?);
                return Some(a);
            }
        }
    }
    if let Some(x) = p.strip_prefix("can't ") {
        if let Some(action) = object_action(x) {
            return Some(vec![Restriction::CantBe { what: fc, action }]);
        }
        // "can't attack, block, or transform": each.
        if x.contains(", ") {
            let items: Vec<&str> = x
                .split(", ")
                .map(|i| i.strip_prefix("or ").unwrap_or(i))
                .collect();
            if items.len() >= 3 && x.contains(", or ") {
                let mut out = Vec::new();
                for i in items {
                    out.extend(restriction_predicate(&format!("can't {i}"), f)?);
                }
                return Some(out);
            }
        }
        // "can't become untapped and can't have counters put on it" is two predicates
        // (split by the caller); "can't attack or be enchanted ..." isn't a form cards use.
    }
    None
}

// ---------------------------------------------------------------------------
// Static lines
// ---------------------------------------------------------------------------

fn static_restrictions(rs: Vec<Restriction>, text: &str) -> Vec<Ability> {
    rs.into_iter()
        .map(|r| {
            AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(r))),
                text,
            )
        })
        .collect()
}

/// "no more than N creature(s) can attack/block [you / ~] each combat" (CR 508.1c,
/// 509.1b).
fn no_more_than(l: &str) -> Option<Vec<Restriction>> {
    let r = l.strip_prefix("no more than ")?;
    let (n, rest) = count(r)?;
    let rest = rest
        .strip_prefix("creatures ")
        .or_else(|| rest.strip_prefix("creature "))?;
    let rest = rest.strip_suffix(" each combat")?;
    Some(match rest {
        "can attack" => vec![Restriction::MaxAttackers(n)],
        "can block" => vec![Restriction::MaxBlockers(n)],
        "can attack you" => vec![Restriction::MaxAttackersAgainst {
            player: Some(PlayerFilter::You),
            object: None,
            n,
        }],
        "can attack ~" => vec![Restriction::MaxAttackersAgainst {
            player: None,
            object: Some(Filter::Source),
            n,
        }],
        _ => return None,
    })
}

/// "All [creatures] able to block [~ / enchanted creature / equipped creature] do so":
/// each such creature blocks it if able (CR 509.1c).
fn all_able_to_block(l: &str) -> Option<Vec<Restriction>> {
    let r = l.strip_prefix("all ")?.strip_suffix(" do so")?;
    let (blockers, attacker) = r.split_once(" able to block ")?;
    let attacker = match attacker {
        "~" => Filter::Source,
        "enchanted creature" | "equipped creature" => Filter::AttachedToSource,
        "~ or enchanted creature" => Filter::Or(vec![Filter::Source, Filter::AttachedToSource]),
        _ => return None,
    };
    let (b, plural) = whole_object_phrase(blockers)?;
    if !plural {
        return None;
    }
    // "All creatures able to block ~ do so" is the lure form.
    if matches!(&b, Filter::Type(CardType::Creature)) {
        return Some(vec![Restriction::MustBeBlockedByAll(attacker)]);
    }
    Some(vec![Restriction::MustBlockAttacker {
        blocker: b,
        attacker,
    }])
}

/// A player group as the subject of a restriction: "players", "your opponents", "each
/// opponent", "you", "enchanted player".
fn player_group(s: &str) -> Option<PlayerFilter> {
    Some(match s {
        "players" | "each player" => PlayerFilter::Any,
        "your opponents" | "each opponent" => PlayerFilter::Opponent,
        "you" => PlayerFilter::You,
        _ => return None,
    })
}

/// The creatures a player group controls ("your opponents" → creatures your opponents
/// control).
fn controlled_by_group(who: &PlayerFilter, f: Filter) -> Option<Filter> {
    Some(match who {
        PlayerFilter::Any => f,
        PlayerFilter::Opponent => f.opp_controls(),
        PlayerFilter::You => f.you_control(),
        _ => return None,
    })
}

/// "[players] can't block with [creatures]", "[players] can't attack with [creatures]",
/// "[players] can't block with more than one creature".
fn players_cant_use(who: &PlayerFilter, p: &str) -> Option<Vec<Restriction>> {
    if let Some(r) = p.strip_prefix("can't block with more than ") {
        let (n, noun) = count(r)?;
        if !creatures_word(noun) {
            return None;
        }
        return Some(vec![Restriction::MaxBlockersOf {
            who: who.clone(),
            n,
        }]);
    }
    for (prefix, attack) in [("can't block with ", false), ("can't attack with ", true)] {
        if let Some(r) = p.strip_prefix(prefix) {
            let (f, plural) = whole_object_phrase(r)?;
            if !plural || filter_mentions(&f, &|x| matches!(x, Filter::ControlledBy(_))) {
                return None;
            }
            let f = controlled_by_group(who, f)?;
            return Some(vec![if attack {
                Restriction::CantAttack(f)
            } else {
                Restriction::CantBlock(f)
            }]);
        }
    }
    None
}

/// "[kind] cards in graveyards [and libraries] can't enter the battlefield" (CR 614.17d):
/// cards coming from those zones.
fn cards_cant_enter(l: &str) -> Option<Vec<Restriction>> {
    let subject = l.strip_suffix(" can't enter the battlefield")?;
    let (cards, zones) = subject.split_once(" in ")?;
    let zones: Vec<ZoneKind> = match zones {
        "graveyards" => vec![ZoneKind::Graveyard],
        "graveyards and libraries" | "libraries and graveyards" => {
            vec![ZoneKind::Graveyard, ZoneKind::Library]
        }
        "libraries" => vec![ZoneKind::Library],
        _ => return None,
    };
    let (f, plural) = whole_object_phrase(cards)?;
    if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Card | Filter::PermanentCard)) {
        return None;
    }
    let zone = Filter::Or(zones.into_iter().map(Filter::InZone).collect());
    Some(vec![Restriction::CantEnterFrom {
        what: Filter::and(vec![f, zone]),
    }])
}

fn put_counters_prevented(on_objects: Option<Filter>, on_players: Option<PlayerFilter>) -> StaticEffect {
    StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::PutCounters {
            on_objects,
            on_players,
            kind: None,
        },
        action: ReplacementAction::Prevent,
        self_replacement: false,
        optional: false,
    })
}

fn restriction_static(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l.trim());
    if let Some(rs) = no_more_than(l)
        .or_else(|| all_able_to_block(l))
        .or_else(|| cards_cant_enter(l))
    {
        return Some(static_restrictions(rs, text));
    }
    // "Players can't get counters." (CR 122.1).
    if let Some(who) = l
        .strip_suffix(" can't get counters")
        .and_then(player_group)
    {
        return Some(vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(put_counters_prevented(None, Some(who)))),
            text,
        )]);
    }
    // "Counters can't be put on artifacts, creatures, enchantments, or lands."
    if let Some(r) = l.strip_prefix("counters can't be put on ") {
        let (f, plural) = whole_object_phrase(&union_nouns(r))?;
        if !plural {
            return None;
        }
        return Some(vec![AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(put_counters_prevented(
                Some(Filter::and(vec![f, Filter::Permanent])),
                None,
            ))),
            text,
        )]);
    }
    for (head, who) in [
        ("players ", PlayerFilter::Any),
        ("your opponents ", PlayerFilter::Opponent),
        ("each opponent ", PlayerFilter::Opponent),
        ("you ", PlayerFilter::You),
    ] {
        if let Some(p) = l.strip_prefix(head) {
            if let Some(rs) = players_cant_use(&who, p) {
                return Some(static_restrictions(rs, text));
            }
        }
    }
    None
}

inventory::submit! { StaticPattern { name: "restriction grammar: static restrictions", priority: 105, parse: restriction_static } }

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// A leading duration: "until your next turn, ", "until end of turn, ", "this turn, ".
fn leading_duration(l: &str) -> (Option<Duration>, &str) {
    for (p, d) in [
        ("until your next turn, ", Duration::UntilYourNextTurn),
        ("until end of turn, ", Duration::EndOfTurn),
        ("until the end of your next turn, ", Duration::UntilEndOfYourNextTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            return (Some(d), r);
        }
    }
    (None, l)
}

/// The duration of a restriction effect: leading, trailing ("this turn", "for the rest
/// of the game"), or none.
fn effect_duration(l: &str) -> Option<(Duration, &str)> {
    let (lead, l) = leading_duration(l);
    if let Some(r) = l.strip_suffix(" for the rest of the game") {
        return lead.is_none().then_some((Duration::Permanent, r));
    }
    let (d, r) = duration_suffix(l);
    match (lead, d) {
        (Some(d), Duration::Permanent) => Some((d, r)),
        (None, Duration::Permanent) => r
            .strip_suffix(" this combat")
            .map(|r| (Duration::EndOfCombat, r)),
        (None, d) => Some((d, r)),
        (Some(_), _) => None,
    }
}

fn add(rs: Vec<Restriction>, duration: Duration) -> Effect {
    Effect::seq(
        rs.into_iter()
            .map(|restriction| Effect::AddRestriction {
                restriction,
                duration: duration.clone(),
            })
            .collect(),
    )
}

/// A spell subject: "creature spells you cast", "other spells you control", "spells".
fn spells_subject(s: &str) -> Option<Filter> {
    let (s, other) = match s.strip_prefix("other ") {
        Some(r) => (r, true),
        None => (s, false),
    };
    let (head, rel) = if let Some(h) = s
        .strip_suffix(" you cast")
        .or_else(|| s.strip_suffix(" you control"))
    {
        (h, Some(PlayerRel::You))
    } else if let Some(h) = s
        .strip_suffix(" your opponents cast")
        .or_else(|| s.strip_suffix(" your opponents control"))
    {
        (h, Some(PlayerRel::Opponent))
    } else {
        (s, None)
    };
    let mut f = if head == "spells" {
        Filter::Spell
    } else {
        let (f, plural) = whole_object_phrase(&union_nouns(head))?;
        if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
            return None;
        }
        f
    };
    if let Some(rel) = rel {
        f = Filter::and(vec![f, Filter::ControlledBy(rel)]);
    }
    if other {
        f = Filter::and(vec![f, Filter::Other]);
    }
    Some(f)
}

/// A static ability a spell has: "This spell can't be countered".
fn cant_be_countered_ability() -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantBeCountered(
        Filter::Source,
    )));
    s.zone = FunctionZone::Stack;
    AbilityDef::new(AbilityKind::Static(s), "This spell can't be countered.")
}

/// "[spells] can't be countered [this turn]", "Target spell can't be countered", "The
/// next [creature] spell you cast this turn can't be countered" (CR 611.2f).
fn spells_cant_be_countered_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    // CR 611.2f: the next such spell gains the ability as it's cast.
    if let Some(r) = l
        .strip_prefix("the next ")
        .and_then(|r| r.strip_suffix(" you cast this turn can't be countered"))
    {
        let filter = if r == "spell" {
            Filter::Any
        } else {
            let head = r.strip_suffix(" spell")?;
            let (f, _) = whole_object_phrase(&format!("{head} spells"))?;
            crate::oracle::patterns::statics::without_spell(f)?
        };
        return Some(Effect::NextSpell {
            filter,
            mods: vec![Modification::AddAbility(cant_be_countered_ability())],
            expires: Duration::EndOfTurn,
        });
    }
    let (mut dur, main) = effect_duration(l).unwrap_or((Duration::Permanent, l));
    let mut subject = main.strip_suffix(" can't be countered")?;
    // "Creature spells you cast this turn can't be countered."
    if let Some(s) = subject.strip_suffix(" this turn") {
        if !matches!(dur, Duration::Permanent) {
            return None;
        }
        (dur, subject) = (Duration::EndOfTurn, s);
    }
    // "Target spell can't be countered": that spell, for as long as it's on the stack.
    if subject == "target spell" {
        let (what, rest) = crate::oracle::effects::object_ref(subject, b)?;
        if !end(&rest).is_empty() || !matches!(what, Sel::Target(_)) {
            return None;
        }
        return Some(add(
            vec![Restriction::CantBeCountered(Filter::In(Box::new(what)))],
            dur,
        ));
    }
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    let f = spells_subject(subject)?;
    Some(add(vec![Restriction::CantBeCountered(f)], dur))
}

inventory::submit! { EffectPattern { name: "restriction grammar: spells can't be countered", priority: 105, parse: spells_cant_be_countered_effect } }

/// A player subject of an effect: "players", "your opponents", "each opponent", "target
/// player", "that player", "they".
fn effect_players(s: &str, b: &mut Builder) -> Option<PlayerFilter> {
    if let Some(pf) = player_group(s) {
        return Some(pf);
    }
    let (r, rest) = crate::oracle::effects::player_ref(s, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(match r {
        PlayerRef::You => PlayerFilter::You,
        PlayerRef::EachOpponent => PlayerFilter::Opponent,
        PlayerRef::EachPlayer => PlayerFilter::Any,
        PlayerRef::Target(_)
        | PlayerRef::ControllerOf(_)
        | PlayerRef::OwnerOf(_)
        | PlayerRef::DefendingPlayer
        | PlayerRef::TriggerPlayer => PlayerFilter::Ref(Box::new(r)),
        _ => return None,
    })
}

/// "[players] can't gain life / search libraries / draw cards [this turn / for the rest
/// of the game / until your next turn]".
fn players_cant_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, main) = effect_duration(l)?;
    let (subject, pred) = main.split_once(" can't ")?;
    let who = effect_players(subject, b)?;
    let rs = match pred {
        "gain life" => vec![Restriction::CantGainLife(who)],
        "search libraries" => vec![Restriction::CantSearch(who)],
        "draw cards" => vec![Restriction::MaxDrawsPerTurn(who, 0)],
        "draw cards or gain life" => vec![
            Restriction::MaxDrawsPerTurn(who.clone(), 0),
            Restriction::CantGainLife(who),
        ],
        "lose life" => vec![Restriction::CantLoseLife(who)],
        _ => return None,
    };
    Some(add(rs, dur))
}

inventory::submit! { EffectPattern { name: "restriction grammar: players can't [action] for a duration", priority: 105, parse: players_cant_effect } }

/// "[objects] [restriction] [duration]" with any duration, for the restrictions this
/// grammar adds and the targeting ones ("Target creature can't be the target of spells
/// or abilities your opponents control this turn", "Until your next turn, target creature
/// can't attack or block").
fn objects_restriction_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, main) = effect_duration(l)?;
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    // "Its activated abilities can't be activated this turn": the object "it" names.
    let (main, possessive) = match main
        .strip_prefix("its activated abilities ")
        .map(|r| ("it", "its", r))
        .or_else(|| {
            main.strip_prefix("their activated abilities ")
                .map(|r| ("they", "their", r))
        }) {
        Some((pronoun, poss, r)) => (pronoun.to_string(), Some(format!("{poss} activated abilities {r}"))),
        None => (main.to_string(), None),
    };
    let (what, rest) = crate::oracle::effects::object_ref(&main, b)?;
    let rest = possessive.unwrap_or(rest);
    let subject = main.strip_suffix(rest.as_str()).unwrap_or(&main).trim();
    let f = subject_filter(&what, subject)?;
    let rs = restriction_predicate(end(&rest), &f)?;
    // Only restrictions a resolving effect can lock onto the objects it names.
    let ok = rs.iter().all(|r| {
        matches!(
            r,
            Restriction::CantAttack(_)
                | Restriction::CantBlock(_)
                | Restriction::CantAttackOrBlock(_)
                | Restriction::CantBeBlocked(_)
                | Restriction::MustAttack(_)
                | Restriction::MustBlock(_)
                | Restriction::MustBeBlocked(_)
                | Restriction::CantBeBlockedBy { .. }
                | Restriction::CantBeTargeted { .. }
                | Restriction::CantBe { .. }
                | Restriction::ExtraBlocks { .. }
                | Restriction::CantAttackAlone(_)
                | Restriction::CantBlockAlone(_)
                | Restriction::CantTransform(_)
                | Restriction::CantActivate { .. }
        )
    });
    if !ok || rs.is_empty() {
        return None;
    }
    Some(add(rs, dur))
}

inventory::submit! { EffectPattern { name: "restriction grammar: objects [restriction] for a duration", priority: 120, parse: objects_restriction_effect } }

/// Whether a filter describes a class of objects a rule-modifying effect can keep
/// applying to (CR 611.2c): qualities, and players fixed as the effect begins
/// ("creatures target player controls", "creatures the active player controls", see
/// `bind_target_players` in `resolve.rs`).
fn restriction_class(f: &Filter) -> bool {
    match f {
        Filter::And(v) | Filter::Or(v) => v.iter().all(restriction_class),
        Filter::Not(x) => restriction_class(x),
        Filter::ControlledBy(PlayerRel::Target(_))
        | Filter::ControlledByPlayer(_)
        | Filter::Named(_)
        | Filter::Other => true,
        other => crate::oracle::effects::is_class_filter(other),
    }
}

/// The filter of a restriction's subject: a class of objects, or the specific objects
/// named (locked in as the effect begins).
fn subject_filter(what: &Sel, subject: &str) -> Option<Filter> {
    Some(match what {
        Sel::All(f) if restriction_class(f) => f.clone(),
        Sel::All(_) | Sel::None | Sel::Players(_) => return None,
        // A pronoun with nothing else to refer to falls back to the source.
        Sel::This if !(subject.starts_with('~') || subject == "it") => return None,
        _ => Filter::In(Box::new(what.clone())),
    })
}

/// Requirements with a player or object: "[objects] attack(s) [player] [this turn / each
/// combat] if able", "[creature] blocks [creature] this turn if able", "[creature]
/// attacks or blocks this turn if able" (CR 508.1d, 509.1c).
fn requirement_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (lead, l) = leading_duration(l);
    let (body, dur) = if let Some(r) = l.strip_suffix(" this turn if able") {
        (r, Duration::EndOfTurn)
    } else if let Some(r) = l.strip_suffix(" this combat if able") {
        (r, Duration::EndOfCombat)
    } else if let Some(r) = l.strip_suffix(" each combat if able") {
        (r, lead.clone()?)
    } else {
        return None;
    };
    let dur = match (&lead, dur) {
        (Some(d), Duration::EndOfTurn) if l.ends_with(" each combat if able") => d.clone(),
        (Some(_), _) if !l.ends_with(" each combat if able") => return None,
        (_, d) => d,
    };
    let (what, rest) = crate::oracle::effects::object_ref(body, b)?;
    let subject = body.strip_suffix(rest.as_str()).unwrap_or(body).trim();
    let f = subject_filter(&what, subject)?;
    let rest = rest.trim();
    let rs = if matches!(rest, "attacks or blocks" | "attack or block") {
        vec![Restriction::MustAttack(f.clone()), Restriction::MustBlock(f)]
    } else if let Some(p) = rest
        .strip_prefix("attacks ")
        .or_else(|| rest.strip_prefix("attack "))
    {
        let defender = match p {
            "you" => PlayerFilter::You,
            "a player" => PlayerFilter::Any,
            _ => {
                let (r, tail) = crate::oracle::effects::player_ref(p, b)?;
                if !end(&tail).is_empty() {
                    return None;
                }
                match r {
                    PlayerRef::You => PlayerFilter::You,
                    PlayerRef::Target(_) | PlayerRef::TriggerPlayer | PlayerRef::ControllerOf(_) => {
                        PlayerFilter::Ref(Box::new(r))
                    }
                    _ => return None,
                }
            }
        };
        vec![Restriction::MustAttackPlayer {
            attackers: f,
            defender,
        }]
    } else if let Some(p) = rest
        .strip_prefix("blocks ")
        .or_else(|| rest.strip_prefix("block "))
    {
        if !p.starts_with("target ") {
            return None;
        }
        let (a, tail) = crate::oracle::effects::object_ref(p, b)?;
        if !end(&tail).is_empty() || !matches!(a, Sel::Target(_)) {
            return None;
        }
        vec![Restriction::MustBlockAttacker {
            blocker: f,
            attacker: Filter::In(Box::new(a)),
        }]
    } else {
        return None;
    };
    Some(add(rs, dur))
}

inventory::submit! { EffectPattern { name: "restriction grammar: requirements with a player or object", priority: 110, parse: requirement_effect } }

/// A list of whole groups as a subject: "Green creatures and white creatures", "White
/// creatures and blue creatures".
fn group_list(s: &str) -> Option<Filter> {
    let parts: Vec<&str> = s.split(" and ").collect();
    if parts.len() < 2 {
        return None;
    }
    let mut v = Vec::new();
    for p in parts {
        let (f, plural) = whole_object_phrase(p)?;
        if !plural {
            return None;
        }
        v.push(f);
    }
    Some(Filter::Or(v))
}

/// Subjects the core group parser doesn't know: "creatures named Lightning Rager",
/// "goaded creatures your opponents control", lists of whole groups.
fn extra_subject(s: &str) -> Option<Filter> {
    if let Some(f) = group_list(s) {
        return Some(f);
    }
    if let Some(r) = s.strip_prefix("goaded ") {
        let (f, plural) = whole_object_phrase(r)?;
        return plural.then(|| {
            Filter::and(vec![
                f,
                Filter::Custom(crate::kw::combat_limits::GOADED.into()),
            ])
        });
    }
    let (noun, name) = s.split_once(" named ")?;
    let (f, plural) = whole_object_phrase(noun)?;
    if !plural || name.is_empty() {
        return None;
    }
    Some(Filter::and(vec![f, Filter::Named(name.into())]))
}

/// "[extra subject] [restriction]" as a static ability, and with a duration as an effect.
fn extra_subject_static(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if ctx.is_spell() {
        return None;
    }
    let l = end(l.trim());
    for (i, _) in l.match_indices(" can") {
        let (subj, pred) = (&l[..i], l[i + 1..].trim());
        let subj = subj.replace('~', ctx.card_name);
        let Some(f) = extra_subject(&subj.to_lowercase()) else {
            continue;
        };
        let rs = restriction_predicate(pred, &f)?;
        return Some(static_restrictions(rs, text));
    }
    None
}

inventory::submit! { StaticPattern { name: "restriction grammar: named, goaded and listed subjects", priority: 105, parse: extra_subject_static } }

fn extra_subject_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, main) = effect_duration(l)?;
    if matches!(dur, Duration::Permanent) {
        return None;
    }
    for (i, _) in main.match_indices(" can") {
        let (subj, pred) = (&main[..i], main[i + 1..].trim());
        let subj = subj.replace('~', b.ctx.card_name);
        let Some(f) = extra_subject(&subj.to_lowercase()) else {
            continue;
        };
        let rs = restriction_predicate(pred, &f)?;
        return Some(add(rs, dur));
    }
    None
}

inventory::submit! { EffectPattern { name: "restriction grammar: named and listed subjects for a duration", priority: 110, parse: extra_subject_effect } }

/// "with even mana values", "with an odd mana value" (CR 202.3).
fn even_odd_mana_value<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for (p, name) in [
        ("with even mana values", crate::kw::combat_limits::EVEN_MANA_VALUE),
        ("with an even mana value", crate::kw::combat_limits::EVEN_MANA_VALUE),
        ("with odd mana values", crate::kw::combat_limits::ODD_MANA_VALUE),
        ("with an odd mana value", crate::kw::combat_limits::ODD_MANA_VALUE),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') {
                return Some((Filter::Custom(name.into()), rest));
            }
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "restriction grammar: with even/odd mana values", priority: 100, parse: even_odd_mana_value } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicates() {
        let f = Filter::Source;
        let one = |p: &str| object_predicate(p, &f).map(|v| v.len());
        assert_eq!(one("can only attack alone"), Some(1));
        assert_eq!(one("can block an additional seven creatures each combat"), Some(1));
        assert_eq!(
            one("can block an additional ninety-nine creatures each combat"),
            Some(1)
        );
        assert_eq!(one("can't be blocked except by six or more creatures"), Some(1));
        assert_eq!(one("must be blocked by two or more creatures if able"), Some(1));
        assert_eq!(one("must be blocked by exactly one creature if able"), Some(1));
        assert_eq!(one("can't block or be blocked by non-spirit creatures"), Some(2));
        assert_eq!(one("can't be the target of abilities your opponents control"), Some(1));
        assert_eq!(
            one("can't be the target of black or red spells your opponents control"),
            Some(1)
        );
        assert_eq!(one("can't become untapped"), Some(1));
        assert_eq!(one("can't be enchanted by other auras"), Some(1));
        assert_eq!(one("can't eat"), None);
        assert!(no_more_than("no more than two creatures can attack you each combat").is_some());
        assert!(no_more_than("no more than one creature can attack ~ each combat").is_some());
        assert!(all_able_to_block("all walls able to block ~ do so").is_some());
    }
}
