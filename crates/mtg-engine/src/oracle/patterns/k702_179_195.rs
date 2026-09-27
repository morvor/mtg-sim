//! Oracle patterns for the keywords of CR 702.179–702.195 (max speed is in
//! `k702_178_max_speed.rs`):
//!
//! * "your speed increases by N" / "increase your speed by N" (CR 702.179c);
//! * "[card] gains harmonize until end of turn. Its harmonize cost is equal to its mana
//!   cost." (CR 702.180a);

use super::{AbilityPattern, EffectPattern, FollowupPattern};
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
