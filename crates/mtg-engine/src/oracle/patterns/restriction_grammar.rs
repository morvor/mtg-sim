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
//!   able to block ~ do so", "[players] can't block with [creatures]", "Players can't get
//!   counters", "Counters can't be put on artifacts, creatures, enchantments, or lands",
//!   "If a creature you control attacks, ~ also attacks if able".
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
        // CR 708.7 (see `rule_statics::face_up`).
        "can't be turned face up" => return Some(vec![Restriction::CantTurnFaceUp(fc)]),
        "can't transform" => return Some(vec![Restriction::CantTransform(fc)]),
        "must be blocked each combat if able" => return Some(vec![Restriction::MustBeBlocked(fc)]),
        // "Enchanted creature gets +1/+1 and has first strike, and all creatures able to
        // block it do so": each creature able to block the subject blocks it (CR 509.1c).
        "all creatures able to block it do so" | "all creatures able to block them do so" => {
            return Some(vec![Restriction::MustBeBlockedByAll(fc)])
        }
        "must be blocked" | "must be blocked each combat" => {
            return Some(vec![Restriction::MustBeBlocked(fc)])
        }
        // "can't attack its owner (or planeswalkers its owner controls)".
        "can't attack its owner" | "can't attack its owner or planeswalkers its owner controls" => {
            return Some(vec![Restriction::CantAttackPlayer {
                attackers: fc,
                defender: PlayerFilter::Ref(Box::new(PlayerRef::OwnerOf(Box::new(Sel::This)))),
                planeswalkers: p.ends_with("planeswalkers its owner controls"),
                battles: false,
            }])
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
        let (n, noun) = if creatures_word(r) { (1, r) } else { count(r)? };
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
    // "Your opponents can't play land cards from graveyards."
    if let Some(z) = p.strip_prefix("can't play land cards from ") {
        let zone = match z {
            "graveyards" => ZoneKind::Graveyard,
            "libraries" => ZoneKind::Library,
            "exile" => ZoneKind::Exile,
            _ => return None,
        };
        return Some(vec![Restriction::CantPlayLandCards {
            who: who.clone(),
            what: Filter::InZone(zone),
        }]);
    }
    // "Players can't play nonbasic lands with the same name as a nontoken permanent."
    // (Cornered Market): land cards matching the description can't be played.
    if let Some(r) = p.strip_prefix("can't play ") {
        let (f, plural) = whole_object_phrase(r)?;
        if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Type(CardType::Land))) {
            return None;
        }
        return Some(vec![Restriction::CantPlayLandCards {
            who: who.clone(),
            what: f,
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

fn put_counters_prevented(
    on_objects: Option<Filter>,
    on_players: Option<PlayerFilter>,
) -> StaticEffect {
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
    // "As long as ~ is tapped, no more than one creature can attack you each combat."
    if let Some(r) = l.strip_prefix("as long as ") {
        let (c, rest) = r.split_once(", ")?;
        let cond = super::activation_restrictions::condition(c, ctx)?;
        let mut v = restriction_static(rest, text, ctx)?;
        for a in &mut v {
            let AbilityKind::Static(st) = &mut std::sync::Arc::make_mut(a).kind else {
                return None;
            };
            if st.condition.is_some() {
                return None;
            }
            st.condition = Some(cond.clone());
        }
        return Some(v);
    }
    if let Some(rs) = no_more_than(l).or_else(|| all_able_to_block(l)) {
        return Some(static_restrictions(rs, text));
    }
    // "Players can't get counters." (CR 122.1).
    if let Some(who) = l.strip_suffix(" can't get counters").and_then(player_group) {
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
    // "Noncreature spells with mana value 4 or greater can't be cast."
    if let Some(subject) = l.strip_suffix(" can't be cast") {
        let (f, plural) = whole_object_phrase(&union_nouns(subject))?;
        if !plural || !filter_mentions(&f, &|x| matches!(x, Filter::Spell)) {
            return None;
        }
        let what = crate::oracle::patterns::statics::without_spell(f)?;
        return Some(static_restrictions(
            vec![Restriction::CantCast {
                who: PlayerFilter::Any,
                what,
            }],
            text,
        ));
    }
    // "Players can't draw cards or gain life."
    if let Some(who) = l
        .strip_suffix(" can't draw cards or gain life")
        .and_then(player_group)
    {
        return Some(static_restrictions(
            vec![
                Restriction::MaxDrawsPerTurn(who.clone(), 0),
                Restriction::CantGainLife(who),
            ],
            text,
        ));
    }
    // "All creatures attack enchanted creature's controller each combat if able."
    if let Some(d) = l
        .strip_prefix("all creatures attack ")
        .and_then(|r| r.strip_suffix(" each combat if able"))
    {
        let defender = match d {
            "enchanted creature's controller" => {
                PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(Box::new(Sel::AttachedTo))))
            }
            "you" => PlayerFilter::You,
            _ => return None,
        };
        return Some(static_restrictions(
            vec![Restriction::MustAttackPlayer {
                attackers: Filter::creature(),
                defender,
            }],
            text,
        ));
    }
    // "If a creature you control attacks, ~ also attacks if able."
    if let Some(r) = attack_together(l) {
        return Some(static_restrictions(vec![r], text));
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

/// "Spells with flash you cast cost {1} less to cast and can't be countered.": the cost
/// change, and the same spells can't be countered (CR 601.2f, 701.6).
fn cost_and_cant_be_countered(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let head = l.strip_suffix(" and can't be countered")?;
    let mut abilities =
        crate::oracle::statics::parse_static(&format!("{}.", &text[..head.len()]), ctx)?;
    if abilities.len() != 1 {
        return None;
    }
    let AbilityKind::Static(st) = &abilities[0].kind else {
        return None;
    };
    let StaticEffect::CostModifier(cm) = &st.effect else {
        return None;
    };
    let CostTarget::Spells(f) = &cm.applies_to else {
        return None;
    };
    let rel = match cm.who {
        PlayerRel::You => PlayerRel::You,
        PlayerRel::Opponent => PlayerRel::Opponent,
        _ => return None,
    };
    let what = Filter::and(vec![f.clone(), Filter::Spell, Filter::ControlledBy(rel)]);
    let mut s = StaticAbility::new(StaticEffect::Restriction(Restriction::CantBeCountered(
        what,
    )));
    s.condition = st.condition.clone();
    abilities.push(AbilityDef::new(AbilityKind::Static(s), text));
    Some(abilities)
}

inventory::submit! { StaticPattern { name: "restriction grammar: [spells] cost less and can't be countered", priority: 105, parse: cost_and_cant_be_countered } }

/// "Enchanted creature gets +4/+4 and has first strike, and all creatures able to block
/// it do so.": the rest of the line, and a blocking requirement on the same object(s)
/// under the same condition (CR 509.1c).
fn and_all_able_to_block_it(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let (head, lure) = if let Some(h) = l
        .strip_suffix(", and all creatures able to block it do so")
        .or_else(|| l.strip_suffix(" and all creatures able to block it do so"))
    {
        (h, true)
    } else if let Some(h) = l.strip_suffix(" and can't have counters put on it") {
        (h, false)
    } else {
        return None;
    };
    let head_text = &text[..head.len()];
    let mut abilities = crate::oracle::statics::parse_static(&format!("{head_text}."), ctx)?;
    let AbilityKind::Static(first) = &abilities.first()?.kind else {
        return None;
    };
    let affected = match &first.effect {
        StaticEffect::Continuous { affected, .. } => affected,
        StaticEffect::Restriction(Restriction::CantBe { what, .. }) => what,
        _ => return None,
    };
    if !matches!(affected, Filter::AttachedToSource | Filter::Source) {
        return None;
    }
    let mut s = StaticAbility::new(if lure {
        StaticEffect::Restriction(Restriction::MustBeBlockedByAll(affected.clone()))
    } else {
        // "can't have counters put on it" (CR 614.1, see `r122_cant_have_counters_put`).
        put_counters_prevented(Some(affected.clone()), None)
    });
    s.condition = first.condition.clone();
    abilities.push(AbilityDef::new(AbilityKind::Static(s), text));
    Some(abilities)
}

inventory::submit! { StaticPattern { name: "restriction grammar: ..., and all creatures able to block it do so", priority: 105, parse: and_all_able_to_block_it } }

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// A leading duration: "until your next turn, ", "until end of turn, ", "this turn, ".
fn leading_duration(l: &str) -> (Option<Duration>, &str) {
    for (p, d) in [
        ("until your next turn, ", Duration::UntilYourNextTurn),
        ("until end of turn, ", Duration::EndOfTurn),
        (
            "until the end of your next turn, ",
            Duration::UntilEndOfYourNextTurn,
        ),
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
    // "Each opponent can't block with more than one creature this combat."
    if let Some(rs) = players_cant_use(&who, &format!("can't {pred}")) {
        return Some(add(rs, dur));
    }
    // "You can't sacrifice those creatures this turn." (only you control them).
    if let Some(o) = pred.strip_prefix("sacrifice ") {
        if !matches!(who, PlayerFilter::You) {
            return None;
        }
        let (what, rest) = crate::oracle::effects::object_ref(o, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        let f = subject_filter(&what, o)?;
        if !matches!(f, Filter::In(_)) {
            return None;
        }
        // A player sacrifices only permanents they control (CR 701.21a): while you
        // control them.
        let f = Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]);
        return Some(add(vec![Restriction::CantBeSacrificed(f)], dur));
    }
    // "You can't attack that player this turn."
    if let Some(d) = pred.strip_prefix("attack ") {
        let defender = effect_players(d, b)?;
        let attackers = match who {
            PlayerFilter::You => Filter::creature().you_control(),
            _ => return None,
        };
        return Some(add(
            vec![Restriction::CantAttackPlayer {
                attackers,
                defender,
                planeswalkers: false,
                battles: false,
            }],
            dur,
        ));
    }
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
    // "can't be blocked this turn except by creatures with flying": the duration is in
    // the middle.
    if let Some((x, y)) = l.split_once(" this turn except by ") {
        return objects_restriction_effect(&format!("{x} except by {y} this turn"), b);
    }
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
        Some((pronoun, poss, r)) => (
            pronoun.to_string(),
            Some(format!("{poss} activated abilities {r}")),
        ),
        None => (main.to_string(), None),
    };
    // "~ and up to one other target creature can't be blocked this turn": both.
    let (main, first) = match main.strip_prefix("~ and ") {
        Some(r) if possessive.is_none() => (r.to_string(), Some(Filter::Source)),
        _ => (main, None),
    };
    let (what, rest) = crate::oracle::effects::object_ref(&main, b)?;
    let rest = possessive.unwrap_or(rest);
    let subject = main.strip_suffix(rest.as_str()).unwrap_or(&main).trim();
    let mut f = subject_filter(&what, subject)?;
    if let Some(first) = first {
        if !matches!(what, Sel::Target(_)) {
            return None;
        }
        f = Filter::Or(vec![first, f]);
    }
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

/// Subjects naming what an earlier instruction did: "each creature dealt damage this
/// way" (fixed as the effect begins, CR 608.2c).
fn damaged_this_way(subject: &str) -> Option<Filter> {
    matches!(
        subject,
        "each creature dealt damage this way"
            | "creatures dealt damage this way"
            | "a creature dealt damage this way"
    )
    .then(|| {
        Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::DAMAGED))),
            Filter::creature(),
        ])
    })
}

