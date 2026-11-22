//! Rulings batch S23 — other casting rulings: life loss below zero in a multiplayer game
//! (CR 119.3, 107.1b), and hybrid spells being each of their colors however they're paid
//! for (CR 202.2d).

use crate::r_s01_common::*;
use crate::r_s14_common::triggers_from;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn exsanguinate_players_can_lose_more_life_than_they_have() {
    cr!("119.3", "107.1b", "704.5a");
    ruling!(
        "Exsanguinate",
        "Players can lose more life than they have. For example, say you're playing a multiplayer game in which one of your opponents has 3 life and your other opponent has 10 life. If you cast Exsanguinate with X of 4, your opponents will wind up at -1 life and 6 life, respectively. You'll gain 8 life."
    );
    supported("Exsanguinate");
    // "Each opponent loses X life. You gain life equal to the life lost this way."
    let mut t = TestGame::new(3);
    t.g.players[1].life = 3;
    t.g.players[2].life = 10;
    t.lands(P0, "Swamp", 6);
    let card = t.hand(P0, "Exsanguinate");
    t.cast(P0, card).x(4).go();
    t.resolve_all();
    assert_eq!(t.life(P1), -1);
    assert_eq!(t.life(P2), 6);
    assert_eq!(t.life(P0), 28);
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P2));
}

#[test]
fn a_hybrid_spell_cast_with_one_color_is_still_multicolored() {
    cr!("202.2d", "105.2b", "107.4e");
    ruling!(
        "Pyroconvergence",
        "The hybrid spells in the Return to Ravnica set are multicolored, even if you cast them with one color of mana."
    );
    supported("Pyroconvergence");
    supported("Lobber Crew");
    supported("Dryad Militant");
    // Pyroconvergence: "Whenever you cast a multicolored spell, this enchantment deals 2
    // damage to any target." Lobber Crew: "Whenever you cast a multicolored spell, untap
    // this creature." Dryad Militant ({G/W}) is cast with {G} only.
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Pyroconvergence");
    let crew = t.battlefield(P0, "Lobber Crew");
    t.g.tap(crew);
    t.lands(P0, "Forest", 1);
    let militant = t.hand(P0, "Dryad Militant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, militant).go();
    assert!(t.obj(spell).chars.colors.is_multicolored());
    t.settle();
    assert_eq!(triggers_from(&t, pyro), 1);
    assert_eq!(triggers_from(&t, crew), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!t.obj_now(crew).tapped);
}
