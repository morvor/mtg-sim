//! Rulings batch P216 — haste (CR 702.10) and its interactions: evasion gained after
//! blocks, losing haste (a static source leaving, transforming), copies with haste, and
//! gaining control of creatures.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::untapped_lands;
use crate::r_s06_common::has_kw;
use crate::r_s10_common::attacking;
use crate::r_s17_common::enter_transformed;
use crate::r_s21_common::blocks_now;
use crate::r_s25_common::*;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn evasion_gained_after_blocks_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Gingerbrute",
        "Activating Gingerbrute's middle ability after it has become blocked by a creature without haste won't cause it to become unblocked."
    );
    ruling!(
        "Resilient Roadrunner",
        "Activating Resilient Roadrunner's last ability after it has become blocked by a creature without haste won't cause it to become unblocked."
    );
    for (name, mana) in [("Gingerbrute", 1), ("Resilient Roadrunner", 3)] {
        supported(name);
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, name);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", mana);
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(bears, attacker)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        t.activate(P0, attacker, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(blocks_now(&t), vec![(bears, attacker)], "{name}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

#[test]
fn satyrs_lose_haste_when_gallia_leaves() {
    cr!("302.6", "702.10b", "506.4");
    ruling!(
        "Gallia of the Endless Dance",
        "If Gallia leaves the battlefield, Satyrs you control lose haste. If they haven’t been under your control since the turn began and don’t otherwise have haste, they can’t attack this turn. If they’ve already attacked, they remain attacking creatures."
    );
    supported("Gallia of the Endless Dance");
    supported("Satyr Wayfinder");
    // Before attacking: the new Satyr can't attack any more.
    let mut t = TestGame::new(2);
    let gallia = t.battlefield(P0, "Gallia of the Endless Dance");
    let satyr = t.battlefield_sick(P0, "Satyr Wayfinder");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, satyr));
    destroy(&mut t, gallia);
    assert!(!has_kw(&t, satyr, KeywordKind::Haste));
    assert!(!can_attack(&mut t, satyr));
    // After attacking: it stays an attacking creature.
    let mut t = TestGame::new(2);
    let gallia = t.battlefield(P0, "Gallia of the Endless Dance");
    let satyr = t.battlefield_sick(P0, "Satyr Wayfinder");
    attack_with(&mut t, &[(satyr, Entity::Player(P1))]);
    destroy(&mut t, gallia);
    assert!(attacking(&t, satyr));
    t.advance_to(P0, Step::EndOfCombat);
    // Satyr Wayfinder is 1/1.
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_creature_that_transforms_into_a_face_without_haste_cant_attack() {
    cr!("302.6", "712.8a");
    ruling!(
        "Geier Reach Bandit // Vildin-Pack Alpha",
        "If a creature with haste transforms and no longer has haste on the same turn that it comes under your control, such as if you cast a Geier Reach Bandit when you already control a Vildin-Pack Alpha and choose to transform it, it won’t be able to attack that turn."
    );
    supported("Geier Reach Bandit // Vildin-Pack Alpha");
    for transform in [false, true] {
        let mut t = TestGame::new(2);
        // The Alpha sees itself enter: it stays an Alpha.
        t.answer_yes(P0, false);
        let alpha = enter_transformed(&mut t, P0, "Geier Reach Bandit // Vildin-Pack Alpha");
        t.resolve_all();
        assert_eq!(name_now(&t, alpha), "Vildin-Pack Alpha");
        let card = t.hand(P0, "Geier Reach Bandit // Vildin-Pack Alpha");
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 2);
        t.cast(P0, card).go();
        t.answer_yes(P0, transform);
        t.resolve_all();
        let bandit = t.g.current(card);
        assert_eq!(name_now(&t, bandit) == "Vildin-Pack Alpha", transform);
        assert_eq!(has_kw(&t, bandit, KeywordKind::Haste), !transform);
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(can_attack(&mut t, bandit), !transform);
    }
}

#[test]
fn kiki_jikis_token_copies_only_the_printed_creature_plus_haste() {
    cr!("707.2", "707.9a", "702.10b");
    ruling!(
        "Kiki-Jiki, Mirror Breaker",
        "The token copies exactly what was printed on the original creature (except that the copy also has haste) and nothing else"
    );
    supported("Kiki-Jiki, Mirror Breaker");
    let mut t = TestGame::new(2);
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    t.g.tap(bears);
    assert_eq!(t.pt(bears), (7, 7));
    t.activate(P0, kiki, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    let copies: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(copies.len(), 1);
    let token = copies[0];
    assert_eq!(name_now(&t, token), "Grizzly Bears");
    assert_eq!(t.pt(token), (2, 2));
    assert_eq!(t.counters(token, "+1/+1"), 0);
    assert!(!t.obj(token).tapped);
    assert!(has_kw(&t, token, KeywordKind::Haste));
    assert!(!has_kw(&t, bears, KeywordKind::Haste));
}

#[test]
fn keldon_overseer_can_take_an_untapped_creature_or_untap_your_own() {
    cr!("702.33d", "603.4");
    ruling!(
        "Keldon Overseer",
        "You can target and gain control of an untapped creature with Keldon Overseer's ability. You can also untap a creature you already control and give it haste."
    );
    supported("Keldon Overseer");
    // An opponent's untapped creature.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Keldon Overseer");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, card).kicked(true).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert!(!t.obj_now(giant).tapped);
    assert!(has_kw(&t, giant, KeywordKind::Haste));
    // P0's own tapped, summoning-sick creature.
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Hill Giant");
    t.g.tap(mine);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "Keldon Overseer");
    t.answer_targets(P0, &[obj(mine)]);
    t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P0);
    assert!(!t.obj_now(mine).tapped);
    assert!(has_kw(&t, mine, KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, mine));
}