/// The subject at the start of `s` and the rest: [`damaged_this_way`] or an object
/// reference.
fn effect_subject(s: &str, b: &mut Builder) -> Option<(Filter, String)> {
    for p in [
        "each creature dealt damage this way",
        "creatures dealt damage this way",
        "a creature dealt damage this way",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            return Some((damaged_this_way(p)?, rest.to_string()));
        }
    }
    let (what, rest) = crate::oracle::effects::object_ref(s, b)?;
    let subject = s.strip_suffix(rest.as_str()).unwrap_or(s).trim();
    Some((subject_filter(&what, subject)?, rest))
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
    let (f, rest) = effect_subject(body, b)?;
    let rest = rest.trim();
    let rs = if matches!(rest, "attacks" | "attack") {
        vec![Restriction::MustAttack(f)]
    } else if matches!(rest, "blocks" | "block") {
        vec![Restriction::MustBlock(f)]
    } else if matches!(rest, "must be blocked" | "must be blocked each combat") {
        vec![Restriction::MustBeBlocked(f)]
    } else if matches!(rest, "attacks or blocks" | "attack or block") {
        vec![
            Restriction::MustAttack(f.clone()),
            Restriction::MustBlock(f),
        ]
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
                    PlayerRef::Target(_)
                    | PlayerRef::TriggerPlayer
                    | PlayerRef::ControllerOf(_) => PlayerFilter::Ref(Box::new(r)),
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

/// "[objects] don't untap during [whose] next untap step" (CR 502.3): for specific
/// objects, through their controllers' next untap steps; for a group a player controls,
/// through that player's next untap step; "during your next untap step", through its
/// controller's.
fn doesnt_untap_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (subject, whose) = l
        .split_once(" don't untap during ")
        .or_else(|| l.split_once(" doesn't untap during "))?;
    let whose = whose
        .strip_suffix(" next untap step")
        .or_else(|| whose.strip_suffix(" next untap steps"))?;
    // "Creatures don't untap during target player's next untap step": the creatures that
    // player controls.
    if let Some(pf) = whose.strip_suffix("'s").and_then(|w| match w {
        "target player" => Some(PlayerFilter::Any),
        "target opponent" => Some(PlayerFilter::Opponent),
        _ => None,
    }) {
        let (f, plural) = whole_object_phrase(subject)?;
        if !plural || filter_mentions(&f, &|x| matches!(x, Filter::ControlledBy(_))) {
            return None;
        }
        let text = if matches!(pf, PlayerFilter::Any) {
            "target player"
        } else {
            "target opponent"
        };
        let slot = b.add_target(TargetSpec::player(pf, text), text);
        b.it_player = PlayerRef::Target(slot);
        return Some(Effect::AddRestriction {
            restriction: Restriction::DoesntUntap(Filter::and(vec![
                f,
                Filter::ControlledBy(PlayerRel::Target(slot)),
            ])),
            duration: Duration::ThroughNextUntapStep,
        });
    }
    let (what, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let duration = match whose {
        "its controller's" | "their controller's" | "their controllers'" => {
            Duration::ThroughNextUntapStep
        }
        "your" => Duration::ThroughYourNextUntapStep,
        _ => return None,
    };
    // Specific objects only: groups use the patterns that bind their player.
    let f = match &what {
        Sel::Target(_) | Sel::Var(_) | Sel::TriggerObject => Filter::In(Box::new(what)),
        Sel::This if subject.starts_with('~') => Filter::In(Box::new(what)),
        _ => return None,
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(f),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "restriction grammar: [objects] don't untap during a next untap step", priority: 110, parse: doesnt_untap_effect } }

/// "All creatures your opponents control able to block that creature this turn do so."
fn all_able_to_block_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let r = l.strip_prefix("all ")?;
    let (r, dur) = if let Some(r) = r.strip_suffix(" this turn do so") {
        (r, Duration::EndOfTurn)
    } else if let Some(r) = r.strip_suffix(" this combat do so") {
        (r, Duration::EndOfCombat)
    } else {
        return None;
    };
    let (blockers, attacker) = r.split_once(" able to block ")?;
    let (bf, plural) = whole_object_phrase(blockers)?;
    if !plural || !restriction_class(&bf) {
        return None;
    }
    let (what, rest) = crate::oracle::effects::object_ref(attacker, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let a = subject_filter(&what, attacker)?;
    if !matches!(a, Filter::In(_)) {
        return None;
    }
    Some(add(
        vec![Restriction::MustBlockAttacker {
            blocker: bf,
            attacker: a,
        }],
        dur,
    ))
}

inventory::submit! { EffectPattern { name: "restriction grammar: all [creatures] able to block [it] this turn do so", priority: 110, parse: all_able_to_block_effect } }

/// "If [a creature ...] attacks, [creatures] (also) attack(s) if able" (CR 508.1d).
fn attack_together(l: &str) -> Option<Restriction> {
    let r = l.strip_prefix("if ")?;
    let (trigger, rest) = r.split_once(" attacks, ")?;
    let attackers = rest
        .strip_suffix(" also attacks if able")
        .or_else(|| rest.strip_suffix(" attacks if able"))
        .or_else(|| rest.strip_suffix(" attack if able"))?;
    let triggers = match trigger {
        "~" => Filter::Source,
        _ => {
            let t = trigger
                .strip_prefix("a ")
                .or_else(|| trigger.strip_prefix("an "))?;
            let (f, plural) = whole_object_phrase(t)?;
            if plural {
                return None;
            }
            f
        }
    };
    let (attackers, same_controller) = match attackers {
        "~" => (Filter::Source, false),
        "all creatures that opponent controls" | "all creatures that player controls" => {
            // Each opponent's creatures attack if one of that player's creatures does.
            if !filter_mentions(&triggers, &|x| matches!(x, Filter::ControlledBy(_))) {
                return None;
            }
            (Filter::creature(), true)
        }
        _ => {
            let a = attackers.strip_prefix("all ")?;
            let (f, plural) = whole_object_phrase(a)?;
            if !plural {
                return None;
            }
            (f, false)
        }
    };
    Some(Restriction::AttackTogether {
        attackers,
        triggers,
        same_controller,
    })
}

/// "Until your next turn, creatures your opponents control attack each combat if able
/// and attack a player other than you if able."
fn attack_other_than_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (lead, l) = leading_duration(l);
    let dur = lead?;
    let (subject, other) = l
        .strip_suffix(" attack a player other than you if able")
        .or_else(|| l.strip_suffix(" attacks a player other than you if able"))
        .map(|r| (r, PlayerFilter::You))?;
    let subject = subject
        .strip_suffix(" attack each combat if able and")
        .or_else(|| subject.strip_suffix(" attacks each combat if able and"))?;
    let (what, rest) = crate::oracle::effects::object_ref(subject, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    let f = subject_filter(&what, subject)?;
    Some(add(
        vec![
            Restriction::MustAttack(f.clone()),
            Restriction::MustAttackOtherThan {
                attackers: f,
                players: other,
            },
        ],
        dur,
    ))
}

inventory::submit! { EffectPattern { name: "restriction grammar: attack a player other than you if able", priority: 110, parse: attack_other_than_effect } }

/// "Until your next turn, creatures can't attack you (or planeswalkers you control)
/// unless their controller pays {2} / 2 life for each of those creatures" (CR 508.1d,
/// 508.1h).
fn attack_tax_effect(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (lead, l) = leading_duration(l);
    let dur = lead?;
    let (planeswalkers, rest) = if let Some(r) = l.strip_prefix(
        "creatures can't attack you or planeswalkers you control unless their controller pays ",
    ) {
        (true, r)
    } else {
        (
            false,
            l.strip_prefix("creatures can't attack you unless their controller pays ")?,
        )
    };
    let cost_s = rest.strip_suffix(" for each of those creatures")?;
    let cost_s = if cost_s.ends_with(" life") {
        format!("pay {cost_s}")
    } else {
        cost_s.to_string()
    };
    let (cost, _) = crate::oracle::costs::parse_cost(&cost_s)?;
    Some(add(
        vec![Restriction::AttackCost {
            attackers: Filter::creature(),
            defender: PlayerFilter::You,
            planeswalkers,
            cost,
        }],
        dur,
    ))
}

inventory::submit! { EffectPattern { name: "restriction grammar: attack taxes for a duration", priority: 110, parse: attack_tax_effect } }

/// "~ gains shroud until end of turn and doesn't untap during your next untap step."
fn gains_and_doesnt_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let head = l.strip_suffix(" and doesn't untap during your next untap step")?;
    if !head.ends_with(" until end of turn") || !head.starts_with('~') {
        return None;
    }
    let modify = crate::oracle::effects::parse_simple(head, b)?;
    if !matches!(
        &modify,
        Effect::Modify {
            what: Sel::This,
            ..
        }
    ) {
        return None;
    }
    Some(Effect::seq(vec![
        modify,
        Effect::AddRestriction {
            restriction: Restriction::DoesntUntap(Filter::In(Box::new(Sel::This))),
            duration: Duration::ThroughYourNextUntapStep,
        },
    ]))
}

