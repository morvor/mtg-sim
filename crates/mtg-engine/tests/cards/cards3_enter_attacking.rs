//! Combat grammar IV: creatures put onto the battlefield attacking a named player or
//! blocking a named creature (CR 508.4, 509.4): Kaalia of the Vast, Owlbear Cub, Zara,
//! Hans Eriksson, Shark Shredder, Aetherplasm, Living Laser, Senu, Summoner's Grimoire.

use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn multi(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

fn attack_of(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attack_target(t.g.current(id)))
}

fn new_named(t: &TestGame, name: &str, before: &[ObjectId]) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| !before.contains(o))
        .collect()
}

#[test]
fn kaalia_puts_the_creature_onto_the_battlefield_attacking_that_opponent() {
    cr!("508.4");
    supported("Kaalia of the Vast");
    let mut t = multi(3);
    let kaalia = t.battlefield(P0, "Kaalia of the Vast");
    let angel = t.hand(P0, "Serra Angel");
    to_combat(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(angel)]);
    // Were P0 asked what the Angel attacks, it would pick P1: it may attack only P2.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    declare(&mut t, P0, &[(kaalia, Entity::Player(P2))]);
    t.resolve_all();
    let a = t.named_on_battlefield("Serra Angel");
    assert_eq!(a.len(), 1);
    assert!(t.g.obj(a[0]).tapped);
    assert_eq!(attack_of(&t, a[0]), Some(Entity::Player(P2)));
}

#[test]
fn owlbear_cub_digs_a_creature_attacking_that_player() {
    cr!("508.4");
    supported("Owlbear Cub");
    let mut t = multi(3);
    let cub = t.battlefield(P0, "Owlbear Cub");
    t.lands(P2, "Forest", 8);
    let wurm = t.library_top(P0, "Craw Wurm");
    to_combat(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    declare(&mut t, P0, &[(cub, Entity::Player(P2))]);
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert!(t.g.obj(w[0]).tapped);
    assert_eq!(attack_of(&t, w[0]), Some(Entity::Player(P2)));
}

#[test]
fn zara_steals_a_creature_attacking_the_player_whose_hand_it_was() {
    cr!("508.4");
    supported("Zara, Renegade Recruiter");
    let mut t = multi(3);
    let zara = t.battlefield(P0, "Zara, Renegade Recruiter");
    let giant = t.hand(P1, "Hill Giant");
    to_combat(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    declare(&mut t, P0, &[(zara, Entity::Player(P1))]);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.g.obj(g[0]).controller, P0);
    assert_eq!(attack_of(&t, g[0]), Some(Entity::Player(P1)));
    // It returns to its owner's hand at the beginning of the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn hans_eriksson_puts_the_creature_attacking_and_it_fights_him() {
    cr!("508.4", "603.12");
    supported("Hans Eriksson");
    let mut t = multi(3);
    let hans = t.battlefield(P0, "Hans Eriksson");
    t.library_top(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    declare(&mut t, P0, &[(hans, Entity::Player(P2))]);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(attack_of(&t, b[0]), Some(Entity::Player(P2)));
    // The bears (2/2) and Hans (1/4) fought.
    assert_eq!(t.g.obj(t.g.current(hans)).damage, 2);
    assert_eq!(t.g.obj(b[0]).damage, 1);
}

#[test]
fn hans_eriksson_puts_a_noncreature_card_into_the_hand_without_a_fight() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    let hans = t.battlefield(P0, "Hans Eriksson");
    t.library_top(P0, "Forest");
    declare(&mut t, P0, &[(hans, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.g.obj(t.g.current(hans)).damage, 0);
}

#[test]
fn shark_shredder_reanimates_attacking_the_damaged_player() {
    cr!("508.4");
    supported("Shark Shredder, Killer Clone");
    let mut t = multi(3);
    let shark = t.battlefield(P0, "Shark Shredder, Killer Clone");
    let wurm = t.graveyard(P2, "Craw Wurm");
    to_combat(&mut t, P0);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    declare(&mut t, P0, &[(shark, Entity::Player(P2))]);
    t.advance_to_step(Step::CombatDamage);
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert_eq!(t.g.obj(w[0]).controller, P0);
    assert!(t.g.obj(w[0]).tapped);
    assert_eq!(attack_of(&t, w[0]), Some(Entity::Player(P2)));
}

#[test]
fn aetherplasm_swaps_in_a_creature_blocking_the_attacker() {
    cr!("509.4");
    supported("Aetherplasm");
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Aetherplasm");
    let wurm = t.hand(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    declare(&mut t, P1, &[(giant, Entity::Player(P0))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    declare_blocks(&mut t, P0, &[(plasm, giant)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Aetherplasm"));
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.blockers_of(giant), vec![w[0]]);
    to_end_of_combat(&mut t);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn living_laser_tokens_enter_tapped_and_attacking_and_are_exiled() {
    cr!("508.4", "609.3");
    supported("Living Laser");
    let mut t = TestGame::new(2);
    let laser = t.battlefield(P0, "Living Laser");
    for _ in 0..2 {
        let c = t.hand(P0, "Forest");
        t.g.discard(P0, c, None);
    }
    declare(&mut t, P0, &[(laser, Entity::Player(P1))]);
    t.resolve_all();
    let tokens = new_named(&t, "Living Laser", &[laser]);
    assert_eq!(tokens.len(), 2);
    for &k in &tokens {
        assert!(t.g.obj(k).tapped);
        assert_eq!(attack_of(&t, k), Some(Entity::Player(P1)));
    }
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20 - 12);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Living Laser"), vec![laser]);
}

#[test]
fn senu_returns_from_exile_attacking_when_a_legend_is_unblocked() {
    cr!("508.4");
    supported("Senu, Keen-Eyed Protector");
    let mut t = TestGame::new(2);
    let hans = t.battlefield(P0, "Kaalia of the Vast");
    t.exile(P0, "Senu, Keen-Eyed Protector");
    declare(&mut t, P0, &[(hans, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    let s = t.named_on_battlefield("Senu, Keen-Eyed Protector");
    assert_eq!(s.len(), 1);
    assert!(!t.g.obj(s[0]).tapped);
    assert_eq!(attack_of(&t, s[0]), Some(Entity::Player(P1)));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 20 - 2 - 2);
}

#[test]
fn summoners_grimoire_enchantment_creature_enters_attacking_others_dont() {
    cr!("508.4");
    supported("Summoner's Grimoire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let book = t.battlefield(P0, "Summoner's Grimoire");
    t.g.attach(book, Entity::Object(bears));
    let nyx = t.hand(P0, "Nyxborn Courser");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(nyx)]);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let n = t.named_on_battlefield("Nyxborn Courser");
    assert_eq!(n.len(), 1);
    assert!(t.g.obj(n[0]).tapped);
    assert_eq!(attack_of(&t, n[0]), Some(Entity::Player(P1)));
}

#[test]
fn summoners_grimoire_other_creature_cards_enter_untapped_and_not_attacking() {
    cr!("508.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let book = t.battlefield(P0, "Summoner's Grimoire");
    t.g.attach(book, Entity::Object(bears));
    let wurm = t.hand(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert!(!t.g.obj(w[0]).tapped);
    assert_eq!(attack_of(&t, w[0]), None);
}
