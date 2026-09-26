//! Oracle text that goes with the keywords of CR 702.111–702.124:
//!
//! * renown (CR 702.112b): "~ is renowned", "as long as ~ is renowned, it has ...",
//!   "when ~ becomes renowned", "whenever a creature you control becomes renowned";
//! * surge and emerge (CR 702.117a, 702.119a): "if its surge cost was paid", "if ~'s
//!   emerge cost was paid";
//! * "Emerge from [quality] [cost]" (CR 702.119b);
//! * crew (CR 702.122): "Crew N. Activate only once each turn.", "whenever ~ becomes
//!   crewed", "whenever ~ crews a Vehicle", "if it was crewed by exactly two creatures",
//!   "~ crews Vehicles as though its power were N greater", "enchanted creature can't
//!   attack, block, or crew Vehicles".

use super::{
    AbilityPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern, TriggerPattern,
};
use crate::oracle::effects::Builder;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::kw::renown::RENOWNED;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Renown (CR 702.112)
// ---------------------------------------------------------------------------

fn renowned() -> Filter {
    Filter::Custom(SmolStr::new(RENOWNED))
}

/// "~ is renowned", "it's renowned", "~ isn't renowned": the renowned designation of the
/// permanent itself (CR 702.112b).
fn renowned_condition(c: &str) -> Option<Condition> {
    let yes = match end(c) {
        "~ is renowned" | "it's renowned" | "it is renowned" => true,
        "~ isn't renowned" | "~ is not renowned" | "it isn't renowned" | "it's not renowned" => {
            false
        }
        _ => return None,
    };
    let f = if yes {
        renowned()
    } else {
        Filter::not(renowned())
    };
    Some(Condition::SelMatches(Sel::This, f))
}

inventory::submit! { ConditionPattern { name: "k702.112 ~ is renowned", priority: 100, parse: renowned_condition } }

/// "As long as ~ is renowned, it has [abilities]": "it" is the permanent itself.
fn renowned_static(l: &str, _text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("as long as ~ is renowned, ")?;
    let rest = r.strip_prefix("it ")?;
    crate::oracle::statics::parse_static(&format!("as long as ~ is renowned, ~ {rest}"), ctx)
}

inventory::submit! { StaticPattern { name: "k702.112 as long as ~ is renowned", priority: 100, parse: renowned_static } }

/// "If it's renowned, [effect]" after an instruction about another object ("Target
/// creature gets +1/+1 until end of turn. ... If it's renowned, untap it."): "it" is that
/// object, not the source.
fn if_its_renowned_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(rest) = end(l).strip_prefix("if it's renowned, ") else {
        return false;
    };
    if matches!(b.it, Sel::This) {
        return false;
    }
    let subject = b.it.clone();
    let Some(then) = crate::oracle::effects::parse_clause(rest, b) else {
        return false;
    };
    *prev = Effect::seq(vec![
        prev.clone(),
        Effect::If {
            cond: Condition::SelMatches(subject, renowned()),
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "k702.112 if it's renowned", priority: 100, apply: if_its_renowned_followup } }

/// "~ becomes renowned", "a creature you control becomes renowned": a permanent became
/// renowned (the [`RENOWNED`] event).
fn becomes_renowned(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let subj = end(r).strip_suffix(" becomes renowned")?;
    // "It" is the permanent itself, or the one that became renowned.
    let it = if subj == "~" {
        Sel::This
    } else {
        Sel::TriggerObject
    };
    let f = if subj == "~" {
        Filter::Source
    } else {
        let s = subj
            .strip_prefix("a ")
            .or_else(|| subj.strip_prefix("an "))
            .or_else(|| subj.strip_prefix("another "))?;
        let (f, plural, tail) = parse_object_phrase(s)?;
        if plural || !end(tail).is_empty() {
            return None;
        }
        if subj.starts_with("another ") {
            Filter::and(vec![f, Filter::Other])
        } else {
            f
        }
    };
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::PlayerAction {
                name: SmolStr::new(RENOWNED),
                who: PlayerRel::Any,
            }),
            cond: Condition::SelMatches(Sel::TriggerObject, f),
        },
        it,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.112 becomes renowned", priority: 100, parse: becomes_renowned } }

