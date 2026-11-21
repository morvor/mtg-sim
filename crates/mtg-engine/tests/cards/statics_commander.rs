//! Statics about commanders ("commander creatures you control", "commanders you
//! control", lieutenant), compound lines with several subjects, and "~ and enchanted
//! creature each get".

use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
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

/// A permanent that is `p`'s commander.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    id
}

#[test]
fn commander_creatures_you_control_get_a_bonus() {
    cr!("903.3", "613.4c");
    ruling!(
        "Bastion Protector",
        "applies to any commander creature you control, whether you own it or not"
    );
    compiles("Bastion Protector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bastion Protector");
    let mine = commander(&mut t, P0, "Grizzly Bears");
    let theirs = commander(&mut t, P1, "Grizzly Bears");
    let plain = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(mine), (4, 4));
    assert!(has(&t, mine, KeywordKind::Indestructible));
    assert_eq!(t.pt(theirs), (2, 2));
    assert_eq!(t.pt(plain), (2, 2));
    // An opponent's commander under your control counts too.
    let mc = t.battlefield(P0, "Mind Control");
    t.attach(mc, Entity::Object(theirs));
    t.settle();
    assert_eq!(t.pt(theirs), (4, 4));
}

#[test]
fn commanders_you_control_have_keywords() {
    cr!("903.3", "613.1f");
    compiles("Falthis, Shadowcat Familiar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Falthis, Shadowcat Familiar");
    let mine = commander(&mut t, P0, "Grizzly Bears");
    let plain = t.battlefield(P0, "Hill Giant");
    assert!(has(&t, mine, KeywordKind::Menace));
    assert!(has(&t, mine, KeywordKind::Deathtouch));
    assert!(!has(&t, plain, KeywordKind::Menace));
}

#[test]
fn commander_creatures_you_own_have_a_triggered_ability() {
    cr!("903.3", "613.1f", "603.6a");
    ruling!(
        "Candlekeep Sage",
        "each of them will have that ability"
    );
    compiles("Candlekeep Sage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Candlekeep Sage");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].is_commander = true;
    let before = t.hand_size(P0);
    let on_bf = t
        .g
        .move_object(bears, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.settle();
    t.resolve_all();
    // "When this creature enters ..., draw a card."
    assert_eq!(t.hand_size(P0), before);
    assert!(t.obj_now(on_bf).is_commander);
    // A second commander you own has it too.
    t.library_top(P0, "Island");
    let giant = t.hand(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].is_commander = true;
    let size = t.hand_size(P0);
    t.g.move_object(giant, Zone::Battlefield, MoveCause::Effect, Some(P0))
        .unwrap();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), size);
    // A non-commander creature doesn't have it.
    let other = t.hand(P0, "Grizzly Bears");
    let size = t.hand_size(P0);
    t.g.move_object(other, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), size - 1);
}

#[test]
fn lieutenant_applies_only_while_you_control_your_commander() {
    cr!("611.3a", "903.3");
    ruling!(
        "Angelic Field Marshal",
        "Lieutenant abilities apply only if your commander is on the battlefield and under your control."
    );
    compiles("Angelic Field Marshal");
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P0, "Angelic Field Marshal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(marshal), (3, 3));
    assert!(!has(&t, bears, KeywordKind::Vigilance));
    // Another player's commander doesn't count.
    commander(&mut t, P1, "Hill Giant");
    assert_eq!(t.pt(marshal), (3, 3));
    commander(&mut t, P0, "Hill Giant");
    assert_eq!(t.pt(marshal), (5, 5));
    assert!(has(&t, bears, KeywordKind::Vigilance));
    assert!(has(&t, marshal, KeywordKind::Vigilance));
}

#[test]
fn lieutenant_with_two_subjects() {
    cr!("611.3a", "613.4c");
    compiles("Thunderfoot Baloth");
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P0, "Thunderfoot Baloth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    commander(&mut t, P0, "Hill Giant");
    // "~ gets +2/+2 and other creatures you control get +2/+2 and have trample."
    assert_eq!(t.pt(baloth), (7, 7));
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has(&t, bears, KeywordKind::Trample));
}

#[test]
fn this_and_enchanted_creature_each_get() {
    cr!("613.4c", "113.6");
    ruling!(
        "Nighthowler",
        "Nighthowler's last ability functions only from the battlefield."
    );
    compiles("Nighthowler");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nighthowler");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Island");
    t.settle();
    assert_eq!(t.pt(n), (2, 2));
    // A Nighthowler card in a graveyard counts, but its own ability doesn't work there.
    let dead = t.graveyard(P1, "Nighthowler");
    t.settle();
    assert_eq!(t.pt(n), (3, 3));
    assert_eq!(t.pt(dead), (0, 0));
}
