//! Rulings batch P069 — a permission to cast or play a card from exile doesn't change when
//! the card can be cast or played, nor its costs: a creature or sorcery card only in its
//! controller's main phase with an empty stack (CR 307.1, 302.1, 601.3), a land only as a
//! land play (CR 305.2, 305.3), a card's mana cost paid unless the permission says
//! otherwise (CR 601.2f–h).

use crate::r_p069_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::types::counters;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's creature `attacker` deals combat damage to P1 (unblocked); its triggers resolve.
fn connect(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

#[test]
fn ixhel_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "302.1");
    ruling!(
        "Ixhel, Scion of Atraxa",
        "Ixhel doesn't change when you can cast an exiled card. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Ixhel, Scion of Atraxa");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ixhel, Scion of Atraxa");
    t.g.players[1].counters.insert(counters::POISON.into(), 3);
    let bears = t.library_top(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert!(t.obj_now(bears).face_down);
    mana(&mut t, P0, ManaType::B, 2);
    assert!(!castable(&mut t, P0, bears), "end step");
    // Mana as though it were mana of any color.
    sorcery_timing_only(&mut t, P0, bears, ManaType::B, 2);
}

#[test]
fn mindleecher_doesnt_change_when_you_can_cast_the_cards() {
    cr!("601.3", "302.1");
    ruling!(
        "Mindleecher",
        "Mindleecher doesn’t change when you can cast the exiled cards. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    supported("Mindleecher");
    let mut t = TestGame::new(2);
    let giant = t.library_top(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = t.hand(P0, "Mindleecher");
    mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, m)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Mutate,
        ))
        .target(bears)
        .go();
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.resolve_all();
    assert!(exiled(&t, giant), "{}", t.dump_log());
    sorcery_timing_only(&mut t, P0, giant, ManaType::R, 4);
}

#[test]
fn mnemonic_betrayal_doesnt_change_when_you_can_cast_the_cards() {
    cr!("601.3", "307.1");
    ruling!(
        "Mnemonic Betrayal",
        "Mnemonic Betrayal doesn’t change when you can cast the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    let div = t.graveyard(P1, "Divination");
    let mb = t.hand(P0, "Mnemonic Betrayal");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::B, 2);
    t.cast(P0, mb).go();
    t.resolve_all();
    assert!(exiled(&t, div));
    // Mana of any type.
    sorcery_timing_only(&mut t, P0, div, ManaType::B, 3);
}

#[test]
fn ornate_kanzashi_normal_restrictions_apply() {
    cr!("601.3", "307.1", "304.1");
    ruling!(
        "Ornate Kanzashi",
        "Normal restrictions apply to cards exiled by Ornate Kanzashi (you can’t cast an instant unless you have priority, etc.)."
    );
    // An instant can be cast whenever P0 has priority (here, in combat); a sorcery only in
    // P0's main phase with an empty stack; both for their mana costs.
    for (name, instant) in [("Lightning Bolt", true), ("Divination", false)] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Ornate Kanzashi");
        let card = t.library_top(P1, name);
        mana(&mut t, P0, ManaType::C, 2);
        t.activate(P0, k, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, card));
        if instant {
            t.set_step(P0, Step::DeclareAttackers);
            // Not while another player has priority.
            mana(&mut t, P0, ManaType::R, 1);
            t.g.turn.priority = Some(P1);
            let c = t.g.current(card);
            assert!(t.g.cast_spell(P0, c, CastMethod::Normal).is_err());
            assert_eq!(t.zone(card), Zone::Exile);
            must_pay(&mut t, P0, card, ManaType::R, 1);
        } else {
            sorcery_timing_only(&mut t, P0, card, ManaType::U, 3);
            must_pay(&mut t, P0, card, ManaType::U, 3);
        }
    }
}

#[test]
fn extraordinary_journey_owner_pays_costs_and_follows_timing() {
    cr!("601.3", "302.1", "601.2f");
    ruling!(
        "Extraordinary Journey",
        "Players must pay all costs and follow all normal timing rules for spells cast with Extraordinary Journey's first ability."
    );
    supported("Extraordinary Journey");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ej = t.hand(P0, "Extraordinary Journey");
    mana(&mut t, P0, ManaType::U, 4);
    t.cast(P0, ej).x(1).go();
    t.answer_targets(P0, &[o(bears)]);
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    // Its owner, P1, may cast it: only in P1's main phase, for {1}{G}. Not P0, even with
    // the mana in its own main phase.
    mana(&mut t, P0, ManaType::G, 2);
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P1, Step::PrecombatMain);
    must_pay(&mut t, P1, bears, ManaType::G, 2);
    sorcery_timing_only(&mut t, P1, bears, ManaType::G, 2);
}

