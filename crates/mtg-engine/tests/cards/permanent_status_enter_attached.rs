//! Putting permanents onto the battlefield attached to an object or player (CR 301.5e,
//! 303.4f–i): "return ~ from your graveyard to the battlefield attached to that
//! creature", "put an Aura card from your hand onto the battlefield attached to ~"
//! (pattern in `src/oracle/patterns/attach_control_grammar.rs`).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn attached(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.obj_now(id).attached_to
}

#[test]
fn enter_attached_cards_compile() {
    assert_compiles(&[
        "Academy Researchers",
        "Arachnus Spinner",
        "Bitterheart Witch",
        "Boonweaver Giant",
        "Danitha, Benalia's Hope",
        "Dragon Scales",
        "Dragon Shadow",
        "Eagle's Rescue",
        "Evershrike",
        "Gryff's Boon",
        "Hakim, Loreweaver",
        "Holy Avenger",
        "Iridescent Drake",
        "Magnetic Snuffler",
        "Mantle of the Ancients",
        "Nomad Mythmaker",
        "Runed Crown",
        "Smoke Shroud",
        "Unfinished Business",
    ]);
}

#[test]
fn dragon_scales_returns_attached_to_the_creature_that_entered() {
    cr!("303.4f", "603.6a");
    let mut t = TestGame::new(2);
    let scales = t.graveyard(P0, "Dragon Scales");
    t.answer_yes(P0, true);
    let wurm = t.enter(P0, "Craw Wurm");
    t.resolve_all();
    let scales = t.g.current(scales);
    assert!(t.on_battlefield(scales));
    assert_eq!(attached(&t, scales), Some(Entity::Object(wurm)));
    assert_eq!(t.pt(wurm), (7, 6));
}

#[test]
fn dragon_scales_stays_in_the_graveyard_if_the_creature_is_gone() {
    cr!("303.4i");
    let mut t = TestGame::new(2);
    let scales = t.graveyard(P0, "Dragon Scales");
    t.answer_yes(P0, true);
    let wurm = t.enter(P0, "Craw Wurm");
    t.settle();
    // The creature leaves before the ability resolves: the Aura has nothing to enter
    // attached to, so it stays where it is.
    t.g.destroy(wurm, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(scales)), Zone::Graveyard(P0));
}

#[test]
fn academy_researchers_puts_an_aura_from_hand_onto_it() {
    cr!("303.4f");
    let mut t = TestGame::new(2);
    let strength = t.hand(P0, "Holy Strength");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(strength)]);
    let researchers = t.enter(P0, "Academy Researchers");
    t.resolve_all();
    let strength = t.g.current(strength);
    assert!(t.on_battlefield(strength));
    assert_eq!(attached(&t, strength), Some(Entity::Object(researchers)));
    assert_eq!(t.pt(researchers), (3, 4));
}

#[test]
fn academy_researchers_cant_put_an_aura_that_cant_enchant_it() {
    cr!("303.4i");
    ruling!(
        "Academy Researchers",
        "You can't put an Aura card from your hand onto the battlefield this way if that Aura can't legally enchant Academy Researchers."
    );
    let mut t = TestGame::new(2);
    let growth = t.hand(P0, "Wild Growth");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(growth)]);
    t.enter(P0, "Academy Researchers");
    t.resolve_all();
    assert_eq!(t.zone(growth), Zone::Hand(P0));
}

#[test]
fn gryffs_boon_returns_from_the_graveyard_attached_to_target_creature() {
    cr!("303.4f", "602.5d");
    let mut t = TestGame::new(2);
    let boon = t.graveyard(P0, "Gryff's Boon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, boon, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    let boon = t.g.current(boon);
    assert!(t.on_battlefield(boon));
    assert_eq!(attached(&t, boon), Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn unfinished_business_returns_auras_and_equipment_attached_to_the_returned_creature() {
    cr!("303.4i", "301.5e");
    ruling!(
        "Unfinished Business",
        "Any target Equipment cards that can't legally be attached to the creature will enter the battlefield unattached."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let strength = t.graveyard(P0, "Holy Strength");
    let blade = t.graveyard(P0, "Bonesplitter");
    let spell = t.hand(P0, "Unfinished Business");
    t.lands(P0, "Plains", 5);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .target(Entity::Object(bears))
        .targets(&[Entity::Object(strength), Entity::Object(blade)])
        .go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    for a in [strength, blade] {
        let a = t.g.current(a);
        assert!(t.on_battlefield(a));
        assert_eq!(attached(&t, a), Some(Entity::Object(bears)));
    }
    assert_eq!(t.pt(bears), (5, 4));
}

#[test]
fn unfinished_business_an_aura_that_cant_enchant_the_creature_stays() {
    cr!("303.4i");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let growth = t.graveyard(P0, "Wild Growth");
    let spell = t.hand(P0, "Unfinished Business");
    t.lands(P0, "Plains", 5);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .target(Entity::Object(bears))
        .targets(&[Entity::Object(growth)])
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.zone(t.g.current(growth)), Zone::Graveyard(P0));
}

#[test]
fn nomad_mythmaker_chooses_a_creature_the_aura_can_enchant() {
    cr!("303.4f", "110.2a");
    ruling!(
        "Nomad Mythmaker",
        "You must choose a creature the Aura can legally enchant."
    );
    ruling!(
        "Nomad Mythmaker",
        "can target an Aura card in any graveyard, not just yours."
    );
    let mut t = TestGame::new(2);
    let mythmaker = t.battlefield(P0, "Nomad Mythmaker");
    // Black Knight has protection from white: Holy Strength can't enchant it.
    let knight = t.battlefield(P0, "Black Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.graveyard(P1, "Holy Strength");
    t.lands(P0, "Plains", 1);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, mythmaker, 0, &[Entity::Object(strength)])
        .unwrap();
    t.resolve_all();
    let strength = t.g.current(strength);
    assert!(t.on_battlefield(strength));
    assert_eq!(t.g.obj(strength).controller, P0);
    assert_eq!(attached(&t, strength), Some(Entity::Object(bears)));
    // The Black Knight wasn't offered.
    assert!(!t.asked().iter().any(|(_, d)| matches!(d,
        mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&Entity::Object(knight)))));
}

#[test]
fn bitterheart_witch_puts_a_curse_onto_the_battlefield_attached_to_target_player() {
    cr!("303.4f", "702.5d");
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Bitterheart Witch");
    let curse = t.library_top(P0, "Curse of the Pierced Heart");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(curse)]);
    t.g.destroy(witch, None);
    t.g.flush_events();
    t.resolve_all();
    let curse = t.g.current(curse);
    assert!(t.on_battlefield(curse));
    assert_eq!(attached(&t, curse), Some(Entity::Player(P1)));
}

#[test]
fn magnetic_snuffler_returns_target_equipment_attached_to_it() {
    cr!("301.5e");
    let mut t = TestGame::new(2);
    let blade = t.graveyard(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(blade)]);
    let snuffler = t.enter(P0, "Magnetic Snuffler");
    t.resolve_all();
    let blade = t.g.current(blade);
    assert!(t.on_battlefield(blade));
    assert_eq!(attached(&t, blade), Some(Entity::Object(snuffler)));
}
