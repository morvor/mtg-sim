//! Oracle patterns for the keywords of CR 702.179–702.195 (max speed is in
//! `k702_178_max_speed.rs`):
//!
//! * "your speed increases by N" / "increase your speed by N" (CR 702.179c);
//! * "[card] gains harmonize until end of turn. Its harmonize cost is equal to its mana
//!   cost." (CR 702.180a);
//! * "Tiered" modes (CR 702.183a);
//! * "Each creature you control [...] stations permanents using its toughness rather than
//!   its power", "... crews Vehicles and stations permanents as though its power were N
//!   greater" (CR 702.184c);
//! * warp (CR 702.185): the void condition "a nonland permanent left the battlefield this
//!   turn or a spell was warped this turn", "You may cast ~ from your graveyard using its
//!   warp ability";
//! * "if ~'s mayhem cost was paid" and the like (CR 702.185, 702.187);

use super::{AbilityPattern, ConditionPattern, EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// A value phrase after "where X is": "the number of creature cards in your graveyard",
/// "~'s power", "the number of experience counters you have".
fn x_value(s: &str, ctx: &CompileContext) -> Option<Value> {
    let s = end(s);
    if let Some(kind) = s
        .strip_prefix("the number of ")
        .and_then(|r| r.strip_suffix(" counters you have"))
    {
        return Some(Value::PlayerCounters(PlayerRef::You, kind.into()));
    }
    let mut b = Builder::new(ctx);
    let (v, tail) = crate::oracle::statics::parse_value_phrase(s, &mut b)?;
    end(&tail).is_empty().then_some(v)
}

/// "Mobilize X, where X is [value]" (CR 702.181a), "Firebending X, where X is [value]"
/// (CR 702.189a): the keyword with its X kept as a value, determined as its ability
/// resolves.
fn keyword_x_where(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let (head, value) = end(&lower).split_once(", where x is ")?;
    let kind = match head {
        "mobilize x" => KeywordKind::Mobilize,
        "firebending x" => KeywordKind::Firebending,
        _ => return None,
    };
    let x = x_value(value, ctx)?;
    let text = block.trim();
    let kw = Keyword {
        x: Some(x),
        ..Keyword::new(kind).text(text)
    };
    Some(crate::oracle::keywords::compile_keyword(kw, text))
}

inventory::submit! { AbilityPattern { name: "k702.181/189 keyword x, where x is", priority: 50, parse: keyword_x_where } }

/// "Tiered" followed by "• [Name] — [cost] — [effect]" modes (CR 702.183a): "Choose one.
/// As an additional cost to cast this spell, pay the cost associated with that mode."
/// Compiles to the tiered keyword and a modal spell ability whose modes carry their
/// additional costs (paid for the chosen mode, CR 601.2b, 601.2f).
fn tiered(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
    let mut lines = block.lines();
    if !lines.next()?.trim().eq_ignore_ascii_case("tiered") {
        return None;
    }
    let mut modes = Vec::new();
    for line in lines {
        let l = line.trim().strip_prefix('•')?.trim();
        // The mode's name has no rules meaning.
        let (_name, rest) = l.split_once(" — ")?;
        let (cost, eff) = rest.split_once(" — ")?;
        let (cost, _) = crate::oracle::costs::parse_cost(cost)?;
        let mut b = Builder::new(ctx);
        let effect = crate::oracle::effects::parse_effect_text(eff, &mut b)?;
        modes.push(Mode {
            text: l.to_string(),
            targets: b.targets,
            effect,
            cost: Some(cost),
        });
    }
    if modes.len() < 2 {
        return None;
    }
    let modal = Modal {
        min: Value::c(1),
        max: Value::c(1),
        allow_repeat: false,
        modes,
        per_mode_cost: true,
        chooser: ModeChooser::Controller,
    };
    let mut out =
        crate::oracle::keywords::compile_keyword(Keyword::new(KeywordKind::Tiered), "Tiered");
    out.push(AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body {
                targets: vec![],
                effect: Effect::Noop,
                modal: Some(modal),
            },
        }),
        block.trim(),
    ));
    Some(out)
}

inventory::submit! { AbilityPattern { name: "k702.183 tiered modes", priority: 50, parse: tiered } }

/// Whether the effect grants the keyword `kind` without a cost of its own ("gains
/// harmonize until end of turn").
fn grants_costless(e: &Effect, kind: KeywordKind) -> bool {
    match e {
        Effect::Modify { mods, .. } => mods.iter().any(|m| {
            matches!(m, Modification::AddKeyword(k) if k.kind == kind && k.cost.is_none())
        }),
        Effect::Seq(v) => v.iter().any(|e| grants_costless(e, kind)),
        Effect::If { then, .. } => grants_costless(then, kind),
        Effect::May { effect, .. } | Effect::ForEach { effect, .. } => {
            grants_costless(effect, kind)
        }
        _ => false,
    }
}

