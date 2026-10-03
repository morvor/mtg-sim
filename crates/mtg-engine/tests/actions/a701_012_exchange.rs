//! CR 701.12: exchange.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn activated_index(t: &TestGame, id: ObjectId, text: &str) -> usize {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.to_lowercase().starts_with(&text.to_lowercase()))
        .expect("no such activated ability")
}

#[test]
fn exchanging_control_of_two_creatures() {
    cr!("701.12", "701.12b");
    supported("Switcheroo");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Switcheroo");
    t.cast(P0, s).targets(&[Entity::Object(mine), Entity::Object(theirs)]).go();
    t.resolve();
    assert_eq!(t.obj_now(mine).controller, P1);
    assert_eq!(t.obj_now(theirs).controller, P0);
}

#[test]
fn exchanging_control_of_permanents_with_the_same_controller_does_nothing() {
    cr!("701.12b");
    ruling!(
        "Switcheroo",
        "If the same player controls both creatures when Switcheroo resolves, nothing happens."
    );
    supported("Switcheroo");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Switcheroo");
    t.cast(P0, s).targets(&[Entity::Object(a), Entity::Object(b)]).go();
    t.resolve();
    assert_eq!(t.obj_now(a).controller, P1);
    assert_eq!(t.obj_now(b).controller, P1);
}

#[test]
fn if_the_entire_exchange_cant_be_completed_no_part_of_it_occurs() {
    cr!("701.12a");
    ruling!(
        "Switcheroo",
        "If one of the target creatures is an illegal target when Switcheroo resolves, the exchange won’t happen."
    );
    supported("Switcheroo");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Switcheroo");
    t.cast(P0, s).targets(&[Entity::Object(mine), Entity::Object(theirs)]).go();
    // One of them is destroyed in response.
    t.g.destroy(theirs, None);
    t.resolve();
    assert_eq!(t.obj_now(mine).controller, P0);
}

#[test]
fn exchanging_a_zone_with_an_empty_zone() {
    cr!("701.12d", "701.12f");
    ruling!("Morality Shift", "This spell works even if your graveyard or library is empty.");
    supported("Morality Shift");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    let lib = t.library_size(P0);
    assert_eq!(t.graveyard_size(P0), 0);
    let s = t.hand(P0, "Morality Shift");
    t.cast(P0, s).go();
    t.resolve();
    // The library went to the graveyard; the (empty) graveyard went to the library.
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), lib + 1);
    // With cards in both, they trade places.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    clear_library(&mut t, P0);
    t.library_top(P0, "Island");
    t.library_top(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    let s = t.hand(P0, "Morality Shift");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.library_size(P0), 1);
    assert!(t
        .g
        .find_in_zone(Zone::Library(P0), "Grizzly Bears")
        .first()
        .is_some());
    assert!(t.in_graveyard(P0, "Island"));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn an_exchanged_card_takes_over_the_attachment() {
    cr!("701.12e");
    supported("Arcanum Wings");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wings = t.battlefield(P0, "Arcanum Wings");
    t.g.attach(wings, Entity::Object(bears));
    t.g.recompute();
    t.lands(P0, "Island", 3);
    let strength = t.hand(P0, "Holy Strength");
    // "Aura swap {2}{U}": exchange this Aura with an Aura card in your hand.
    let i = activated_index(&t, wings, "Aura swap");
    t.activate(P0, wings, i, &[]).unwrap();
    t.resolve();
    // Arcanum Wings is no longer attached (it's in the hand); Holy Strength is attached
    // to the Bears.
    assert_eq!(t.zone(wings), Zone::Hand(P0));
    assert_eq!(t.obj_now(strength).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 4));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn exchanging_a_life_total_with_toughness() {
    cr!("701.12g");
    ruling!(
        "Tree of Redemption",
        "you will gain or lose an amount of life necessary so that your life total equals Tree of Redemption's former toughness"
    );
    supported("Tree of Redemption");
    supported("Ajani's Pridemate");
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Redemption");
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[0].life = 7;
    assert_eq!(t.pt(tree).1, 13);
    t.activate(P0, tree, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 13);
    assert_eq!(t.pt(tree).1, 7);
    // The life change was a life gain: "Whenever you gain life" triggered.
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
    // Toughness modifications apply after it's set.
    run(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![tree])),
            mods: vec![Modification::ModifyPT(Value::c(0), Value::c(2))],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(tree).1, 9);
}

#[test]
fn a_player_who_cant_gain_life_cant_be_given_a_higher_total_by_an_exchange() {
    cr!("701.12a", "701.12g");
    supported("Tree of Perdition");
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Perdition");
    // P1 can't gain life; the Tree's toughness is higher than their life total.
    t.battlefield(P0, "Leyline of Punishment");
    t.g.players[1].life = 5;
    t.activate(P0, tree, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // No part of the exchange happened.
    assert_eq!(t.life(P1), 5);
    assert_eq!(t.pt(tree).1, 13);
    // Lowering it is fine.
    let mut t = TestGame::new(2);
    let tree = t.battlefield(P0, "Tree of Perdition");
    t.activate(P0, tree, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.pt(tree).1, 20);
}

#[test]
fn exchanging_two_creatures_powers() {
    cr!("701.12g");
    supported("Serene Master");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Serene Master");
    let giant = t.battlefield(P1, "Hill Giant");
    // P1 attacks with the Giant; P0 blocks with Serene Master (0/2).
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let (mp, mt) = t.pt(master);
    t.attack(&[(giant, Entity::Player(P0))], &[(master, giant)]);
    // Each power became the other's; the Giant dealt 0 damage and took 3.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(master));
    assert_eq!(t.obj_now(master).damage, 0);
    assert_eq!(t.pt(master), (3, mt));
    // Until end of combat.
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(t.pt(master), (mp, mt));
}

#[test]
fn exchanging_text_boxes() {
    cr!("701.12h");
    supported("Exchange of Words");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    // "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(sorcerer), Entity::Object(bears)]);
    let words = t.enter(P0, "Exchange of Words");
    t.resolve_all();
    let has_ping = |t: &TestGame, id| {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .any(|a| matches!(a.kind, AbilityKind::Activated(_)))
    };
    assert!(!has_ping(&t, sorcerer));
    assert!(has_ping(&t, bears));
    t.activate(P0, bears, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // It lasts only while Exchange of Words remains on the battlefield.
    t.g.destroy(t.g.current(words), None);
    t.g.recompute();
    assert!(has_ping(&t, sorcerer));
    assert!(!has_ping(&t, bears));
}
