//! Copy grammar V (cards4-referents-statics, family `copies`).
//!
//! - "Enchanted creature is a copy of the chosen creature." (Metamorphic Alteration),
//!   "Equipped creature is a copy of the last chosen card." (Paleontologist's Pick-Axe //
//!   Dinosaur Headdress): a static ability's copy effect on the permanent the source is
//!   attached to (CR 707.2, 613.2a), copying the object the source's linked choice noted
//!   ([`Modification::CopyOf`], `static_copy.rs`).

use super::StaticPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// Whether the card notes a choice of one object for its other abilities: "As ~ enters,
/// choose a creature." or "..., choose an exiled creature card used to craft ~."
fn card_notes_one_choice(what: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    match what {
        "the chosen creature" => raw.contains(" enters, choose a creature.")
            || raw.contains(" enters, choose another creature"),
        "the last chosen card" => raw.contains(", choose an exiled creature card"),
        _ => false,
    }
}

fn attached_is_copy_of_chosen(l: &str, orig: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let l = end(l);
    let rest = l
        .strip_prefix("enchanted creature is a copy of ")
        .or_else(|| l.strip_prefix("equipped creature is a copy of "))?;
    if !ctx.is_permanent() || !card_notes_one_choice(rest) {
        return None;
    }
    let effect = StaticEffect::Continuous {
        affected: Filter::AttachedToSource,
        mods: vec![Modification::CopyOf(Box::new(Sel::LinkedNoted))],
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(effect)),
        orig,
    )])
}

inventory::submit! { StaticPattern { name: "copies 5: enchanted/equipped creature is a copy of the chosen object", priority: 100, parse: attached_is_copy_of_chosen } }

/// The card chosen to be copied.
const COPIED_CARD: Var = vars::USER + 51_201;

/// "Whenever one or more cards leave your graveyard, put a +1/+1 counter on ~, then you
/// may have ~ become a copy of an artifact or creature card from among those cards until
/// end of turn." (Spirit of Resilience): one of the cards of the triggering batch (CR
/// 603.2c, 603.10a), chosen as the ability resolves; a card that has left the zone it went
/// to is copied as it last existed there (CR 707.2, 608.2h).
fn become_copy_of_one_of_those_cards(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may have ~ become a copy of ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("~ becomes a copy of ")?),
    };
    let (noun, r) = r.split_once(" from among those cards")?;
    let duration = match r.trim() {
        "until end of turn" => Duration::EndOfTurn,
        "" => Duration::Permanent,
        _ => return None,
    };
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("whenever one or more cards leave your graveyard,") || !b.in_trigger {
        return None;
    }
    let noun = noun.strip_prefix("an ").or_else(|| noun.strip_prefix("a "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let choose = Effect::Store {
        var: COPIED_CARD,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(vec![Filter::In(Box::new(Sel::TriggerObjects)), f]),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    let copy = Effect::BecomeCopy {
        what: Sel::This,
        of: Sel::Var(COPIED_CARD),
        duration,
    };
    let e = Effect::seq(vec![choose, copy]);
    Some(if may {
        Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        }
    } else {
        e
    })
}

inventory::submit! { super::EffectPattern { name: "copies 5: become a copy of [a card] from among those cards", priority: 100, parse: become_copy_of_one_of_those_cards } }

/// "You may have ~ enter as a copy of [object]. If you do, it gains haste until end of
/// turn." (The Fourteenth Doctor): an effect that happens only if it enters as a copy
/// ([`Effect::EnterCopyExtra`], CR 614.1c, 707.9), performed on the permanent as it
/// enters.
fn enter_as_copy_if_you_do(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || !block.is_ascii() {
        return None;
    }
    let lower = block.to_lowercase();
    if !lower.starts_with("you may have ~ enter ") {
        return None;
    }
    let i = lower.find(". if you do, ")?;
    let first = &block[..i + 1];
    let rest = end(&lower[i + ". if you do, ".len()..]);
    let rest = rest.strip_prefix("it ")?;
    let body = crate::oracle::effects::parse_body_with_it(&format!("~ {rest}."), ctx, Sel::This)?;
    if !body.targets.is_empty() || body.modal.is_some() {
        return None;
    }
    let mut out = crate::oracle::parse_ability(first, ctx)?;
    // Only the plain optional copy replacement.
    let [a] = out.as_mut_slice() else {
        return None;
    };
    let AbilityKind::Static(st) = &a.kind else {
        return None;
    };
    let StaticEffect::Replacement(d) = &st.effect else {
        return None;
    };
    if !matches!(d.action, ReplacementAction::EnterAsCopy { optional: true, .. }) {
        return None;
    }
    let extra = Effect::EnterCopyExtra {
        only_if: None,
        mods: vec![],
        // Performed as it enters, with the permanent as its source (`actions.rs`).
        effect: Box::new(body.effect),
    };
    std::sync::Arc::make_mut(a).text = block.into();
    out.push(AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::EntersBattlefield(Filter::Source),
                action: ReplacementAction::AsEnters(Box::new(extra)),
                self_replacement: true,
                optional: false,
            },
        ))),
        block,
    ));
    Some(out)
}

inventory::submit! { super::AbilityPattern { name: "copies 5: you may have ~ enter as a copy of [object]. If you do, it [effect]", priority: 40, parse: enter_as_copy_if_you_do } }

/// "copy all spells you control" and "copy all other activated and triggered abilities
/// you control" (Ulalek, Fused Atrocity): each of them, as the instruction is performed
/// (CR 707.10; the second doesn't copy the resolving ability itself; mana abilities never
/// use the stack, CR 605.3a).
fn copy_all_you_control(l: &str, _b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let what = match end(l) {
        "copy all spells you control" | "copy each spell you control" => Filter::and(vec![
            Filter::Spell,
            Filter::ControlledBy(PlayerRel::You),
        ]),
        "copy all other activated and triggered abilities you control"
        | "copy each other activated and triggered ability you control" => Filter::and(vec![
            Filter::InZone(ZoneKind::Stack),
            Filter::ControlledBy(PlayerRel::You),
            Filter::Custom(crate::kw::referents_5::OTHER_ABILITY_ON_STACK.into()),
        ]),
        _ => return None,
    };
    Some(Effect::CopySpell {
        what: Sel::All(what),
        count: Value::c(1),
        new_targets: false,
    })
}

inventory::submit! { super::EffectPattern { name: "copies 5: copy all spells / other abilities you control", priority: 100, parse: copy_all_you_control } }

/// "You may choose new targets for the copies." after several copy instructions: for each
/// of them (CR 707.10c).
fn new_targets_for_all_copies(l: &str, prev: &mut Effect, _b: &mut crate::oracle::effects::Builder) -> bool {
    if end(l) != "you may choose new targets for the copies" {
        return false;
    }
    fn mark(e: &mut Effect) -> usize {
        match e {
            Effect::CopySpell { new_targets, .. } => {
                *new_targets = true;
                1
            }
            Effect::Seq(v) => v.iter_mut().map(mark).sum(),
            Effect::If { then, .. } => mark(then),
            Effect::May { effect, .. } => mark(effect),
            _ => 0,
        }
    }
    let mut copy = prev.clone();
    if mark(&mut copy) < 2 {
        return false;
    }
    *prev = copy;
    true
}

inventory::submit! { super::FollowupPattern { name: "copies 5: you may choose new targets for the copies (several copy instructions)", priority: 90, apply: new_targets_for_all_copies } }
