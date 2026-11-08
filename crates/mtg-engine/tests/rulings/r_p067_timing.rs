//! Rulings batch P067 — when cards exiled by these effects can be cast: a permission
//! doesn't change the card's timing (CR 601.3, 307.1, 305.2), while a card cast during a
//! resolution ignores type-based timing and can't be cast later (CR 608.2g); spells cast
//! during a resolution, and the triggers they cause (CR 603.3).

use crate::r_p067_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `card` into exile as if `by` had exiled it, with `n` counters of `kind` on it.
fn exile_by(t: &mut TestGame, card: ObjectId, by: PlayerId, kind: &str, n: u32) -> ObjectId {
    let x = t
        .g
        .move_object(card, Zone::Exile, mtg_engine::events::MoveCause::Exile, Some(by))
        .unwrap();
    if n > 0 {
        t.g.obj_mut(x).counters.insert(kind.into(), n);
    }
    x
}

/// Asserts that `p` can cast `card` in its main phase with an empty stack only (not in
/// combat, not with a spell on the stack, not in another player's turn).
fn sorcery_timing_only(t: &mut TestGame, p: PlayerId, card: ObjectId, ty: ManaType, n: u32) {
    let other = PlayerId(1 - p.0);
    t.set_step(p, Step::PostcombatMain);
    mana(t, p, ty, n);
    assert!(castable(t, p, card), "main phase, empty stack");
    let bolt = t.hand(other, "Lightning Bolt");
    mana(t, other, ManaType::R, 1);
    t.g.turn.priority = Some(other);
    t.cast(other, bolt).target(other).go();
    assert!(!castable(t, p, card), "stack not empty");
    t.resolve_all();
    t.set_step(p, Step::BeginningOfCombat);
    assert!(!castable(t, p, card), "combat");
    t.set_step(other, Step::PrecombatMain);
    assert!(!castable(t, p, card), "another player's turn");
    t.set_step(p, Step::PrecombatMain);
    mana(t, p, ty, n);
    assert!(castable(t, p, card));
}

#[test]
fn covetous_urge_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "307.1");
    ruling!(
        "Covetous Urge",
        "Covetous Urge's effect doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    supported("Covetous Urge");
    let mut t = TestGame::new(2);
    let div = t.hand(P1, "Divination");
    t.lands(P0, "Island", 4);
    let urge = t.hand(P0, "Covetous Urge");
    t.answer_choose_moved(P0, &[o(div)]);
    t.cast(P0, urge).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, div));
    // Mana as though it were any color: black mana pays for {2}{U}.
    sorcery_timing_only(&mut t, P0, div, ManaType::B, 3);
}

