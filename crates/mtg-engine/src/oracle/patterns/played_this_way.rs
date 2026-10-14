//! Spells cast and lands played "this way" (gap R7; CR 601.3, 400.7h, 400.7i, 603.7):
//!
//! * after an effect gives a permission: "You may play that card this turn. When you play
//!   a card this way, ~ deals 2 damage to each player." (Fires of Mount Doom), "When you
//!   cast it this turn, ~ gains all activated abilities of that card until end of turn."
//!   (Havengul Lich), "Until end of turn, whenever you play a land or cast a spell this
//!   way, ..." (Apple of Eden): a delayed triggered ability (CR 603.7) that triggers only
//!   for a card played with that permission (`permissions::PermissionUse`);
//! * after a static permission: "If you cast a creature spell this way, it gains haste
//!   until end of turn." (Thundermane Dragon) is done as the spell is cast
//!   (`PlayTerms::on_cast`); "When you cast an Equipment spell this way, it gains
//!   \"...\"" (Galea, Kindler of Hope) is a triggered ability of the source;
//! * after a spell cast as an effect resolves: "You may cast that card without paying its
//!   mana cost. If you cast a creature spell this way, it gains haste ..." (Strago and
//!   Relm): that spell.

use super::FollowupPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;
use crate::permissions::PLAYED_THIS_WAY;

fn this_way() -> Filter {
    Filter::Custom(PLAYED_THIS_WAY.into())
}

/// The last instruction of an effect (looking into sequences and permission terms).
fn last(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last),
        Effect::WithPlayTerms { effect, .. } => last(effect),
        _ => e,
    }
}

/// "a creature spell", "an Equipment spell": the spell's qualities.
fn spell_kind(s: &str) -> Option<Filter> {
    let s = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    if s == "spell" {
        return Some(Filter::Any);
    }
    let (f, plural, tail) = parse_object_phrase(s)?;
    (!plural && end(tail).trim().is_empty() && s.ends_with(" spell")).then_some(f)
}

/// The body of an ability about the spell or land played ("it", "its owner").
fn body_about(text: &str, b: &Builder, it: Sel) -> Option<Body> {
    let text = text
        .replace("that card's", "its")
        .replace("that card", "it")
        .replace("that spell", "it");
    super::r600_triggers::reflexive_body_about(&text, b, it)
}

/// "when you play a card this way, X", "when you cast a spell this way, X", "when you
/// cast it this turn, X", "until end of turn, whenever you play a land or cast a spell
/// this way, X": the trigger condition, whether it triggers only once, and X.
fn this_way_trigger(l: &str) -> Option<(TriggerCond, bool, &str)> {
    let (once, r) = if let Some(r) = l.strip_prefix("when ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("until end of turn, whenever ") {
        (false, r)
    } else {
        return None;
    };
    let (event, x) = r.split_once(", ")?;
    let spell = |f: Filter| TriggerCond::CastSpell {
        who: PlayerRel::You,
        filter: Filter::and(vec![f, this_way()]),
    };
    let land = || TriggerCond::LandPlayed {
        who: PlayerRel::You,
        filter: this_way(),
    };
    // A stated duration ("this turn", "until end of turn") ends the delayed ability at
    // the end of the turn and lets it trigger more than once (CR 603.7b).
    let (cond, this_turn) = match event {
        "you play a card this way" => (TriggerCond::AnyOf(vec![spell(Filter::Card), land()]), false),
        "you play a land or cast a spell this way" => {
            (TriggerCond::AnyOf(vec![spell(Filter::Card), land()]), !once)
        }
        "you cast it this turn" | "you cast that card this turn" => (spell(Filter::Any), true),
        _ => {
            let r = event
                .strip_prefix("you cast ")?
                .strip_suffix(" this way")?;
            (spell(spell_kind(r)?), !once)
        }
    };
    if this_turn {
        return Some((TriggerCond::ThisTurn(Box::new(cond)), false, x));
    }
    Some((cond, once, x))
}

/// After an effect that gives a permission to play cards.
fn f_when_played_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(last(prev), Effect::GrantPlayPermission { .. }) {
        return false;
    }
    let Some((trigger, once, x)) = this_way_trigger(end(l).trim()) else {
        return false;
    };
    let Some(body) = body_about(x, b, Sel::TriggerObject) else {
        return false;
    };
    let e = Effect::DelayedTrigger {
        trigger,
        body: Box::new(body),
        once,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "played this way: delayed trigger after a permission", priority: 60, apply: f_when_played_this_way } }

/// `f` without the requirement to be a spell.
fn without_spell(f: Filter) -> Filter {
    match f {
        Filter::Spell => Filter::Any,
        Filter::And(v) => Filter::and(v.into_iter().map(without_spell).collect()),
        other => other,
    }
}

/// "if you cast a creature spell this way, X": the spell's qualities and X.
fn if_cast_this_way(l: &str) -> Option<(Filter, &str)> {
    let r = end(l).trim().strip_prefix("if you cast ")?;
    let (kind, x) = r.split_once(" this way, ")?;
    Some((spell_kind(kind)?, x))
}

/// What "it" (the spell, `it`) gets: X parsed with "it" as that spell.
fn spell_effect(x: &str, b: &Builder, it: Sel) -> Option<Effect> {
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = b.in_trigger;
    sub.it = it;
    let e = parse_effect_text(x, &mut sub)?;
    sub.targets.is_empty().then_some(e)
}

/// After a spell cast as the effect resolves ("You may cast that card without paying its
/// mana cost. If you cast a creature spell this way, it gains haste until end of turn."):
/// `Var(IT)` is the spell, if one was cast.
fn f_if_cast_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !matches!(last(prev), Effect::CastCard { .. }) {
        return false;
    }
    let Some((f, x)) = if_cast_this_way(l) else {
        return false;
    };
    let Some(e) = spell_effect(x, b, Sel::Var(vars::IT)) else {
        return false;
    };
    let cond = Condition::SelMatches(
        Sel::Var(vars::IT),
        Filter::and(vec![f, Filter::InZone(ZoneKind::Stack)]),
    );
    let then = Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, then]);
    true
}

inventory::submit! { FollowupPattern { name: "played this way: if you cast a spell this way (cast as it resolves)", priority: 60, apply: f_if_cast_this_way } }

/// A second sentence of a static permission about the spells cast with it: "If you cast a
/// creature spell this way, it gains haste until end of turn." becomes part of the
/// permission's terms; "When you cast an Equipment spell this way, it gains \"...\"" is
/// another (triggered) ability of the source, returned. None if it's neither.
pub(crate) fn static_rider(
    s: &str,
    terms: &mut PlayTerms,
    text: &str,
    ctx: &CompileContext,
) -> Option<Option<Ability>> {
    let l = end(s).trim().to_lowercase();
    let b = Builder::new(ctx);
    if let Some((f, x)) = if_cast_this_way(&l) {
        let e = spell_effect(x, &b, Sel::Var(vars::IT))?;
        // The object is the spell being cast (its qualities as a card are what's left).
        let f = without_spell(f);
        let on_cast = Effect::If {
            cond: Condition::SelMatches(Sel::Var(vars::IT), f),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        };
        terms.on_cast = Some(Box::new(on_cast));
        return Some(None);
    }
    let (trigger, true, x) = this_way_trigger(&l)? else {
        return None;
    };
    let body = body_about(x, &b, Sel::TriggerObject)?;
    let tr = TriggeredAbility::new(trigger, body);
    Some(Some(AbilityDef::new(AbilityKind::Triggered(tr), text)))
}
