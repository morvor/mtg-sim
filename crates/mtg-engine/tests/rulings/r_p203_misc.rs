//! Rulings batch P203 — cipher, cleave, cloak, cohort, constellation and converge rulings.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, target_candidates};
use crate::r_s03_common::*;
use crate::r_s05_common::move_to;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use mtg_engine::events::Event;
use mtg_engine::kw::cipher::encoded_on;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Cipher -------------------------------------------------------------------------------

/// P0 casts the cipher card `name` targeting `target` and encodes it on a creature with
/// shroud, Argothian Enchantress, which can't be targeted. Asserts it was chosen as the
/// spell resolved and never targeted.
fn encode_on_shroud_creature(name: &str, target: impl Fn(&mut TestGame) -> Entity) {
    supported(name);
    supported("Argothian Enchantress");
    let mut t = TestGame::new(2);
    let ench = t.battlefield(P0, "Argothian Enchantress");
    let tgt = target(&mut t);
    let c = in_hand_with_mana(&mut t, P0, name);
    t.answer_targets(P0, &[tgt]);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.cast(P0, c).go();
    // While casting, nothing about the encoding was chosen.
    assert!(choice_candidates(&t, from, "cipher").is_empty());
    t.answer_choose(P0, &[Entity::Object(ench)]);
    t.resolve_all();
    let cands = choice_candidates(&t, from, "cipher");
    assert_eq!(cands.len(), 1, "{name}");
    assert!(cands[0].contains(&Entity::Object(ench)));
    assert!(!target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(ench))));
    assert_eq!(encoded_on(&t.g, t.g.current(c)), Some(ench), "{name}");
    assert!(!t.g.turn_events.iter().any(
        |e| matches!(e, Event::BecameTarget { target: Entity::Object(o), .. } if *o == ench)
    ));
}

#[test]
fn the_cipher_creature_is_chosen_on_resolution_and_isnt_targeted() {
    cr!("702.99a", "608.2c", "702.18a");
    ruling!(
        "Writ of Return",
        "You choose the creature as the spell resolves. The cipher ability doesn't target that creature."
    );
    ruling!(
        "Arcane Heist",
        "You choose the creature as the spell resolves. The cipher ability doesn’t target that creature."
    );
    // Writ of Return: "Return target creature card from your graveyard to the battlefield
    // tapped."
    encode_on_shroud_creature("Writ of Return", |t| {
        Entity::Object(t.graveyard(P0, "Hill Giant"))
    });
    // Arcane Heist: "You may cast target instant or sorcery card from an opponent's
    // graveyard without paying its mana cost. ..." (P0 declines.)
    encode_on_shroud_creature("Arcane Heist", |t| {
        Entity::Object(t.graveyard(P1, "Divination"))
    });
}

// --- Cleave -------------------------------------------------------------------------------

#[test]
fn inspired_idea_and_null_profusion_apply_in_timestamp_order() {
    cr!("402.2", "613.11", "702.148a");
    ruling!(
        "Inspired Idea",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you are affected by Null Profusion (an enchantment that sets a player's maximum hand size to two) and then resolve Inspired Idea without paying its cleave cost, your maximum hand size would be zero. However, if Null Profusion entered the battlefield after Inspired Idea resolved, your maximum hand size would be two."
    );
    supported("Inspired Idea");
    supported("Null Profusion");
    let max = |t: &mut TestGame| {
        t.g.recompute();
        t.g.player(P0).max_hand_size.map(|m| m.max(0))
    };
    // Null Profusion, then Inspired Idea: 2 - 3, so zero.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Null Profusion");
    assert_eq!(max(&mut t), Some(2));
    let c = in_hand_with_mana(&mut t, P0, "Inspired Idea");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(max(&mut t), Some(0));
    // Inspired Idea, then Null Profusion: two.
    let mut t = TestGame::new(2);
    let c = in_hand_with_mana(&mut t, P0, "Inspired Idea");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(max(&mut t), Some(4));
    t.battlefield(P0, "Null Profusion");
    assert_eq!(max(&mut t), Some(2));
}

// --- Cloak --------------------------------------------------------------------------------

#[test]
fn cryptic_coat_cloaks_even_if_it_left_the_battlefield() {
    cr!("701.58a", "608.2c");
    ruling!(
        "Cryptic Coat",
        "You'll still cloak the top card of your library even if Cryptic Coat isn't on the battlefield as its first ability resolves."
    );
    supported("Cryptic Coat");
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, "Hill Giant");
    let coat = t.enter(P0, "Cryptic Coat");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the Coat returns to its owner's hand.
    move_to(&mut t, coat, Zone::Hand(P0));
    t.resolve_all();
    let c = t.g.current(card);
    assert!(t.on_battlefield(c) && t.obj(c).face_down);
    assert_eq!(t.pt(c), (2, 2));
    assert!(t.in_hand(P0, "Cryptic Coat"));
    assert!(!t
        .g
        .permanents()
        .any(|o| o.attached_to == Some(Entity::Object(c))));
}

// --- Cohort -------------------------------------------------------------------------------