#[test]
fn daxos_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "302.1");
    ruling!(
        "Daxos of Meletis",
        "Daxos doesn't change when you can cast the exiled card. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Daxos of Meletis");
    let mut t = TestGame::new(2);
    let daxos = t.battlefield(P0, "Daxos of Meletis");
    let bears = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(daxos, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert!(exiled(&t, bears));
    assert_eq!(t.life(P0), 22, "life equal to its mana value");
    // During combat (end of combat step): no.
    mana(&mut t, P0, ManaType::W, 2);
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, bears));
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go();
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn dazzling_sphinx_card_is_cast_during_the_resolution_or_not_at_all() {
    cr!("608.2g");
    ruling!(
        "Dazzling Sphinx",
        "Dazzling Sphinx's triggered ability lets you cast the exiled card during the resolution of the ability. It doesn't allow you to wait and cast the card later."
    );
    supported("Dazzling Sphinx");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let sphinx = t.battlefield(P0, "Dazzling Sphinx");
        let div = t.library_top(P1, "Divination");
        t.answer_yes(P0, yes);
        attack_with(&mut t, &[(sphinx, Entity::Player(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        t.resolve();
        if yes {
            // Cast during the resolution: on the stack now, in combat.
            assert_eq!(t.zone(div), Zone::Stack, "{}", t.dump_log());
            t.resolve_all();
            assert_eq!(t.hand_size(P0), 2);
        } else {
            assert_eq!(t.zone(div), Zone::Library(P1), "{}", t.dump_log());
            t.set_step(P0, Step::PostcombatMain);
            mana(&mut t, P0, ManaType::U, 3);
            assert!(!castable(&mut t, P0, div));
        }
    }
}

#[test]
fn fallen_shinobi_and_haldan_dont_change_when_you_can_play_the_cards() {
    cr!("601.3", "305.2", "305.3", "307.1");
    ruling!(
        "Fallen Shinobi",
        "Fallen Shinobi's triggered ability doesn't change when you can play the exiled cards. For example, if a sorcery card is exiled, you can cast it only during your main phase when the stack is empty. If a land card is exiled, you can play it only during your main phase and only if you have an available land play remaining."
    );
    ruling!(
        "Haldan, Avid Arcanist",
        "Haldan doesn't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    supported("Fallen Shinobi");
    supported("Haldan, Avid Arcanist");
    // Fallen Shinobi: P1 exiles Divination and Forest.
    let mut t = TestGame::new(2);
    let shinobi = t.battlefield(P0, "Fallen Shinobi");
    let cards = stack_library(&mut t, P1, &["Divination", "Forest"]);
    attack_with(&mut t, &[(shinobi, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert!(exiled(&t, cards[0]) && exiled(&t, cards[1]), "{}", t.dump_log());
    assert!(!castable(&mut t, P0, cards[0]), "end of combat");
    assert!(!playable_land(&mut t, P0, cards[1]), "end of combat");
    sorcery_timing_only(&mut t, P0, cards[0], ManaType::C, 0);
    assert!(playable_land(&mut t, P0, cards[1]));
    // With the land play used: no.
    let mtn = t.hand(P0, "Mountain");
    t.play_land(P0, mtn).unwrap();
    assert!(!playable_land(&mut t, P0, cards[1]));
    // Haldan: a sorcery and a land card P0 exiled with fetch counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Haldan, Avid Arcanist");
    let div = t.library_top(P1, "Divination");
    let div = exile_by(&mut t, div, P0, "fetch", 1);
    let forest = t.library_top(P1, "Forest");
    let forest = exile_by(&mut t, forest, P0, "fetch", 1);
    sorcery_timing_only(&mut t, P0, div, ManaType::R, 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable_land(&mut t, P0, forest), "combat");
    t.set_step(P0, Step::PrecombatMain);
    assert!(playable_land(&mut t, P0, forest));
    let mtn = t.hand(P0, "Mountain");
    t.play_land(P0, mtn).unwrap();
    assert!(!playable_land(&mut t, P0, forest));
}

#[test]
fn goliath_daydreamer_casts_during_its_attack_trigger_ignoring_type_timing_only() {
    cr!("608.2g", "307.1");
    ruling!(
        "Goliath Daydreamer",
        "Goliath Daydreamer's last ability allows you to cast an exiled card with a dream counter on it only while the ability is resolving and still on the stack. You can't wait to cast the spell later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Goliath Daydreamer");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Goliath Daydreamer");
        let div = t.library_top(P0, "Divination");
        let div = exile_by(&mut t, div, P0, "dream", 1);
        t.answer_choose_moved(P0, if yes { vec![o(div)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        attack_with(&mut t, &[(g, Entity::Player(P1))]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        if yes {
            // A sorcery cast in the declare attackers step.
            assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
        } else {
            assert!(exiled(&t, div));
            block_and_finish(&mut t, P1, &[]);
            t.advance_to(P0, Step::PostcombatMain);
            mana(&mut t, P0, ManaType::U, 3);
            assert!(!castable(&mut t, P0, div));
        }
    }
}

#[test]
fn goliath_daydreamer_doesnt_exile_a_countered_spell() {
    cr!("608.2n", "701.6a");
    ruling!(
        "Goliath Daydreamer",
        "If a spell is countered or otherwise fails to resolve, Goliath Daydreamer's first ability won't exile it."
    );
    for counter in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Goliath Daydreamer");
        let bolt = t.hand(P0, "Lightning Bolt");
        mana(&mut t, P0, ManaType::R, 1);
        t.cast(P0, bolt).target(P1).go();
        t.resolve();
        assert_eq!(t.stack_len(), 1, "the trigger resolved");
        if counter {
            let c = t.hand(P1, "Counterspell");
            mana(&mut t, P1, ManaType::U, 2);
            t.g.turn.priority = Some(P1);
            let b = t.g.current(bolt);
            t.cast(P1, c).target(b).go();
        }
        t.resolve_all();
        if counter {
            assert!(t.in_graveyard(P0, "Lightning Bolt"));
            assert_eq!(t.life(P1), 20);
        } else {
            assert!(exiled(&t, bolt));
            assert_eq!(t.counters(bolt, "dream"), 1);
            assert_eq!(t.life(P1), 17);
        }
    }
}

#[test]
fn heartless_conscription_land_creature_cards_are_played_as_lands() {
    cr!("305.2", "305.3", "305.9");
    ruling!(
        "Heartless Conscription",
        "If any of the cards are land cards (perhaps because they were lands that had become creatures), you can play them from exile, following the normal timing rules for playing lands."
    );
    supported("Heartless Conscription");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let hc = t.hand(P0, "Heartless Conscription");
    mana(&mut t, P0, ManaType::B, 8);
    t.cast(P0, hc).go();
    t.resolve_all();
    assert!(exiled(&t, arbor));
    assert!(!castable(&mut t, P0, arbor), "a land can't be cast");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable_land(&mut t, P0, arbor));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!playable_land(&mut t, P0, arbor));
    t.set_step(P0, Step::PostcombatMain);
    assert!(playable_land(&mut t, P0, arbor));
    t.play_land(P0, t.g.current(arbor)).unwrap();
    assert!(t.on_battlefield(arbor));
    assert_eq!(t.obj_now(arbor).controller, P0);
}

/// P0 casts Hazoret's Undying Fury with a library of exactly `lib` (top first); `picks`
/// are the cards cast during its resolution, in order; `targets` answers their targets.
/// Returns the library cards and the spell.
fn hazoret(t: &mut TestGame, lib: &[&str], picks: &[usize], targets: &[Entity]) -> (Vec<ObjectId>, ObjectId) {
    supported("Hazoret's Undying Fury");
    t.g.players[0].library.clear();
    let cards = stack_library(t, P0, lib);
    let h = t.hand(P0, "Hazoret's Undying Fury");
    mana(t, P0, ManaType::R, 6);
    t.answer_choose_moved(P0, &picks.iter().map(|i| o(cards[*i])).collect::<Vec<_>>());
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    let spell = t.cast(P0, h).go();
    while t.zone(spell) == Zone::Stack {
        t.resolve();
    }
    (cards, spell)
}

#[test]
fn hazorets_undying_fury_each_spell_has_mana_value_5_or_less_not_the_total() {
    cr!("608.2g", "202.3");
    ruling!(
        "Hazoret's Undying Fury",
        "Each individual spell you cast this way must have mana value 5 or less. Their total mana value could be greater."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let (cards, _) = hazoret(
        &mut t,
        &["Hill Giant", "Serra Angel", "Shivan Dragon", "Mesa Unicorn"],
        &[0, 1],
        &[],
    );
    assert!(!was_offered(&t, from, cards[2]), "Shivan Dragon has mana value 6");
    t.resolve_all();
    assert!(t.on_battlefield(cards[0]) && t.on_battlefield(cards[1]), "{}", t.dump_log());
    assert!(exiled(&t, cards[2]) && exiled(&t, cards[3]));
}

#[test]
fn hazorets_undying_fury_spells_can_target_earlier_ones_or_it_but_it_leaves_soon() {
    cr!("608.2g", "608.2n", "608.2b");
    ruling!(
        "Hazoret's Undying Fury",
        "If a spell you cast this way targets another spell, it may target a spell you cast earlier during the resolution of Hazoret's Undying Fury. It may target Hazoret's Undying Fury as well, but Hazoret's Undying Fury will be put into your graveyard soon afterwards and the spell's target will become illegal."
    );
    // Lightning Bolt, then Twincast copying it.
    let mut t = TestGame::new(2);
    let lib = ["Lightning Bolt", "Twincast", "Hill Giant", "Hill Giant"];
    target_named_when_offered(&mut t, P0, "Lightning Bolt");
    let mut t2 = TestGame::new(2);
    let (cards, _) = hazoret(&mut t, &lib, &[0, 1], &[Entity::Player(P1)]);
    let bolt = t.g.current(cards[0]);
    let twin = t.g.current(cards[1]);
    assert_eq!(t.g.stack, vec![bolt, twin], "{}", t.dump_log());
    assert_eq!(crate::r_s25_common::targets_of(&t, twin), vec![o(bolt)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // Cancel targeting Hazoret's Undying Fury: the Fury goes to the graveyard as it
    // finishes resolving, and Cancel's only target is then illegal.
    let (cards, fury) = hazoret(&mut t2, &["Cancel", "Hill Giant", "Hill Giant", "Hill Giant"], &[0], &[]);
    let cancel = t2.g.current(cards[0]);
    assert_eq!(crate::r_s25_common::targets_of(&t2, cancel), vec![o(fury)]);
    assert!(t2.in_graveyard(P0, "Hazoret's Undying Fury"));
    t2.resolve_all();
    assert!(t2.in_graveyard(P0, "Cancel"));
    assert!(!crate::r_s07_common::resolved(&t2, cancel));
}

#[test]
fn hazorets_undying_fury_triggers_wait_until_casting_is_done_and_resolve_first() {
    cr!("603.3", "608.2g", "405.5");
    ruling!(
        "Hazoret's Undying Fury",
        "If any abilities trigger as you cast any of those cards, they won't be put on the stack until after you're done casting them. They'll resolve before any of those spells."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let (cards, _) = hazoret(
        &mut t,
        &["Lightning Bolt", "Lava Axe", "Hill Giant", "Hill Giant"],
        &[0, 1],
        &[Entity::Player(P1), Entity::Player(P1)],
    );
    // Both spells were cast before either trigger was put on the stack.
    let bolt = t.g.current(cards[0]);
    let axe = t.g.current(cards[1]);
    assert_eq!(&t.g.stack[..2], &[bolt, axe], "{}", t.dump_log());
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, ""), 2);
    // (One Elemental from casting the Fury itself.)
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    t.resolve();
    t.resolve();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 3);
    assert_eq!(t.g.stack, vec![bolt, axe]);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "{}", t.dump_log());
}

#[test]
fn two_hazorets_undying_fury_untap_effects_both_wear_off_in_the_next_untap_step() {
    cr!("502.3");
    ruling!(
        "Hazoret's Undying Fury",
        "If more than one spell says that lands you control don't untap during your next untap step, the effects will all wear off during that untap step. You'll untap lands you control during your untap step after that one."
    );
    supported("Hazoret's Undying Fury");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 2);
    for l in &lands {
        t.g.obj_mut(*l).tapped = true;
    }
    for _ in 0..2 {
        let h = t.hand(P0, "Hazoret's Undying Fury");
        mana(&mut t, P0, ManaType::R, 6);
        t.answer_choose_moved(P0, &[]);
        t.cast(P0, h).go();
        t.resolve_all();
    }
    t.advance_to(P0, Step::Upkeep);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped), "first untap step");
    for l in &lands {
        t.g.obj_mut(*l).tapped = true;
    }
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped), "the one after {}", t.dump_log());
}

#[test]
fn emergent_ultimatum_spells_can_target_it_until_its_exiled() {
    cr!("608.2g", "608.2b");
    ruling!(
        "Emergent Ultimatum",
        "Emergent Ultimatum is still on the stack while you cast the cards you found, so those spells could target it. However, once it’s exiled, it will no longer be a legal target."
    );
    supported("Emergent Ultimatum");
    let mut t = TestGame::new(2);
    let cancel = t.library_top(P0, "Cancel");
    let bears = t.library_top(P0, "Grizzly Bears");
    let eu = t.hand(P0, "Emergent Ultimatum");
    for (ty, n) in [(ManaType::B, 2), (ManaType::G, 3), (ManaType::U, 2)] {
        mana(&mut t, P0, ty, n);
    }
    t.answer_choose_moved(P0, &[o(cancel), o(bears)]);
    // P1 shuffles Grizzly Bears back; P0 casts Cancel targeting Emergent Ultimatum.
    t.answer_choose_moved(P1, &[o(bears)]);
    t.answer_choose_moved(P0, &[o(cancel)]);
    t.answer_yes(P0, true);
    let spell = t.cast(P0, eu).target(P1).go();
    t.resolve();
    let c = t.g.current(cancel);
    assert_eq!(t.zone(cancel), Zone::Stack, "{}", t.dump_log());
    assert_eq!(crate::r_s25_common::targets_of(&t, c), vec![o(spell)]);
    assert!(exiled(&t, spell));
    t.resolve_all();
    assert!(!crate::r_s07_common::resolved(&t, c));
    assert!(t.in_graveyard(P0, "Cancel"));
}
