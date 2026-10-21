//! The worked examples of CR 400 (zones), played out as tests.

use super::r105_util::{card_from_text, put_in_hand};
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
