//! Modal spells and abilities beyond the plain "Choose one —" header (CR 700.2):
//!
//! * "Choose three. You may choose the same mode more than once." (CR 700.2d);
//! * "An opponent chooses one —": another player chooses the mode when the controller
//!   normally would (CR 700.2e); "that player" in a mode is the player who chose;
//! * spree: "+ {cost} — [effect]" modes, each with an additional cost paid if that mode is
//!   chosen (CR 700.2h, 702.172).

use super::{AbilityPattern, BlockGroupPattern};
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::effects::{parse_effect_text, strip_flavor_word, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// Parses bullet lines ("• [effect]") into modes. `it_player` is what "that player" means.
fn bullet_modes(lines: &[&str], ctx: &CompileContext, it_player: &PlayerRef) -> Option<Vec<Mode>> {
    let mut modes = Vec::new();
    for line in lines {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let l = l.strip_prefix('•')?.trim();
        let mut b = Builder::new(ctx);
        b.it_player = it_player.clone();
        let effect = parse_effect_text(strip_flavor_word(l), &mut b)?;
        modes.push(Mode {
            text: l.to_string(),
            targets: b.targets,
            effect,
            cost: None,
        });
    }
    (!modes.is_empty()).then_some(modes)
}

fn modal_ability(modal: Modal, block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() {
        return None;
    }
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

/// "Choose three. You may choose the same mode more than once." followed by bullets: the
/// same mode may be chosen several times, and is then performed that many times in
/// sequence (CR 700.2d).
fn repeatable_modes(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut lines = block.lines();
    let header = lines.next()?.to_lowercase();
    let r = header.strip_prefix("choose ")?;
    let (count, rest) = r.split_once('.')?;
    if end(rest) != "you may choose the same mode more than once" {
        return None;
    }
    let (n, tail) = parse_number(count)?;
    let n = n.as_const()?;
    if !tail.trim().is_empty() {
        return None;
    }
    let rest: Vec<&str> = lines.collect();
    let modes = bullet_modes(&rest, ctx, &PlayerRef::You)?;
    let modal = Modal {
        min: Value::c(n),
        max: Value::c(n),
        allow_repeat: true,
        modes,
        per_mode_cost: false,
        chooser: ModeChooser::Controller,
    };
    modal_ability(modal, block, ctx)
}

inventory::submit! { AbilityPattern { name: "r700 repeatable modes", priority: 70, parse: repeatable_modes } }

/// "An opponent chooses one —" followed by bullets (CR 700.2e): an opponent chooses the
/// mode as the spell is cast; "that player" in a mode is that opponent.
fn opponent_chooses_mode(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut lines = block.lines();
    let header = lines.next()?.to_lowercase();
    let header = header.trim().trim_end_matches(['—', ':', ' ']);
    let (min, max) = match header {
        "an opponent chooses one" => (1, 1),
        _ => return None,
    };
    let rest: Vec<&str> = lines.collect();
    let modes = bullet_modes(&rest, ctx, &PlayerRef::ChosenOpponent)?;
    let modal = Modal {
        min: Value::c(min),
        max: Value::c(max),
        allow_repeat: false,
        modes,
        per_mode_cost: false,
        chooser: ModeChooser::Opponent,
    };
    modal_ability(modal, block, ctx)
}

inventory::submit! { AbilityPattern { name: "r700 opponent chooses mode", priority: 70, parse: opponent_chooses_mode } }

fn is_spree_mode(line: &str) -> bool {
    line.starts_with("+ {") && line.contains(" — ")
}

/// Spree modes are printed one per line: "+ {2}{B} — Destroy target creature.". Groups
/// them into one block.
fn group_spree_modes(blocks: Vec<String>, _ctx: &CompileContext) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for b in blocks {
        if is_spree_mode(&b) {
            if let Some(last) = out.last_mut() {
                if last.lines().all(is_spree_mode) {
                    last.push('\n');
                    last.push_str(&b);
                    continue;
                }
            }
        }
        out.push(b);
    }
    out
}

inventory::submit! { BlockGroupPattern { name: "r700 spree modes", priority: 70, group: group_spree_modes } }

/// "+ {cost} — [effect]" lines: modes with an additional cost, of which one or more are
/// chosen; each chosen mode's cost is added to the total cost (CR 700.2h, 702.172a).
fn spree_modes(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut modes = Vec::new();
    for line in block.lines() {
        if !is_spree_mode(line) {
            return None;
        }
        let (cost, eff) = line.strip_prefix("+ ")?.split_once(" — ")?;
        let (cost, _) = parse_cost(cost)?;
        let mut b = Builder::new(ctx);
        let effect = parse_effect_text(eff, &mut b)?;
        modes.push(Mode {
            text: line.to_string(),
            targets: b.targets,
            effect,
            cost: Some(cost),
        });
    }
    let n = modes.len() as i32;
    let modal = Modal {
        min: Value::c(1),
        max: Value::c(n),
        allow_repeat: false,
        modes,
        per_mode_cost: true,
        chooser: ModeChooser::Controller,
    };
    modal_ability(modal, block, ctx)
}

inventory::submit! { AbilityPattern { name: "r700 spree modes", priority: 70, parse: spree_modes } }