/// "Its harmonize cost is equal to its mana cost.": a granted harmonize ability without
/// a cost of its own is cast for the card's mana cost (see `kw/harmonize.rs`).
fn harmonize_cost_is_mana_cost(s: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    matches!(
        end(s),
        "its harmonize cost is equal to its mana cost"
            | "the harmonize cost is equal to its mana cost"
            | "the harmonize cost is equal to that card's mana cost"
    ) && grants_costless(prev, KeywordKind::Harmonize)
}

inventory::submit! {
    FollowupPattern { name: "k702.180: harmonize cost equal to mana cost", priority: 50, apply: harmonize_cost_is_mana_cost }
}

/// "your speed increases by N", "increase your speed by N" (CR 702.179c).
fn speed_increases(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l
        .strip_prefix("your speed increases by ")
        .or_else(|| l.strip_prefix("increase your speed by "))?;
    let (n, tail) = parse_number(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    let n = n.as_const().filter(|n| *n > 0)?;
    Some(crate::kw::start_your_engines::increase(n as u32))
}

inventory::submit! { EffectPattern { name: "k702.179 speed increases", priority: 60, parse: speed_increases } }

/// "[subject] stations permanents using its toughness rather than its power", "[subject]
/// crews Vehicles and stations permanents as though its power were N greater"
/// (CR 702.184c): the subject gets the station (and crew) statics of `kw/crew.rs`.
/// "Each creature you control [...]" gives them to each such creature (see
/// `kw/station.rs`).
fn stations_permanents(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    use crate::kw::crew::{power_bonus, uses_toughness};
    let l = end(l);
    let (subject, kinds, pred): (&str, &[KeywordKind], &str) =
        if let Some((s, p)) = l.split_once(" crews vehicles and stations permanents ") {
            (s, &[KeywordKind::Crew, KeywordKind::Station], p)
        } else if let Some((s, p)) = l.split_once(" stations permanents ") {
            (s, &[KeywordKind::Station], p)
        } else {
            return None;
        };
    let names: Vec<smol_str::SmolStr> = if pred == "using its toughness rather than its power" {
        kinds.iter().map(|k| uses_toughness(*k)).collect()
    } else {
        let n = pred
            .strip_prefix("as though its power were ")?
            .strip_suffix(" greater")?;
        let (n, tail) = parse_number(n)?;
        let n = n.as_const()?;
        if !end(tail).is_empty() {
            return None;
        }
        kinds.iter().map(|k| power_bonus(*k, n)).collect()
    };
    let statics: Vec<Ability> = names
        .into_iter()
        .map(|n| {
            AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(n))),
                text,
            )
        })
        .collect();
    if subject == "~" {
        return Some(statics);
    }
    let (affected, _) =
        crate::oracle::patterns::statics::whole_object_phrase(subject.strip_prefix("each ")?)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected,
            mods: statics.into_iter().map(Modification::AddAbility).collect(),
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.184c stations permanents using", priority: 100, parse: stations_permanents } }

/// Void's condition, "a nonland permanent left the battlefield this turn or a spell was
/// warped this turn", and its parts (CR 702.185c).
fn warp_conditions(c: &str) -> Option<Condition> {
    use crate::kw::warp::{NONLAND_LEFT_THIS_TURN, WARPED_THIS_TURN};
    let custom = |n: &str| Condition::Custom(n.into());
    Some(match end(c) {
        "a nonland permanent left the battlefield this turn or a spell was warped this turn" => {
            Condition::Or(vec![custom(NONLAND_LEFT_THIS_TURN), custom(WARPED_THIS_TURN)])
        }
        "a spell was warped this turn" => custom(WARPED_THIS_TURN),
        "a nonland permanent left the battlefield this turn" => custom(NONLAND_LEFT_THIS_TURN),
        _ => return None,
    })
}

inventory::submit! { ConditionPattern { name: "k702.185c a spell was warped this turn", priority: 50, parse: warp_conditions } }

/// "You may cast ~ from your graveyard using its warp ability." (see `kw/warp.rs`).
fn warp_from_graveyard(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "you may cast ~ from your graveyard using its warp ability" {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::Custom(crate::kw::warp::FROM_GRAVEYARD.into()));
    s.zone = FunctionZone::Graveyard;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "k702.185 cast from graveyard using warp", priority: 50, parse: warp_from_graveyard } }

/// "if ~'s mayhem cost was paid", "if its warp cost was paid", "if this spell's ... cost
/// was paid", "if his/her ... cost was paid" (CR 702.185a, 702.187b).
fn keyword_cost_paid(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = ["its ", "~'s ", "this spell's ", "his ", "her ", "their "]
        .iter()
        .find_map(|p| c.strip_prefix(p))?;
    let name = match r {
        "mayhem cost was paid" => crate::kw::mayhem::MAYHEM,
        "warp cost was paid" => crate::kw::warp::WARP,
        _ => return None,
    };
    Some(Condition::CostPaid(name.into()))
}

inventory::submit! { ConditionPattern { name: "k702.185-190 keyword cost was paid", priority: 50, parse: keyword_cost_paid } }