#[test]
fn fiend_nightveil_and_praetors_grasp_follow_the_normal_rules() {
    cr!("601.3", "302.1", "601.2f");
    ruling!(
        "Fiend of the Shadows",
        "Playing a card exiled with Fiend of the Shadows follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    ruling!(
        "Nightveil Specter",
        "Playing a card exiled with Nightveil Specter follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    ruling!(
        "Praetor's Grasp",
        "Playing a card exiled with Praetor’s Grasp follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    for name in ["Fiend of the Shadows", "Nightveil Specter", "Praetor's Grasp"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = match name {
            "Praetor's Grasp" => {
                let bears = t.library_top(P1, "Grizzly Bears");
                t.library_top(P1, "Hill Giant");
                let g = t.hand(P0, name);
                mana(&mut t, P0, ManaType::B, 3);
                t.answer_choose_moved(P0, &[o(bears)]);
                t.cast(P0, g).target(P1).go();
                t.resolve_all();
                bears
            }
            _ => {
                let c = t.battlefield(P0, name);
                let bears = if name == "Fiend of the Shadows" {
                    t.hand(P1, "Grizzly Bears")
                } else {
                    t.library_top(P1, "Grizzly Bears")
                };
                connect(&mut t, c);
                bears
            }
        };
        assert!(exiled(&t, bears), "{name}: {}", t.dump_log());
        // Praetor's Grasp resolved in P0's main phase; the others' cards were exiled in
        // combat.
        if name != "Praetor's Grasp" {
            mana(&mut t, P0, ManaType::G, 2);
            assert!(!castable(&mut t, P0, bears), "{name}: end of combat");
        }
        sorcery_timing_only(&mut t, P0, bears, ManaType::G, 2);
        must_pay(&mut t, P0, bears, ManaType::G, 2);
    }
}

#[test]
fn share_the_spoils_land_follows_normal_timing() {
    cr!("305.2", "305.3", "116.2a");
    ruling!(
        "Share the Spoils",
        "Playing a card this way follows all the normal timing rules for that card. For example, if you play a land this way, you may do so only during your main phase while the stack is empty and only if you havem't yet played a land (unless another effect allows you to play additional lands)."
    );
    supported("Share the Spoils");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P1, "Forest");
    t.enter(P0, "Share the Spoils");
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    land_timing(&mut t, P0, forest);
}

#[test]
fn dauthi_voidwalker_play_is_subject_to_timing() {
    cr!("601.3", "302.1");
    ruling!(
        "Dauthi Voidwalker",
        "Playing a card with Dauthi Voidwalker's last ability is still subject to normal timing restrictions."
    );
    supported("Dauthi Voidwalker");
    let mut t = TestGame::new(2);
    let dv = t.battlefield(P0, "Dauthi Voidwalker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(exiled(&t, bears));
    assert_eq!(t.counters(bears, "void"), 1);
    t.answer_choose_moved(P0, &[o(bears)]);
    t.activate(P0, dv, 0, &[]).unwrap();
    t.resolve_all();
    // Without paying its mana cost, but only at sorcery speed.
    sorcery_timing_only(&mut t, P0, bears, ManaType::C, 0);
}

#[test]
fn robber_of_the_rich_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "307.1");
    ruling!(
        "Robber of the Rich",
        "Robber of the Rich's effect doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robber of the Rich");
    t.hand(P1, "Island");
    let div = t.library_top(P1, "Divination");
    connect(&mut t, r);
    assert!(exiled(&t, div), "{}", t.dump_log());
    mana(&mut t, P0, ManaType::R, 3);
    assert!(!castable(&mut t, P0, div), "end of combat");
    t.advance_to(P0, Step::PostcombatMain);
    no_mana(&mut t, P0);
    mana(&mut t, P0, ManaType::R, 3);
    assert!(castable(&mut t, P0, div));
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(P1).go();
    assert!(!castable(&mut t, P0, div), "stack not empty");
    t.resolve_all();
    t.advance_to(P0, Step::End);
    mana(&mut t, P0, ManaType::R, 3);
    assert!(!castable(&mut t, P0, div), "end step");
}

