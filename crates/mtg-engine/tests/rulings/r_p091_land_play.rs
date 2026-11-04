//! Rulings batch P091 — playing lands from other zones (Ancient Greenwarden, Hazezon,
//! Courser of Kruphix, Traveling Chocobo, Rocco, The Omenkeel, Faldorn), additional land
//! plays (Aesi, Case of the Locked Hothouse) and "can't play lands" (Turf Wound, Pardic
//! Miner).

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Checks the normal land-play timing for `card` (in a zone a permission lets `P0` play
/// lands from): not in the upkeep, not while the stack isn't empty, once in the main
/// phase, and not after a land was played.
fn normal_land_timing(t: &mut TestGame, card: ObjectId, what: &str) {
    t.set_step(P0, Step::Upkeep);
    assert!(!may_play_land(t, P0, card), "{what}: upkeep");
    t.set_step(P0, Step::PrecombatMain);
    assert!(may_play_land(t, P0, card), "{what}: main phase");
    // A spell on the stack.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!may_play_land(t, P0, card), "{what}: stack not empty");
    t.resolve_all();
    assert!(may_play_land(t, P0, card), "{what}: stack empty again");
    // After a land play from hand, no more.
    let hand_land = t.hand(P0, "Wastes");
    t.play_land(P0, hand_land).unwrap();
    assert!(!may_play_land(t, P0, card), "{what}: already played a land");
}

#[test]
fn land_play_permissions_dont_change_timing_or_the_number_of_land_plays() {
    cr!("305.2", "305.3", "601.3", "505.6b");
    ruling!(
        "Ancient Greenwarden",
        "Ancient Greenwarden doesn't change the times when you can play those lands. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    ruling!(
        "Hazezon, Shaper of Sand",
        "Hazezon doesn't change the times when you can play those land cards. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    ruling!(
        "Courser of Kruphix",
        "Courser of Kruphix doesn't change when you can play lands. You can do so only during your main phase when you have priority and the stack is empty."
    );
    supported("Ancient Greenwarden");
    supported("Courser of Kruphix");
    // Ancient Greenwarden: a land card in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Greenwarden");
    let land = t.graveyard(P0, "Forest");
    normal_land_timing(&mut t, land, "Greenwarden");
    // Hazezon: a Desert card in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hazezon, Shaper of Sand");
    let land = t.graveyard(P0, "Sunscorched Desert");
    normal_land_timing(&mut t, land, "Hazezon");
    // Courser of Kruphix: the top card of the library.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Courser of Kruphix");
    let land = t.library_top(P0, "Forest");
    normal_land_timing(&mut t, land, "Courser");
}

#[test]
fn graveyard_land_permissions_dont_allow_activating_abilities_or_casting_spell_faces() {
    cr!("305.2", "712.12", "702.29a", "602.2");
    ruling!(
        "Ancient Greenwarden",
        "Ancient Greenwarden doesn't allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    ruling!(
        "Hazezon, Shaper of Sand",
        "Hazezon doesn't allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    ruling!(
        "Ancient Greenwarden",
        "Ancient Greenwarden allows you to play a modal double-faced card's land face, but not a nonland face."
    );
    for (perm, land) in [
        ("Ancient Greenwarden", "Secluded Steppe"),
        ("Hazezon, Shaper of Sand", "Desert of the Mindful"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, perm);
        t.lands(P0, "Plains", 3);
        t.lands(P0, "Island", 3);
        let card = t.graveyard(P0, land);
        let acts = actions_for(&mut t, P0, card);
        assert_eq!(acts, vec![Action::PlayLand { card }], "{perm}");
    }
    // Agadeem's Awakening // Agadeem, the Undercrypt: the land face can be played, the
    // sorcery face can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Greenwarden");
    t.lands(P0, "Swamp", 6);
    let mdfc = t.graveyard(P0, "Agadeem's Awakening // Agadeem, the Undercrypt");
    assert!(may_play_land(&mut t, P0, mdfc));
    assert!(!may_cast(&mut t, P0, mdfc));
    t.play_land(P0, mdfc).unwrap();
    assert_eq!(t.named_on_battlefield("Agadeem, the Undercrypt").len(), 1);
}

#[test]
fn chocobo_follows_timing_and_allows_optional_costs_for_birds_from_the_top() {
    cr!("601.3", "305.2", "702.33a", "307.1");
    ruling!(
        "Traveling Chocobo",
        "You must follow all normal timing rules for Bird spells cast and lands played using the middle ability and must pay all costs for spells cast this way. To cast a spell this way, you may choose to pay any alternative costs or optional additional costs the spell may have (such as a kicker cost)."
    );
    supported("Traveling Chocobo");
    supported("Tempest Owl");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Traveling Chocobo");
    let land = t.library_top(P0, "Forest");
    normal_land_timing(&mut t, land, "Chocobo");
    // Tempest Owl (Bird, {1}{U}, kicker {4}{U}: "tap up to three target permanents").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Traveling Chocobo");
    let owl = t.library_top(P0, "Tempest Owl");
    // No mana: it can't be cast (costs must be paid).
    assert!(!may_cast(&mut t, P0, owl));
    t.lands(P0, "Island", 7);
    t.set_step(P0, Step::Upkeep);
    assert!(!may_cast(&mut t, P0, owl), "sorcery timing for a creature spell");
    t.set_step(P0, Step::PrecombatMain);
    assert!(may_cast(&mut t, P0, owl));
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, owl).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.zone(owl), Zone::Battlefield);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(tapped_lands(&t, P0), 7);
}

