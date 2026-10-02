//! Rulings batch P112 — activated abilities with sacrifice costs and the abilities around
//! them: Kels's "whenever you sacrifice" trigger (CR 603.2, 603.3), costs paid once
//! (CR 118.8 / 117.12), last known information of a sacrificed creature (CR 608.2h),
//! targets chosen before costs are paid (CR 602.2b, 601.2c, 601.2h), regeneration
//! (CR 701.19), mana abilities (CR 605) and delayed triggers (CR 603.7).

use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::ability_targets;
use crate::r_s20_common::tap_for_mana;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn indestructible(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.has_keyword(KeywordKind::Indestructible)
}

/// P0's Kels with a Grizzly Bears and three Swamps.
fn kels_game() -> (TestGame, ObjectId, ObjectId) {
    supported("Kels, Fight Fixer");
    let mut t = TestGame::new(2);
    let kels = t.battlefield(P0, "Kels, Fight Fixer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    (t, kels, bears)
}

#[test]
fn kels_trigger_from_a_sacrifice_cost_resolves_first_and_pays_once() {
    cr!("603.2", "603.3", "608.1");
    ruling!(
        "Kels, Fight Fixer",
        "If you sacrifice a permanent as part of a spell or activated ability's cost, Kels's middle ability will resolve before that spell or ability."
    );
    ruling!(
        "Kels, Fight Fixer",
        "Kels's middle ability is a triggered ability, not an activated ability. It doesn't allow you to sacrifice a permanent whenever you want; rather, you need some other way of sacrificing permanents, such as Kels's last ability."
    );
    ruling!(
        "Kels, Fight Fixer",
        "While resolving Kels's middle ability, you can't pay {U/B} more than once to draw more than one card."
    );
    let (mut t, kels, bears) = kels_game();
    // Kels's only activated ability is the indestructible one; the draw is triggered.
    let activated = t
        .obj_now(kels)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    assert_eq!(activated, 1);
    // Plenty of mana: still only one card.
    t.lands(P0, "Swamp", 3);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[bears.into()]);
    t.activate(P0, kels, 0, &[]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    // The trigger resolved first: a card drawn, Kels not indestructible yet.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(!indestructible(&t, kels));
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(indestructible(&t, kels));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kels_can_gain_indestructible_while_already_indestructible() {
    cr!("602.2");
    ruling!(
        "Kels, Fight Fixer",
        "You can activate Kels's last ability even if nothing's about to destroy it and even if it already has indestructible."
    );
    let (mut t, kels, bears) = kels_game();
    let bears2 = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[bears.into()]);
    t.answer_yes(P0, false);
    t.activate(P0, kels, 0, &[]).unwrap();
    t.resolve_all();
    assert!(indestructible(&t, kels));
    assert!(can_activate(&mut t, P0, kels));
    t.answer_choose(P0, &[bears2.into()]);
    t.answer_yes(P0, false);
    t.activate(P0, kels, 0, &[]).unwrap();
    t.resolve_all();
    assert!(indestructible(&t, kels));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(creatures(&t, P0) == vec![t.g.current(kels)]);
}

#[test]
fn kheru_dreadmaw_uses_the_sacrificed_creatures_last_toughness() {
    cr!("608.2h");
    ruling!(
        "Kheru Dreadmaw",
        "Use the toughness of the creature as it last existed on the battlefield to determine how much life you gain."
    );
    supported("Kheru Dreadmaw");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Kheru Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[bears.into()]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[bears.into()]);
    t.activate(P0, maw, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 25);
}

#[test]
fn rhys_sacrificed_for_its_own_regeneration_has_no_effect() {
    cr!("602.2b", "701.19a");
    ruling!(
        "Rhys the Exiled",
        "You may sacrifice Rhys the Exiled to pay for its own regeneration ability. However, since Rhys is no longer on the battlefield, the ability will have no effect."
    );
    supported("Rhys the Exiled");
    let mut t = TestGame::new(2);
    let rhys = t.battlefield(P0, "Rhys the Exiled");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[rhys.into()]);
    t.activate(P0, rhys, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Rhys the Exiled"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Rhys the Exiled"));
    assert!(t.named_on_battlefield("Rhys the Exiled").is_empty());
}

#[test]
fn spectral_lynx_can_regenerate_when_not_at_risk() {
    cr!("701.19a");
    ruling!(
        "Spectral Lynx",
        "You can activate the regeneration ability even if Spectral Lynx isn’t at risk of being destroyed."
    );
    supported("Spectral Lynx");
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P0, "Spectral Lynx");
    t.lands(P0, "Swamp", 1);
    assert!(can_activate(&mut t, P0, lynx));
    t.activate(P0, lynx, 0, &[]).unwrap();
    t.resolve_all();
    // The shield waits for a later destruction this turn.
    destroy(&mut t, lynx);
    assert!(t.on_battlefield(lynx));
    assert!(t.obj_now(lynx).tapped);
}

