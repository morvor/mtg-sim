//! "Choose [targets]." as a sentence of its own (CR 115.1, 601.2c): the targets are
//! chosen as the spell or ability is put on the stack, and choosing them does nothing by
//! itself. Later sentences refer to them ("the chosen spell", "that spell", "those
//! creatures", "them", "each of them").
//!
//! This covers what the narrower "choose target [object]" patterns leave: spells ("Choose
//! target spell.", "Choose target instant or sorcery spell."), several targets of one
//! instance of the word "target" ("Choose two target creatures.", "Choose up to three
//! target cards in graveyards."), lists ("Choose up to one target artifact, up to one
//! target creature, and up to one target land.") and requirements on the targets taken
//! together ("controlled by the same player", "controlled by different players", CR
//! 115.3; see `target_groups.rs`).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// "controlled by the same player" / "controlled by the same opponent" / "controlled by
/// different players" after a target phrase: the requirement, an extra filter, and the
/// rest.
fn controller_group(t: &str) -> Option<(TargetGroup, Option<Filter>, &str)> {
    let t = t.trim_start();
    if let Some(r) = t.strip_prefix("controlled by the same player") {
        return Some((TargetGroup::SameController, None, r));
    }
    if let Some(r) = t.strip_prefix("controlled by the same opponent") {
        return Some((
            TargetGroup::SameController,
            Some(Filter::ControlledBy(PlayerRel::Opponent)),
            r,
        ));
    }
    if let Some(r) = t.strip_prefix("controlled by different players") {
        return Some((TargetGroup::DifferentControllers, None, r));
    }
    None
}

/// The noun later sentences use for a target ("spell", "creature", "card", "permanent").
fn noun(spec: &TargetSpec, text: &str) -> &'static str {
    match &spec.what {
        TargetKind::Spell(_) => "spell",
        TargetKind::SpellOrAbility(_) => "spell",
        _ => {
            let t = text.to_string();
            if t.contains(" card") {
                "card"
            } else if t.contains("creature") && !t.contains(" or ") && !t.contains(", ") {
                "creature"
            } else if t.contains("artifact") && !t.contains(" or ") && !t.contains(", ") {
                "artifact"
            } else {
                "permanent"
            }
        }
    }
}

fn choose_targets(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("choose ")?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone(), b.named.len());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
        b.named.truncate(saved.3);
    };
    let mut rest = r;
    let mut slots: Vec<(u8, &'static str)> = Vec::new();
    loop {
        let Some((mut spec, tail)) = parse_target(rest) else {
            restore(b);
            return None;
        };
        let mut tail = tail;
        if let Some((grp, extra, t2)) = controller_group(tail) {
            if spec.together.is_some() {
                restore(b);
                return None;
            }
            spec.together = Some(grp);
            if let (Some(x), TargetKind::Object(f)) = (extra, &mut spec.what) {
                *f = Filter::and(vec![f.clone(), x]);
            }
            tail = t2;
        }
        // Players are other patterns' ("choose target player"); so are qualifiers this
        // pattern doesn't read.
        if !matches!(
            spec.what,
            TargetKind::Object(_) | TargetKind::Spell(_) | TargetKind::SpellOrAbility(_)
        ) {
            restore(b);
            return None;
        }
        let text = rest[..rest.len() - tail.len()].trim().to_string();
        let n = noun(&spec, &text);
        let same_controller = matches!(spec.together, Some(TargetGroup::SameController));
        let slot = b.add_target(spec, &text);
        // "Choose two target creatures controlled by the same player. That player ...":
        // their controller.
        if same_controller {
            b.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(slot)));
        }
        slots.push((slot, n));
        let t = end(tail);
        if t.is_empty() {
            break;
        }
        rest = match t
            .strip_prefix(", and ")
            .or_else(|| t.strip_prefix(", "))
            .or_else(|| t.strip_prefix("and "))
            .or_else(|| tail.trim_start().strip_prefix("and "))
        {
            Some(x) if x.starts_with("target ") || x.starts_with("up to ") => x,
            _ => {
                restore(b);
                return None;
            }
        };
    }
    // The narrower patterns read one target of one instance of "target" (and two of
    // them joined by "and"): only what they don't.
    let one = |s: u8| {
        let spec = &b.targets[s as usize];
        spec.fixed_min() == Some(1)
            && matches!(spec.max, Value::Const(1))
            && spec.together.is_none()
            && matches!(spec.what, TargetKind::Object(_))
    };
    if slots.len() <= 2 && slots.iter().all(|(s, _)| one(*s)) {
        restore(b);
        return None;
    }
    let all = if slots.len() == 1 {
        Sel::Target(slots[0].0)
    } else {
        Sel::Union(slots.iter().map(|(s, _)| Sel::Target(*s)).collect())
    };
    let same_noun = slots.iter().all(|(_, n)| *n == slots[0].1);
    let mut names: Vec<String> = vec!["the chosen permanents".into(), "the chosen cards".into()];
    if same_noun {
        let n = slots[0].1;
        names.extend([format!("the chosen {n}s"), format!("those {n}s")]);
        let single = slots.len() == 1
            && matches!(b.targets[slots[0].0 as usize].max, Value::Const(1));
        if single {
            names.extend([format!("the chosen {n}"), format!("that {n}")]);
        }
    }
    for name in names {
        b.named.push((name, all.clone()));
    }
    b.it = all;
    Some(Effect::Noop)
}

inventory::submit! { EffectPattern { name: "choice grammar: choose [targets]", priority: 975, parse: choose_targets } }

#[cfg(test)]
mod tests {
    use crate::card;

    #[test]
    fn choose_target_spell_and_groups_compile() {
        for name in ["Swallowed by Leviathan", "Run Away Together", "Incriminate"] {
            let def = card(name);
            assert!(
                def.unsupported_text().is_empty(),
                "{name}: {:?}",
                def.unsupported_text()
            );
        }
    }
}
