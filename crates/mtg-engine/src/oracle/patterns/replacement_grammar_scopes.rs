//! Scopes and remaining event families of the replacement grammar (CR 614.1, 614.10):
//!
//! - "Until end of turn, [replacement effect]" / "This turn, ..." / "Until your next
//!   turn, ...": any replacement effect the compiler understands as a static ability,
//!   created by a resolving spell or ability for that duration (CR 611.2a). "Until end of
//!   turn, if one or more tokens would be created under your control, twice that many of
//!   those tokens are created instead."
//! - Mana: "If a player taps a nonbasic land for mana, it produces colorless mana instead
//!   of any other type.", "if you tap a land you control for mana, it produces {U}
//!   instead of any other type" (CR 106.12b).
//! - Control on entering: "If a creature would enter the battlefield under an opponent's
//!   control [this turn], it enters under your control instead."
//! - Steps and turns: "Skip your next turn.", "you skip your draw step this turn", "If you
//!   would begin your draw step, you may skip that step instead. If you do, [effect]."
//!   (CR 614.10).

use super::replacement_grammar::duration_prefix;
use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::mana::ManaType;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

fn static_ability(effect: StaticEffect, text: &str) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), text)
}

/// Whether a replacement definition can be created by a one-shot effect as it stands:
/// it doesn't refer to its source's own state ("~"), which a static ability would mean.
fn one_shot_ok(d: &ReplacementDef) -> bool {
    let s = serde_json::to_string(d).unwrap_or_default();
    !s.contains("\"Source\"") && !s.contains("\"This\"") && !s.contains("AttachedTo")
}

