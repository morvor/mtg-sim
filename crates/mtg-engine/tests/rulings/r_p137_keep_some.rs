//! Rulings batch P137 — "each player chooses some permanents, then sacrifices the rest"
//! (Cataclysmic Gearhulk, Ajani, Nacatl Avenger, Global Ruin, Razia's Purification,
//! Destined Confrontation) and Liliana of the Veil's piles: a permanent with several types
//! can be chosen for each of them, players choose in turn order knowing the earlier
//! choices, and the unchosen permanents are sacrificed at the same time (CR 101.4).

use crate::r_p137_common::*;
use crate::r_s17_common::put_onto_battlefield;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";

/// P0's Ajani, Nacatl Avenger (with 4 loyalty), put onto the battlefield transformed.
fn avenger(t: &mut TestGame) -> ObjectId {
    supported(AJANI);
    let card = t.graveyard(P0, AJANI);
    let ajani = put_onto_battlefield(t, P0, card, true).expect("Ajani entered");
    assert_eq!(t.obj(ajani).chars.name, "Ajani, Nacatl Avenger");
    let have = t.counters(ajani, counters::LOYALTY);
    t.g.add_counters(Entity::Object(ajani), counters::LOYALTY, 4 - have, None);
    ajani
}

#[test]
fn ajani_avenger_an_artifact_creature_can_be_chosen_as_both() {
    cr!("101.4", "101.4c");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "A permanent with more than one type may be chosen as any of its types. For example, an artifact creature may be chosen as the artifact, the creature, or both. Choosing the same permanent twice this way has the same result as choosing it once."
    );
    let mut t = TestGame::new(2);
    let ajani = avenger(&mut t);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    // The Ornithopter is P1's artifact and P1's creature.
    t.answer_choose(P1, &[obj(thopter)]);
    t.answer_choose(P1, &[obj(thopter)]);
    t.activate(P0, ajani, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(ring));
    assert!(t.in_graveyard(P1, "Grizzly Bears") && t.in_graveyard(P1, "Sol Ring"));
}

#[test]
fn ajani_avenger_an_opponent_without_some_types_chooses_the_others() {
    cr!("101.4", "101.4c");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "If an opponent doesn't control any permanents of one of the types, they'll still choose the permanents of the types they do control."
    );
    let mut t = TestGame::new(2);
    let ajani = avenger(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // P1 has only creatures: they choose the Hill Giant as their creature.
    t.answer_choose(P1, &[obj(giant)]);
    t.activate(P0, ajani, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn cataclysmic_gearhulk_artifact_creature_chosen_as_both() {
    cr!("101.4", "101.4c");
    ruling!(
        "Cataclysmic Gearhulk",
        "A permanent with more than one type may be chosen as either of the types or both. For example, an artifact creature may be chosen as the artifact, the creature, or both. Choosing the same permanent twice this way has the same result as choosing it once."
    );
    ruling!(
        "Cataclysmic Gearhulk",
        "Cataclysmic Gearhulk will be sacrificed to its own ability if you don't choose it as your artifact or your creature."
    );
    supported("Cataclysmic Gearhulk");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let ring = t.battlefield(P0, "Sol Ring");
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_choose(P0, &[obj(thopter)]);
    let gearhulk = t.enter(P0, "Cataclysmic Gearhulk");
    t.settle();
    t.resolve_all();
    // The Ornithopter was P0's artifact and creature: the Gearhulk and the Sol Ring
    // weren't chosen.
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(gearhulk) && !t.on_battlefield(ring));
    assert!(t.in_graveyard(P0, "Cataclysmic Gearhulk"));
}

#[test]
fn cataclysmic_gearhulk_players_choose_in_turn_order_then_sacrifice_at_once() {
    cr!("101.4", "101.4c");
    ruling!(
        "Cataclysmic Gearhulk",
        "First the player whose turn it is chooses permanents they control, then each other player in turn order does the same. Each player will know the choices made by players who chose before them. All of the unchosen nonland permanents are then sacrificed simultaneously."
    );
    supported("Cataclysmic Gearhulk");
    // On P1's turn, P1 chooses first, then P2, then P0.
    let mut t = TestGame::new(3);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let mut kept = vec![];
    let mut lost = vec![];
    for p in [P0, P1, P2] {
        let bears = t.battlefield(p, "Grizzly Bears");
        let giant = t.battlefield(p, "Hill Giant");
        if p != P0 {
            t.answer_choose(p, &[obj(giant)]);
        }
        kept.push(giant);
        lost.push(bears);
    }
    let seen = [P0, P1, P2].map(|p| {
        watch(&mut t, p, is_choose, |g| {
            (
                g.permanents().count(),
                g.known_apnap_choices(PlayerId(0))
                    .iter()
                    .filter(|(_, c)| c.is_some())
                    .count(),
            )
        })
    });
    let gearhulk = t.enter(P0, "Cataclysmic Gearhulk");
    // P0's artifact is the Gearhulk, and its creature the Hill Giant.
    t.answer_choose(P0, &[obj(gearhulk)]);
    t.answer_choose(P0, &[obj(kept[0])]);
    t.settle();
    let n = t.g.permanents().count();
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = choosers_since(&t, from);
    // Each player is asked for the types they control (a creature; the Gearhulk is also
    // P0's artifact).
    let mut firsts = order.clone();
    firsts.dedup();
    assert_eq!(firsts, vec![P1, P2, P0]);
    assert_eq!(seen[1].lock().unwrap()[0], (n, 0));
    assert_eq!(seen[2].lock().unwrap()[0], (n, 1));
    assert_eq!(seen[0].lock().unwrap()[0], (n, 2));
    for k in kept.iter().chain([&gearhulk]) {
        assert!(t.on_battlefield(*k));
    }
    for l in &lost {
        assert!(!t.on_battlefield(*l));
    }
}

#[test]
fn global_ruin_a_dual_land_can_be_chosen_for_both_types() {
    cr!("101.4", "305.6");
    ruling!(
        "Global Ruin",
        "If a land counts as multiple basic land types, you can choose it for either or for both of those land types."
    );
    supported("Global Ruin");
    let mut t = TestGame::new(2);
    let tundra = t.battlefield(P1, "Tundra");
    let plains = t.battlefield(P1, "Plains");
    let island = t.battlefield(P1, "Island");
    // P1 chooses the Tundra as their Plains and as their Island.
    t.answer_choose(P1, &[obj(tundra)]);
    t.answer_choose(P1, &[obj(tundra)]);
    t.lands(P0, "Plains", 5);
    let spell = t.hand(P0, "Global Ruin");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(tundra));
    assert!(!t.on_battlefield(plains) && !t.on_battlefield(island));
}

