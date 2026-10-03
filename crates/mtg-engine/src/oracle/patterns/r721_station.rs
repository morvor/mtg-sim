//! Station cards (CR 721): station symbols "{N+} [abilities] [P/T]" — printed "N+ |
//! [abilities]", with the card's power/toughness box in its highest striation.

use super::{AbilityPattern, BlockGroupPattern};
use crate::ability::*;
use crate::oracle::CompileContext;
use crate::types::*;

/// Marks a station striation during grouping: "{STATION N} " (or "{STATION N PT} " for
/// the striation holding the power/toughness box).
const MARK: &str = "{STATION ";

fn is_station_card(ctx: &CompileContext) -> bool {
    ctx.keywords.iter().any(|k| k.eq_ignore_ascii_case("station"))
}

/// "9+ | Flying, first strike" → (9, "Flying, first strike").
fn symbol_line(line: &str) -> Option<(u32, &str)> {
    let (n, rest) = line.trim().split_once("+ | ")?;
    Some((n.trim().parse().ok()?, rest))
}

/// Groups each station symbol's striation: the symbol's line and the lines after it, up
/// to the next symbol (CR 721.2, 721.3). The highest symbol's striation holds the card's
/// power/toughness box.
fn group_striations(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    if !is_station_card(ctx) {
        return blocks;
    }
    let max = blocks
        .iter()
        .filter_map(|b| symbol_line(b).map(|(n, _)| n))
        .max();
    let has_pt = ctx.power.is_some() && ctx.toughness.is_some();
    let mut out: Vec<String> = Vec::new();
    let mut in_striation = false;
    for b in blocks {
        if let Some((n, rest)) = symbol_line(&b) {
            let pt = if has_pt && Some(n) == max { " PT" } else { "" };
            out.push(format!("{MARK}{n}{pt}}} {rest}"));
            in_striation = true;
        } else if in_striation {
            let last = out.last_mut().unwrap();
            last.push('\n');
            last.push_str(b.trim());
        } else {
            out.push(b);
        }
    }
    out
}

inventory::submit! { BlockGroupPattern { name: "r721 station striations", priority: 70, group: group_striations } }

/// "{N+}[abilities]" / "{N+}[abilities][P/T]": "As long as this permanent has N or more
/// charge counters on it, it has [abilities] [and is a creature with base power and
/// toughness [P/T] in addition to its other types]" (CR 721.2a, 721.2b).
fn station_striation(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.strip_prefix(MARK)?;
    let (head, text) = r.split_once("} ")?;
    let (n, pt) = match head.strip_suffix(" PT") {
        Some(n) => (n, true),
        None => (head, false),
    };
    let n: u32 = n.trim().parse().ok()?;
    let mut mods = Vec::new();
    if pt {
        let p: i32 = ctx.power?.trim().parse().ok()?;
        let t: i32 = ctx.toughness?.trim().parse().ok()?;
        mods.push(Modification::AddTypes(vec![CardType::Creature]));
        mods.push(Modification::SetPT(Some(Value::c(p)), Some(Value::c(t))));
    }
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        for a in crate::oracle::parse_ability(line, ctx)? {
            mods.push(Modification::AddAbility(a));
        }
    }
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    });
    s.condition = Some(Condition::Compare(
        Value::CountersOn(Box::new(Sel::This), Some(counters::CHARGE.into())),
        Cmp::Ge,
        Value::c(n as i32),
    ));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), &block[MARK.len()..])])
}

inventory::submit! { AbilityPattern { name: "r721 station symbol", priority: 70, parse: station_striation } }
