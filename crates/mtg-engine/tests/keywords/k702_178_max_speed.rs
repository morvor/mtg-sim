//! CR 702.178 Max speed: "Max speed — [Ability]" means "As long as your speed is 4, this
//! object has '[Ability].'" (pattern in `src/oracle/patterns/k702_178_max_speed.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn set_speed(t: &mut TestGame, p: PlayerId, speed: u32) {
    t.g.players[p.idx()].speed = Some(speed);
    t.recompute();
}

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn max_speed_cards_compile() {
    assert_supported(&[
        "Burnout Bashtronaut",
        "Walking Sarcophagus",
        "Far Fortune, End Boss",
        "Embalmed Ascendant",
        "Goblin Surveyor",
        "Endrider Catalyzer",
        "Racers' Scoreboard",
    ]);
}

#[test]
fn a_static_ability_applies_only_at_max_speed() {
    cr!("702.178a", "702.179e");
    let mut t = TestGame::new(2);
    let bash = t.battlefield(P0, "Burnout Bashtronaut");
    let cat = t.battlefield(P0, "Walking Sarcophagus");
    let double = |t: &TestGame| t.obj_now(bash).chars.has_keyword(KeywordKind::DoubleStrike);
    set_speed(&mut t, P0, 3);
    assert!(!double(&t));
    assert_eq!(t.pt(cat), (2, 1));
    // Max speed is speed 4.
    set_speed(&mut t, P0, 4);
    assert!(double(&t));
    assert_eq!(t.pt(cat), (3, 3));
    // It's the controller's speed that counts.
    set_speed(&mut t, P0, 1);
    set_speed(&mut t, P1, 4);
    assert!(!double(&t));
    assert_eq!(t.pt(cat), (2, 1));
}

#[test]
fn a_replacement_effect_applies_only_at_max_speed() {
    cr!("702.178a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Far Fortune, End Boss");
    set_speed(&mut t, P0, 3);
    let bolt = |t: &mut TestGame| {
        t.lands(P0, "Mountain", 1);
        let b = t.hand(P0, "Lightning Bolt");
        t.cast(P0, b).target(P1).go();
        t.resolve();
    };
    bolt(&mut t);
    assert_eq!(t.life(P1), 17);
    set_speed(&mut t, P0, 4);
    bolt(&mut t);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn a_triggered_ability_triggers_only_at_max_speed() {
    cr!("702.178a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Embalmed Ascendant");
    set_speed(&mut t, P0, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    set_speed(&mut t, P0, 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn an_activated_ability_can_be_activated_only_at_max_speed() {
    cr!("702.178a");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Endrider Catalyzer");
    set_speed(&mut t, P0, 3);
    assert!(t.activate(P0, cat, 0, &[]).is_err());
    assert!(!t.obj_now(cat).tapped);
    set_speed(&mut t, P0, 4);
    assert!(t.activate(P0, cat, 0, &[]).is_ok());
    assert!(t.obj_now(cat).tapped);
}

#[test]
fn a_granted_ability_functions_from_the_zone_it_states() {
    cr!("702.178a", "702.178b");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let surveyor = t.graveyard(P0, "Goblin Surveyor");
    t.lands(P0, "Mountain", 3);
    set_speed(&mut t, P0, 3);
    assert!(t.activate(P0, surveyor, 0, &[]).is_err());
    set_speed(&mut t, P0, 4);
    let hand = t.hand_size(P0);
    t.activate(P0, surveyor, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_exile("Goblin Surveyor"));
    assert_eq!(t.hand_size(P0), hand + 1);
}
