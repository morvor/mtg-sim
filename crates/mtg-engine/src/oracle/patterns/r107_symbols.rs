//! Symbols (CR 107.4–107.18): level symbols of leveler cards (107.8), Saga chapter
//! symbols (107.15), Class level bars (107.16), energy {E} (107.14) and ticket {TK}
//! (107.17) costs and effects, pawprint {P} modes (107.18), and "{H}" in rules text
//! (107.4g).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::patterns::{AbilityPattern, BlockGroupPattern, EffectPattern};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::counters;

/// "LEVEL 2-6" → (2, Some(6)); "LEVEL 7+" → (7, None).
fn level_symbol(line: &str) -> Option<(u32, Option<u32>)> {
    let r = line.trim().strip_prefix("LEVEL ")?;
    if let Some(n) = r.strip_suffix('+') {
        return Some((n.trim().parse().ok()?, None));
    }
    let (a, b) = r.split_once('-')?;
    Some((a.trim().parse().ok()?, Some(b.trim().parse().ok()?)))
}

/// "[Cost]: Level N" → (cost text, N).
fn class_level_bar(line: &str) -> Option<(&str, u32)> {
    let (cost, rest) = crate::oracle::split_cost(line.trim())?;
    let n = rest.trim().strip_prefix("Level ")?.trim().parse().ok()?;
    if !cost.contains('{') {
        return None;
    }
    Some((cost, n))
}

fn is_pawprint_header(line: &str) -> bool {
    let l = line.to_lowercase();
    l.starts_with("choose up to ") && l.contains("{p} worth of modes")
}

/// Groups the lines of a leveler striation, a class level section, or a pawprint modal
/// spell into single blocks.
fn group_striations(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    // Saga chapter symbols look like ability words ("I, II — ..."): mark them so they
    // reach the chapter pattern intact.
    let saga = ctx.type_line.subtypes.iter().any(|s| s.as_str() == "Saga");
    let blocks: Vec<String> = blocks
        .into_iter()
        .map(|b| match (saga, b.split_once(" — ")) {
            (true, Some((head, eff))) if chapter_numbers(head).is_some() => {
                format!("{CHAPTER}{head} — {eff}")
            }
            _ => b,
        })
        .collect();
    let mut out: Vec<String> = Vec::new();
    // What the current group is: a level symbol, a class level bar, or pawprint modes.
    #[derive(PartialEq)]
    enum G {
        None,
        Level,
        Class,
        Pawprint,
    }
    let mut g = G::None;
    for b in blocks {
        let starts = if level_symbol(&b).is_some() {
            Some(G::Level)
        } else if class_level_bar(&b).is_some() {
            Some(G::Class)
        } else if is_pawprint_header(&b) {
            Some(G::Pawprint)
        } else {
            None
        };
        match starts {
            Some(kind) => {
                g = kind;
                out.push(b);
            }
            None if g == G::Pawprint && !b.starts_with("{P}") => {
                g = G::None;
                out.push(b);
            }
            None if g != G::None => {
                let last = out.last_mut().unwrap();
                last.push('\n');
                last.push_str(&b);
            }
            None => out.push(b),
        }
    }
    out
}

inventory::submit! { BlockGroupPattern { name: "r107 striations", priority: 70, group: group_striations } }

/// Parses the abilities printed within a striation.
fn striation_abilities(lines: &[&str], ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut out = Vec::new();
    for l in lines {
        out.extend(crate::oracle::parse_ability(l, ctx)?);
    }
    Some(out)
}

/// "{LEVEL N1-N2} [Abilities] [P/T]" (CR 107.8a) and "{LEVEL N3+} [Abilities] [P/T]"
/// (CR 107.8b): as long as the creature has that many level counters, it has base power
/// and toughness [P/T] and has [abilities].
fn leveler_striation(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut lines = block.lines();
    let (lo, hi) = level_symbol(lines.next()?)?;
    let mut pt: Option<(i32, i32)> = None;
    let mut rest: Vec<&str> = Vec::new();
    for l in lines {
        let t = l.trim();
        if pt.is_none() {
            if let Some((p, tt)) = t.split_once('/') {
                if let (Ok(p), Ok(tt)) = (p.parse::<i32>(), tt.parse::<i32>()) {
                    pt = Some((p, tt));
                    continue;
                }
            }
        }
        rest.push(t);
    }
    let abilities = striation_abilities(&rest, ctx)?;
    let level = || Value::CountersOn(Box::new(Sel::This), Some(counters::LEVEL.into()));
    let mut conds = vec![Condition::Compare(level(), Cmp::Ge, Value::c(lo as i32))];
    if let Some(hi) = hi {
        conds.push(Condition::Compare(level(), Cmp::Le, Value::c(hi as i32)));
    }
    let mut mods = Vec::new();
    if let Some((p, t)) = pt {
        mods.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
    }
    mods.extend(abilities.into_iter().map(Modification::AddAbility));
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    });
    s.condition = Some(Condition::And(conds));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), block)])
}