inventory::submit! { EffectPattern { name: "restriction grammar: gains [ability] until end of turn and doesn't untap", priority: 110, parse: gains_and_doesnt_untap } }

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
        (
            "with even mana values",
            crate::kw::combat_limits::EVEN_MANA_VALUE,
        ),
        (
            "with an even mana value",
            crate::kw::combat_limits::EVEN_MANA_VALUE,
        ),
        (
            "with odd mana values",
            crate::kw::combat_limits::ODD_MANA_VALUE,
        ),
        (
            "with an odd mana value",
            crate::kw::combat_limits::ODD_MANA_VALUE,
        ),
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

/// "with {X} in their mana costs", "with {X} in its mana cost" (CR 107.3).
fn with_x_in_mana_cost<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for p in ["with {x} in their mana costs", "with {x} in its mana cost"] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') {
                return Some((Filter::HasX, rest));
            }
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "restriction grammar: with {X} in their mana costs", priority: 100, parse: with_x_in_mana_cost } }

/// "Until your next turn, up to one target creature gets -3/-0 and its activated
/// abilities can't be activated.": the P/T change and the restriction, both for the
/// duration.
fn pump_and_restriction_effect(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (lead, main) = leading_duration(l);
    let dur = lead?;
    let (first, second) = main.split_once(" and ")?;
    let gets = first.find(" gets ")?;
    let dur_text = match dur {
        Duration::UntilYourNextTurn => "until your next turn",
        Duration::EndOfTurn => "until end of turn",
        _ => return None,
    };
    let modify = crate::oracle::effects::parse_simple(&format!("{first} {dur_text}"), b)?;
    let Effect::Modify { what, .. } = &modify else {
        return None;
    };
    let subject = first[..gets].trim();
    let f = subject_filter(what, subject)?;
    let rs = restriction_predicate(second, &f)?;
    if rs.is_empty() {
        return None;
    }
    let mut v = vec![modify];
    v.push(add(rs, dur));
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "restriction grammar: [duration], [object] gets +X/+Y and [restriction]", priority: 110, parse: pump_and_restriction_effect } }