#[test]
fn akoum_flameseekers_discard_isnt_optional() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Akoum Flameseeker",
        "Discarding a card is part of the ability’s effect and isn’t optional. As the cohort ability resolves, if you have a card in your hand, you must discard one, even if the card you may have wanted to discard as you activated the ability is no longer in your hand."
    );
    supported("Akoum Flameseeker");
    // "Cohort — {T}, Tap an untapped Ally you control: Discard a card. If you do, draw a
    // card." P0 activates holding Shock and Hill Giant; the Shock leaves P0's hand before
    // the ability resolves: the Giant must be discarded.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Akoum Flameseeker");
    let b = t.battlefield(P0, "Akoum Flameseeker");
    let shock = t.hand(P0, "Shock");
    t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.activate(P0, a, 0, &[]).unwrap();
    move_to(&mut t, shock, Zone::Exile);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 1);
    assert!(!t.in_hand(P0, "Hill Giant"));
    // With an empty hand as it resolves, nothing is discarded, so nothing is drawn.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Akoum Flameseeker");
    let b = t.battlefield(P0, "Akoum Flameseeker");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.activate(P0, a, 0, &[]).unwrap();
    move_to(&mut t, shock, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn zadas_commando_cant_be_summoning_sick_but_the_other_ally_can() {
    cr!("302.6", "602.5a");
    ruling!(
        "Zada's Commando",
        "To activate a cohort ability, the Ally with that ability must have been under your control continuously since the beginning of your most recent turn. Informally, it can't have \"summoning sickness.\" However, the other Ally you tap can be one that just came under your control. (Note that tapping the second Ally doesn't use {T} [the tap symbol].)"
    );
    supported("Zada's Commando");
    // "Cohort — {T}, Tap an untapped Ally you control: This creature deals 1 damage to
    // target opponent or planeswalker."
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Zada's Commando");
    let new = t.battlefield_sick(P0, "Zada's Commando");
    assert!(!can_activate(&mut t, P0, new));
    assert!(can_activate(&mut t, P0, old));
    t.answer_choose(P0, &[Entity::Object(new)]);
    t.activate(P0, old, 0, &[Entity::Player(P1)]).unwrap();
    assert!(t.obj(old).tapped && t.obj(new).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

// --- Constellation ------------------------------------------------------------------------

#[test]
fn calix_can_target_itself_with_its_constellation_ability() {
    cr!("115.1", "603.3d");
    ruling!(
        "Calix, Guided by Fate",
        "Calix may be the target of its own constellation ability. This includes when Calix itself enters the battlefield."
    );
    supported("Calix, Guided by Fate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let calix = t.enter(P0, "Calix, Guided by Fate");
    t.answer_targets(P0, &[Entity::Object(calix)]);
    t.settle();
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&Entity::Object(calix)));
    assert!(cands[0].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.counters(calix, counters::PLUS1), 1);
    // Another enchantment entering: Calix again.
    t.answer_targets(P0, &[Entity::Object(calix)]);
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(t.counters(calix, counters::PLUS1), 2);
}

#[test]
fn a_copy_of_protean_thaumaturge_has_its_constellation_ability() {
    cr!("707.2");
    ruling!(
        "Protean Thaumaturge",
        "If something becomes a copy of Protean Thaumaturge, it also has the constellation ability."
    );
    supported("Protean Thaumaturge");
    supported("Clone");
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Protean Thaumaturge");
    t.answer_choose(P0, &[Entity::Object(pt)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Protean Thaumaturge");
    // An enchantment entering: both constellation abilities trigger.
    t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "become a copy"), 2);
}

#[test]
fn goldenhide_ox_doesnt_force_any_specific_creature_to_block() {
    cr!("509.1c");
    ruling!(
        "Goldenhide Ox",
        "The constellation ability doesn't force any specific creature to block the target creature."
    );
    supported("Goldenhide Ox");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Goldenhide Ox");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(y, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, bears), (y, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[]));
}

#[test]
fn agent_of_erebos_must_target_you_if_youre_the_only_legal_target() {
    cr!("603.3d", "702.11c");
    ruling!(
        "Agent of Erebos",
        "The constellation ability is mandatory. If you are the only legal target (perhaps because your opponent has hexproof), you must target yourself and exile all the cards from your graveyard."
    );
    supported("Agent of Erebos");
    supported("Leyline of Sanctity");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Sanctity");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Shock");
    let from = t.asked().len();
    t.enter(P0, "Agent of Erebos");
    t.resolve_all();
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![Entity::Player(P0)]]);
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Shock"));
    assert_eq!(t.graveyard_size(P1), 1);
}

// --- Converge -----------------------------------------------------------------------------

#[test]
fn mana_spent_on_a_cost_increase_counts_for_converge() {
    cr!("207.2c", "601.2f");
    ruling!(
        "Crystalline Crawler",
        "If there are any alternative or additional costs to cast a spell with a converge ability, the mana spent to pay those costs will count. For example, if an effect makes Crystalline Crawler cost {1} more to cast, you could pay {W}{U}{B}{R}{G} to cast it and have it enter the battlefield with five +1/+1 counters."
    );
    supported("Crystalline Crawler");
    supported("Sphere of Resistance");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        t.g.players[0].mana_pool.add_type(ty, 1);
    }
    let c = t.hand(P0, "Crystalline Crawler");
    t.cast(P0, c).go();
    t.resolve_all();
    let crawler = t.named_on_battlefield("Crystalline Crawler")[0];
    assert_eq!(t.counters(crawler, counters::PLUS1), 5);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}