#[test]
fn additional_land_play_effects_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Aesi, Tyrant of Gyre Strait",
        "Aesi's ability is cumulative with other effects that let you play additional lands, such as the one from Exploration."
    );
    ruling!(
        "Case of the Locked Hothouse",
        "The effect of Case of the Locked Hothouse that allows you to play an additional land is cumulative with similar effects. For example, if you control two Case of the Locked Hothouses, you'll be able to play three lands during each of your turns."
    );
    supported("Aesi, Tyrant of Gyre Strait");
    supported("Case of the Locked Hothouse");
    for (a, b) in [
        ("Aesi, Tyrant of Gyre Strait", "Exploration"),
        ("Case of the Locked Hothouse", "Case of the Locked Hothouse"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, a);
        t.battlefield(P0, b);
        for i in 0..3 {
            let f = t.hand(P0, "Forest");
            assert!(t.play_land(P0, f).is_ok(), "{a}: land {i}");
        }
        let f = t.hand(P0, "Forest");
        assert!(t.play_land(P0, f).is_err(), "{a}: a fourth land");
    }
}

#[test]
fn cant_play_lands_doesnt_stop_putting_lands_or_activating_land_abilities() {
    cr!("305.2", "305.4", "602.1");
    ruling!(
        "Turf Wound",
        "Stops the player from playing a land, but not from putting a land onto the battlefield using a spell or ability."
    );
    ruling!(
        "Pardic Miner",
        "The ability won’t stop lands from being put onto the battlefield by a spell or ability."
    );
    ruling!(
        "Pardic Miner",
        "The ability won’t stop players from activating the abilities of lands. It only stops land cards from being played."
    );
    supported("Turf Wound");
    supported("Pardic Miner");
    for how in ["Turf Wound", "Pardic Miner"] {
        let mut t = TestGame::new(2);
        if how == "Turf Wound" {
            t.lands(P0, "Mountain", 3);
            let wound = t.hand(P0, "Turf Wound");
            t.cast(P0, wound).target(Entity::Player(P0)).go();
        } else {
            let miner = t.battlefield(P0, "Pardic Miner");
            t.activate(P0, miner, 0, &[Entity::Player(P0)]).unwrap();
        }
        t.resolve_all();
        let f = t.hand(P0, "Forest");
        assert!(t.play_land(P0, f).is_err(), "{how}: can't play a land");
        // Rampant Growth puts a land onto the battlefield.
        t.lands(P0, "Forest", 2);
        t.library_top(P0, "Plains");
        let growth = t.hand(P0, "Rampant Growth");
        t.answer_choose(P0, &[]);
        t.cast(P0, growth).go();
        let before = t.g.find_in_zone(Zone::Battlefield, "Plains").len();
        let plains = t.g.player(P0).library.iter().copied().find(|id| {
            t.g.obj(*id).chars.name == "Plains"
        });
        t.clear_answers();
        t.answer_choose(P0, &[Entity::Object(plains.unwrap())]);
        t.resolve_all();
        assert_eq!(
            t.g.find_in_zone(Zone::Battlefield, "Plains").len(),
            before + 1,
            "{how}: put onto the battlefield"
        );
        // A land's activated (mana) ability can still be activated.
        let karoo = t.battlefield(P0, "Gruul Turf");
        assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, karoo, "{R}{G}"));
    }
}

