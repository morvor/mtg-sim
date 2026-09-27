//! Oracle text of the keywords of CR 702.168–702.177 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Exhaust — [cost]: [effect]" (CR 702.177a), "Whenever you activate an exhaust
//!   ability", "During your turn, as long as you haven't activated an exhaust ability this
//!   turn, you may activate exhaust abilities as though they haven't been activated"
//!   (CR 702.177b); "[Vehicle] becomes an artifact creature" (what many exhaust abilities
//!   of Vehicles do);
//! * saddle (CR 702.171): "~ is saddled", "whenever ~ becomes saddled [for the first time
//!   each turn]", "whenever ~ saddles a Mount [or crews a Vehicle] [during your main
//!   phase]", "[Mount] becomes saddled until end of turn" (and "creature that saddled it
//!   this turn" in `oracle/phrases.rs`);
//! * "if this spell's freerunning cost was paid" (CR 702.173a);
//! * plot (CR 702.170): "it becomes plotted", "when ~ becomes plotted", plotting from the
//!   top of the library and cheaper plotting from hand;
//! * "Gift a [something]" (CR 702.174a–b, the keyword and its second ability), "if the
//!   gift was promised", "if the gift wasn't promised" (CR 702.174k), "whenever you give a
//!   gift" (CR 702.174c).

use super::{
    AbilityPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern, TriggerPattern,
};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use crate::types::CardType;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Exhaust (CR 702.177)
// ---------------------------------------------------------------------------

/// "Exhaust — [cost]: [effect]" (CR 702.177a): the activated ability, which can be
/// activated only once (see `kw/exhaust.rs`). Its text keeps the "Exhaust" label, which
/// makes it an exhaust ability.
fn exhaust(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let rest = t.strip_prefix("Exhaust — ")?;
    let abilities = crate::oracle::parse_ability(rest, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Activated(act.clone()),
        t,
    )])
}

inventory::submit! { AbilityPattern { name: "k702.177 exhaust", priority: 100, parse: exhaust } }

/// "Whenever you activate an exhaust ability" (Rangers' Refueler); "... that isn't a mana
/// ability" (Sala, Deck Boss), whose "it" is the ability on the stack ("copy it").
fn exhaust_activated(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    use crate::kw::exhaust::{EXHAUST_ACTIVATED, EXHAUST_ACTIVATED_NONMANA};
    match end(r) {
        "you activate an exhaust ability" => Some((
            TriggerCond::Custom(SmolStr::new(EXHAUST_ACTIVATED)),
            Sel::TriggerObject,
            PlayerRef::You,
        )),
        "you activate an exhaust ability that isn't a mana ability" => Some((
            TriggerCond::Custom(SmolStr::new(EXHAUST_ACTIVATED_NONMANA)),
            Sel::TriggerSpell,
            PlayerRef::You,
        )),
        _ => None,
    }
}

/// "When you next activate an exhaust ability that isn't a mana ability this turn, copy
/// it." (Pit Automaton): a delayed triggered ability that triggers only once, this turn
/// (CR 603.7b–c).
fn when_you_next_activate_exhaust(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("when you next ")?;
    let (event, eff) = r.split_once(" this turn, ")?;
    let (trigger, it, it_player) = exhaust_activated(&format!("you {event}"))?;
    let body = crate::oracle::effects::parse_trigger_body(eff, b.ctx, it, it_player)?;
    if body.modal.is_some() || !body.targets.is_empty() {
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger: TriggerCond::ThisTurn(Box::new(trigger)),
        body: Box::new(body),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "k702.177 when you next activate an exhaust ability this turn", priority: 100, parse: when_you_next_activate_exhaust } }

inventory::submit! { TriggerPattern { name: "k702.177 you activate an exhaust ability", priority: 100, parse: exhaust_activated } }

