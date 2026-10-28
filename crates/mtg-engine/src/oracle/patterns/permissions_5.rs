//! Permissions grammar V (round 4): permissions over cards a player exiled (exile
//! provenance, `kw/exile_provenance.rs`), and the draw replacements that feed them.
//!
//! * "You may play lands and cast noncreature spells from among cards you exiled that have
//!   fetch counters on them, ..." (Haldan, Avid Arcanist): the cards you put into exile.
//! * "Once each turn, you may play a card from exile with a collection counter on it if it
//!   was exiled by an ability you controlled, ..." (Evelyn, the Covetous).
//! * "Each player may play lands and cast spells from among cards they exiled with ~ this
//!   turn." (Uba Mask), "Each player may look at cards they exiled with ~, and they may
//!   play lands and cast spells from among those cards." (Shared Fate): each player's
//!   permission for the cards linked to the source that they exiled (CR 607.2a).
//! * "If a player would draw a card, that player exiles that card face up instead." (Uba
//!   Mask: the card they would draw), "If a player would draw a card, that player exiles
//!   the top card of one of their opponents' libraries face down instead." (Shared Fate).
//!
//! Each is read as the "you" form the permission grammar knows, then given the player
//! who exiled the cards.

use super::AbilityPattern;
use crate::ability::*;
use crate::kw::exile_provenance::{EACH_MAY_LOOK_AT_OWN_EXILED, EXILED_BY_THEM, EXILED_BY_YOU};
use crate::oracle::CompileContext;
use std::sync::Arc;

/// Compiles `rewritten` as the block `block` and changes each ability with `f`; the
/// abilities keep the block's text.
fn compile_as(
    rewritten: &str,
    block: &str,
    ctx: &CompileContext,
    f: impl Fn(&mut AbilityDef) -> Option<()>,
) -> Option<Vec<Ability>> {
    let v = crate::oracle::parse_ability(rewritten, ctx)?;
    let mut out = Vec::new();
    for a in v {
        let mut a = (*a).clone();
        f(&mut a)?;
        a.text = block.to_string();
        out.push(Arc::new(a));
    }
    Some(out)
}

/// The play permission of a static ability.
fn permission(a: &mut AbilityDef) -> Option<&mut PlayPermission> {
    match &mut a.kind {
        AbilityKind::Static(s) => match &mut s.effect {
            StaticEffect::PlayPermission(pp) => Some(pp),
            _ => None,
        },
        _ => None,
    }
}

fn add_quality(pp: &mut PlayPermission, q: Filter) {
    let what = std::mem::replace(&mut pp.what, Filter::Any);
    pp.what = Filter::and(vec![what, q]);
}

