//! Rulings batch P216 — goad (CR 701.15): who a goaded creature must attack in
//! multiplayer games, which creatures a goad effect affects (locked in as it resolves),
//! static goading, and goading creatures already in combat.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s09_common::*;
use crate::r_s10_common::{attacking, blocking};
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` casts the sorcery `name` (no targets) in their main phase and it resolves.
fn sorcery(t: &mut TestGame, p: PlayerId, name: &str) {
    supported(name);
    t.set_step(p, Step::PrecombatMain);
    cast_new(t, p, name, &[]);
    t.resolve_all();
}

fn goaders(t: &TestGame, id: ObjectId) -> Vec<PlayerId> {
    let mut v = t.g.goaders(t.g.current(id));
    v.sort();
    v
}

#[test]
fn a_creature_goaded_by_several_opponents_attacks_one_that_didnt_goad_it() {
    cr!("701.15b", "701.15c", "508.1d");
    ruling!(
        "Ghoulish Impetus",
        "Attacking with a goaded creature doesn't cause it to stop being goaded. If a creature you control has been goaded by multiple opponents, it must attack one of your opponents that hasn't goaded it, as that fulfills the maximum number of goad requirements. If a creature you control has been goaded by each of your opponents, the creature must attack an opponent (rather than a planeswalker or battle), but you choose which opponent it attacks."
    );
    ruling!(
        "Taunt from the Rampart",
        "If a creature you control has been goaded by multiple opponents, it must attack one of your opponents that hasn't goaded it, as that fulfills the maximum number of goad requirements. If a creature you control has been goaded by each of your opponents, you choose which opponent it attacks."
    );
    supported("Ghoulish Impetus");
    let mut t = TestGame::new(4);
    let bears = t.battlefield(P3, "Grizzly Bears");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    // Goaded by P0 (Ghoulish Impetus) and P1 (Taunt from the Rampart).
    attach_new(&mut t, P0, "Ghoulish Impetus", bears);
    sorcery(&mut t, P1, "Taunt from the Rampart");
    assert_eq!(goaders(&t, bears), vec![P0, P1]);
    to_combat(&mut t, P3);
    let at = |e: Entity| vec![(bears, e)];
    assert!(legal_attack(&mut t, &at(Entity::Player(P2))));
    for e in [Entity::Player(P0), Entity::Player(P1), obj(ajani)] {
        assert!(!legal_attack(&mut t, &at(e)), "{e:?}");
    }
    assert!(!legal_attack(&mut t, &[]));
    // Goaded by P2 too: any opponent, but not a planeswalker.
    sorcery(&mut t, P2, "Disrupt Decorum");
    assert_eq!(goaders(&t, bears), vec![P0, P1, P2]);
    to_combat(&mut t, P3);
    for p in [P0, P1, P2] {
        assert!(legal_attack(&mut t, &at(Entity::Player(p))));
    }
    assert!(!legal_attack(&mut t, &at(obj(ajani))));
    // Attacking doesn't end it.
    let attacks = declare(&mut t, P3, &at(Entity::Player(P1)));
    assert_eq!(attacks, at(Entity::Player(P1)));
    assert_eq!(goaders(&t, bears), vec![P0, P1, P2]);
}

#[test]
fn a_goaded_creature_that_cant_attack_other_players_attacks_a_planeswalker_or_the_goader() {
    cr!("701.15b", "508.1d");
    ruling!(
        "Taunt from the Rampart",
        "If the creature doesn't meet any of the above exceptions and can attack, it must attack a player other than the controller of the spell or ability that goaded it if able. If the creature can't attack any of those players but could otherwise attack, it must attack an opposing planeswalker (controlled by any opponent) or the player that goaded it."
    );
    ruling!(
        "Taunting Sliver",
        "If the creature doesn't meet any of the above exceptions and can attack, it must attack a player other than the controller of the spell or ability that goaded it if able. If the creature can't attack any of those players but could otherwise attack, it must attack a planeswalker an opponent controls, a battle an opponent protects, or a player who goaded it."
    );
    supported("Taunting Sliver");
    supported("Blazing Archon");
    // Taunt from the Rampart (P0); P2's Blazing Archon: "Creatures can't attack you."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    let other_pw = t.battlefield(P2, "Ajani Goldmane");
    t.battlefield(P2, "Blazing Archon");
    sorcery(&mut t, P0, "Taunt from the Rampart");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P2))]));
    for e in [Entity::Player(P0), obj(ajani), obj(other_pw)] {
        assert!(legal_attack(&mut t, &[(bears, e)]), "{e:?}");
    }
    // Taunting Sliver's granted trigger, in a two-player game: P0 or P0's planeswalker.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Taunting Sliver");
    t.resolve_all();
    assert_eq!(goaders(&t, bears), vec![P0]);
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    for e in [Entity::Player(P0), obj(ajani)] {
        assert!(legal_attack(&mut t, &[(bears, e)]), "{e:?}");
    }
}

#[test]
fn goad_effects_affect_only_the_creatures_there_as_they_resolve() {
    cr!("701.15a", "611.2c", "701.15d");
    ruling!(
        "Disrupt Decorum",
        "Creatures that enter the battlefield after Disrupt Decorum resolves won't be goaded."
    );
    ruling!(
        "Geode Rager",
        "Creatures that enter the battlefield or come under the target player's control after Geode Rager's ability has resolved won't be goaded."
    );
    ruling!(
        "Geode Rager",
        "Being goaded more than once by the same player is redundant."
    );
    // Disrupt Decorum.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    sorcery(&mut t, P0, "Disrupt Decorum");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(goaders(&t, bears), vec![P0]);
    assert!(goaders(&t, giant).is_empty());
    // Geode Rager: landfall twice, targeting P1.
    supported("Geode Rager");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Geode Rager");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    for _ in 0..2 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Forest");
        t.resolve_all();
    }
    assert_eq!(t.obj_now(bears).goaded_by, vec![P0]);
    assert_eq!(goaders(&t, bears), vec![P0]);
    let late = t.battlefield(P1, "Llanowar Elves");
    give_control(&mut t, mine, P1);
    assert!(goaders(&t, late).is_empty());
    assert!(goaders(&t, mine).is_empty());
}

#[test]
fn marisis_goad_is_locked_in_and_outlasts_marisi() {
    cr!("701.15a", "611.2c", "510.3a");
    ruling!(
        "Marisi, Breaker of the Coil",
        "Creatures that enter the battlefield after Marisi's second ability resolves won't be goaded."
    );
    ruling!(
        "Marisi, Breaker of the Coil",
        "The goaded creatures remain goaded even if Marisi leaves the battlefield before your next turn."
    );
    supported("Marisi, Breaker of the Coil");
    let mut t = TestGame::new(2);
    let marisi = t.battlefield(P0, "Marisi, Breaker of the Coil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(marisi, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(goaders(&t, bears), vec![P0]);
    let late = t.battlefield(P1, "Hill Giant");
    destroy(&mut t, marisi);
    assert_eq!(goaders(&t, bears), vec![P0]);
    assert!(goaders(&t, late).is_empty());
    // In P1's turn the Bears must attack; the Giant needn't.
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, Entity::Player(P0))]));
}

#[test]
fn goading_a_blocker_or_attackers_doesnt_remove_them_from_combat() {
    cr!("701.15a", "506.4");
    ruling!(
        "Bothersome Quasit",
        "Goading a creature after it has been declared as a blocker will not remove it from combat."
    );
    ruling!(
        "Take the Bait",
        "Untapping and goading the attacking creatures won't remove them from combat, even if they're attacking you."
    );
    supported("Bothersome Quasit");
    supported("Take the Bait");
    // Bothersome Quasit ("Goaded creatures your opponents control can't block"): P0 casts
    // a noncreature spell after blockers are declared.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bothersome Quasit");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, giant)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(blocking(&t, bears));
    cast_new(&mut t, P0, "Giant Growth", &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(goaders(&t, bears), vec![P0]);
    assert!(blocking(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    // Take the Bait, cast by the defending player during P1's combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P1);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(t.obj_now(bears).tapped);
    cast_new(&mut t, P0, "Take the Bait", &[]);
    t.resolve_all();
    assert!(attacking(&t, bears));
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(goaders(&t, bears), vec![P0]);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn baeloths_goad_lasts_only_while_its_condition_holds() {
    cr!("701.15a", "611.3a");
    ruling!(
        "Baeloth Barrityl, Entertainer",
        "Unlike most similar effects, Baeloth Barrityl's first ability doesn't cause a creature to be goaded until your next turn. Rather, it causes it to be goaded for as long as Baeloth Barrityl is on battlefield, has the ability, and has sufficient power."
    );
    supported("Baeloth Barrityl, Entertainer");
    // Baeloth is 2/5.
    let mut t = TestGame::new(2);
    let baeloth = t.battlefield(P0, "Baeloth Barrityl, Entertainer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(goaders(&t, elves), vec![P0]);
    assert!(goaders(&t, giant).is_empty());
    // Still goaded in P0's next turn and beyond.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(goaders(&t, elves), vec![P0]);
    // The Elves get bigger: no longer.
    cast_new(&mut t, P1, "Giant Growth", &[obj(elves)]);
    t.resolve_all();
    assert!(goaders(&t, elves).is_empty());
    // Baeloth gets bigger: the Giant is goaded; Baeloth leaves: nothing is.
    cast_new(&mut t, P0, "Giant Growth", &[obj(baeloth)]);
    t.resolve_all();
    assert_eq!(goaders(&t, giant), vec![P0]);
    destroy(&mut t, baeloth);
    assert!(goaders(&t, giant).is_empty());
    assert!(goaders(&t, elves).is_empty());
}