#[test]
fn rocco_exiled_cards_follow_the_normal_rules_and_the_trigger_order() {
    cr!("305.2", "601.3", "603.3b", "101.4");
    ruling!(
        "Rocco, Street Chef",
        "Playing the card you exiled with Rocco follows the normal rules for playing that card. You must pay its costs, and you must follow all applicable timing rules."
    );
    ruling!(
        "Rocco, Street Chef",
        "Unless an effect allows you to play additional lands that turn, you can play a land card you exiled with Rocco only if you haven't played a land yet that turn."
    );
    ruling!(
        "Rocco, Street Chef",
        "If Rocco's last ability triggers because a spell was cast, that ability will resolve before that spell does. If the ability triggers because a land was played, the land will be on the battlefield by the time you put the ability on the stack."
    );
    // P1 controls Rocco; at P1's end step each player exiles their top card: P0 a Forest,
    // P1 a Grizzly Bears.
    let mut t = TestGame::new(2);
    let rocco = t.battlefield(P1, "Rocco, Street Chef");
    t.advance_to(P1, Step::PostcombatMain);
    let forest = t.library_top(P0, "Forest");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    let forest = t.g.current(forest);
    let bears = t.g.current(bears);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
    // P0's turn: P0 plays the exiled Forest. P0's Sporemound triggers too: P0 is the
    // active player, so its trigger goes on the stack first and Rocco's (P1's) resolves
    // first.
    t.advance_to(P0, Step::PrecombatMain);
    let mound = t.battlefield(P0, "Sporemound");
    // Already played a land: can't play the exiled one.
    let mut t2 = t.g.clone();
    std::mem::swap(&mut t.g, &mut t2);
    let other = t.hand(P0, "Plains");
    t.play_land(P0, other).unwrap();
    assert!(!may_play_land(&mut t, P0, forest));
    std::mem::swap(&mut t.g, &mut t2);
    assert!(may_play_land(&mut t, P0, forest));
    t.answer_targets(P1, &[Entity::Object(rocco)]);
    t.play_land(P0, forest).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.zone(forest), Zone::Battlefield);
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).controller, P1);
    assert_eq!(triggers_of(&t, mound), 1);
    t.resolve_all();
    assert_eq!(t.counters(rocco, "+1/+1"), 1);
    // P1's turn: the Bears can't be cast in the upkeep, or without mana.
    t.advance_to(P1, Step::Upkeep);
    t.lands(P1, "Forest", 2);
    assert!(!may_cast(&mut t, P1, bears), "timing");
    t.advance_to(P1, Step::PrecombatMain);
    assert!(may_cast(&mut t, P1, bears));
    t.answer_targets(P1, &[Entity::Object(rocco)]);
    t.cast(P1, bears).go();
    t.settle();
    // Rocco's trigger is above the Bears spell.
    let top = *t.g.stack.last().unwrap();
    assert!(triggers_of(&t, rocco) == 1 && top != t.g.current(bears));
    t.resolve();
    assert_eq!(t.counters(rocco, "+1/+1"), 2);
    assert_eq!(t.zone(bears), Zone::Stack);
}

#[test]
fn omenkeel_lands_follow_normal_rules_and_outlive_the_omenkeel() {
    cr!("305.2", "611.2a", "702.122a");
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "The Omenkeel doesn't allow you to play additional lands during your turn, nor does it allow you to play lands at times you normally can't."
    );
    ruling!(
        "Cosima, God of the Voyage // The Omenkeel",
        "You can play lands from among cards exiled with The Omenkeel for as long as they remain exiled, even if The Omenkeel leaves the battlefield or another player gains control of it."
    );
    supported("Cosima, God of the Voyage // The Omenkeel");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Cosima, God of the Voyage // The Omenkeel");
    t.lands(P0, "Island", 2);
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let keel = t.named_on_battlefield("The Omenkeel")[0];
    t.g.objects[keel.0 as usize].summoning_sick = false;
    let crewer = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, keel, &[crewer]));
    t.resolve_all();
    let cards = stack_library(&mut t, P1, &["Forest", "Island", "Lightning Bolt"]);
    t.attack(&[(keel, Entity::Player(P1))], &[]);
    t.resolve_all();
    let exiled: Vec<ObjectId> = cards.iter().map(|c| t.g.current(*c)).collect();
    for c in &exiled {
        assert_eq!(t.zone(*c), Zone::Exile);
    }
    // The Omenkeel leaves the battlefield; the permission remains.
    crate::r_s02_common::destroy(&mut t, keel);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(may_play_land(&mut t, P0, exiled[0]));
    assert!(!may_cast(&mut t, P0, exiled[2]), "only lands");
    t.play_land(P0, exiled[0]).unwrap();
    // Only one land per turn.
    assert!(!may_play_land(&mut t, P0, exiled[1]));
    // Not in the end step either.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!may_play_land(&mut t, P0, exiled[1]));
    t.advance_to(P0, Step::Upkeep);
    assert!(!may_play_land(&mut t, P0, exiled[1]));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(may_play_land(&mut t, P0, exiled[1]));
}

#[test]
fn faldorn_lands_from_exile_follow_the_normal_rules() {
    cr!("305.2", "601.3");
    ruling!(
        "Faldorn, Dread Wolf Herald",
        "You must pay all costs and follow all normal timing rules for cards played this way. For example, you may only play a land from exile this way during your main phase while the stack is empty, and only if you haven't played a land yet this turn."
    );
    supported("Faldorn, Dread Wolf Herald");
    let mut t = TestGame::new(2);
    let faldorn = t.battlefield(P0, "Faldorn, Dread Wolf Herald");
    t.lands(P0, "Wastes", 1);
    t.hand(P0, "Grizzly Bears");
    let land = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[]);
    t.activate(P0, faldorn, 0, &[]).unwrap();
    t.resolve_all();
    let land = t.g.current(land);
    assert_eq!(t.zone(land), Zone::Exile);
    normal_land_timing(&mut t, land, "Faldorn");
}
