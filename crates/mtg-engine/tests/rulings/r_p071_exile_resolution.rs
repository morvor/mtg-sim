//! Rulings batch P071 — permissions to play exiled cards "this turn", and casting exiled
//! cards while the effect that exiled them resolves (CR 608.2g): costs (CR 601.2b,
//! 601.2f–h, 118.9), timing (CR 117.1a, 307.1, 305.2, 305.3), and choices made on
//! resolution (CR 608.2d).

use crate::r_p071_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::kwa::manifest;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Runs the game to the beginning of P0's next upkeep, putting `top` on top of P1's
/// library after P1's draw step. Returns that card.
fn to_p0_upkeep_with_p1_top(t: &mut TestGame, top: &str) -> ObjectId {
    t.advance_to(P1, Step::End);
    let card = t.library_top(P1, top);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    card
}

// ---------------------------------------------------------------------------
// Costs
// ---------------------------------------------------------------------------

#[test]
fn ornate_kanzashi_cards_are_paid_for() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Ornate Kanzashi",
        "You must pay all costs to play cards exiled by Ornate Kanzashi."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Ornate Kanzashi");
    let div = t.library_top(P1, "Divination");
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, k, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    must_pay(&mut t, P0, div, ManaType::U, 3);
    mana(&mut t, P0, ManaType::R, 3);
    assert!(!castable(&mut t, P0, div), "red mana");
}

#[test]
fn evolving_door_card_is_paid_for_as_it_resolves() {
    cr!("608.2g", "601.2f", "601.2h");
    ruling!(
        "Evolving Door",
        "You must pay all costs for a spell you cast this way."
    );
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let door = t.battlefield(P0, "Evolving Door");
        let thopter = t.battlefield(P0, "Ornithopter");
        t.lands(P0, "Wastes", 1);
        let bears = t.library_top(P0, "Grizzly Bears");
        t.answer_choose(P0, &[o(thopter)]);
        t.answer_choose_moved(P0, &[o(bears)]);
        t.answer_yes(P0, true);
        t.activate(P0, door, 0, &[]).unwrap();
        if pay {
            mana(&mut t, P0, ManaType::G, 2);
        }
        t.resolve_all();
        if pay {
            assert!(t.on_battlefield(bears), "{}", t.dump_log());
            assert_eq!(t.g.players[0].mana_pool.total(), 0);
        } else {
            assert!(exiled(&t, bears), "{}", t.dump_log());
            // And the permission ended with the resolution.
            mana(&mut t, P0, ManaType::G, 2);
            assert!(!castable(&mut t, P0, bears));
        }
    }
}

#[test]
fn urianger_cards_are_paid_for_less_two_with_normal_timing() {
    cr!("601.2f", "302.1");
    ruling!(
        "Urianger Augurelt",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Urianger Augurelt's last ability."
    );
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urianger Augurelt");
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.activate(P0, u, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, giant), "{}", t.dump_log());
    let u = t.g.current(u);
    t.g.objects[u.0 as usize].tapped = false;
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    // {3}{R} less {2}.
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, giant), "one mana");
    must_pay(&mut t, P0, giant, ManaType::R, 2);
    sorcery_timing_this_turn(&mut t, P0, giant, ManaType::R, 2);
}

#[test]
fn ryan_sinclair_free_cast_pays_kicker_and_mandatory_additional_costs() {
    cr!("118.9a", "601.2b", "601.2f", "702.33a");
    ruling!(
        "Ryan Sinclair",
        "You may pay optional additional costs to cast that spell, such as kicker costs. If it has any mandatory additional costs, you must pay those."
    );
    // Burst Lightning (mana value 1), kicked for {4}.
    let mut t = TestGame::new(2);
    let ryan = t.battlefield(P0, "Ryan Sinclair");
    t.lands(P0, "Mountain", 4);
    let burst = t.library_top(P0, "Burst Lightning");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(ryan, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Burst Lightning"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 16);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    let _ = burst;
    // Village Rites: a creature is sacrificed (Grizzly Bears).
    let mut t = TestGame::new(2);
    let ryan = t.battlefield(P0, "Ryan Sinclair");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Village Rites");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(ryan, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Village Rites"));
    assert_eq!(t.hand_size(P0), hand + 2);
}

// ---------------------------------------------------------------------------
// "This turn": timing
// ---------------------------------------------------------------------------