#[test]
fn razias_purification_a_player_with_fewer_than_three_keeps_everything() {
    cr!("101.4");
    ruling!(
        "Razia's Purification",
        "If a player doesn’t control three permanents, that player chooses all the permanents they do control and doesn’t sacrifice anything."
    );
    supported("Razia's Purification");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    give_mana_for(&mut t, P0, "Razia's Purification");
    let mine = t.battlefield(P0, "Hill Giant");
    let spell = t.hand(P0, "Razia's Purification");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears) && t.on_battlefield(forest));
    // P0 had more than three: three remain.
    assert!(t.g.permanents_controlled_by(P0).len() == 3);
    let _ = mine;
}

#[test]
fn destined_confrontation_keeps_any_creatures_with_total_power_four_or_less() {
    cr!("101.4");
    ruling!(
        "Destined Confrontation",
        "Destined Confrontation has each player choose any number of creatures and then checks that the total power of creatures each player chose this way is 4 or less. For example, you could save two 2/2 creatures, or a 1/1 and a 3/3 creature, but not all four of those creatures."
    );
    supported("Destined Confrontation");
    // P1 keeps two 2/2s; P0 tries to keep all four of its creatures and can't.
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Runeclaw Bear");
    let one = t.battlefield(P1, "Elvish Mystic");
    let three = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(b1), obj(b2)]);
    let mine: Vec<ObjectId> = ["Grizzly Bears", "Runeclaw Bear", "Elvish Mystic", "Hill Giant"]
        .iter()
        .map(|n| t.battlefield(P0, n))
        .collect();
    t.answer_choose(P0, &mine.iter().map(|o| obj(*o)).collect::<Vec<_>>());
    give_mana_for(&mut t, P0, "Destined Confrontation");
    let spell = t.hand(P0, "Destined Confrontation");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(b1) && t.on_battlefield(b2));
    assert!(!t.on_battlefield(one) && !t.on_battlefield(three));
    let power: i32 = creatures(&t, P0).iter().map(|c| t.pt(*c).0).sum();
    assert!(power <= 4, "P0 kept total power {power}");
    assert!(creatures(&t, P0).len() < 4);
    // A 1/1 and a 3/3 is fine too.
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Runeclaw Bear");
    let one = t.battlefield(P1, "Elvish Mystic");
    let three = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(one), obj(three)]);
    give_mana_for(&mut t, P0, "Destined Confrontation");
    let spell = t.hand(P0, "Destined Confrontation");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
    assert!(t.on_battlefield(one) && t.on_battlefield(three));
}

#[test]
fn liliana_of_the_veil_an_empty_pile_sacrifices_nothing() {
    cr!("700.3", "700.3a");
    ruling!(
        "Liliana of the Veil",
        "A pile can be empty. If the player chooses an empty pile, no permanents will be sacrificed."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    t.g.add_counters(Entity::Object(lili), counters::LOYALTY, 3, None);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    // P0 puts nothing in the first pile; P1 chooses that (empty) pile.
    t.answer_choose(P0, &[]);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, lili, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears) && t.on_battlefield(forest));
    assert_eq!(t.graveyard_size(P1), 0);
}