/// "... from among cards you exiled that have fetch counters on them ..." (Haldan, Avid
/// Arcanist), "... a card from exile with a collection counter on it if it was exiled by
/// an ability you controlled ..." (Evelyn, the Covetous).
fn cards_you_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let (rewritten, owned) = if block.contains(" from among cards you exiled ") {
        (
            block.replacen(" from among cards you exiled ", " from among cards you own in exile ", 1),
            true,
        )
    } else if block.contains(" if it was exiled by an ability you controlled") {
        (block.replacen(" if it was exiled by an ability you controlled", "", 1), false)
    } else {
        return None;
    };
    compile_as(&rewritten, block, ctx, |a| {
        let pp = permission(a)?;
        if pp.zone != ZoneKind::Exile || pp.who != PlayerRel::You {
            return None;
        }
        if owned {
            // Not only cards you own: the cards you exiled.
            let Filter::And(v) = &mut pp.what else {
                return None;
            };
            let i = v
                .iter()
                .position(|f| matches!(f, Filter::OwnedBy(PlayerRel::You)))?;
            v[i] = Filter::Custom(EXILED_BY_YOU.into());
        } else {
            add_quality(pp, Filter::Custom(EXILED_BY_YOU.into()));
        }
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: cards you exiled", priority: 40, parse: cards_you_exiled } }

/// "Each player may play lands and cast spells from among cards they exiled with ~ this
/// turn.", "Each player may look at cards they exiled with ~, and they may play lands and
/// cast spells from among those cards.": each player's permission for the cards linked to
/// the source that they exiled.
fn each_player_cards_they_exiled(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block.contains('\n') {
        return None;
    }
    let rewritten = if let Some(r) = block.strip_prefix("Each player may look at cards they exiled with ~, and they may ") {
        format!("You may look at cards exiled with ~, and you may {r}")
    } else {
        let r = block.strip_prefix("Each player may ")?;
        if !r.contains(" from among cards they exiled with ~") {
            return None;
        }
        format!(
            "You may {}",
            r.replacen(" from among cards they exiled with ~", " from among cards exiled with ~", 1)
        )
    };
    compile_as(&rewritten, block, ctx, |a| {
        if let AbilityKind::Static(s) = &mut a.kind {
            if let StaticEffect::Custom(n) = &mut s.effect {
                if n == crate::kw::look_at_exiled_with::MAY_LOOK_AT_LINKED {
                    *n = EACH_MAY_LOOK_AT_OWN_EXILED.into();
                    return Some(());
                }
                return None;
            }
        }
        let pp = permission(a)?;
        if pp.zone != ZoneKind::Exile || pp.who != PlayerRel::You {
            return None;
        }
        // The cards linked to the source (CR 607.2a) that this player exiled.
        if !format!("{:?}", pp.what).contains("In(Linked)") {
            return None;
        }
        pp.who = PlayerRel::Any;
        add_quality(pp, Filter::Custom(EXILED_BY_THEM.into()));
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: each player, cards they exiled with ~", priority: 40, parse: each_player_cards_they_exiled } }

/// "If a player would draw a card, that player exiles that card face up instead." (Uba
/// Mask): the card they would draw, the top card of their library, exiled by them with
/// the source (CR 614.6, 607.2a).
fn draw_exiles_that_card(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = block.strip_prefix("If a player would draw a card, that player exiles ")?;
    if r != "that card face up instead." && r != "that card instead." {
        return None;
    }
    compile_as(
        "If a player would draw a card, that player exiles the top card of their library instead.",
        block,
        ctx,
        |_| Some(()),
    )
}

inventory::submit! { AbilityPattern { name: "permissions 5: a player exiles the card they would draw instead", priority: 40, parse: draw_exiles_that_card } }

/// "If a player would draw a card, that player exiles the top card of one of their
/// opponents' libraries face down instead." (Shared Fate): the player chooses one of their
/// opponents and exiles that player's top card face down, with the source (CR 406.3,
/// 607.2a).
fn draw_exiles_opponents_top(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if block
        != "If a player would draw a card, that player exiles the top card of one of their opponents' libraries face down instead."
    {
        return None;
    }
    let instead = Effect::AsPlayer {
        who: PlayerRef::TriggerPlayer,
        effect: Box::new(Effect::seq(vec![
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Opponent,
            },
            Effect::Exile {
                what: Sel::TopOfLibrary(PlayerRef::ChosenOpponent, Value::c(1)),
                face_down: true,
                link: true,
            },
        ])),
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::Any),
                action: ReplacementAction::Instead(Box::new(instead)),
                self_replacement: false,
                optional: false,
            },
        ))),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "permissions 5: a player exiles an opponent's top card face down instead of drawing", priority: 40, parse: draw_exiles_opponents_top } }

/// The triggered ability's body effect.
fn trigger_body(a: &mut AbilityDef) -> Option<&mut Effect> {
    match &mut a.kind {
        AbilityKind::Triggered(t) => Some(&mut t.body.effect),
        _ => None,
    }
}

/// "Whenever a creature deals combat damage to one of your opponents, its controller looks
/// at the top card of that opponent's library and exiles it face down. They may play that
/// card for as long as it remains exiled. Mana of any type can be spent to cast a spell
/// this way." (Gonti, Night Minister): the creature's controller does it all as if they
/// were "you" (they look, exile, and get the permission and its terms); the card is
/// exiled by them.
fn its_controller_looks_and_exiles(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (trigger, rest) = block.split_once(", its controller looks at ")?;
    if !trigger.starts_with("Whenever ") || block.contains('\n') {
        return None;
    }
    let (first, others) = rest.split_once(". ")?;
    let (what, how) = first.split_once(" and exiles it")?;
    let what = what.replace("that opponent's library", "that player's library");
    let others = others.strip_prefix("They may ")?;
    let rewritten = format!(
        "{trigger}, look at {what}, then exile it{how}. You may {others}"
    );
    // Who "its controller" is, as the trigger's text reads it.
    let probe = crate::oracle::parse_ability(
        &format!("{trigger}, its controller draws a card."),
        ctx,
    )?;
    let mut probe = (*probe.first()?.clone()).clone();
    let Effect::Draw { who, .. } = trigger_body(&mut probe)? else {
        return None;
    };
    let who = who.clone();
    if !matches!(who, PlayerRef::ControllerOf(_)) {
        return None;
    }
    compile_as(&rewritten, block, ctx, |a| {
        let body = trigger_body(a)?;
        let e = std::mem::replace(body, Effect::Noop);
        *body = Effect::AsPlayer {
            who: who.clone(),
            effect: Box::new(e),
        };
        Some(())
    })
}

inventory::submit! { AbilityPattern { name: "permissions 5: its controller looks at and exiles a card; they may play it", priority: 40, parse: its_controller_looks_and_exiles } }