#[test]
fn dead_mans_chest_doesnt_change_when_you_can_cast_the_cards() {
    cr!("601.3", "302.1");
    ruling!(
        "Dead Man's Chest",
        "The effect of Dead Man's Chest doesn't change when you can cast the exiled cards. For example, if you exile a creature card without flash, you can cast it only during your main phase when the stack is empty."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let chest = t.battlefield(P0, "Dead Man's Chest");
    t.g.attach(chest, Entity::Object(bears));
    let cards = stack_library(&mut t, P1, &["Hill Giant", "Forest"]);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(exiled(&t, cards[0]) && exiled(&t, cards[1]));
    sorcery_timing_only(&mut t, P0, cards[0], ManaType::U, 4);
}

#[test]
fn grinning_totem_card_uses_normal_timing_and_its_mana_cost() {
    cr!("601.3", "307.1", "601.2f");
    ruling!(
        "Grinning Totem",
        "The exiled card is played using the normal rules for timing and priority for its card type, as well as any other applicable restrictions. You’ll have to pay its mana cost. The only thing that’s different is you’re casting it from the Exile zone."
    );
    supported("Grinning Totem");
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Grinning Totem");
    let div = t.library_top(P1, "Divination");
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_choose_moved(P0, &[o(div)]);
    t.activate(P0, totem, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    sorcery_timing_only(&mut t, P0, div, ManaType::U, 3);
    must_pay(&mut t, P0, div, ManaType::U, 3);
}

#[test]
fn thada_adel_card_uses_artifact_timing_land_play_and_its_mana_cost() {
    cr!("601.3", "305.2", "305.3", "601.2f");
    ruling!(
        "Thada Adel, Acquisitor",
        "The exiled card is played using the normal timing rules for an artifact, as well as any other applicable restrictions. For example, you can't play the card during your combat phase unless it has flash. Similarly, if the exiled card is an artifact land, you can't play it if you've already played a land that turn. If it's a nonland card, you'll have to pay its mana cost. The only thing that's different is you're playing it from the exile zone."
    );
    supported("Thada Adel, Acquisitor");
    for name in ["Mind Stone", "Seat of the Synod"] {
        let mut t = TestGame::new(2);
        let thada = t.battlefield(P0, "Thada Adel, Acquisitor");
        let card = t.library_top(P1, name);
        t.answer_choose_moved(P0, &[o(card)]);
        connect(&mut t, thada);
        assert!(exiled(&t, card), "{name}: {}", t.dump_log());
        if name == "Mind Stone" {
            mana(&mut t, P0, ManaType::C, 2);
            assert!(!castable(&mut t, P0, card), "combat phase");
            sorcery_timing_only(&mut t, P0, card, ManaType::C, 2);
            must_pay(&mut t, P0, card, ManaType::C, 2);
        } else {
            land_timing(&mut t, P0, card);
        }
    }
}

#[test]
fn idol_of_endurance_doesnt_change_when_you_may_cast_creature_spells() {
    cr!("601.3", "302.1", "702.8a");
    ruling!(
        "Idol of Endurance",
        "The last ability of Idol of Endurance doesn't change when you may cast those creature spells. Normally this means during your main phase while the stack is empty, but you can cast a creature spell with flash at other times."
    );
    supported("Idol of Endurance");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let viper = t.graveyard(P0, "Ambush Viper");
    let idol = t.enter(P0, "Idol of Endurance");
    t.resolve_all();
    assert!(exiled(&t, bears) && exiled(&t, viper));
    mana(&mut t, P0, ManaType::W, 2);
    t.activate(P0, idol, 0, &[]).unwrap();
    t.resolve_all();
    sorcery_timing_only(&mut t, P0, bears, ManaType::C, 0);
    // Ambush Viper has flash: castable in combat, and in another player's turn.
    t.set_step(P0, Step::DeclareAttackers);
    assert!(castable(&mut t, P0, viper));
    t.set_step(P1, Step::End);
    assert!(castable(&mut t, P0, viper));
}
