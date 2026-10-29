//! Repeated processes, "the other" and per-player exchanges in older Oracle wordings
//! (cards4-paper-effects, families `repeat-loops`, `referents`, `player-actions`).

use crate::combat2_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn helm_setup(x: i64, leyline: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let helm = t.battlefield(P0, "Helm of Obedience");
    if leyline {
        t.battlefield(P0, "Leyline of the Void");
    }
    t.lands(P0, "Plains", x as usize);
    // Library from the top: Island, Forest, Grizzly Bears, Mountain.
    for c in ["Mountain", "Grizzly Bears", "Forest", "Island"] {
        t.library_top(P1, c);
    }
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, helm, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    (t, helm)
}

#[test]
fn helm_of_obedience_mills_until_a_creature_and_takes_it() {
    cr!("701.17a", "608.2c");
    ruling!(
        "Helm of Obedience",
        "You put the creature card onto the battlefield even if you can’t sacrifice Helm of Obedience"
    );
    supported("Helm of Obedience");
    let (t, helm) = helm_setup(5, false);
    assert!(t.in_graveyard(P1, "Island"));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(!t.in_graveyard(P1, "Mountain"));
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert!(!t.on_battlefield(helm));
}

#[test]
fn helm_of_obedience_stops_after_x_cards() {
    cr!("701.17a");
    let (t, helm) = helm_setup(2, false);
    assert!(t.in_graveyard(P1, "Island"));
    assert!(t.in_graveyard(P1, "Forest"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.on_battlefield(helm));
}

#[test]
fn helm_of_obedience_mills_the_whole_library_against_leyline_of_the_void() {
    cr!("701.17a", "614.1a");
    ruling!(
        "Helm of Obedience",
        "If an effect like that of Leyline of the Void prevents cards from being put into your opponent’s graveyard, the process described in the first sentence of Helm of Obedience’s effect will never stop. Your opponent’s entire library will be exiled, even if X is 1."
    );
    let (t, helm) = helm_setup(1, true);
    assert_eq!(t.library_size(P1), 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Mountain"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.on_battlefield(helm));
}

#[test]
fn forgotten_lore_the_opponent_chooses_again_while_you_pay() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Forgotten Lore",
        "As soon as you decline to pay {G}, the last card that was chosen by the opponent is put into your hand."
    );
    supported("Forgotten Lore");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Forgotten Lore");
    t.lands(P0, "Forest", 2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let cs = t.graveyard(P0, "Counterspell");
    t.graveyard(P0, "Dark Ritual");
    t.answer_choose(P1, &[Entity::Object(bolt)]);
    t.answer_yes(P0, true);
    // Bolt was chosen already: the second choice must be another card.
    t.answer_choose(P1, &[Entity::Object(cs)]);
    t.answer_yes(P0, false);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert!(t.in_hand(P0, "Counterspell"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P0, "Dark Ritual"));
    // The second choice didn't offer Lightning Bolt.
    let asked = t.asked();
    let offers: Vec<&Vec<Entity>> = asked
        .iter()
        .filter(|(p, _)| *p == P1)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offers.len(), 2);
    assert!(offers[0].contains(&Entity::Object(bolt)));
    assert!(!offers[1].contains(&Entity::Object(bolt)));
}

#[test]
fn cannibalize_exiles_the_chosen_one_and_grows_the_other() {
    cr!("608.2d", "115.1");
    ruling!(
        "Cannibalize",
        "You choose the two target creatures during announcement. You choose which gets the counters and which is to get exiled on resolution."
    );
    supported("Cannibalize");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Cannibalize");
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(giant, "+1/+1"), 2);
}

#[test]
fn juxtapose_exchanges_each_players_greatest_creature_and_artifact() {
    cr!("701.12a", "701.12b");
    ruling!(
        "Juxtapose",
        "If one of the players doesn't control a creature at the time the exchange would be made, that part of the effect does nothing, but the exchange of artifacts will still happen."
    );
    supported("Juxtapose");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Juxtapose");
    t.lands(P0, "Island", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let thopter = t.battlefield(P0, "Ornithopter");
    let ring = t.battlefield(P1, "Sol Ring");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.obj_now(giant).controller, P1);
    assert_eq!(t.obj_now(wurm).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(elves).controller, P1);
    // Ornithopter is an artifact creature with the greatest (0) of P0's artifacts.
    assert_eq!(t.obj_now(thopter).controller, P1);
    assert_eq!(t.obj_now(ring).controller, P0);

    // No creature on one side: only the artifacts change hands.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let spell = t.hand(P0, "Juxtapose");
    t.lands(P0, "Island", 5);
    let giant = t.battlefield(P0, "Hill Giant");
    let ring = t.battlefield(P1, "Sol Ring");
    let mox = t.battlefield(P0, "Mox Pearl");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(ring).controller, P0);
    assert_eq!(t.obj_now(mox).controller, P1);
}