/// "no permanents named ~ are on the battlefield", "there are no [permanents] on the
/// battlefield".
fn none_on_battlefield(c: &str) -> Option<Condition> {
    let c = end(c);
    let noun = c
        .strip_prefix("no ")
        .and_then(|r| r.strip_suffix(" are on the battlefield"))
        .or_else(|| {
            c.strip_prefix("there are no ")
                .and_then(|r| r.strip_suffix(" on the battlefield"))
        })?;
    let (f, plural) = whole_object_phrase(noun)?;
    if !plural {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![f, Filter::Permanent])),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { super::ConditionPattern { name: "restriction grammar: no [permanents] are on the battlefield", priority: 100, parse: none_on_battlefield } }

/// "that doesn't have first strike, double strike, vigilance, or haste", "that don't
/// have flying": none of the listed keyword abilities (CR 702).
fn without_keywords<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let list = ["that doesn't have ", "that don't have "]
        .iter()
        .find_map(|p| r.strip_prefix(p))?;
    // The keywords, separated by ", ", ", or " and " or "; the rest after the last one.
    let mut kws = Vec::new();
    let mut rest = list;
    loop {
        // The longest keyword name at the start.
        let mut best: Option<(crate::keywords::KeywordKind, usize)> = None;
        for (i, _) in rest
            .char_indices()
            .chain(std::iter::once((rest.len(), ' ')))
        {
            if i == 0 || (i < rest.len() && !rest[i..].starts_with([' ', ','])) {
                continue;
            }
            if let Some(k) = crate::keywords::KeywordKind::from_name(&rest[..i]) {
                best = Some((k, i));
            }
        }
        let (k, i) = best?;
        kws.push(Filter::HasKeyword(k));
        rest = &rest[i..];
        if let Some(r) = rest
            .strip_prefix(", or ")
            .or_else(|| rest.strip_prefix(" or "))
            .or_else(|| rest.strip_prefix(", "))
        {
            rest = r;
            continue;
        }
        break;
    }
    let any = if kws.len() == 1 {
        kws.pop()?
    } else {
        Filter::Or(kws)
    };
    Some((Filter::not(any), rest))
}