/// "During your turn, as long as you haven't activated an exhaust ability this turn, you
/// may activate exhaust abilities as though they haven't been activated." (Elvish
/// Refueler; CR 702.177b).
fn exhaust_again(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    (end(l)
        == "during your turn, as long as you haven't activated an exhaust ability this turn, \
            you may activate exhaust abilities as though they haven't been activated")
        .then(|| {
            vec![AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
                    crate::kw::exhaust::AS_THOUGH_NOT_ACTIVATED.into(),
                ))),
                text,
            )]
        })
}

inventory::submit! { StaticPattern { name: "k702.177b activate exhaust abilities as though they haven't been activated", priority: 100, parse: exhaust_again } }

/// "~ becomes an artifact creature" (an exhaust ability of a Vehicle: "Exhaust — {3}:
/// This Vehicle becomes an artifact creature. Put a +1/+1 counter on it."): for as long as
/// it remains on the battlefield.
fn becomes_artifact_creature(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes an artifact creature")?;
    let saved = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subj, b)?;
    if !end(&rest).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::Modify {
        what,
        mods: vec![Modification::AddTypes(vec![
            CardType::Artifact,
            CardType::Creature,
        ])],
        duration: Duration::Permanent,
    })
}

inventory::submit! { EffectPattern { name: "k702.177 ~ becomes an artifact creature", priority: 100, parse: becomes_artifact_creature } }