// ---------------------------------------------------------------------------
// Surge, emerge (CR 702.117a, 702.119a)
// ---------------------------------------------------------------------------

/// "Emerge from [quality] [cost]" (CR 702.119b): emerge whose sacrificed permanent is a
/// [quality] permanent (the keyword's filter) rather than a creature.
fn emerge_from(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    let rest = lower.strip_prefix("emerge from ")?;
    let at = rest.find('{')?;
    let quality = crate::oracle::keywords::quality_phrase(rest[..at].trim())?;
    let cost_at = "emerge from ".len() + at;
    let cost = crate::oracle::keywords::parse_keyword_cost(&t[cost_at..])?;
    let kw = Keyword {
        cost: Some(cost),
        filter: Some(quality),
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Emerge)
    };
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.119b emerge from [quality]", priority: 100, parse: emerge_from } }

/// "you sacrifice ~ while casting a spell with emerge" (Foul Emissary): sacrificed for any
/// reason while its controller casts a spell with emerge.
fn sacrificed_while_casting_emerge(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if end(r) != "you sacrifice ~ while casting a spell with emerge" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::YouSacrifice(Filter::Source)),
            cond: Condition::Custom(SmolStr::new(
                crate::kw::emerge::CASTING_A_SPELL_WITH_EMERGE,
            )),
        },
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.119 you sacrifice ~ while casting a spell with emerge", priority: 100, parse: sacrificed_while_casting_emerge } }

/// "its surge cost was paid", "~'s surge cost was paid", "this spell's emerge cost was
/// paid", "you cast it for its surge cost".
fn alt_cost_paid(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c
        .strip_prefix("its ")
        .or_else(|| c.strip_prefix("~'s "))
        .or_else(|| c.strip_prefix("this spell's "));
    let name = match r {
        Some("surge cost was paid") => crate::kw::surge::SURGE,
        Some("emerge cost was paid") => crate::kw::emerge::EMERGE,
        _ => match c {
            "you cast it for its surge cost" | "you cast ~ for its surge cost" => {
                crate::kw::surge::SURGE
            }
            _ => return None,
        },
    };
    Some(Condition::CostPaid(SmolStr::new(name)))
}

inventory::submit! { ConditionPattern { name: "k702.117/119 surge or emerge cost was paid", priority: 100, parse: alt_cost_paid } }

// ---------------------------------------------------------------------------
// Crew (CR 702.122)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Partner (CR 702.124)
// ---------------------------------------------------------------------------