inventory::submit! { FilterSuffixPattern { name: "restriction grammar: that doesn't have [keywords]", priority: 100, parse: without_keywords } }

/// "with no abilities", "with abilities" (CR 113).
fn with_abilities<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    for (p, has) in [("with no abilities", false), ("with abilities", true)] {
        if let Some(rest) = r.strip_prefix(p) {
            if rest.is_empty() || rest.starts_with(' ') {
                let f = if has {
                    Filter::HasAbilities
                } else {
                    Filter::not(Filter::HasAbilities)
                };
                return Some((f, rest));
            }
        }
    }
    None
}

inventory::submit! { FilterSuffixPattern { name: "restriction grammar: with (no) abilities", priority: 100, parse: with_abilities } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicates() {
        let f = Filter::Source;
        let one = |p: &str| object_predicate(p, &f).map(|v| v.len());
        assert_eq!(one("can only attack alone"), Some(1));
        assert_eq!(
            one("can block an additional seven creatures each combat"),
            Some(1)
        );
        assert_eq!(
            one("can block an additional ninety-nine creatures each combat"),
            Some(1)
        );
        assert_eq!(
            one("can't be blocked except by six or more creatures"),
            Some(1)
        );
        assert_eq!(
            one("must be blocked by two or more creatures if able"),
            Some(1)
        );
        assert_eq!(
            one("must be blocked by exactly one creature if able"),
            Some(1)
        );
        assert_eq!(
            one("can't block or be blocked by non-spirit creatures"),
            Some(2)
        );
        assert_eq!(
            one("can't be the target of abilities your opponents control"),
            Some(1)
        );
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
