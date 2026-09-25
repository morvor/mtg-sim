//! Oracle patterns for CR 603 wording: reflexive triggered abilities ("When you do, ...",
//! CR 603.12), abilities that count their own resolutions ("When this ability resolves for
//! the third time this turn, ...", CR 603.7h), "Do this only once each turn" (CR 603.2h),
//! and "sacrifice another [object]".

use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Parses the text of a reflexive (or similar immediately-created) triggered ability into
/// its own body: it has its own targets. "That creature" refers to the objects the
/// preceding instruction acted on.
fn reflexive_body(text: &str, b: &Builder) -> Option<Body> {
    let text = text
        .replace("that creature's power", "its power")
        .replace("that creature's toughness", "its toughness");
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::IT);
    sub.it_player = b.it_player.clone();
    let effect = parse_effect_text(&text, &mut sub)?;
    Some(Body {
        targets: sub.targets,
        effect,
        modal: None,
    })
}

/// "When you do, [effect]" / "When you don't, [effect]" (CR 603.12).
fn when_you_do(l: &str, b: &mut Builder) -> Option<Effect> {
    let (r, did) = if let Some(r) = l.strip_prefix("when you do, ") {
        (r, true)
    } else if let Some(r) = l.strip_prefix("when you don't, ") {
        (r, false)
    } else {
        return None;
    };
    let body = reflexive_body(r, b)?;
    let reflexive = Effect::Reflexive {
        body: Box::new(body),
    };
    let cond = if did {
        Condition::PrevHappened
    } else {
        Condition::Not(Box::new(Condition::PrevHappened))
    };
    Some(Effect::If {
        cond,
        then: Box::new(reflexive),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "When this ability resolves for the Nth time this turn, [effect]" (CR 603.7h): a
/// delayed triggered ability created only during that resolution, triggering as it ends.
fn resolves_for_the_nth_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("when this ability resolves for the ")?;
    let (w, rest) = split_word(r);
    let n = match w {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        _ => return None,
    };
    let rest = rest.strip_prefix("time this turn, ")?;
    let body = reflexive_body(rest, b)?;
    Some(Effect::If {
        cond: Condition::Compare(Value::TimesResolvedThisTurn, Cmp::Eq, Value::c(n)),
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    })
}

/// "sacrifice another creature" — the controller of the effect sacrifices one.
fn sacrifice_another(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("sacrifice ")?;
    if !r.starts_with("another ") {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Sacrifice {
        who: PlayerRef::You,
        filter: f,
        count: Value::c(1),
    })
}

/// A triggered ability ending in "Do this only once each turn." (CR 603.2h).
fn do_this_only_once(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    if !(lower.starts_with("when") || lower.starts_with("at ")) {
        return None;
    }
    let body = t.strip_suffix("Do this only once each turn.")?.trim_end();
    let a = crate::oracle::triggers::parse_triggered(body, ctx)?;
    let AbilityKind::Triggered(mut tr) = a.kind.clone() else {
        return None;
    };
    tr.do_once_per_turn = true;
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

inventory::submit! { EffectPattern { name: "reflexive: when you do", priority: 0, parse: when_you_do } }
inventory::submit! { EffectPattern { name: "resolves for the nth time", priority: 0, parse: resolves_for_the_nth_time } }
inventory::submit! { EffectPattern { name: "sacrifice another", priority: 0, parse: sacrifice_another } }
inventory::submit! { AbilityPattern { name: "do this only once each turn", priority: 0, parse: do_this_only_once } }
