//! An activated ability's own cost changes (CR 602.2b, 601.2f, 118.7): "This ability costs
//! {1} less to activate for each legendary creature you control." (the Channel lands),
//! "This ability costs {2} less to activate if you control a legendary creature.", "This
//! ability costs {X} less to activate, where X is the power of the creature it targets.",
//! "This ability costs {1} more to activate for each card in your hand."
//!
//! The sentence ends the ability's effect text (before any activation restriction) and
//! becomes an [`OwnCostChange`] of the ability. Values and conditions about "the creature
//! it targets" are judged once the ability's targets are chosen (CR 601.2c, 601.2f); see
//! `activation_costs::add_own_cost_changes`.
//!
//! The same sentence after a keyword that defines an activated ability ("Equip {3}. This
//! ability costs {1} less to activate for each other Equipment you control.") changes the
//! cost of that keyword's ability of this object: a static ability of the object
//! ([`CostTarget::ActivatedAbilities`] with the keyword's class).

use super::costs_casting_self::{cost_change, cost_condition, for_each_value, leading_mana};
use super::AbilityPattern;
use crate::ability::*;
use crate::mana::ManaSymbol;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;
use crate::oracle::CompileContext;

const THE_TARGET: &str = "the creature it targets";

/// `v` with `~` meaning the ability's (single) target instead of its source.
fn self_to_target(v: Value) -> Option<Value> {
    let mut j = serde_json::to_value(&v).ok()?;
    fn walk(j: &mut serde_json::Value) {
        match j {
            serde_json::Value::String(s) if s == "This" => {
                *j = serde_json::json!({ "Target": 0 });
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(walk),
            serde_json::Value::Object(m) => m.values_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut j);
    serde_json::from_value(j).ok()
}

/// A value phrase, where "the creature it targets" is the ability's target: parsed with
/// that phrase as `~`, which must then be the only object the value refers to as `~`.
fn targeted_value(s: &str, parse: impl Fn(&str) -> Option<Value>) -> Option<Value> {
    if !s.contains(THE_TARGET) {
        if s.contains("target") {
            return None;
        }
        return parse(s);
    }
    if s.contains('~') || s.matches(THE_TARGET).count() != 1 {
        return None;
    }
    self_to_target(parse(&s.replace(THE_TARGET, "~"))?)
}

/// "it targets a creature with power 2 or less": the ability's target matches.
fn targets_condition(c: &str) -> Option<Condition> {
    let r = c.strip_prefix("it targets ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::SelMatches(Sel::Target(0), f))
}

/// Parses "this ability costs {N} less/more to activate [for each ... | if ... | during
/// your turn | , where X is ...]" (lowercase, without the final period).
/// `single_target`: the ability has exactly one target, so "it targets" and "the creature
/// it targets" can be judged by it.
pub fn parse_own_cost_change(
    l: &str,
    single_target: bool,
    ctx: &CompileContext,
) -> Option<OwnCostChange> {
    let r = end(l).strip_prefix("this ability costs ")?;
    let (mana, r) = leading_mana(r)?;
    let r = r.trim_start();
    let (more, tail) = if let Some(t) = r.strip_prefix("less to activate") {
        (false, t)
    } else if let Some(t) = r.strip_prefix("more to activate") {
        (true, t)
    } else {
        return None;
    };
    let tail = tail.trim();
    if !single_target && tail.contains("target") {
        return None;
    }
    let is_x = mana.symbols.as_slice() == [ManaSymbol::X];
    if mana.has_x() && !is_x {
        return None;
    }
    let (change, condition) = if is_x {
        let v = tail.strip_prefix(", where x is ")?;
        let v = targeted_value(v, |v| {
            let (v, rest) = parse_value_phrase(v, &mut Builder::new(ctx))?;
            end(&rest).is_empty().then_some(v)
        })?;
        let change = if more {
            CostChange::IncreaseGeneric(v)
        } else {
            CostChange::ReduceGeneric(v)
        };
        (change, None)
    } else if let Some(fe) = tail.strip_prefix("for each ") {
        let fe = end(fe);
        let v = if fe.contains(THE_TARGET) {
            targeted_value(fe, |s| super::statics::parse_for_each(s, Some(&Sel::This)))?
        } else {
            for_each_value(fe)?
        };
        (cost_change(mana, more, Some(v))?, None)
    } else if tail.is_empty() {
        return None;
    } else {
        let c = end(tail);
        let cond = match c.strip_prefix("if ").and_then(targets_condition) {
            Some(t) => t,
            None => {
                let cond = cost_condition(c, ctx)?;
                // The conditions about spells cast before "this spell" (and anything about
                // targets) are a spell's own; not this ability's.
                if matches!(&cond, Condition::Custom(_)) || mentions(&cond, "Targets") {
                    return None;
                }
                cond
            }
        };
        (cost_change(mana, more, None)?, Some(cond))
    };
    Some(OwnCostChange { change, condition })
}

fn mentions<T: serde::Serialize>(x: &T, word: &str) -> bool {
    serde_json::to_string(x).is_ok_and(|s| s.contains(&format!("\"{word}\"")))
}

/// Splits a final "This ability costs ..." sentence off an activated ability's effect
/// text: (the rest, the sentence in lowercase).
pub fn split_own_cost_sentence(eff: &str) -> Option<(&str, String)> {
    let lower = eff.to_lowercase();
    let i = lower.rfind("this ability costs ")?;
    let head = eff[..i].trim_end();
    if !(head.is_empty() || head.ends_with('.')) {
        return None;
    }
    let sentence = end(&lower[i..]);
    // One sentence.
    if sentence.contains(". ") {
        return None;
    }
    Some((head, sentence.to_string()))
}

/// "[Keyword line]. This ability costs ...": the keyword, plus a static ability changing
/// the cost of that keyword's ability of this object.
fn keyword_with_own_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || text.contains(':') {
        return None;
    }
    let (head, sentence) = split_own_cost_sentence(text)?;
    let head = head.strip_suffix('.')?;
    let mut abilities = crate::oracle::keywords::parse_keyword_line(head, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Keyword(kw) = &a.kind else {
        return None;
    };
    let kind = kw.kind;
    // The keyword must define exactly one activated ability with at most one target.
    let derived = crate::keyword_impls::derived_abilities(kw);
    let [d] = derived.as_slice() else {
        return None;
    };
    let AbilityKind::Activated(act) = &d.kind else {
        return None;
    };
    let single = act.body.targets.len() == 1 && act.body.targets[0].max.as_const() == Some(1);
    let oc = parse_own_cost_change(&sentence, single, ctx)?;
    let mut scope = AbilityScope::new(Filter::Source, AbilityClass::Keyword(kind));
    // A requirement on the target becomes the scope's targeting requirement.
    if let Some(Condition::SelMatches(Sel::Target(0), f)) = &oc.condition {
        scope.targeting = Some(f.clone());
    }
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ActivatedAbilities(Box::new(scope)),
        who: PlayerRel::Any,
        change: oc.change,
    }));
    // Other conditions are the static ability's condition.
    if !matches!(&oc.condition, Some(Condition::SelMatches(Sel::Target(0), _))) {
        s.condition = oc.condition;
    }
    abilities.push(AbilityDef::new(AbilityKind::Static(s), &sentence));
    Some(abilities)
}

