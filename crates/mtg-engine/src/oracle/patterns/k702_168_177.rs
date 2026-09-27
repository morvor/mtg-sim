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

/// "Whenever you activate an exhaust ability" (Rangers' Refueler).
fn exhaust_activated(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    (end(r) == "you activate an exhaust ability").then(|| {
        (
            TriggerCond::Custom(SmolStr::new(crate::kw::exhaust::EXHAUST_ACTIVATED)),
            Sel::TriggerObject,
            PlayerRef::You,
        )
    })
}

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
