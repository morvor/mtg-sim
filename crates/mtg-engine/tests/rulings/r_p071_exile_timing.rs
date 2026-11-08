//! Rulings batch P071 — a permission to play a card from exile doesn't change when it can
//! be played, nor its costs: a creature or sorcery card only in its controller's main
//! phase with an empty stack (CR 117.1a, 307.1, 601.3), a land only as a land play in its
//! controller's turn (CR 305.2, 305.3), the card's costs paid unless the permission says
//! otherwise (CR 601.2f–h).

use crate::r_p071_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// "For as long as it remains exiled": timing across turns
// ---------------------------------------------------------------------------

/// P0's creature connects with P1 while `name`'s ability exiles the top card of P1's
/// library (`top`). Returns that card.
fn exiled_by_combat_damage(t: &mut TestGame, name: &str, top: &str) -> ObjectId {
    let card = t.library_top(P1, top);
    let attacker = match name {
        "Dream-Thief's Bandana" => {
            let bears = t.battlefield(P0, "Grizzly Bears");
            let b = t.battlefield(P0, name);
            t.g.attach(b, Entity::Object(bears));
            bears
        }
        _ => t.battlefield(P0, name),
    };
    connect(t, &[attacker]);
    assert!(exiled(t, card), "{name}: {}", t.dump_log());
    card
}

#[test]
fn combat_damage_exiles_play_lands_only_as_land_plays() {
    cr!("305.2", "305.3", "116.2a");
    ruling!(
        "Gonti, Canny Acquisitor",
        "You pay all costs and follow all normal timing rules for cards played from exile with Gonti’s last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Dream-Thief's Bandana",
        "You pay all costs and follow all normal timing rules for cards played from exile with Dream-Thief's Bandana's first ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Predators' Hour",
        "You must pay all costs to play cards exiled this way, and you must follow all normal timing permissions and restrictions. For example, if you play a land this way, you may do so only during your main phase while the stack is empty and only if you haven't yet played a land (unless another effect allows you to play additional lands)."
    );
    for name in [
        "Gonti, Canny Acquisitor",
        "Dream-Thief's Bandana",
        "Predators' Hour",
    ] {
        let mut t = TestGame::new(2);
        let forest = if name == "Predators' Hour" {
            let bears = t.battlefield(P0, "Grizzly Bears");
            let forest = t.library_top(P1, "Forest");
            let ph = t.hand(P0, name);
            mana(&mut t, P0, ManaType::B, 2);
            t.cast(P0, ph).go();
            t.resolve_all();
            connect(&mut t, &[bears]);
            assert!(exiled(&t, forest), "{name}: {}", t.dump_log());
            forest
        } else {
            exiled_by_combat_damage(&mut t, name, "Forest")
        };
        assert!(t.obj_now(forest).face_down, "{name}");
        land_timing(&mut t, P0, forest);
        // And a spell exiled this way is paid for (any type of mana).
        let mut t = TestGame::new(2);
        let div = if name == "Predators' Hour" {
            let bears = t.battlefield(P0, "Grizzly Bears");
            let div = t.library_top(P1, "Divination");
            let ph = t.hand(P0, name);
            mana(&mut t, P0, ManaType::B, 2);
            t.cast(P0, ph).go();
            t.resolve_all();
            connect(&mut t, &[bears]);
            div
        } else {
            exiled_by_combat_damage(&mut t, name, "Divination")
        };
        t.set_step(P0, Step::PostcombatMain);
        must_pay(&mut t, P0, div, ManaType::B, 3);
        sorcery_timing_only(&mut t, P0, div, ManaType::B, 3);
    }
}

