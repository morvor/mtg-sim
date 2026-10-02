//! Linked abilities where the second refers to the player the first targeted (CR 607.1):
//! "When ~ enters, target player loses 6 life." / "When ~ leaves the battlefield, that
//! player gains 6 life." (Laquatus's Champion, Soul Scourge).
//!
//! The ability with "that player" has no referent of its own, so it's grouped with the
//! earlier ability of the card that targets a player: that ability notes its target as it
//! resolves ([`Effect::NoteLinked`]), and "that player" is the noted player — or, while
//! that ability is still on the stack, its target ([`PlayerRef::LinkedNoted`], see
//! `linked_notes.rs`).

use super::{AbilityPattern, BlockGroupPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

/// Marks the second ability of a grouped pair.
const MARK: &str = "\u{2}linked-that-player\u{2}";

/// The abilities of a block as the compiler accepts them (see `oracle::compile`).
fn parsed(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    crate::oracle::parse_ability(block, ctx).filter(|v| {
        !v.iter().any(|a| {
            super::oracle_hardening_referents::has_no_referent(a)
                || crate::repeat_process::has_stray_repeat(a)
                || super::filters_relational::unresolved(a)
        })
    })
}

/// The body of a triggered or activated ability.
fn body_mut(a: &mut AbilityKind) -> Option<&mut Body> {
    match a {
        AbilityKind::Triggered(t) => Some(&mut t.body),
        AbilityKind::Activated(x) => Some(&mut x.body),
        _ => None,
    }
}

/// The first ability, noting the player it targets, and the second, whose "that player"
/// is that player.
fn build(first: &str, second: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = second.to_lowercase();
    if !lower.contains("that player") || lower.contains("target") {
        return None;
    }
    if parsed(second, ctx).is_some() {
        return None;
    }
    let v = parsed(first, ctx)?;
    let [a] = v.as_slice() else {
        return None;
    };
    let mut kind = a.kind.clone();
    let body = body_mut(&mut kind)?;
    if body.modal.is_some() {
        return None;
    }
    // Exactly one target, a player.
    let [spec] = body.targets.as_slice() else {
        return None;
    };
    if !matches!(spec.what, TargetKind::Player(_)) {
        return None;
    }
    let effect = std::mem::take(&mut body.effect);
    body.effect = Effect::seq(vec![
        Effect::NoteLinked {
            what: Sel::Players(PlayerRef::Target(0)),
            replace: false,
        },
        effect,
    ]);
    let probe = second
        .replace("that player", "the noted player")
        .replace("That player", "The noted player");
    let rest = parsed(&probe, ctx)?;
    let mut out = vec![AbilityDef::with_link(kind, first, a.link)];
    out.extend(
        rest.into_iter()
            .map(|b| AbilityDef::with_link(b.kind.clone(), second, b.link)),
    );
    Some(out)
}

/// Groups an ability with "that player" and no referent with the last earlier ability of
/// the card that targets a player.
fn group(blocks: Vec<String>, ctx: &CompileContext) -> Vec<String> {
    if !blocks
        .iter()
        .any(|b| b.to_lowercase().contains("that player"))
    {
        return blocks;
    }
    let mut blocks = blocks;
    let mut i = 1;
    while i < blocks.len() {
        let found = (0..i)
            .rev()
            .find(|&j| !blocks[j].contains(MARK) && build(&blocks[j], &blocks[i], ctx).is_some());
        if let Some(j) = found {
            let second = blocks.remove(i);
            blocks[j] = format!("{}\n{MARK}{}", blocks[j], second);
        } else {
            i += 1;
        }
    }
    blocks
}

fn parse_group(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (first, second) = block.split_once(&format!("\n{MARK}"))?;
    build(first, second, ctx)
}

inventory::submit! { BlockGroupPattern { name: "r607 linked: that player the earlier ability targeted", priority: 90, group } }
inventory::submit! { AbilityPattern { name: "r607 linked: that player the earlier ability targeted", priority: 0, parse: parse_group } }

/// "Choose a creature card exiled with ~." on a card whose linked ability refers to "the
/// last chosen card" (Koh, the Face Stealer): the chosen card is noted for it, replacing
/// the card chosen before (CR 607.2d, 607.2e).
fn choose_card_for_linked(l: &str, _b: &mut Builder) -> Option<Effect> {
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("the last chosen card") {
        return None;
    }
    let r = end(l).strip_prefix("choose ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let r = r.strip_suffix(" exiled with ~")?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).is_empty() {
        return None;
    }
    let filter = Filter::and(vec![
        f,
        Filter::In(Box::new(Sel::Linked)),
        Filter::InZone(ZoneKind::Exile),
    ]);
    Some(Effect::NoteLinked {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter,
            count: Value::c(1),
            up_to: false,
            store: None,
        },
        replace: true,
    })
}

inventory::submit! { EffectPattern { name: "r607 linked: choose a card exiled with ~ (the last chosen card)", priority: 50, parse: choose_card_for_linked } }