inventory::submit! { AbilityPattern { name: "keyword line. this ability costs less/more", priority: 80, parse: keyword_with_own_cost } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Layout;
    use crate::types::TypeLine;

    fn parse(s: &str, single: bool) -> Option<OwnCostChange> {
        let tl = TypeLine::parse("Artifact");
        let ctx = CompileContext {
            card_name: "X",
            full_name: "X",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        parse_own_cost_change(&s.to_lowercase(), single, &ctx)
    }

    #[test]
    fn own_cost_sentences() {
        for s in [
            "This ability costs {1} less to activate for each legendary creature you control.",
            "This ability costs {1} less to activate for each basic land type among lands you control.",
            "This ability costs {1} less to activate for each other artifact you control.",
            "This ability costs {1} less to activate for each quest counter on ~.",
            "This ability costs {2} less to activate if you control a legendary creature.",
            "This ability costs {1} less to activate during your turn.",
            "This ability costs {1} more to activate for each card in your hand.",
            "This ability costs {X} less to activate, where X is the greatest power among Wurms you control.",
        ] {
            assert!(parse(s, false).is_some(), "{s}");
        }
        let oc = parse(
            "This ability costs {X} less to activate, where X is the power of the creature it targets.",
            true,
        )
        .unwrap();
        assert!(format!("{oc:?}").contains("Target(0)"), "{oc:?}");
        assert!(parse(
            "This ability costs {X} less to activate, where X is the power of the creature it targets.",
            false,
        )
        .is_none());
        let oc = parse(
            "This ability costs {2} less to activate if it targets a creature with power 2 or less.",
            true,
        )
        .unwrap();
        assert!(matches!(
            oc.condition,
            Some(Condition::SelMatches(Sel::Target(0), _))
        ));
    }
}
#[cfg(test)]
mod probe {
    #[test]
    fn probe_costs() {
        let Ok(f) = std::env::var("COST_PROBE") else {
            return;
        };
        for l in std::fs::read_to_string(f).unwrap().lines() {
            let r = crate::oracle::costs::parse_cost(l);
            eprintln!("{} {l:?} => {r:?}", if r.is_some() { "OK  " } else { "FAIL" });
        }
    }
}