#[test]
fn fire_lord_ozai_creature_waits_for_your_main_phase_unless_it_has_flash() {
    cr!("302.1", "702.8a", "601.3");
    ruling!(
        "Fire Lord Ozai",
        "You must follow normal time restrictions for cards cast using Fire Lord Ozai's activated ability. For example, if you exile a creature card, it can only be cast in your main phase when the stack is empty. Notably, you usually cannot cast a creature card exiled with this ability on your opponent's turn unless it has flash."
    );
    // In P1's turn: Grizzly Bears can't be cast; Ambush Viper (flash) can.
    for (name, ok) in [("Grizzly Bears", false), ("Ambush Viper", true)] {
        let mut t = TestGame::new(2);
        let ozai = t.battlefield(P0, "Fire Lord Ozai");
        let card = t.library_top(P1, name);
        t.set_step(P1, Step::PrecombatMain);
        t.g.turn.priority = Some(P0);
        mana(&mut t, P0, ManaType::C, 6);
        t.activate(P0, ozai, 0, &[]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, card), "{name}: {}", t.dump_log());
        assert_eq!(castable(&mut t, P0, card), ok, "{name}");
    }
    // In P0's turn: only in a main phase with an empty stack, without paying.
    let mut t = TestGame::new(2);
    let ozai = t.battlefield(P0, "Fire Lord Ozai");
    let bears = t.library_top(P1, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 6);
    t.activate(P0, ozai, 0, &[]).unwrap();
    t.resolve_all();
    sorcery_timing_this_turn(&mut t, P0, bears, ManaType::C, 0);
}

#[test]
fn primordial_mist_card_is_played_with_normal_timing_and_costs() {
    cr!("305.2", "307.1", "601.2f");
    ruling!(
        "Primordial Mist",
        "You must follow the normal timing permissions and restrictions of each card you play."
    );
    for name in ["Divination", "Forest"] {
        let mut t = TestGame::new(2);
        let mist = t.battlefield(P0, "Primordial Mist");
        let card = t.library_top(P0, name);
        let fd = manifest::put_face_down(&mut t.g, card, P0, manifest::MANIFESTED, Some(mist))
            .expect("manifested");
        t.answer_choose(P0, &[o(fd)]);
        t.activate(P0, mist, 0, &[]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, fd), "{name}: {}", t.dump_log());
        assert!(!t.obj_now(fd).face_down);
        if name == "Forest" {
            land_timing_this_turn(&mut t, P0, fd);
        } else {
            t.set_step(P0, Step::PostcombatMain);
            must_pay(&mut t, P0, fd, ManaType::U, 3);
            sorcery_timing_this_turn(&mut t, P0, fd, ManaType::U, 3);
        }
    }
}

