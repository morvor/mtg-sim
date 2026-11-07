//! Group subjects narrowed by a state ("as long as it's a creature"), "with no
//! abilities", and stacked thresholds ("also get ... as long as ...").

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

#[test]
fn creatures_with_no_abilities() {
    cr!("613.4c", "113.1");
    ruling!(
        "Muraganda Petroglyphs",
        "Muraganda Petroglyphs won't apply to that creature because it has gained an ability"
    );
    compiles("Muraganda Petroglyphs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let other = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(angel), (4, 4));
    // Gaining an ability stops the bonus.
    let aura = t.battlefield(P0, "Arcane Flight");
    t.attach(aura, Entity::Object(other));
    t.settle();
    assert_eq!(t.pt(other), (3, 3));
}

#[test]
fn each_land_gets_a_bonus_as_long_as_its_a_creature() {
    cr!("611.3a", "613.4c");
    ruling!("Earth Surge", "It doesn't turn lands into creatures.");
    compiles("Earth Surge");
    compiles("Wind Zendikon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Earth Surge");
    let island = t.battlefield(P0, "Island");
    let forest = t.battlefield(P1, "Forest");
    let z = t.battlefield(P0, "Wind Zendikon");
    t.attach(z, Entity::Object(island));
    t.settle();
    assert_eq!(t.pt(island), (4, 4));
    assert!(!t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn each_untapped_creature_that_isnt_attacking() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Arcades Sabboth",
        "it gets +0/+2 from its own third ability"
    );
    compiles("Arcades Sabboth");
    let mut t = TestGame::new(2);
    let arcades = t.battlefield(P0, "Arcades Sabboth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(tapped);
    t.settle();
    assert_eq!(t.pt(arcades), (7, 9));
    assert_eq!(t.pt(bears), (2, 4));
    assert_eq!(t.pt(tapped), (2, 2));
    // An attacking creature loses the bonus (Grizzly Bears has no vigilance: it taps).
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.is_attacking(bears));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn stacked_thresholds_with_also() {
    cr!("611.3a", "613.4c");
    compiles("Jetmir, Nexus of Revels");
    let mut t = TestGame::new(2);
    let jetmir = t.battlefield(P0, "Jetmir, Nexus of Revels");
    let a = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(a), (2, 2));
    t.battlefield(P0, "Grizzly Bears");
    t.settle();
    // Three creatures: +1/+0 and vigilance.
    assert_eq!(t.pt(a), (3, 2));
    assert!(has(&t, a, KeywordKind::Vigilance));
    assert!(!has(&t, a, KeywordKind::Trample));
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    t.settle();
    // Six: another +1/+0 and trample.
    assert_eq!(t.pt(a), (4, 2));
    assert_eq!(t.pt(jetmir), (7, 4));
    assert!(has(&t, a, KeywordKind::Trample));
    assert!(!has(&t, a, KeywordKind::DoubleStrike));
}