inventory::submit! { AbilityPattern { name: "r107 leveler striation", priority: 70, parse: leveler_striation } }

/// "[Cost]: Level N — [Abilities]" (CR 107.16a): "[Cost]: This Class's level becomes N.
/// Activate only if this Class is level N-1 and only as a sorcery" and "As long as this
/// Class is level N or greater, it has [abilities]."
fn class_level_section(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut lines = block.lines();
    let (cost_s, n) = class_level_bar(lines.next()?)?;
    let rest: Vec<&str> = lines.map(str::trim).filter(|l| !l.is_empty()).collect();
    let abilities = striation_abilities(&rest, ctx)?;
    let (cost, _) = crate::oracle::costs::parse_cost(cost_s)?;
    let mut act = ActivatedAbility::new(cost, Body::effect(Effect::SetClassLevel { level: n }));
    act.timing = ActivationTiming::Sorcery;
    act.condition = Some(Condition::Compare(
        Value::ClassLevel,
        Cmp::Eq,
        Value::c(n as i32 - 1),
    ));
    let mut out = vec![AbilityDef::new(
        AbilityKind::Activated(act),
        format!("{cost_s}: Level {n}"),
    )];
    if !abilities.is_empty() {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: abilities
                .into_iter()
                .map(Modification::AddAbility)
                .collect(),
        });
        s.condition = Some(Condition::Compare(
            Value::ClassLevel,
            Cmp::Ge,
            Value::c(n as i32),
        ));
        out.push(AbilityDef::new(AbilityKind::Static(s), block));
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "r107 class level bar", priority: 70, parse: class_level_section } }

fn roman(s: &str) -> Option<u32> {
    Some(match s.trim() {
        "I" => 1,
        "II" => 2,
        "III" => 3,
        "IV" => 4,
        "V" => 5,
        "VI" => 6,
        "VII" => 7,
        _ => return None,
    })
}

/// Marks a Saga chapter line during grouping.
const CHAPTER: &str = "{CHAPTER} ";

/// "I, II" → [1, 2].
fn chapter_numbers(head: &str) -> Option<Vec<u32>> {
    let mut ns = Vec::new();
    for part in head.split(',') {
        ns.push(roman(part)?);
    }
    Some(ns)
}

/// "{rN}—[Effect]" / "{rN1}, {rN2}—[Effect]" chapter abilities (CR 107.15a–b): triggered
/// abilities that trigger when lore counters bring the count from below N to N or more.
fn saga_chapter(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (head, eff) = block.strip_prefix(CHAPTER)?.split_once(" — ")?;
    let ns = chapter_numbers(head)?;
    let block = &block[CHAPTER.len()..];
    let body = crate::oracle::effects::parse_body(eff, ctx)?;
    let name = format!(
        "chapter:{}",
        ns.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let tr = TriggeredAbility::new(TriggerCond::Custom(name.into()), body);
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r107 saga chapter", priority: 70, parse: saga_chapter } }

/// "Choose up to five {P} worth of modes. You may choose the same mode more than once."
/// followed by "{P} — ...", "{P}{P} — ..." modes (CR 107.18, 700.2i).
fn pawprint_modes(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let mut lines = block.lines();
    let header = lines.next()?.to_lowercase();
    let r = header.strip_prefix("choose up to ")?;
    let (n, rest) = parse_number(r)?;
    let budget = n.as_const()? as u32;
    let rest = rest.trim();
    let rest = rest.strip_prefix("{p} worth of modes")?;
    let allow_repeat = rest.contains("you may choose the same mode more than once");
    let mut modes = Vec::new();
    for l in lines {
        let l = l.trim();
        let (_, eff) = l.split_once(" — ")?;
        let mut b = Builder::new(ctx);
        let effect = crate::oracle::effects::parse_effect_text(eff, &mut b)?;
        modes.push(Mode {
            text: l.to_string(),
            targets: b.targets,
            effect,
            cost: None,
        });
    }
    if modes.is_empty() {
        return None;
    }
    let modal = Modal {
        min: Value::c(0),
        max: Value::c(budget as i32),
        allow_repeat,
        modes,
        per_mode_cost: false,
        chooser: ModeChooser::Pawprints(budget),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(modal),
            },
        }),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "r107 pawprint modes", priority: 70, parse: pawprint_modes } }

/// Counts leading "{E}" / "{TK}" symbols: "{e}{e}" → 2.
fn count_symbols(s: &str, sym: &str) -> Option<(i32, String)> {
    let mut n = 0;
    let mut rest = s.trim();
    while let Some(r) = rest.strip_prefix(sym) {
        n += 1;
        rest = r;
    }
    (n > 0).then(|| (n, rest.to_string()))
}