#[test]
fn rev_and_ramirez_cards_are_sorceries_paid_for_in_your_main_phase() {
    cr!("307.1", "601.2f", "601.2h");
    ruling!(
        "Rev, Tithe Extractor",
        "You pay all costs and follow all timing rules for spells cast with the permission granted by Rev's last ability. For example, if the exiled card is a sorcery, you may cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Ramirez DePietro, Pillager",
        "You must pay all costs and follow all normal timing rules for spells cast with Ramirez DePietro, Pillager's last ability."
    );
    for name in ["Rev, Tithe Extractor", "Ramirez DePietro, Pillager"] {
        let mut t = TestGame::new(2);
        let div = exiled_by_combat_damage(&mut t, name, "Divination");
        // (Rev's Treasure is used up.)
        for tr in with_subtype(&t, P0, "Treasure") {
            t.g.objects[tr.0 as usize].tapped = true;
        }
        // Neither lets mana be spent as though it were another color: {2}{U}.
        mana(&mut t, P0, ManaType::B, 3);
        t.set_step(P0, Step::PostcombatMain);
        assert!(!castable(&mut t, P0, div), "{name}: black mana");
        must_pay(&mut t, P0, div, ManaType::U, 3);
        sorcery_timing_only(&mut t, P0, div, ManaType::U, 3);
    }
}