/// The partner abilities written on their own line (CR 702.124a): "Partner with [name]"
/// (the name may contain commas), "Partner—[text]", "Choose a Background", "Doctor's
/// companion". The last two aren't the partner keyword (CR 702.124n): they're static
/// abilities that function before the game (see `kw/partner.rs`).
fn partner_line(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::partner::{CHOOSE_A_BACKGROUND, DOCTORS_COMPANION};
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    let custom = |name: &str| {
        let mut s = StaticAbility::new(StaticEffect::Custom(SmolStr::new(name)));
        s.zone = FunctionZone::Anywhere;
        Some(vec![AbilityDef::new(AbilityKind::Static(s), t)])
    };
    match lower.as_str() {
        "choose a background" => return custom(CHOOSE_A_BACKGROUND),
        "doctor's companion" => return custom(DOCTORS_COMPANION),
        _ => {}
    }
    let named = lower
        .strip_prefix("partner with ")
        .is_some_and(|n| !n.trim().is_empty());
    let text = lower
        .strip_prefix("partner—")
        .is_some_and(|x| !x.trim().is_empty() && !x.contains('{'));
    if !named && !text {
        return None;
    }
    let kw = Keyword::new(KeywordKind::Partner).text(t);
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.124 partner abilities", priority: 100, parse: partner_line } }

/// "Put your commander into your hand from the command zone" (Command Beacon): with two
/// commanders there, its owner chooses one (CR 702.124e).
fn put_your_commander_into_your_hand(l: &str, _b: &mut Builder) -> Option<Effect> {
    if end(l) != "put your commander into your hand from the command zone" {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![
                Filter::Commander,
                Filter::OwnedBy(PlayerRel::You),
                Filter::InZone(ZoneKind::Command),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "k702.124e put your commander into your hand", priority: 100, parse: put_your_commander_into_your_hand } }

/// "Crew N. Activate only once each turn.": the crew keyword with that restriction (kept
/// in its text, see `kw/crew.rs`).
fn crew_once_each_turn(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let lower = t.to_lowercase();
    let n = lower
        .strip_prefix("crew ")?
        .strip_suffix(". activate only once each turn")?;
    let n: i32 = n.trim().parse().ok()?;
    let kw = Keyword::with_n(KeywordKind::Crew, n).text(t);
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "k702.122 crew N, activate only once each turn", priority: 100, parse: crew_once_each_turn } }

/// "~ becomes crewed [for the first time each turn]" (CR 702.122e): a crew ability of the
/// Vehicle resolved. "~ crews a Vehicle" (CR 702.122b): it was tapped to pay for a
/// Vehicle's crew ability; "that Vehicle" is the trigger object.
fn crew_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    use crate::kw::crew::{CREWED, CREWS_A_VEHICLE};
    let r = end(r);
    if r == "~ crews a vehicle" {
        return Some((
            TriggerCond::Custom(SmolStr::new(CREWS_A_VEHICLE)),
            Sel::TriggerObject,
            PlayerRef::TriggerPlayer,
        ));
    }
    let (first_time, subj) = match r.strip_suffix(" becomes crewed for the first time each turn")
    {
        Some(s) => (true, s),
        None => (false, r.strip_suffix(" becomes crewed")?),
    };
    if subj != "~" {
        return None;
    }
    let cond = TriggerCond::Where {
        trigger: Box::new(TriggerCond::PlayerAction {
            name: SmolStr::new(CREWED),
            who: PlayerRel::Any,
        }),
        cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
    };
    // "It" is the Vehicle itself.
    Some((
        if first_time {
            TriggerCond::FirstTimeEachTurn(Box::new(cond))
        } else {
            cond
        },
        Sel::This,
        PlayerRef::TriggerPlayer,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.122 becomes crewed, crews a vehicle", priority: 100, parse: crew_triggers } }

/// "[Vehicle] becomes an artifact creature until end of turn": what a crew ability does
/// (CR 702.122a), as an instruction of its own ("Whenever ~ becomes crewed, up to one other
/// target Vehicle you control becomes an artifact creature until end of turn.").
fn becomes_artifact_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes an artifact creature until end of turn")?;
    let saved = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subj, b)?;
    if !end(&rest).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddTypes(vec![
            crate::types::CardType::Artifact,
            crate::types::CardType::Creature,
        ])],
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "k702.122 becomes an artifact creature until end of turn", priority: 100, parse: becomes_artifact_creature } }