/// "[Until end of turn, | This turn, | Until your next turn, ] [replacement effect]".
fn p_scoped_static(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (dur, r) = duration_prefix(l);
    let dur = dur?;
    if !r.starts_with("if ") {
        return None;
    }
    let abilities = crate::oracle::statics::parse_static(r, b.ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let AbilityKind::Static(st) = &a.kind else {
        return None;
    };
    let StaticEffect::Replacement(def) = &st.effect else {
        return None;
    };
    if st.condition.is_some() || !one_shot_ok(def) {
        return None;
    }
    Some(Effect::AddReplacement {
        def: def.clone(),
        duration: dur,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: until end of turn, [static replacement]", priority: 160, parse: p_scoped_static } }

/// "{U}", "colorless mana".
fn mana_type(s: &str) -> Option<ManaType> {
    let s = s.trim();
    if s == "colorless mana" {
        return Some(ManaType::C);
    }
    let inner = s.strip_prefix('{')?.strip_suffix('}')?;
    let mut chars = inner.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    ManaType::from_letter(c)
}

/// "If [a player taps | you tap] [a land] for mana, it produces [type] instead of any
/// other type."
fn mana_instead(l: &str) -> Option<ReplacementDef> {
    let r = end(l.trim()).strip_prefix("if ")?;
    let (cond, action) = r.split_once(", it produces ")?;
    let t = mana_type(action.strip_suffix(" instead of any other type")?)?;
    let (you, subj) = if let Some(x) = cond.strip_prefix("a player taps ") {
        (false, x)
    } else {
        (true, cond.strip_prefix("you tap ")?)
    };
    let subj = subj.strip_suffix(" for mana")?;
    let s = subj
        .strip_prefix("a ")
        .or_else(|| subj.strip_prefix("an "))?;
    let (f, plural, rest) = parse_object_phrase(s)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    // Only a permanent's controller can activate its mana abilities (CR 602.2).
    let f = if you {
        Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])
    } else {
        f
    };
    Some(ReplacementDef {
        event: ReplacementEvent::ProduceMana(f),
        action: ReplacementAction::ManaTypeInstead(t),
        self_replacement: false,
        optional: false,
    })
}

fn s_mana_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let def = mana_instead(l)?;
    Some(vec![static_ability(StaticEffect::Replacement(def), text)])
}

inventory::submit! { StaticPattern { name: "replacement grammar: if a player taps a land for mana, it produces [type] instead", priority: 150, parse: s_mana_instead } }

/// "If a creature would enter the battlefield under an opponent's control [this turn], it
/// enters under your control instead." → (definition, "this turn").
fn enters_under_your_control(l: &str) -> Option<(ReplacementDef, bool)> {
    let r = end(l.trim()).strip_prefix("if ")?;
    let (ev, act) = r.split_once(", ")?;
    if act != "it enters under your control instead" {
        return None;
    }
    let (ev, this_turn) = match ev.strip_suffix(" this turn") {
        Some(x) => (x, true),
        None => (ev, false),
    };
    let subj = ev
        .strip_suffix(" would enter the battlefield under an opponent's control")
        .or_else(|| ev.strip_suffix(" would enter under an opponent's control"))?;
    let s = subj.strip_prefix("a ").or_else(|| subj.strip_prefix("an "))?;
    let (f, plural, rest) = parse_object_phrase(s)?;
    if plural || !rest.trim().is_empty() {
        return None;
    }
    Some((
        ReplacementDef {
            // The controller it would enter under (CR 614.12).
            event: ReplacementEvent::EntersBattlefield(Filter::and(vec![
                f,
                Filter::ControlledBy(PlayerRel::Opponent),
            ])),
            action: ReplacementAction::EnterUnderControl(PlayerRef::You),
            self_replacement: false,
            optional: false,
        },
        this_turn,
    ))
}

fn s_enters_under_your_control(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (def, this_turn) = enters_under_your_control(l)?;
    if this_turn {
        return None;
    }
    Some(vec![static_ability(StaticEffect::Replacement(def), text)])
}

inventory::submit! { StaticPattern { name: "replacement grammar: enters under your control instead", priority: 150, parse: s_enters_under_your_control } }

fn p_enters_under_your_control(l: &str, _b: &mut Builder) -> Option<Effect> {
    let (def, this_turn) = enters_under_your_control(l)?;
    if !this_turn {
        return None;
    }
    Some(Effect::AddReplacement {
        def,
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: this turn, enters under your control instead", priority: 150, parse: p_enters_under_your_control } }

/// "Skip your next turn." (an instruction to you), "you skip your draw step this turn"
/// (the draw step of this turn, which hasn't begun).
fn p_skip(l: &str, _b: &mut Builder) -> Option<Effect> {
    let step = match end(l.trim()) {
        "skip your next turn" => StepKind::Turn,
        "you skip your draw step this turn" | "skip your draw step this turn" => {
            StepKind::Draw
        }
        _ => return None,
    };
    Some(Effect::Skip {
        who: PlayerRef::You,
        step,
    })
}

inventory::submit! { EffectPattern { name: "replacement grammar: skip your next turn", priority: 150, parse: p_skip } }

/// "If you would begin your draw step, you may skip that step instead. If you do, you gain
/// 2 life." (CR 614.10b: the action happens as the next step begins).
fn s_skip_step_instead(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l.trim());
    let r = l.strip_prefix("if you would begin your ")?;
    let (step, r) = r.split_once(", ")?;
    let step = match step {
        "draw step" => StepKind::Draw,
        "upkeep step" | "upkeep" => StepKind::Upkeep,
        _ => return None,
    };
    let (optional, r) = match r.strip_prefix("you may ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let (first, more) = match r.split_once(". ") {
        Some((a, m)) => (a, Some(m)),
        None => (r, None),
    };
    if first != "skip that step instead" {
        return None;
    }
    let action = match more {
        None => ReplacementAction::Prevent,
        Some(m) => {
            let m = m.strip_prefix("if you do, ")?;
            let mut b = Builder::new(ctx);
            let e = crate::oracle::effects::parse_effect_text(&format!("{m}."), &mut b)?;
            if !b.targets.is_empty() {
                return None;
            }
            ReplacementAction::Instead(Box::new(e))
        }
    };
    Some(vec![static_ability(
        StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::SkipStep {
                step,
                whose: PlayerRel::You,
            },
            action,
            self_replacement: false,
            optional,
        }),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacement grammar: if you would begin your draw step, you may skip it", priority: 150, parse: s_skip_step_instead } }

/// The spell's own damage can't be prevented this turn (see
/// `damage_removal_prevention::f_the_damage_cant_be_prevented`).
fn unpreventable() -> Effect {
    Effect::AddRestriction {
        restriction: Restriction::SourceDamageCantBePrevented(Filter::In(Box::new(Sel::This))),
        duration: Duration::EndOfTurn,
    }
}

/// "~ deals 4 damage to any target. If [condition], instead ~ deals 6 damage to that
/// permanent or player and the damage can't be prevented." (Lightning Surge, Urza's Rage):
/// the amount changes and the damage can't be prevented if the condition holds as the
/// spell resolves (CR 608.2c).
fn f_instead_unpreventable(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if !b.ctx.is_spell() {
        return false;
    }
    let Some((c, r)) = l.strip_prefix("if ").and_then(|r| r.split_once(", instead ")) else {
        return false;
    };
    let Some(r) = r
        .strip_prefix("~ deals ")
        .or_else(|| r.strip_prefix("it deals "))
        .and_then(|r| r.strip_suffix(" and the damage can't be prevented"))
    else {
        return false;
    };
    let Some((n, tail)) = parse_number(r) else {
        return false;
    };
    if !matches!(
        tail.trim(),
        "damage to that permanent or player" | "damage to that creature" | "damage to that player"
    ) {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let Effect::DealDamage {
        source: Sel::This,
        to,
        ..
    } = &*prev
    else {
        return false;
    };
    let to = to.clone();
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(Effect::seq(vec![
            unpreventable(),
            Effect::DealDamage {
                source: Sel::This,
                amount: n,
                to,
            },
        ])),
        otherwise: Box::new(old),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "replacement grammar: if [condition], instead ~ deals N damage and the damage can't be prevented", priority: 150, apply: f_instead_unpreventable } }

/// "If [condition], ~ can't be countered and the damage can't be prevented." on an
/// instant or sorcery (Demonfire, Banefire): static abilities that function while the
/// spell is on the stack, as long as the condition holds (CR 113.6).
fn a_cant_be_countered_or_prevented(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let text = block.trim();
    let lower = text.to_lowercase();
    let c = end(&lower)
        .strip_prefix("if ")?
        .strip_suffix(", ~ can't be countered and the damage can't be prevented")?;
    // "If X is 5 or more": the X of the spell on the stack (CR 107.3).
    let cond = match c
        .strip_prefix("x is ")
        .and_then(|r| r.strip_suffix(" or more"))
        .and_then(|n| n.parse::<i32>().ok())
    {
        Some(n) => Condition::Compare(Value::XOf(Box::new(Sel::This)), Cmp::Ge, Value::c(n)),
        None => crate::oracle::statics::parse_condition(c, ctx)?,
    };
    let mut out = Vec::new();
    for r in [
        Restriction::CantBeCountered(Filter::Source),
        Restriction::SourceDamageCantBePrevented(Filter::Source),
    ] {
        let mut s = StaticAbility::new(StaticEffect::Restriction(r));
        s.zone = FunctionZone::Stack;
        s.condition = Some(cond.clone());
        out.push(AbilityDef::new(AbilityKind::Static(s), text));
    }
    Some(out)
}

inventory::submit! { super::AbilityPattern { name: "replacement grammar: if [condition], ~ can't be countered and the damage can't be prevented", priority: 150, parse: a_cant_be_countered_or_prevented } }

/// "If [condition], search your library for two basic land cards instead of one." (Reclaim
/// the Wastes), "... put up to two creature cards from among the revealed cards into your
/// hand instead of one." (Gather the Pack): the previous instruction with a different
/// number, if the condition holds as the spell resolves (CR 608.2c). The number replaced
/// must be the only such count of that instruction.
fn f_instead_of_number(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some((c, r)) = l.strip_prefix("if ").and_then(|r| r.split_once(", ")) else {
        return false;
    };
    let Some((text, old_word)) = r.rsplit_once(" instead of ") else {
        return false;
    };
    let Some((Value::Const(old), "")) = parse_number(old_word).map(|(v, r)| (v, r.trim())) else {
        return false;
    };
    // The new number: after "up to" or "for" in the instruction.
    let new = text.split(' ').collect::<Vec<_>>().windows(2).find_map(|w| {
        if matches!(w[0], "to" | "for" | "cast" | "put") {
            match parse_number(w[1]) {
                Some((Value::Const(n), _)) if w[1] != "a" && w[1] != "an" => Some(n),
                _ => None,
            }
        } else {
            None
        }
    });
    let Some(new) = new else {
        return false;
    };
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    use serde_json::Value as J;
    fn walk(v: &mut J, old: i32, new: i32, hits: &mut u32) {
        match v {
            J::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if matches!(k.as_str(), "count" | "take")
                        && *x == serde_json::json!({ "Const": old })
                    {
                        *hits += 1;
                        *x = serde_json::json!({ "Const": new });
                    } else {
                        walk(x, old, new, hits);
                    }
                }
            }
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, old, new, hits)),
            _ => {}
        }
    }
    let Ok(mut json) = serde_json::to_value(&*prev) else {
        return false;
    };
    let mut hits = 0;
    walk(&mut json, old, new, &mut hits);
    if hits != 1 {
        return false;
    }
    let Ok(modified) = serde_json::from_value::<Effect>(json) else {
        return false;
    };
    let original = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::If {
        cond,
        then: Box::new(modified),
        otherwise: Box::new(original),
    };
    true
}

inventory::submit! { super::FollowupPattern { name: "replacement grammar: if [condition], [instruction] instead of [number]", priority: 150, apply: f_instead_of_number } }

/// On an instant or sorcery, an ability-word paragraph "Spell mastery — If [condition],
/// [instruction] instead of [number]." modifies the instruction of the paragraph before
/// it (CR 207.2c: the ability word has no rules meaning): the two are parsed as one
/// ability.
fn group_instead_of_number(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    if !ctx.is_spell() {
        return blocks;
    }
    let mut out: Vec<String> = Vec::new();
    for b in blocks {
        let stripped = crate::oracle::strip_ability_word(&b);
        let lower = stripped.to_lowercase();
        let joins = stripped.len() < b.len()
            && lower.starts_with("if ")
            && lower.contains(" instead of ")
            && !out.is_empty();
        if joins {
            let prev = out.pop().expect("checked");
            out.push(format!("{} {}", prev.trim_end(), stripped.trim()));
        } else {
            out.push(b);
        }
    }
    out
}

inventory::submit! { super::BlockGroupPattern { name: "replacement grammar: spell mastery instead of [number] joins the previous paragraph", priority: 150, group: group_instead_of_number } }
