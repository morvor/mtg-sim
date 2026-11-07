//! Rulings batch P052 — drawbacks: a static ability that depends on how many creatures
//! block (CR 611.3a, 506.4), casting and land-play restrictions and where they function
//! (CR 113.6, 113.6e), "unless" checked on resolution (CR 608.2), "can't be equipped"
//! (CR 701.3b), and Heaven // Earth's two values of X (CR 107.3, 709.3).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn rampaging_cyclops_recovers_when_a_blocker_leaves_combat() {
    cr!("611.3a", "506.4", "509.1h");
    ruling!(
        "Rampaging Cyclops",
        "If all but one of Rampaging Cyclops’s blockers are removed from combat, Rampaging Cyclops’s effect immediately stops reducing its power."
    );
    supported("Rampaging Cyclops");
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Rampaging Cyclops");
    old(&mut t, cyclops);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(cyclops), (4, 4));
    attack_with(&mut t, &[(cyclops, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(a, cyclops), (b, cyclops)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(t.pt(cyclops), (2, 4));
    t.g.destroy(a, None);
    t.settle();
    // One blocker left: the Cyclops is still blocked, at full power.
    assert!(t.g.combat.as_ref().unwrap().is_blocked(cyclops));
    assert_eq!(t.pt(cyclops), (4, 4));
}

#[test]
fn rock_jockeys_restrictions_function_in_hand_and_on_the_battlefield() {
    cr!("113.6", "113.6e");
    supported("Rock Jockey");
    ruling!(
        "Rock Jockey",
        "The first ability applies only while this card is in your hand (or someplace else you can cast it from)."
    );
    // A land played: Rock Jockey can't be cast.
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Mountain");
    let jockey = t.hand(P0, "Rock Jockey");
    t.play_land(P0, land).expect("land play");
    fill_pool(&mut t, P0, 3);
    assert!(t.cast(P0, jockey).try_go().is_err());
    // A Rock Jockey on the battlefield doesn't stop another being cast... and no land was
    // played: it can be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rock Jockey");
    let jockey = t.hand(P0, "Rock Jockey");
    fill_pool(&mut t, P0, 3);
    assert!(t.cast(P0, jockey).try_go().is_ok());
    ruling!(
        "Rock Jockey",
        "The second ability applies only while this card is on the battlefield."
    );
    // Cast this turn and on the battlefield: no land can be played.
    t.resolve_all();
    let land = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, land).is_err());
    // In hand (not cast), or on the battlefield without having been cast: lands can be
    // played.
    let mut t = TestGame::new(2);
    t.hand(P0, "Rock Jockey");
    t.enter(P0, "Rock Jockey");
    t.settle();
    let land = t.hand(P0, "Mountain");
    assert!(t.play_land(P0, land).is_ok());
}

/// P0's Hill Giant attacks P1 (blocked by P1's Grizzly Bears if `blocked`), then after
/// combat P0 equips it with Thirsting Axe; returns whether the Giant is still on the
/// battlefield after P0's end step began.
fn axe_equipped_after_combat(blocked: bool) -> bool {
    supported("Thirsting Axe");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    old(&mut t, giant);
    let axe = t.battlefield(P0, "Thirsting Axe");
    let bears = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(giant, pl(P1))]);
    let blocks = if blocked { vec![(bears, giant)] } else { vec![] };
    block_and_finish(&mut t, P1, &blocks);
    assert_eq!(t.life(P1), if blocked { 20 } else { 17 });
    t.advance_to(P0, Step::PostcombatMain);
    fill_pool(&mut t, P0, 2);
    t.activate(P0, axe, 0, &[obj(giant)]).expect("equip");
    t.resolve_all();
    assert_eq!(t.obj_now(axe).attached_to, Some(obj(giant)));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.on_battlefield(giant)
}

#[test]
fn thirsting_axe_checks_the_creature_equipped_as_the_end_step_begins() {
    cr!("603.4", "301.5");
    ruling!(
        "Thirsting Axe",
        "Thirsting Axe’s triggered ability checks whether the creature it’s attached to at the moment your end step begins dealt combat damage to a creature earlier in the turn. It doesn’t matter whether Thirsting Axe was attached to it (or to any creature at all) as combat damage was dealt."
    );
    assert!(axe_equipped_after_combat(true));
    assert!(!axe_equipped_after_combat(false));
}

#[test]
fn vampire_lacerator_checks_the_opponents_life_only_on_resolution() {
    cr!("608.2");
    ruling!(
        "Vampire Lacerator",
        "Whether an opponent has 10 or less life is checked only as Vampire Lacerator's ability resolves."
    );
    supported("Vampire Lacerator");
    for (at_trigger, at_resolution, loses) in [(11, 10, false), (10, 11, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vampire Lacerator");
        t.g.players[1].life = at_trigger;
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        // The trigger is put on the stack whatever the opponent's life total.
        assert_eq!(t.stack_len(), 1);
        t.g.players[1].life = at_resolution;
        t.resolve_all();
        assert_eq!(t.life(P0), if loses { 19 } else { 20 });
    }
}

#[test]
fn an_equip_ability_can_target_goblin_brawler_but_does_nothing() {
    cr!("701.3b", "702.6a");
    ruling!(
        "Goblin Brawler",
        "You can activate an equip ability that targets Goblin Brawler, but the Equipment will fail to move onto it."
    );
    supported("Goblin Brawler");
    let mut t = TestGame::new(2);
    let brawler = t.battlefield(P0, "Goblin Brawler");
    let bonesplitter = t.battlefield(P0, "Bonesplitter");
    fill_pool(&mut t, P0, 1);
    t.activate(P0, bonesplitter, 0, &[obj(brawler)])
        .expect("equip can target Goblin Brawler");
    t.resolve_all();
    assert_eq!(t.obj_now(bonesplitter).attached_to, None);
    assert_eq!(t.pt(brawler), (2, 2));
}

#[test]
fn crosiss_catacombs_can_return_a_tapped_land() {
    cr!("603.2");
    ruling!(
        "Crosis's Catacombs",
        "You can return a land that is tapped or untapped."
    );
    supported("Crosis's Catacombs");
    let mut t = TestGame::new(2);
    let island = t.battlefield(P0, "Island");
    t.g.objects[island.0 as usize].tapped = true;
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(island)]);
    let lair = t.enter(P0, "Crosis's Catacombs");
    t.resolve_all();
    assert_eq!(t.zone(island), Zone::Hand(P0));
    assert!(t.on_battlefield(lair));
}

#[test]
fn heaven_and_earth_each_choose_their_own_x() {
    cr!("107.3", "709.3", "702.127a");
    ruling!(
        "Heaven // Earth",
        "You don't have to choose the same value for X while casting Earth as you did while casting Heaven."
    );
    supported("Heaven // Earth");
    let mut t = TestGame::new(2);
    let flier = t.battlefield(P1, "Wind Drake");
    let giant = t.battlefield(P1, "Hill Giant");
    let card = t.hand(P0, "Heaven // Earth");
    fill_pool(&mut t, P0, 6);
    t.cast(P0, card).method(CastMethod::Half(0)).x(2).go();
    t.resolve_all();
    assert!(!t.on_battlefield(flier));
    assert!(t.on_battlefield(giant));
    let card = t.g.current(card);
    fill_pool(&mut t, P0, 6);
    t.cast(P0, card)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Aftermath))
        .x(3)
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}