/// "you get {E}{E}" (CR 107.14) and "you get {TK}" (CR 107.17): the player gets that many
/// energy / ticket counters.
fn get_player_counters(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you get ")?;
    if let Some(rest) = r.strip_prefix("that many {e}") {
        if rest.trim().is_empty() || rest.trim().starts_with('(') {
            return Some(Effect::AddPlayerCounters {
                who: PlayerRef::You,
                kind: counters::ENERGY.into(),
                n: Value::EventAmount,
            });
        }
    }
    for (sym, kind) in [("{e}", counters::ENERGY), ("{tk}", counters::TICKET)] {
        if let Some((n, rest)) = count_symbols(r, sym) {
            let rest = rest.trim();
            if rest.is_empty() || rest.starts_with('(') {
                return Some(Effect::AddPlayerCounters {
                    who: PlayerRef::You,
                    kind: kind.into(),
                    n: Value::c(n),
                });
            }
        }
    }
    None
}

inventory::submit! { EffectPattern { name: "r107 get energy or tickets", priority: 70, parse: get_player_counters } }

/// Activated abilities whose costs include "Pay {E}{E}" (CR 107.14) or ticket symbols
/// "{TK}{TK}" (CR 107.17a): paying removes that many energy / ticket counters from the
/// player.
fn energy_or_ticket_cost(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (cost_s, eff_s) = crate::oracle::split_cost(block.trim())?;
    let mut extra: Vec<CostPart> = Vec::new();
    let mut kept: Vec<String> = Vec::new();
    for part in cost_s.split(", ") {
        let lower = part.trim().to_lowercase();
        let (sym, kind, text) = if let Some(r) = lower.strip_prefix("pay ") {
            ("{e}", counters::ENERGY, r.to_string())
        } else if lower.starts_with("{tk}") {
            ("{tk}", counters::TICKET, lower.clone())
        } else {
            kept.push(part.trim().to_string());
            continue;
        };
        let (count, rest) = match text.strip_prefix("x ") {
            // "Pay X {E}": X is announced as the ability is activated (CR 107.3a).
            Some(r) if r.trim() == sym => (Value::X, String::new()),
            _ => {
                let (n, rest) = count_symbols(&text, sym)?;
                (Value::c(n), rest)
            }
        };
        if !rest.trim().is_empty() {
            return None;
        }
        extra.push(CostPart::PayPlayerCounters {
            kind: kind.into(),
            count,
        });
    }
    if extra.is_empty() {
        return None;
    }
    let (mut cost, _) = if kept.is_empty() {
        (Cost::free(), false)
    } else {
        crate::oracle::costs::parse_cost(&kept.join(", "))?
    };
    cost.parts.extend(extra);
    let (eff_text, timing, max_per_turn, any_player) =
        crate::oracle::costs::split_activation_restrictions(eff_s);
    let body = crate::oracle::effects::parse_body(eff_text, ctx)?;
    let is_mana = crate::oracle::effects::is_mana_effect(&body.effect) && body.targets.is_empty();
    let mut act = ActivatedAbility::new(cost, body);
    act.timing = timing;
    act.max_per_turn = max_per_turn;
    act.any_player = any_player;
    act.is_mana_ability = is_mana;
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "r107 energy and ticket costs", priority: 70, parse: energy_or_ticket_cost } }

/// "Whenever you cast a spell with {H} in its mana cost, ~ deals damage equal to that
/// spell's mana value to any target." (Rage Extractor; CR 107.4g: {H} means any
/// Phyrexian mana symbol.)
fn phyrexian_spell_trigger(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let eff = lower.strip_prefix("whenever you cast a spell with {h} in its mana cost, ")?;
    let eff = eff.replace("that spell's mana value", "that spell mana value");
    let body = if let Some(r) =
        end(&eff).strip_prefix("~ deals damage equal to that spell mana value to ")
    {
        let (spec, tail) = parse_any_target(r)?;
        if !end(tail).is_empty() {
            return None;
        }
        Body::simple(
            vec![spec],
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::ManaValueOf(Box::new(Sel::TriggerSpell)),
                to: Sel::Target(0),
            },
        )
    } else {
        crate::oracle::effects::parse_trigger_body(&eff, ctx, Sel::TriggerSpell, PlayerRef::You)?
    };
    let tr = TriggeredAbility::new(
        TriggerCond::CastSpell {
            who: PlayerRel::You,
            filter: Filter::and(vec![Filter::Spell, Filter::HasPhyrexianMana]),
        },
        body,
    );
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), block)])
}

inventory::submit! { AbilityPattern { name: "r107 phyrexian spell trigger", priority: 70, parse: phyrexian_spell_trigger } }
