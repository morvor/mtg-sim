//! The worked examples of CR 400 (zones), played out as tests.

use super::r105_util::{card_from_text, put_in_hand};
use super::r703_common::supported;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Plays out the CR 400.6 example: a spell deals 5 damage to a player with 5 life and 5
/// damage to that player's Exquisite Archangel. `exile` answers the player's choice of
/// where the Archangel goes (`None`: no answer, the default).
fn archangel_and_its_controller_take_lethal_damage(exile: Option<bool>) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let angel = t.battlefield(P0, "Exquisite Archangel");
    t.g.players[0].life = 5;
    let spell = card_from_text(
        "Twin Blast",
        "{2}{R}",
        "Instant",
        None,
        "Twin Blast deals 5 damage to target player and 5 damage to target creature.",
    );
    let blast = put_in_hand(&mut t, P1, spell);
    t.lands(P1, "Mountain", 3);
    t.cast(P1, blast)
        .targets(&[Entity::Player(P0), Entity::Object(angel)])
        .go();
    if let Some(exile) = exile {
        t.answer_yes(P0, exile);
    }
    t.resolve();
    (t, angel)
}

#[test]
fn exquisite_archangel_and_lethal_damage_its_controller_chooses_where_it_goes() {
    cr!("400.6", "704.3", "704.5a", "704.5g", "614.1a");
    cr_example!("400.6", "Exquisite Archangel");
    for exile in [true, false] {
        let (t, angel) = archangel_and_its_controller_take_lethal_damage(Some(exile));
        // The player's life total becomes their starting life total.
        assert!(!t.has_lost(P0), "{}", t.dump_log());
        assert_eq!(t.life(P0), 20);
        assert!(!t.on_battlefield(angel));
        // ... and they chose whether the Archangel goes to exile or to the graveyard.
        assert_eq!(t.in_exile("Exquisite Archangel"), exile, "{}", t.dump_log());
        assert_eq!(t.in_graveyard(P0, "Exquisite Archangel"), !exile);
        // The choice was the Archangel's controller's.
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        // Only an Archangel put into its owner's graveyard by the lethal damage was destroyed.
        let destroyed = t
            .g
            .turn_events
            .iter()
            .any(|e| matches!(e, Event::Destroyed { obj, .. } if *obj == angel));
        assert_eq!(destroyed, !exile, "{}", t.dump_log());
    }
}

#[test]
fn lichs_mirror_and_a_creature_dying_at_the_same_time_its_owner_chooses_where_it_goes() {
    // CR 400.6 with another loss replacement: Lich's Mirror ("If you would lose the game,
    // instead shuffle your hand, your graveyard, and all permanents you own into your
    // library, then draw seven cards and your life total becomes 20.") and a creature with
    // lethal damage, both state-based actions at once.
    cr!("400.6", "704.3");
    for to_library in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lich's Mirror");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.players[0].life = 0;
        t.g.objects[bears.0 as usize].damage = 2;
        t.answer_yes(P0, to_library);
        t.settle();
        assert!(!t.has_lost(P0), "{}", t.dump_log());
        assert_eq!(t.life(P0), 20);
        assert!(!t.on_battlefield(bears));
        assert_eq!(t.hand_size(P0), 7);
        assert_eq!(t.in_graveyard(P0, "Grizzly Bears"), !to_library);
        let in_library = t
            .g
            .players[0]
            .library
            .iter()
            .any(|id| t.obj_now(*id).chars.name == "Grizzly Bears");
        // (It may have been drawn among the seven cards.)
        assert_eq!(
            in_library || t.in_hand(P0, "Grizzly Bears"),
            to_library,
            "{}",
            t.dump_log()
        );
    }
}

/// P0 is at 0 life and their Grizzly Bears has lethal damage: both state-based actions
/// are performed at once (CR 704.3).
fn zero_life_and_a_dying_bear(t: &mut TestGame) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 0;
    t.g.objects[bears.0 as usize].damage = 2;
    bears
}

/// No one was asked to choose between a replacement's move and a state-based action's.
fn no_move_choice(t: &TestGame) -> bool {
    !t.asked().iter().any(|(_, d)| match d {
        Decision::YesNo { prompt, .. } => prompt.contains("state-based action"),
        _ => false,
    })
}

#[test]
fn a_loss_replacement_that_moves_nothing_applies_though_its_source_dies_at_the_same_time() {
    // Like Nira, Hellkite Duelist's replacement (which the engine doesn't compile yet), on
    // a creature: it replaces the loss without moving anything, and the creature, dealt
    // lethal damage at the same time, was still on the battlefield as the state-based
    // actions were performed (CR 704.3). It simply dies; nothing is contradictory (CR 400.6).
    cr!("704.3", "704.5a", "704.5g", "614.1a");
    let mut t = TestGame::new(2);
    let def = card_from_text(
        "Last Duelist",
        "{3}",
        "Creature — Dragon",
        Some((3, 3)),
        "If you would lose the game, instead draw three cards and your life total becomes 5.",
    );
    assert!(def.is_fully_supported(), "{:?}", def.unsupported_text());
    let duelist = t.custom(P0, def, Zone::Battlefield);
    t.g.players[0].life = 0;
    t.g.objects[duelist.0 as usize].damage = 3;
    let hand = t.hand_size(P0);
    t.settle();
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 5);
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(!t.on_battlefield(duelist));
    assert!(t.in_graveyard(P0, "Last Duelist"));
    assert!(no_move_choice(&t));
}

#[test]
fn the_golden_throne_and_lilianas_other_contract_with_a_creature_dying_at_the_same_time() {
    // Each replacement moves only its own source, which no state-based action moves.
    cr!("704.3", "614.1a");
    supported("The Golden Throne");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Golden Throne");
    zero_life_and_a_dying_bear(&mut t);
    t.settle();
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 1);
    assert!(t.in_exile("The Golden Throne"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(no_move_choice(&t));

    supported("Liliana's Other Contract");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana's Other Contract");
    zero_life_and_a_dying_bear(&mut t);
    t.settle();
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    // It returned transformed: "You can't lose the game."
    assert_eq!(t.named_on_battlefield("Liliana's Undead Minion").len(), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(no_move_choice(&t));
    assert_eq!(t.life(P0), 0);
}

#[test]
fn a_replacement_shuffling_the_graveyard_doesnt_shuffle_a_creature_dying_at_the_same_time() {
    // Like Puresteel Angel's emblem (which the engine doesn't compile yet): "shuffle your
    // graveyard into your library" and a creature dying are performed at the same time
    // (CR 704.3): the creature isn't in the graveyard yet, so it stays there.
    cr!("704.3", "614.1a");
    let mut t = TestGame::new(2);
    let def = card_from_text(
        "Graveyard Mirror",
        "{4}",
        "Artifact",
        None,
        "If you would lose the game, instead shuffle your graveyard into your library and your life total becomes 20.",
    );
    assert!(def.is_fully_supported(), "{:?}", def.unsupported_text());
    t.custom(P0, def, Zone::Battlefield);
    t.graveyard(P0, "Hill Giant");
    let bears = zero_life_and_a_dying_bear(&mut t);
    t.settle();
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    assert!(!t.in_graveyard(P0, "Hill Giant"), "{}", t.dump_log());
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(no_move_choice(&t));
}