/// "Exhaust abilities of other permanents you control cost {2} less to activate." (Boom
/// Scholar): a cost modifier for the exhaust abilities of those sources (CR 601.2f, 602.2b).
fn exhaust_cost_modifier(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("exhaust abilities of ")?;
    let (sources, rest) = r.split_once(" cost {")?;
    let (n, rest) = rest.split_once('}')?;
    let n: i32 = n.parse().ok()?;
    let change = match rest {
        " less to activate" => CostChange::ReduceGeneric(Value::c(n)),
        " more to activate" => CostChange::IncreaseGeneric(Value::c(n)),
        _ => return None,
    };
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(sources)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::CostModifier(
            CostModifier {
                applies_to: CostTarget::KeywordAbilitiesOf(
                    crate::keywords::KeywordKind::Exhaust,
                    f,
                ),
                who: PlayerRel::Any,
                change,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.177 exhaust abilities of [permanents] cost less", priority: 100, parse: exhaust_cost_modifier } }

// ---------------------------------------------------------------------------
// Saddle (CR 702.171)
// ---------------------------------------------------------------------------

/// The saddled designation of the permanent itself (CR 702.171b).
fn this_saddled() -> Condition {
    Condition::SelMatches(
        Sel::This,
        Filter::Custom(SmolStr::new(crate::kw::saddle::SADDLED)),
    )
}

/// "~ is saddled", "~ isn't saddled" (Caustic Bronco, Archmage's Newt).
fn saddled_condition(c: &str) -> Option<Condition> {
    match end(c) {
        "~ is saddled" | "it's saddled" | "it is saddled" => Some(this_saddled()),
        "~ isn't saddled" | "~ is not saddled" | "it isn't saddled" | "it's not saddled" => {
            Some(Condition::Not(Box::new(this_saddled())))
        }
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "k702.171b ~ is saddled", priority: 100, parse: saddled_condition } }

/// "Whenever ~ becomes saddled [for the first time each turn]" (Stubborn Burrowfiend):
/// the permanent became saddled (CR 702.171a–b). "Whenever ~ saddles a Mount [or crews a
/// Vehicle] [during your main phase]" (Canyon Vaulter): it was tapped to pay for a saddle
/// ability (CR 702.171c); "that Mount" is the trigger object.
fn saddle_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    use crate::kw::crew::CREWS_A_VEHICLE;
    use crate::kw::saddle::{BECAME_SADDLED, SADDLES_A_MOUNT};
    let r = end(r);
    if let Some(rest) = r.strip_prefix("~ saddles a mount") {
        let (crews, rest) = match rest.strip_prefix(" or crews a vehicle") {
            Some(x) => (true, x),
            None => (false, rest),
        };
        let main = match rest {
            "" => false,
            " during your main phase" => true,
            _ => return None,
        };
        let saddles = TriggerCond::Custom(SmolStr::new(SADDLES_A_MOUNT));
        let mut trigger = if crews {
            TriggerCond::AnyOf(vec![
                saddles,
                TriggerCond::Custom(SmolStr::new(CREWS_A_VEHICLE)),
            ])
        } else {
            saddles
        };
        if main {
            trigger = TriggerCond::Where {
                trigger: Box::new(trigger),
                cond: Condition::And(vec![
                    Condition::YourTurn,
                    Condition::Phase(PhaseCond::MainPhase),
                ]),
            };
        }
        return Some((trigger, Sel::TriggerObject, PlayerRef::TriggerPlayer));
    }
    let (first_time, subj) = match r.strip_suffix(" becomes saddled for the first time each turn") {
        Some(s) => (true, s),
        None => (false, r.strip_suffix(" becomes saddled")?),
    };
    if subj != "~" {
        return None;
    }
    let cond = TriggerCond::Where {
        trigger: Box::new(TriggerCond::PlayerAction {
            name: SmolStr::new(BECAME_SADDLED),
            who: PlayerRel::Any,
        }),
        cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
    };
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

inventory::submit! { TriggerPattern { name: "k702.171 becomes saddled, saddles a mount", priority: 100, parse: saddle_triggers } }

/// "[Mount] becomes saddled until end of turn" (Guidelight Matrix, Kolodin, Triumph
/// Caster): the saddled designation from an effect (CR 702.171b).
fn becomes_saddled(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes saddled until end of turn")?;
    let saved = b.targets.len();
    let (what, rest) = crate::oracle::effects::object_ref(subj, b)?;
    if !end(&rest).is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(crate::kw::saddle::becomes_saddled(what))
}

inventory::submit! { EffectPattern { name: "k702.171 becomes saddled until end of turn", priority: 100, parse: becomes_saddled } }

/// Whether a filter is about the creatures that saddled the source this turn.
fn about_saddlers(f: &Filter) -> bool {
    match f {
        Filter::Custom(n) => n.as_str() == crate::kw::saddle::SADDLED_IT_THIS_TURN,
        Filter::And(v) => v.iter().any(about_saddlers),
        _ => false,
    }
}

/// "any number of creatures that saddled it this turn", "up to one creature that saddled
/// it this turn", "a creature that saddled it this turn": chosen as the effect happens
/// (CR 702.171c).
fn saddlers(s: &str) -> Option<(Sel, String)> {
    let (count, up_to, r) = if let Some(r) = s.strip_prefix("any number of ") {
        (Value::c(99), true, r)
    } else if let Some(r) = s.strip_prefix("up to one ") {
        (Value::c(1), true, r)
    } else if let Some(r) = s.strip_prefix("a ") {
        (Value::c(1), false, r)
    } else {
        return None;
    };
    let (f, _, rest) = crate::oracle::phrases::parse_object_phrase(r)?;
    if !about_saddlers(&f) {
        return None;
    }
    Some((
        Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count,
            up_to,
            store: None,
        },
        rest.to_string(),
    ))
}

/// "return any number of creatures that saddled it this turn to their owner's hand"
/// (Rambling Possum).
fn return_saddlers(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (what, rest) = saddlers(end(l).strip_prefix("return ")?)?;
    if !matches!(
        rest.trim(),
        "to their owner's hand" | "to their owners' hands" | "to its owner's hand"
    ) {
        return None;
    }
    Some(Effect::Move {
        what,
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "k702.171c return creatures that saddled it", priority: 100, parse: return_saddlers } }

/// "at end of combat, exile it and up to one creature that saddled it this turn, then
/// return those cards to the battlefield under their owner's control" (Fortune, Loyal
/// Steed): a delayed triggered ability (CR 603.7) that exiles the Mount (as the ability
/// that created it knew it) and a creature that saddled it chosen then, and returns them
/// as new objects (CR 400.7).
fn flicker_with_saddler_at_end_of_combat(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("at end of combat, exile ")?;
    let (it, r) = crate::oracle::effects::object_ref(r, b)?;
    if !matches!(it, Sel::This) {
        return None;
    }
    let r = r.trim().strip_prefix("and ")?;
    let (who, rest) = r.split_once(", then return those cards to the battlefield")?;
    if !matches!(
        rest,
        " under their owner's control" | " under their owners' control"
    ) {
        return None;
    }
    let (chosen, tail) = saddlers(who)?;
    if !tail.trim().is_empty() {
        return None;
    }
    const MOUNT: Var = vars::USER + 1718;
    let mut to = Destination::battlefield();
    to.controller = Some(PlayerRef::OwnerOf(Box::new(Sel::Var(vars::IT))));
    Some(Effect::seq(vec![
        Effect::Store {
            var: MOUNT,
            sel: Sel::This,
        },
        Effect::AtNext {
            step: TriggerStep::EndOfCombat,
            effect: Box::new(Effect::seq(vec![
                Effect::Exile {
                    what: Sel::Union(vec![Sel::Var(MOUNT), chosen]),
                    face_down: false,
                    link: false,
                },
                Effect::Move {
                    what: Sel::Var(vars::IT),
                    to,
                },
            ])),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "k702.171c at end of combat, exile it and a creature that saddled it", priority: 100, parse: flicker_with_saddler_at_end_of_combat } }

/// "choose a nonlegendary creature that saddled it this turn and create a tapped and
/// attacking token that's a copy of it" (Calamity, Galloping Inferno): the chosen creature
/// is what "it" refers to afterward.
fn choose_saddler_and(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let (who, rest) = r.split_once(" and ")?;
    let (chosen, tail) = saddlers(who)?;
    if !tail.trim().is_empty() {
        return None;
    }
    const CHOSEN: Var = vars::USER + 1719;
    let saved = b.it.clone();
    b.it = Sel::Var(CHOSEN);
    let Some(e) = crate::oracle::effects::parse_clause(rest, b) else {
        b.it = saved;
        return None;
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN,
            sel: chosen,
        },
        e,
    ]))
}

inventory::submit! { EffectPattern { name: "k702.171c choose a creature that saddled it and ...", priority: 100, parse: choose_saddler_and } }

// ---------------------------------------------------------------------------
// Freerunning (CR 702.173)
// ---------------------------------------------------------------------------

/// "this spell's freerunning cost was paid" (Monastery Raid).
fn freerunning_paid(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "this spell's freerunning cost was paid"
            | "~'s freerunning cost was paid"
            | "its freerunning cost was paid"
    )
    .then(|| Condition::CostPaid(crate::kw::freerunning::FREERUNNING.into()))
}

inventory::submit! { ConditionPattern { name: "k702.173 freerunning cost was paid", priority: 100, parse: freerunning_paid } }

/// "Assassin spells you cast have freerunning {B}{B}." (Ezio Auditore da Firenze),
/// "Creature spells you cast gain offspring {2} as you cast them." (Zinnia, Valley's
/// Voice): the spells gain the keyword as they're cast (CR 610.5; see `next_spell.rs`),
/// so its cost can be paid as they're cast, and a permanent spell's offspring ability
/// keeps working on the battlefield (CR 400.7b).
fn spells_you_cast_gain_cost_keyword(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    use crate::keywords::KeywordKind;
    let l = end(l);
    let (subject, rest) = l.split_once(" spells you cast ")?;
    let kw_text = rest
        .strip_prefix("have ")
        .or_else(|| rest.strip_prefix("gain ")?.strip_suffix(" as you cast them"))?;
    let kws = crate::oracle::keywords::parse_keyword_line(kw_text, ctx)?;
    let [kw] = kws.as_slice() else {
        return None;
    };
    let AbilityKind::Keyword(k) = &kw.kind else {
        return None;
    };
    if !matches!(k.kind, KeywordKind::Freerunning | KeywordKind::Offspring) || k.cost.is_none() {
        return None;
    }
    let quality = match CardType::from_word(subject) {
        Some(t) => Filter::Type(t),
        None => {
            let mut c = subject.chars();
            let name: String = c
                .next()?
                .to_uppercase()
                .chain(c)
                .collect();
            crate::types::subtype_kind(&name)?;
            Filter::Subtype(SmolStr::new(name))
        }
    };
    let s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::and(vec![
            quality,
            Filter::Spell,
            Filter::ControlledBy(PlayerRel::You),
        ]),
        mods: vec![Modification::AddKeyword(k.clone())],
    });
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "k702.173/175 [quality] spells you cast have freerunning / gain offspring", priority: 100, parse: spells_you_cast_gain_cost_keyword } }