#[test]
fn satyr_hedonist_is_a_mana_ability_that_does_not_use_the_stack() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Satyr Hedonist",
        "Satyr Hedonist's ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Satyr Hedonist");
    let mut t = TestGame::new(2);
    let satyr = t.battlefield(P0, "Satyr Hedonist");
    t.lands(P0, "Mountain", 1);
    assert!(tap_for_mana(&mut t, P0, satyr, "Add"));
    assert_eq!(t.stack_len(), 0);
    assert!(t.in_graveyard(P0, "Satyr Hedonist"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);
}

#[test]
fn protomatter_powder_cannot_return_itself() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Protomatter Powder",
        "You can’t sacrifice Protomatter Powder to return itself to the battlefield. First you choose the target (at which time it’s still on the battlefield), then you pay the costs (at which time you sacrifice it)."
    );
    supported("Protomatter Powder");
    let mut t = TestGame::new(2);
    let powder = t.battlefield(P0, "Protomatter Powder");
    t.lands(P0, "Plains", 5);
    // No artifact card in the graveyard: no legal target, so it can't be activated.
    assert!(!can_activate(&mut t, P0, powder));
    let thopter = t.graveyard(P0, "Ornithopter");
    assert_eq!(ability_targets(&mut t, powder, 0), vec![Entity::Object(thopter)]);
    t.activate(P0, powder, 0, &[thopter.into()]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert!(t.in_graveyard(P0, "Protomatter Powder"));
}

#[test]
fn savage_gorilla_draws_nothing_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Savage Gorilla",
        "You do not get to draw a card if the target is not legal on resolution."
    );
    supported("Savage Gorilla");
    for spoil in [false, true] {
        let mut t = TestGame::new(2);
        let gorilla = t.battlefield(P0, "Savage Gorilla");
        let victim = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Island", 1);
        t.lands(P0, "Swamp", 1);
        let hand = t.hand_size(P0);
        t.activate(P0, gorilla, 0, &[victim.into()]).unwrap();
        if spoil {
            destroy(&mut t, victim);
        }
        t.resolve_all();
        if spoil {
            assert_eq!(t.hand_size(P0), hand);
        } else {
            assert_eq!(t.pt(victim), (3, 1));
            assert_eq!(t.hand_size(P0), hand + 1);
        }
    }
}

#[test]
fn bagel_and_schmear_share_without_a_target_just_draws() {
    cr!("601.2c");
    ruling!(
        "Bagel and Schmear",
        "You can activate Bagel and Schmear's first ability with no target. In that case, you will only draw a card when it resolves."
    );
    supported("Bagel and Schmear");
    let mut t = TestGame::new(2);
    let bagel = t.battlefield(P0, "Bagel and Schmear");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[]);
    t.activate(P0, bagel, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(bears, mtg_engine::types::counters::PLUS1), 0);
    assert!(t.in_graveyard(P0, "Bagel and Schmear"));
}

#[test]
fn puppet_conjurer_sacrifices_any_homunculus() {
    cr!("701.21a");
    ruling!(
        "Puppet Conjurer",
        "The Homunculus you sacrifice due to the second ability isn’t limited to being a Homunculus token put onto the battlefield by Puppet Conjurer."
    );
    supported("Puppet Conjurer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puppet Conjurer");
    t.battlefield(P0, "Sneaky Homunculus");
    into_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sneaky Homunculus"));
    assert_eq!(t.named_on_battlefield("Puppet Conjurer").len(), 1);
}

#[test]
fn transluminant_in_the_end_step_waits_for_the_next_end_step() {
    cr!("603.7a", "603.2b");
    ruling!(
        "Transluminant",
        "If you use this ability during a turn’s end phase, the chance to put “at end of turn”-triggered abilities on the stack has passed. You won’t get the Spirit creature token until the beginning of the next end step."
    );
    supported("Transluminant");
    let mut t = TestGame::new(2);
    let lum = t.battlefield(P0, "Transluminant");
    t.lands(P0, "Plains", 1);
    t.set_step(P0, Step::End);
    t.activate(P0, lum, 0, &[]).unwrap();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    t.advance_to(P1, Step::Upkeep);
    assert!(tokens(&t, P0).is_empty());
    t.advance_to(P1, Step::End);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert!(t.obj_now(toks[0]).chars.has_subtype("Spirit"));
    assert_eq!(t.zone(lum), Zone::Graveyard(P0));
}