#[test]
fn quintorius_card_is_cast_with_sorcery_timing() {
    cr!("307.1", "601.3");
    ruling!(
        "Quintorius, Loremaster",
        "You must follow all applicable timing rules for casting the exiled card. For example, if the card is a sorcery card, you can cast it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quintorius, Loremaster");
    let div = t.graveyard(P0, "Divination");
    t.answer_targets(P0, &[o(div)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    let spirit = with_subtype(&t, P0, "Spirit")[0];
    mana(&mut t, P0, ManaType::R, 2);
    mana(&mut t, P0, ManaType::W, 1);
    t.answer_choose(P0, &[o(spirit)]);
    t.activate(P0, q, 0, &[o(div)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(spirit), "{}", t.dump_log());
    assert!(!castable(&mut t, P0, div), "end step");
    sorcery_timing_this_turn(&mut t, P0, div, ManaType::C, 0);
}

#[test]
fn the_temporal_anchor_cards_follow_normal_timing_and_are_paid_for() {
    cr!("305.2", "307.1", "601.2f");
    ruling!(
        "The Temporal Anchor",
        "You must follow all normal timing rules for a card you play using The Temporal Anchor's last ability and, if it's a spell, you must pay its costs to cast it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Temporal Anchor");
    let div = t.library_top(P0, "Divination");
    let forest = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![forest, div]));
    mtg_engine::library::scry(&mut t.g, P0, 2);
    t.resolve_all();
    assert!(exiled(&t, div) && exiled(&t, forest), "{}", t.dump_log());
    land_timing(&mut t, P0, forest);
    must_pay(&mut t, P0, div, ManaType::U, 3);
    sorcery_timing_only(&mut t, P0, div, ManaType::U, 3);
}

#[test]
fn the_ruinous_powers_card_follows_normal_timing() {
    cr!("305.2", "307.1");
    ruling!(
        "The Ruinous Powers",
        "You must follow all normal timing rules when casting a spell or playing a land using the permission granted by Ruinous Powers."
    );
    for name in ["Divination", "Forest"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Ruinous Powers");
        let card = to_p0_upkeep_with_p1_top(&mut t, name);
        assert!(exiled(&t, card), "{name}: {}", t.dump_log());
        if name == "Forest" {
            assert!(!playable_land(&mut t, P0, card), "upkeep");
            land_timing_this_turn(&mut t, P0, card);
        } else {
            mana(&mut t, P0, ManaType::R, 3);
            assert!(!castable(&mut t, P0, card), "upkeep");
            sorcery_timing_this_turn(&mut t, P0, card, ManaType::R, 3);
        }
    }
}

#[test]
fn laughing_jasper_flint_creature_waits_for_your_main_phase() {
    cr!("302.1", "601.3");
    ruling!(
        "Laughing Jasper Flint",
        "You pay all costs and follow all normal timing rules for cards played from exile with Laughing Jasper Flint’s last ability. For example, if you exile a creature card this way, you must wait until your main phase to cast it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laughing Jasper Flint");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bears = to_p0_upkeep_with_p1_top(&mut t, "Grizzly Bears");
    assert!(exiled(&t, bears), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::B, 2);
    assert!(!castable(&mut t, P0, bears), "upkeep");
    t.set_step(P0, Step::PrecombatMain);
    must_pay(&mut t, P0, bears, ManaType::B, 2);
    sorcery_timing_this_turn(&mut t, P0, bears, ManaType::B, 2);
}

#[test]
fn ugin_and_stolen_goods_free_cards_still_follow_timing() {
    cr!("302.1", "307.1", "601.3");
    ruling!(
        "Ugin, Eye of the Storms",
        "You must follow the normal timing permissions and restrictions of each spell you cast using the permission granted by Ugin’s last ability."
    );
    ruling!(
        "Stolen Goods",
        "You must follow the normal timing restrictions of the exiled card."
    );
    // Ugin's −11 finds Steel Overseer, a colorless creature card.
    let mut t = TestGame::new(2);
    let ugin = t.battlefield(P0, "Ugin, Eye of the Storms");
    t.g.objects[ugin.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 11);
    let so = t.library_top(P0, "Steel Overseer");
    t.answer_choose_moved(P0, &[o(so)]);
    t.activate(P0, ugin, 2, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, so), "{}", t.dump_log());
    sorcery_timing_this_turn(&mut t, P0, so, ManaType::C, 0);
    // Stolen Goods: Divination.
    let mut t = TestGame::new(2);
    let div = t.library_top(P1, "Divination");
    let sg = t.hand(P0, "Stolen Goods");
    mana(&mut t, P0, ManaType::U, 4);
    t.cast(P0, sg).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    sorcery_timing_this_turn(&mut t, P0, div, ManaType::C, 0);
}

#[test]
fn norin_creature_is_cast_again_only_in_a_main_phase() {
    cr!("302.1", "601.3");
    ruling!(
        "Norin, Swift Survivalist",
        "You pay all costs and follow all timing rules for cards played this way. For example, if a card exiled with Norin's last ability is a creature card without flash, you may cast that creature spell only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Norin, Swift Survivalist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(wall, bears)]);
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears), "end of combat");
    t.set_step(P0, Step::PostcombatMain);
    must_pay(&mut t, P0, bears, ManaType::G, 2);
    sorcery_timing_this_turn(&mut t, P0, bears, ManaType::G, 2);
}

// ---------------------------------------------------------------------------
// Cast while the ability resolves
// ---------------------------------------------------------------------------

#[test]
fn izzet_chemister_cards_are_cast_only_as_its_ability_resolves() {
    cr!("608.2g", "307.1");
    ruling!(
        "Izzet Chemister",
        "You must cast any of the exiled cards you wish to cast while Izzet Chemister's last ability is resolving. You can't cast them later in the turn. A sorcery cast this way will be cast at a time you normally wouldn't be able to."
    );
    let mut t = TestGame::new(2);
    let ic = t.battlefield(P0, "Izzet Chemister");
    let div = t.graveyard(P0, "Divination");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    for card in [div, bolt] {
        let ic = t.g.current(ic);
        t.g.objects[ic.0 as usize].tapped = false;
        mana(&mut t, P0, ManaType::R, 1);
        t.activate(P0, ic, 0, &[o(card)]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, card), "{}", t.dump_log());
    }
    // In P1's end step: Divination is cast (a sorcery, at instant speed); Lightning Bolt
    // isn't.
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    let ic = t.g.current(ic);
    t.g.objects[ic.0 as usize].tapped = false;
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose_moved(P0, &[o(div)]);
    let hand = t.hand_size(P0);
    t.activate(P0, ic, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(exiled(&t, bolt));
    // Later, in P0's turn, Lightning Bolt can't be cast from exile.
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, bolt));
}