// ---------------------------------------------------------------------------
// Gift (CR 702.174)
// ---------------------------------------------------------------------------

/// "Gift a card", "Gift a tapped Fish", ...: the gift keyword (its [something] kept in its
/// text) and its second ability, where the keyword is printed (CR 702.174a–b, 702.174j;
/// see `kw/gift.rs`).
fn gift(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::keywords::{Keyword, KeywordKind};
    use crate::kw::gift::{gift_ability, GiftKind};
    let t = block.trim().trim_end_matches('.');
    let words = t.strip_prefix("Gift ")?;
    let kind = GiftKind::parse(words)?;
    let kw = Keyword {
        text: Some(SmolStr::new(t)),
        ..Keyword::new(KeywordKind::Gift)
    };
    Some(vec![
        AbilityDef::new(AbilityKind::Keyword(kw), t),
        gift_ability(kind, ctx.is_spell(), t),
    ])
}

inventory::submit! { AbilityPattern { name: "k702.174 gift a [something]", priority: 100, parse: gift } }

/// "the gift was promised", "the gift wasn't promised" (CR 702.174k).
fn gift_promised(c: &str) -> Option<Condition> {
    use crate::kw::gift::promised;
    match end(c) {
        "the gift was promised" | "its gift was promised" | "this spell's gift was promised" => {
            Some(promised())
        }
        "the gift wasn't promised" | "the gift was not promised" => {
            Some(Condition::Not(Box::new(promised())))
        }
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "k702.174k the gift was promised", priority: 100, parse: gift_promised } }