#[test]
fn savvy_trader_land_is_a_land_play() {
    cr!("305.2", "305.3");
    ruling!(
        "Savvy Trader",
        "You pay all costs and follow all normal timing rules for the card played from exile with Savvy Trader’s first ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    t.answer_targets(P0, &[o(forest)]);
    t.enter(P0, "Savvy Trader");
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    land_timing(&mut t, P0, forest);
}

#[test]
fn savvy_trader_card_stays_playable_by_you_after_it_leaves_or_changes_control() {
    cr!("601.3", "109.5", "611.2a");
    ruling!(
        "Savvy Trader",
        "You may play the card exiled with Savvy Trader’s first ability even if Savvy Trader leaves the battlefield. If another player gains control of Savvy Trader, that player can’t play the card, and you still can."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let st = t.enter(P0, "Savvy Trader");
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    // P1 gains control of Savvy Trader (Control Magic): P1 can't cast the card, P0 still
    // can.
    t.set_step(P1, Step::PrecombatMain);
    let cm = t.hand(P1, "Control Magic");
    mana(&mut t, P1, ManaType::U, 4);
    t.cast(P1, cm).target(st).go();
    t.resolve_all();
    assert_eq!(t.obj_now(st).controller, P1, "{}", t.dump_log());
    mana(&mut t, P1, ManaType::G, 2);
    assert!(!castable(&mut t, P1, bears), "the new controller");
    t.set_step(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::G, 2);
    assert!(castable(&mut t, P0, bears), "{}", t.dump_log());
    // Savvy Trader leaves the battlefield: P0 still can.
    let st = t.g.current(st);
    t.g.destroy(st, None);
    t.settle();
    assert!(!t.on_battlefield(st));
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn siphon_insight_and_expensive_taste_cards_include_lands_on_normal_timing() {
    cr!("305.2", "305.3", "307.1");
    ruling!("Siphon Insight", "You may play the card even if it is a land.");
    ruling!(
        "Siphon Insight",
        "You must still follow all normal timing rules to play a card exiled with Siphon Insight."
    );
    ruling!(
        "Decadent Dragon // Expensive Taste",
        "You may play the cards exiled with Expensive Taste even if they are land cards."
    );
    ruling!(
        "Decadent Dragon // Expensive Taste",
        "You must still follow all normal timing rules to play cards exiled with Expensive Taste."
    );
    for name in ["Siphon Insight", "Decadent Dragon // Expensive Taste"] {
        let mut t = TestGame::new(2);
        let cards = stack_library(&mut t, P1, &["Forest", "Divination"]);
        let spell = t.hand(P0, name);
        mana(&mut t, P0, ManaType::B, 3);
        mana(&mut t, P0, ManaType::U, 1);
        if name == "Siphon Insight" {
            // Exile the Forest, put Divination on the bottom; then (another game) the
            // reverse.
            t.answer_choose(P0, &[o(cards[0])]);
            t.cast(P0, spell).target(P1).go();
        } else {
            t.cast(P0, spell)
                .method(mtg_engine::object::CastMethod::Half(1))
                .target(P1)
                .go();
        }
        t.resolve_all();
        assert!(exiled(&t, cards[0]), "{name}: {}", t.dump_log());
        land_timing(&mut t, P0, cards[0]);
        let div = if name == "Siphon Insight" {
            let mut t2 = TestGame::new(2);
            let cards = stack_library(&mut t2, P1, &["Forest", "Divination"]);
            let spell = t2.hand(P0, name);
            mana(&mut t2, P0, ManaType::B, 1);
            mana(&mut t2, P0, ManaType::U, 1);
            t2.answer_choose(P0, &[o(cards[1])]);
            t2.cast(P0, spell).target(P1).go();
            t2.resolve_all();
            assert!(exiled(&t2, cards[1]), "{}", t2.dump_log());
            sorcery_timing_only(&mut t2, P0, cards[1], ManaType::B, 3);
            continue;
        } else {
            cards[1]
        };
        assert!(exiled(&t, div));
        sorcery_timing_only(&mut t, P0, div, ManaType::U, 3);
    }
}

#[test]
fn intellect_devourer_and_kheru_mind_eater_lands_need_a_land_play() {
    cr!("305.2", "305.2b", "305.3");
    ruling!(
        "Intellect Devourer",
        "You must still follow all normal timing rules for cards played this way. For example, you may play a land exiled this way only during your main phase and only if you haven't played a land yet this turn."
    );
    ruling!(
        "Kheru Mind-Eater",
        "You may play land cards exiled with Kheru Mind-Eater only if you have an available land play this turn."
    );
    // Intellect Devourer: P1's only card in hand, a Forest, is exiled as it enters.
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    t.enter(P0, "Intellect Devourer");
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    land_timing(&mut t, P0, forest);
    // Kheru Mind-Eater: P1 exiles their only card in hand, a Forest.
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    let k = t.battlefield(P0, "Kheru Mind-Eater");
    connect(&mut t, &[k]);
    assert!(exiled(&t, forest), "{}", t.dump_log());
    land_timing(&mut t, P0, forest);
}

#[test]
fn tinybones_lands_follow_normal_timing() {
    cr!("305.2", "305.3");
    ruling!(
        "Tinybones, Bauble Burglar",
        "You pay all costs and follow all normal timing rules for cards played with the permission granted by Tinybones's second ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let tb = t.battlefield(P0, "Tinybones, Bauble Burglar");
    let forest = t.hand(P1, "Forest");
    mana(&mut t, P0, ManaType::B, 4);
    t.activate(P0, tb, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert_eq!(t.counters(forest, "stash"), 1);
    land_timing(&mut t, P0, forest);
}

#[test]
fn shadow_of_the_enemy_and_heartless_conscription_creatures_are_cast_normally() {
    cr!("302.1", "601.2f", "601.2h");
    ruling!(
        "Shadow of the Enemy",
        "You pay all costs and follow all normal timing rules for spells cast this way."
    );
    ruling!(
        "Heartless Conscription",
        "You pay all costs and follow all normal timing rules for cards played from exile this way. For example, if the exiled card is a creature card without flash, you may cast it only during your main phase while the stack is empty."
    );
    for name in ["Shadow of the Enemy", "Heartless Conscription"] {
        let mut t = TestGame::new(2);
        let giant = if name == "Shadow of the Enemy" {
            t.graveyard(P1, "Hill Giant")
        } else {
            t.battlefield(P1, "Hill Giant")
        };
        let s = t.hand(P0, name);
        mana(&mut t, P0, ManaType::B, 8);
        if name == "Shadow of the Enemy" {
            t.cast(P0, s).target(P1).go();
        } else {
            t.cast(P0, s).go();
        }
        t.resolve_all();
        assert!(exiled(&t, giant), "{name}: {}", t.dump_log());
        // Mana of any type, but all four of it.
        must_pay(&mut t, P0, giant, ManaType::B, 4);
        mana(&mut t, P0, ManaType::B, 3);
        assert!(!castable(&mut t, P0, giant), "{name}: three mana");
        sorcery_timing_only(&mut t, P0, giant, ManaType::B, 4);
    }
}

#[test]
fn thranduils_decree_card_is_cast_with_its_types_timing() {
    cr!("302.1", "601.3");
    ruling!(
        "Thranduil's Decree",
        "You must still follow any timing restrictions and permissions, including those based on the card's type."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    mana(&mut t, P1, ManaType::G, 2);
    let spell = t.cast(P1, bears).go();
    let d = t.hand(P0, "Thranduil's Decree");
    mana(&mut t, P0, ManaType::U, 6);
    t.g.turn.priority = Some(P0);
    t.cast(P0, d).target(spell).go();
    t.resolve();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    // P1's turn: not castable; P0's main phase with an empty stack: free.
    assert!(!castable(&mut t, P0, bears), "P1's turn");
    sorcery_timing_only(&mut t, P0, bears, ManaType::C, 0);
}

#[test]
fn durnan_card_pays_its_cost_reduced_by_undaunted_with_normal_timing() {
    cr!("702.125a", "601.2f", "302.1");
    ruling!(
        "Durnan of the Yawning Portal",
        "You must still pay all costs (as modified by the undaunted ability) and follow all normal timing rules when casting a spell this way."
    );
    // Three players: Hill Giant ({3}{R}) costs {1}{R} with two opponents.
    let mut t = TestGame::new(3);
    let giant = t.library_top(P0, "Hill Giant");
    let d = t.battlefield(P0, "Durnan of the Yawning Portal");
    t.answer_choose_moved(P0, &[o(giant)]);
    attack_with(&mut t, &[(d, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, giant), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::R, 2);
    assert!(!castable(&mut t, P0, giant), "combat");
    t.set_step(P0, Step::PostcombatMain);
    no_mana(&mut t, P0);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(!castable(&mut t, P0, giant), "one mana");
    must_pay(&mut t, P0, giant, ManaType::R, 2);
    sorcery_timing_only(&mut t, P0, giant, ManaType::R, 2);
    t.cast(P0, t.g.current(giant)).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn shared_fate_cards_need_the_right_colors_of_mana() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Shared Fate",
        "You need to pay the costs of any cards you play from the Exile zone. This could be a problem if you don't have the right colors of mana available."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shared Fate");
    let div = t.library_top(P1, "Divination");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(exiled(&t, div), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::R, 3);
    assert!(!castable(&mut t, P0, div), "no blue mana");
    no_mana(&mut t, P0);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, div));
}

#[test]
fn opposition_agent_and_breeches_cards_follow_normal_timing() {
    cr!("305.2", "305.3", "307.1");
    ruling!(
        "Opposition Agent",
        "You must follow the normal timing permissions and restrictions for each exiled card. If one is a land, you can't play it unless you have land plays available."
    );
    ruling!(
        "Breeches, Brazen Plunderer",
        "You must follow the normal timing permissions and restrictions for each exiled card. If one is a land, you can't play it unless you have land plays available."
    );
    // Opposition Agent: P1 searches with Rampant Growth and Demonic Tutor in their turn;
    // the cards found are exiled; P0 may play them, on P0's timing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    let forest = t.library_top(P1, "Forest");
    let div = t.library_top(P1, "Divination");
    t.set_step(P1, Step::PrecombatMain);
    let rg = t.hand(P1, "Rampant Growth");
    let dt = t.hand(P1, "Demonic Tutor");
    mana(&mut t, P1, ManaType::G, 2);
    t.answer_choose_moved(P0, &[o(forest)]);
    t.cast(P1, rg).go();
    t.resolve_all();
    mana(&mut t, P1, ManaType::B, 2);
    t.answer_choose_moved(P0, &[o(div)]);
    t.cast(P1, dt).go();
    t.resolve_all();
    assert!(exiled(&t, forest) && exiled(&t, div), "{}", t.dump_log());
    assert!(!playable_land(&mut t, P0, forest), "P1's turn");
    mana(&mut t, P0, ManaType::R, 3);
    assert!(!castable(&mut t, P0, div), "P1's turn");
    land_timing(&mut t, P0, forest);
    sorcery_timing_only(&mut t, P0, div, ManaType::R, 3);
    // Breeches: "this turn".
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Breeches, Brazen Plunderer");
    let forest = exiled_by_combat_damage_with(&mut t, b, "Forest");
    land_timing_this_turn(&mut t, P0, forest);
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Breeches, Brazen Plunderer");
    let div = exiled_by_combat_damage_with(&mut t, b, "Divination");
    sorcery_timing_this_turn(&mut t, P0, div, ManaType::R, 3);
}

/// `attacker` (P0's) connects with P1, whose library has `top` on top; returns that card,
/// asserted exiled.
fn exiled_by_combat_damage_with(t: &mut TestGame, attacker: ObjectId, top: &str) -> ObjectId {
    let card = t.library_top(P1, top);
    connect(t, &[attacker]);
    assert!(exiled(t, card), "{}", t.dump_log());
    card
}

#[test]
fn breeches_exiles_only_from_opponents_dealt_damage() {
    cr!("603.2c", "120.1");
    ruling!(
        "Breeches, Brazen Plunderer",
        "You'll exile a card from each opponent who was dealt damage, not from all of your opponents."
    );
    let mut t = TestGame::new(3);
    let b = t.battlefield(P0, "Breeches, Brazen Plunderer");
    let c1 = t.library_top(P1, "Divination");
    let c2 = t.library_top(P2, "Divination");
    connect(&mut t, &[b]);
    assert!(exiled(&t, c1), "{}", t.dump_log());
    assert_eq!(t.zone(c2), Zone::Library(P2));
}

// ---------------------------------------------------------------------------
// "This turn" / "until end of turn" permissions
// ---------------------------------------------------------------------------

#[test]
fn daxos_card_is_paid_for_with_alternative_or_additional_costs() {
    cr!("118.9", "601.2b", "601.2f", "702.33a");
    ruling!(
        "Daxos of Meletis",
        "You must pay all costs to cast the exiled card. You may pay alternative or additional costs. If the card has any mandatory additional costs, you must pay those."
    );
    // Kicked Burst Lightning: {R} + kicker {4}, mana of any color.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daxos of Meletis");
    let burst = exiled_by_combat_damage_with(&mut t, d, "Burst Lightning");
    mana(&mut t, P0, ManaType::W, 1);
    assert!(castable(&mut t, P0, burst));
    no_mana(&mut t, P0);
    assert!(!castable(&mut t, P0, burst), "must pay");
    mana(&mut t, P0, ManaType::W, 5);
    t.cast(P0, t.g.current(burst)).kicked(true).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // Village Rites: the sacrifice is mandatory (Daxos itself is sacrificed).
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daxos of Meletis");
    let rites = exiled_by_combat_damage_with(&mut t, d, "Village Rites");
    mana(&mut t, P0, ManaType::W, 1);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[o(d)]);
    t.cast(P0, t.g.current(rites)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Daxos of Meletis"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 2);
    // Invigorate's alternative cost (an opponent gains 3 life, with a Forest) can be paid.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daxos of Meletis");
    t.battlefield(P0, "Forest");
    let inv = exiled_by_combat_damage_with(&mut t, d, "Invigorate");
    let alt = crate::r_p068_common::methods(&mut t, P0, inv)
        .into_iter()
        .find(|m| *m != mtg_engine::object::CastMethod::Normal)
        .expect("Invigorate's alternative cost");
    t.cast(P0, t.g.current(inv)).method(alt).target(d).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 21, "{}", t.dump_log());
    assert_eq!(t.pt(d), (6, 6));
}