#[test]
fn gix_plays_cards_as_it_resolves_and_lands_only_with_a_land_play_in_your_turn() {
    cr!("608.2g", "305.2", "305.2b", "305.3");
    ruling!(
        "Gix, Yawgmoth Praetor",
        "You may play a land this way only during your own turn and only if you have not yet played a land this turn."
    );
    ruling!(
        "Gix, Yawgmoth Praetor",
        "You must play the cards as you resolve the last ability. You can't wait and play them later."
    );
    // `when`: 0 = P0's turn with a land play, 1 = P0's turn with the land play used,
    // 2 = P1's turn.
    for when in 0..3 {
        let mut t = TestGame::new(2);
        let gix = t.battlefield(P0, "Gix, Yawgmoth Praetor");
        let lib = stack_library(&mut t, P1, &["Forest", "Divination"]);
        let d1 = t.hand(P0, "Hill Giant");
        let d2 = t.hand(P0, "Hill Giant");
        t.lands(P0, "Swamp", 7);
        match when {
            1 => {
                let w = t.hand(P0, "Wastes");
                t.play_land(P0, w).unwrap();
            }
            2 => {
                t.set_step(P1, Step::PrecombatMain);
                t.g.turn.priority = Some(P0);
            }
            _ => {}
        }
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.answer_choose(P0, &[o(d1), o(d2)]);
        t.activate(P0, gix, 0, &[Entity::Player(P1)]).unwrap();
        t.answer_choose_moved(P0, &[o(lib[0]), o(lib[1])]);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        // Divination was cast either way.
        assert!(t.in_graveyard(P1, "Divination"), "{when}: {}", t.dump_log());
        if when == 0 {
            assert!(t.on_battlefield(lib[0]), "{when}: {}", t.dump_log());
        } else {
            assert!(exiled(&t, lib[0]), "{when}: {}", t.dump_log());
            // And it can't be played later, in P0's next main phase with a land play.
            t.advance_to(P0, Step::PrecombatMain);
            assert!(!playable_land(&mut t, P0, lib[0]), "{when}");
        }
    }
}

#[test]
fn etrata_exiles_a_creature_that_cant_be_turned_face_up_and_casts_it() {
    cr!("701.58g", "608.2g");
    ruling!(
        "Etrata, Deadly Fugitive",
        "You might be unable to turn a face-down creature face up because it's an instant or sorcery. Alternatively, abilities such as that of Karlov Watchdog might prevent you from turning face-down creatures face up altogether. In those cases, you'll exile that creature, and then you'll choose whether or not to cast that card without paying its mana cost."
    );
    // A cloaked Lightning Bolt: exiled and cast for free.
    let mut t = TestGame::new(2);
    let etrata = t.battlefield(P0, "Etrata, Deadly Fugitive");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let fd = manifest::put_face_down(&mut t.g, bolt, P0, manifest::CLOAKED, Some(etrata))
        .expect("cloaked");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::B, 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, fd, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
    // A cloaked Grizzly Bears in P1's turn with P1's Karlov Watchdog: exiled and cast (at
    // instant speed, during the resolution).
    let mut t = TestGame::new(2);
    let etrata = t.battlefield(P0, "Etrata, Deadly Fugitive");
    t.battlefield(P1, "Karlov Watchdog");
    let bears = t.library_top(P0, "Grizzly Bears");
    let fd = manifest::put_face_down(&mut t.g, bears, P0, manifest::CLOAKED, Some(etrata))
        .expect("cloaked");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::B, 3);
    t.answer_yes(P0, true);
    t.activate(P0, fd, 0, &[]).unwrap();
    t.resolve_all();
    let now = t.g.current(fd);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert!(!t.g.obj(now).face_down);
    assert_eq!(t.g.obj(now).chars.name, "Grizzly Bears");
}

/// P0's Eye of the Storm or Torrential Gearhulk (`how`) gives P0 a chance to cast Panic
/// ("Cast this spell only during combat before blockers are declared.") during a
/// resolution in `step`. Returns whether P1's Grizzly Bears can't block (Panic was cast).
fn panic_cast_during_resolution(how: &str, step: Step) -> bool {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, step);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(bears)]);
    if how == "Eye of the Storm" {
        // Panic is already exiled with Eye of the Storm (cast in an earlier combat); now P0
        // casts Opt (an instant): both are copied.
        let eye = t.battlefield(P0, "Eye of the Storm");
        let panic = t.hand(P0, "Panic");
        let saved = t.g.turn.step;
        t.set_step(P0, Step::BeginningOfCombat);
        mana(&mut t, P0, ManaType::R, 1);
        t.clear_answers();
        t.answer_yes(P0, false);
        t.cast(P0, panic).target(bears).go();
        t.resolve_all();
        assert!(exiled(&t, panic), "{}", t.dump_log());
        let _ = eye;
        t.clear_answers();
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[o(bears)]);
        t.set_step(P0, saved);
        let opt = t.hand(P0, "Opt");
        mana(&mut t, P0, ManaType::U, 1);
        t.cast(P0, opt).go();
        t.resolve_all();
    } else {
        let panic = t.graveyard(P0, "Panic");
        t.answer_targets(P0, &[o(panic)]);
        t.enter(P0, "Torrential Gearhulk");
        t.resolve_all();
    }
    !crate::r_p123_common::can_block(&mut t, bears)
}