/// "Whenever you give a gift" (Jolly Gerbils; CR 702.174c).
fn give_a_gift(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "you give a gift").then(|| {
        (
            TriggerCond::PlayerAction {
                name: SmolStr::new(crate::kw::gift::GAVE_GIFT),
                who: PlayerRel::You,
            },
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

inventory::submit! { TriggerPattern { name: "k702.174c you give a gift", priority: 100, parse: give_a_gift } }

/// "If the gift was promised, instead [effect with targets of its own]." after a sentence
/// ("Counter target creature spell. If the gift was promised, instead counter target
/// spell."): the targets are alternatives chosen as the spell is cast, depending on
/// whether the optional cost was paid (CR 702.174m, 601.2c; also "if this spell was
/// kicked"): the new effect's targets are chosen only if it was, the previous sentence's
/// only if it wasn't.
fn if_paid_instead_with_targets(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", instead ") else {
        return false;
    };
    if matches!(prev, Effect::Noop | Effect::Seq(_)) {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let cast_time = match &cond {
        Condition::CostPaid(_) => true,
        Condition::Not(inner) => matches!(**inner, Condition::CostPaid(_)),
        _ => false,
    };
    if !cast_time {
        return false;
    }
    let first_new = b.targets.len();
    let Some(e) = crate::oracle::effects::parse_effect_text(x, b) else {
        b.targets.truncate(first_new);
        return false;
    };
    if b.targets.len() == first_new {
        // No targets of its own: the general "If [condition], instead [effect]" handles it.
        b.targets.truncate(first_new);
        return false;
    }
    // The previous sentence's targets: those it refers to, chosen only if its effect can
    // happen.
    let old = std::mem::replace(prev, Effect::Noop);
    let not = match &cond {
        Condition::Not(inner) => (**inner).clone(),
        c => Condition::Not(Box::new(c.clone())),
    };
    let old_json = serde_json::to_string(&old).unwrap_or_default();
    for (i, spec) in b.targets[..first_new].iter_mut().enumerate() {
        if spec.condition.is_none() && old_json.contains(&format!("{{\"Target\":{i}}}")) {
            spec.condition = Some(not.clone());
        }
    }
    for spec in &mut b.targets[first_new..] {
        spec.condition = Some(cond.clone());
    }
    *prev = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { FollowupPattern { name: "k702.174m if the gift was promised, instead [effect with targets]", priority: 90, apply: if_paid_instead_with_targets } }

/// "[subject] also [does something]" ("If the gift was promised, that creature also gains
/// indestructible until end of turn."): "also" only stresses that the earlier instructions
/// still happen.
fn also(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = l.split_once(" also ")?;
    if subject.is_empty() || rest.contains(" also ") || subject.contains(',') {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{subject} {rest}"), b)
}

inventory::submit! { EffectPattern { name: "k702.174 [subject] also [effect]", priority: 150, parse: also } }

/// "up to two target creature cards each with mana value 2 or less" (Dewdrop Cure): "each"
/// restates that the quality applies to every one of the targets.
fn targets_each_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" target ") || !l.contains(" each with ") {
        return None;
    }
    crate::oracle::effects::parse_clause(&l.replacen(" each with ", " with ", 1), b)
}

inventory::submit! { EffectPattern { name: "k702.174 target [objects] each with [quality]", priority: 150, parse: targets_each_with } }

/// "You and permanents you control gain hexproof until end of turn." (Dawn's Truce): the
/// same instruction for the player and for the group.
fn you_and_group_gain(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let rest = l.strip_prefix("you and ")?;
    let (group, what) = rest.split_once(" gain ")?;
    if group.contains(" and ") || group.contains(',') {
        return None;
    }
    let saved = b.targets.len();
    let you = crate::oracle::effects::parse_clause(&format!("you gain {what}"), b);
    let them = crate::oracle::effects::parse_clause(&format!("{group} gain {what}"), b);
    match (you, them) {
        (Some(a), Some(c)) => Some(Effect::Seq(vec![a, c])),
        _ => {
            b.targets.truncate(saved);
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "k702.174 you and [group] gain [ability]", priority: 150, parse: you_and_group_gain } }

// ---------------------------------------------------------------------------
// Disguise (CR 702.168)
// ---------------------------------------------------------------------------

/// "exile up to X other target creatures from the battlefield and/or creature cards from
/// graveyards" (Aurelia's Vindicator, whose X is its disguise cost's, CR 702.168e; Angel
/// of Serenity): each target is either a creature or a creature card in a graveyard, and
/// the cards are exiled with the source (CR 607.2a, "the exiled cards").
fn exile_creatures_and_or_graveyard_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("exile up to ")?;
    let (n, rest) = crate::oracle::phrases::parse_number(r)?;
    let text = "other target creatures from the battlefield and/or creature cards from graveyards";
    if end(rest.trim()) != text {
        return None;
    }
    let filter = Filter::Or(vec![
        Filter::and(vec![
            Filter::creature(),
            Filter::InZone(ZoneKind::Battlefield),
            Filter::Other,
        ]),
        Filter::and(vec![
            Filter::Card,
            Filter::Type(CardType::Creature),
            Filter::InZone(ZoneKind::Graveyard),
        ]),
    ]);
    let spec = TargetSpec {
        min: 0,
        max: n,
        ..TargetSpec::object(filter, text)
    };
    let slot = b.add_target(spec, &format!("up to {text}"));
    Some(Effect::Exile {
        what: Sel::Target(slot),
        face_down: false,
        link: true,
    })
}

inventory::submit! { EffectPattern { name: "k702.168e exile up to N other target creatures and/or creature cards from graveyards", priority: 100, parse: exile_creatures_and_or_graveyard_cards } }

/// "Disguise {5}{R}. This cost is reduced by {1} for each instant and sorcery card in your
/// graveyard." (Fugitive Codebreaker): the disguise cost paid to turn it face up
/// (CR 702.168d) is reduced (see `kw/morph_face_up.rs`).
fn disguise_cost_reduced(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim().trim_end_matches('.');
    let (kw, rest) = t.split_once(". ")?;
    let cost = kw.strip_prefix("Disguise ")?;
    let cost = crate::oracle::keywords::parse_keyword_cost(cost)?;
    let each = rest
        .to_lowercase()
        .strip_prefix("this cost is reduced by {1} for each ")?
        .to_string();
    let filter = match crate::oracle::patterns::statics::parse_for_each(&each, None)? {
        Value::CardsInGraveyard(PlayerRef::You, f) => Filter::and(vec![
            f,
            Filter::Card,
            Filter::InZone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        Value::Count(f) => f,
        _ => return None,
    };
    let kw = crate::kw::morph_face_up::disguise_reduced_for_each(cost, filter, kw);
    Some(vec![AbilityDef::new(AbilityKind::Keyword(kw), t)])
}

inventory::submit! { AbilityPattern { name: "k702.168 disguise cost reduced for each", priority: 100, parse: disguise_cost_reduced } }

// ---------------------------------------------------------------------------
// Solved (CR 702.169): what Cases are solved by
// ---------------------------------------------------------------------------

/// "N or more" / "no": a count condition on `v`.
fn at_least(r: &str, v: Value) -> Option<(Condition, &str)> {
    if let Some(rest) = r.strip_prefix("no ") {
        return Some((Condition::Compare(v, Cmp::Eq, Value::c(0)), rest));
    }
    let (n, rest) = crate::oracle::phrases::parse_number(r)?;
    let rest = rest.trim_start().strip_prefix("or more ")?;
    Some((Condition::Compare(v, Cmp::Ge, n), rest))
}

/// Conditions about what happened this turn, as Cases are solved by (CR 719.3a; see
/// `kw/solved.rs`): "three or more creatures attacked this turn" (Case of the Gateway
/// Express), "no creatures attacked this turn"; "three or more creature cards were put
/// into graveyards from anywhere this turn" (Case of the Gorgon's Kiss); "three or more
/// sources you controlled dealt damage this turn" (Case of the Burning Masks); "you've
/// cast four or more instant and sorcery spells this turn" (Case of the Ransacked Lab,
/// Arclight Phoenix).
fn this_turn_counts(c: &str) -> Option<Condition> {
    use crate::kw::solved::*;
    let c = end(c);
    let custom = |n: &str| Value::Custom(SmolStr::new(n));
    if let Some(r) = c.strip_prefix("you've cast ") {
        let r = r.strip_suffix(" this turn")?;
        let (n, rest) = crate::oracle::phrases::parse_number(r)?;
        let spells = rest.trim_start().strip_prefix("or more ")?;
        let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(spells)?;
        if !plural || !end(tail).is_empty() || !spells.ends_with("spells") {
            return None;
        }
        return Some(Condition::Compare(
            Value::SpellsCastThisTurn(PlayerRef::You, f),
            Cmp::Ge,
            n,
        ));
    }
    for (what, value) in [
        ("creatures attacked this turn", CREATURES_ATTACKED),
        (
            "creature cards were put into graveyards from anywhere this turn",
            CREATURE_CARDS_TO_GRAVEYARDS,
        ),
        (
            "sources you controlled dealt damage this turn",
            SOURCES_YOU_CONTROLLED_DEALT_DAMAGE,
        ),
    ] {
        if let Some(r) = c.strip_suffix(what) {
            let r = format!("{r} ");
            let (cond, rest) = at_least(&r, custom(value))?;
            if !rest.trim().is_empty() {
                return None;
            }
            return Some(cond);
        }
    }
    None
}

inventory::submit! { ConditionPattern { name: "k702.169 counts of what happened this turn", priority: 100, parse: this_turn_counts } }

/// "You may look at the top card of your library any time, and you may play lands [and
/// cast creature and enchantment spells] from the top of your library." (Case of the
/// Locked Hothouse, Radha, Heart of Keld): two static abilities in one sentence.
fn look_at_top_and_play(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let rest = end(l)
        .strip_prefix("you may look at the top card of your library any time, and you may ")?;
    let mut out = crate::oracle::statics::parse_static(
        "You may look at the top card of your library any time.",
        ctx,
    )?;
    out.extend(crate::oracle::statics::parse_static(
        &format!("You may {rest}."),
        ctx,
    )?);
    Some(
        out.into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), text))
            .collect(),
    )
}

inventory::submit! { StaticPattern { name: "k702.169 look at the top card any time, and play from the top", priority: 100, parse: look_at_top_and_play } }

// ---------------------------------------------------------------------------
// Plot (CR 702.170)
// ---------------------------------------------------------------------------

/// "It becomes plotted." after an instruction that exiled a card (Aven Interrupter, Kellan
/// Joins Up; CR 702.170c).
fn becomes_plotted(l: &str, b: &mut Builder) -> Option<Effect> {
    let subj = end(l).strip_suffix(" becomes plotted")?;
    let what = match subj {
        "it" | "that card" => b.it.clone(),
        _ => return None,
    };
    Some(crate::kw::plot::becomes_plotted(what))
}

inventory::submit! { EffectPattern { name: "k702.170c it becomes plotted", priority: 100, parse: becomes_plotted } }

/// "exile a nonland card with mana value 3 or less from your hand" (Kellan Joins Up, Jace
/// Reawakened): the player chooses such a card in their hand; "it" is the exiled card
/// afterward ("If you do, it becomes plotted.").
fn exile_card_from_hand(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("exile a ")?
        .strip_suffix(" from your hand")?;
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::And(vec![
                f,
                Filter::Card,
                Filter::InZone(ZoneKind::Hand),
                Filter::OwnedBy(PlayerRel::You),
            ]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "k702.170c exile a [quality] card from your hand", priority: 100, parse: exile_card_from_hand } }

/// "When ~ becomes plotted" (CR 702.170a, 702.170c): the card in exile became plotted.
fn becomes_plotted_trigger(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "~ becomes plotted").then(|| {
        (
            TriggerCond::Where {
                trigger: Box::new(TriggerCond::PlayerAction {
                    name: SmolStr::new(crate::kw::plot::BECAME_PLOTTED),
                    who: PlayerRel::Any,
                }),
                cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
            },
            Sel::This,
            PlayerRef::TriggerPlayer,
        )
    })
}

inventory::submit! { TriggerPattern { name: "k702.170 ~ becomes plotted", priority: 100, parse: becomes_plotted_trigger } }

/// "When this card becomes plotted, [effect]" (Longhorn Sharpshooter): the ability
/// functions while the card is in exile, where it becomes plotted.
fn when_becomes_plotted(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    if !t.to_lowercase().starts_with("when ~ becomes plotted,") {
        return None;
    }
    let a = crate::oracle::triggers::parse_triggered(t, ctx)?;
    let AbilityKind::Triggered(tr) = &a.kind else {
        return None;
    };
    let mut tr = tr.clone();
    tr.zone = FunctionZone::Exile;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { AbilityPattern { name: "k702.170 when ~ becomes plotted", priority: 100, parse: when_becomes_plotted } }

/// "You may plot nonland cards from the top of your library." (CR 702.170f); "The top card
/// of your library has plot. The plot cost is equal to its mana cost."; "Plotting cards
/// from your hand costs {N} less." (CR 702.170e). All from Fblthp, Lost on the Range and
/// Doc Aurlock, Grizzled Genius.
fn plot_statics(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::plot::{from_hand_costs_less, PLOT_FROM_LIBRARY_TOP, TOP_CARD_HAS_PLOT};
    let name = match end(l) {
        "you may plot nonland cards from the top of your library" => {
            SmolStr::new(PLOT_FROM_LIBRARY_TOP)
        }
        "the top card of your library has plot. the plot cost is equal to its mana cost" => {
            SmolStr::new(TOP_CARD_HAS_PLOT)
        }
        other => {
            let n = other
                .strip_prefix("plotting cards from your hand costs {")?
                .strip_suffix("} less")?
                .parse::<u32>()
                .ok()?;
            from_hand_costs_less(n)
        }
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(name))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.170 plot statics", priority: 100, parse: plot_statics } }