/// "it was crewed by exactly two creatures", "it was crewed by two or more creatures": the
/// creatures tapped to pay for the crew ability whose resolution triggered the ability
/// (CR 702.122e), the event's amount.
fn crewed_by_n(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("it was crewed by ")
        .or_else(|| end(c).strip_prefix("~ was crewed by "))?;
    let (exactly, r) = match r.strip_prefix("exactly ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let (n, rest) = crate::oracle::phrases::parse_number(r)?;
    let cmp = match (exactly, end(rest)) {
        (true, "creatures" | "creature") => Cmp::Eq,
        (false, "or more creatures") => Cmp::Ge,
        _ => return None,
    };
    Some(Condition::Compare(Value::EventAmount, cmp, n))
}

inventory::submit! { ConditionPattern { name: "k702.122e it was crewed by N creatures", priority: 100, parse: crewed_by_n } }

/// "an Assassin crewed it this turn": a creature with that quality crewed the source this
/// turn (CR 702.122c).
fn quality_crewed_it(c: &str) -> Option<Condition> {
    let subj = end(c).strip_suffix(" crewed it this turn")?;
    let s = subj
        .strip_prefix("a ")
        .or_else(|| subj.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(s)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Exists(Filter::and(vec![
        f,
        Filter::Custom(SmolStr::new(crate::kw::crew::CREWED_IT_THIS_TURN)),
    ])))
}

inventory::submit! { ConditionPattern { name: "k702.122c a [quality] crewed it this turn", priority: 100, parse: quality_crewed_it } }

/// "~ crews Vehicles as though its power were N greater", "~ saddles Mounts and crews
/// Vehicles as though its power were N greater", "~ crews Vehicles using its toughness
/// rather than its power" (see `kw/crew.rs`).
fn crews_vehicles_as_though(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::crew::{power_bonus, uses_toughness};
    let r = end(l).strip_prefix("~ ")?;
    let (kinds, rest): (&[KeywordKind], &str) =
        if let Some(x) = r.strip_prefix("saddles mounts and crews vehicles ") {
            (&[KeywordKind::Saddle, KeywordKind::Crew], x)
        } else if let Some(x) = r.strip_prefix("crews vehicles ") {
            (&[KeywordKind::Crew], x)
        } else {
            return None;
        };
    let names: Vec<SmolStr> = if rest == "using its toughness rather than its power" {
        kinds.iter().map(|k| uses_toughness(*k)).collect()
    } else {
        let n = rest
            .strip_prefix("as though its power were ")?
            .strip_suffix(" greater")?;
        let (n, tail) = crate::oracle::phrases::parse_number(n)?;
        let n = n.as_const()?;
        if !end(tail).is_empty() {
            return None;
        }
        kinds.iter().map(|k| power_bonus(*k, n)).collect()
    };
    Some(
        names
            .into_iter()
            .map(|n| {
                AbilityDef::new(
                    AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(n))),
                    text,
                )
            })
            .collect(),
    )
}

inventory::submit! { StaticPattern { name: "k702.122 crews vehicles as though", priority: 100, parse: crews_vehicles_as_though } }

/// "Enchanted creature can't attack, block, or crew Vehicles[, and its activated abilities
/// ...]" (CR 702.122d): the rest is an ordinary static ability.
fn cant_crew_vehicles(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let (subject, rest) = l.split_once(" can't attack, block, or crew vehicles")?;
    if !subject.starts_with("enchanted ") {
        return None;
    }
    // "... or crew Vehicles. Its activated abilities can't be activated ...": a second
    // static ability in the same paragraph.
    let mut out = match rest.strip_prefix(". ") {
        Some(next) => {
            let mut v = crate::oracle::statics::parse_static(
                &format!("{subject} can't attack or block"),
                ctx,
            )?;
            // "Its" is the enchanted permanent's.
            let next = match next.strip_prefix("its ") {
                Some(r) => format!("{subject}'s {r}"),
                None => next.to_string(),
            };
            v.extend(crate::oracle::statics::parse_static(&next, ctx)?);
            v
        }
        None => crate::oracle::statics::parse_static(
            &format!("{subject} can't attack or block{rest}"),
            ctx,
        )?,
    };
    out.push(AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            crate::kw::crew::cant_tap_for(KeywordKind::Crew),
        ))),
        text,
    ));
    Some(out)
}

inventory::submit! { StaticPattern { name: "k702.122d can't crew vehicles", priority: 100, parse: cant_crew_vehicles } }