#[test]
fn eye_of_the_storm_and_torrential_gearhulk_casts_keep_cast_only_restrictions() {
    cr!("608.2g", "601.3", "506.8");
    ruling!(
        "Eye of the Storm",
        "You must still follow any restrictions on when you can cast the spell, such as “cast only during combat” or “cast only on your own turn.”"
    );
    ruling!(
        "Torrential Gearhulk",
        "If you cast the card, you do so as part of the resolution of Torrential Gearhulk's triggered ability. You can't wait to cast it later in the turn. Timing restrictions (such as \"Cast [this card] only during combat\") still apply."
    );
    for how in ["Eye of the Storm", "Torrential Gearhulk"] {
        assert!(
            panic_cast_during_resolution(how, Step::BeginningOfCombat),
            "{how}: in combat"
        );
        assert!(
            !panic_cast_during_resolution(how, Step::PostcombatMain),
            "{how}: main phase"
        );
    }
}

// ---------------------------------------------------------------------------
// Choices on resolution
// ---------------------------------------------------------------------------

#[test]
fn legend_of_yangchen_targets_on_the_stack_and_asks_on_resolution() {
    cr!("601.2c", "603.3d", "608.2d", "714.2b");
    ruling!(
        "The Legend of Yangchen // Avatar Yangchen",
        "You must choose a target opponent for The Legend of Yangchen's second chapter ability when it is put on the stack. You will not choose whether or not that opponent will draw cards until the ability resolves."
    );
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let saga = t.battlefield(P0, "The Legend of Yangchen // Avatar Yangchen");
        t.g.objects[saga.0 as usize]
            .counters
            .insert(counters::LORE.into(), 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let from = t.asked().len();
        t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        let asked = t.asked()[from..].to_vec();
        assert!(asked
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
        assert!(!asked.iter().any(|(_, d)| matches!(d, Decision::YesNo { .. })));
        let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
        let from = t.asked().len();
        t.answer_yes(P0, yes);
        t.resolve();
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        let n = if yes { 3 } else { 0 };
        assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + n, h1 + n), "{yes}");
    }
}

#[test]
fn jace_architect_chooses_an_opponent_on_resolution_without_targeting_and_picks_the_pile() {
    cr!("115.1", "608.2d", "700.3");
    ruling!(
        "Jace, Architect of Thought",
        "You pick one of your opponents when Jace's second ability resolves. The ability doesn't target that opponent. All players may see the revealed cards and offer opinions. You (not your opponent) choose which pile is put into your hand and which pile is put on the bottom of your library."
    );
    // P2 has hexproof (Leyline of Sanctity): still a legal choice.
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    t.battlefield(P2, "Leyline of Sanctity");
    let lib = stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    let from = t.asked().len();
    t.activate(P0, jace, 1, &[]).unwrap();
    let ab = *t.g.stack.last().unwrap();
    assert!(t.g.obj(ab)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .all(|c| c.targets.iter().all(|x| x.is_empty())));
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P2, &[o(lib[0])]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.resolve();
    let asked = t.asked()[from..].to_vec();
    // P0 chose the opponent, P2 separated the piles, P0 chose the pile.
    assert!(matches!(&asked[0], (P0, Decision::ChooseEntities { candidates, .. })
        if candidates.contains(&Entity::Player(P2))));
    assert!(asked.iter().any(|(p, d)| *p == P2
        && matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("piles"))));
    assert!(asked.iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("pile"))));
    assert!(!asked.iter().any(|(p, d)| *p == P2
        && matches!(d, Decision::ChooseOption { prompt, .. } if prompt.contains("pile"))));
    assert_eq!(t.zone(lib[1]), Zone::Hand(P0), "{}", t.dump_log());
    assert_eq!(t.zone(lib[2]), Zone::Hand(P0));
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library[0], t.g.current(lib[0]), "on the bottom");
}
