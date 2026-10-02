//! Animate Dead, Dance of the Dead and Necromancy: Auras that enchant a card in a graveyard
//! (CR 303.4a, 702.5a), swap their enchant ability in layer 6, and enchant only the
//! creature they put onto the battlefield (CR 607.2c, 704.5m).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

/// P0 casts `aura` (with Swamps) targeting `card`; the Aura spell resolves. Returns the
/// Aura spell's card.
fn cast_aura_on(t: &mut TestGame, aura: &str, swamps: usize, card: ObjectId) -> ObjectId {
    t.lands(P0, "Swamp", swamps);
    let a = t.hand(P0, aura);
    t.g.turn.priority = Some(P0);
    t.cast(P0, a).target(card).go();
    t.resolve();
    t.g.current(a)
}

fn enchant_text(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Enchant)
        .map(|k| k.text.as_deref().unwrap_or("").to_lowercase())
        .collect()
}

#[test]
fn animate_dead_enters_attached_to_the_card_then_returns_it_and_attaches_again() {
    cr!("303.4a", "702.5a", "704.5m", "613.1f", "607.2c", "400.7");
    ruling!(
        "Animate Dead",
        "You target a creature card in a graveyard when you cast it. It enters the battlefield attached to that card. Then it returns that card to the battlefield, and attaches itself to the card again"
    );
    ruling!(
        "Dance of the Dead",
        "It enters attached to that card. Then it returns that card to the battlefield, and attaches itself to that card again"
    );
    assert_supported("Animate Dead");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, bears);
    // The Aura is on the battlefield, attached to the card in the graveyard; state-based
    // actions leave it there.
    t.settle();
    assert!(t.on_battlefield(ad));
    assert_eq!(t.obj_now(ad).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(enchant_text(&t, ad), vec!["enchant creature card in a graveyard"]);
    // Its enters ability returns the card under P0's control and attaches the Aura to the
    // new object.
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    assert_ne!(now, bears);
    assert_eq!(t.obj_now(now).controller, P0);
    let ad_now = t.g.current(ad);
    // Animate Dead never moved.
    assert_eq!(ad_now, ad);
    assert!(t.on_battlefield(ad));
    assert_eq!(t.obj_now(ad).attached_to, Some(Entity::Object(now)));
    assert_eq!(
        enchant_text(&t, ad),
        vec!["enchant creature put onto the battlefield with ~"]
    );
    // Enchanted creature gets -1/-0.
    assert_eq!(t.pt(now), (1, 2));
}

#[test]
fn animate_dead_can_target_a_creature_card_with_shroud() {
    cr!("702.18a", "303.4a");
    ruling!(
        "Animate Dead",
        "A creature card with shroud may be targeted by Animate Dead, and Animate Dead will become attached to the creature that enters the battlefield."
    );
    let mut t = TestGame::new(2);
    let ench = t.graveyard(P0, "Argothian Enchantress");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, ench);
    t.resolve_all();
    let now = t.g.current(ench);
    assert!(t.on_battlefield(now));
    assert!(t.obj_now(now).chars.has_keyword(KeywordKind::Shroud));
    assert_eq!(t.obj_now(ad).attached_to, Some(Entity::Object(now)));
}

#[test]
fn a_creature_with_protection_from_black_is_returned_then_sacrificed() {
    cr!("702.16c", "704.5m", "603.7a", "303.4c");
    ruling!(
        "Animate Dead",
        "If the creature put onto the battlefield has protection from black—or if the creature can't legally be enchanted by Animate Dead for another reason—Animate Dead won't be able to attach to it."
    );
    ruling!(
        "Necromancy",
        "If the creature card put onto the battlefield has protection from black (or anything that prevents this from legally being attached), this won't be able to attach to it."
    );
    let mut t = TestGame::new(2);
    // Protection works only on the battlefield: the card in the graveyard can be targeted
    // and enchanted.
    let knight = t.graveyard(P1, "White Knight");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, knight);
    t.settle();
    assert_eq!(t.obj_now(ad).attached_to, Some(Entity::Object(knight)));
    // The ability resolves: the Knight returns; Animate Dead can't attach to it and goes
    // to the graveyard; its delayed ability has the Knight sacrificed.
    t.resolve();
    let now = t.g.current(knight);
    assert!(t.on_battlefield(now));
    t.settle();
    assert!(t.in_graveyard(P0, "Animate Dead"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "White Knight"));
    assert!(t.named_on_battlefield("White Knight").is_empty());
}

#[test]
fn when_animate_dead_leaves_the_battlefield_the_creature_is_sacrificed() {
    cr!("603.7a", "603.7c", "701.21a");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, bears);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    // P1 destroys Animate Dead.
    t.lands(P1, "Plains", 2);
    let d = t.hand(P1, "Disenchant");
    t.g.turn.priority = Some(P1);
    t.cast(P1, d).target(ad).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Animate Dead"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn if_animate_dead_left_before_its_ability_resolves_nothing_happens() {
    cr!("603.4", "608.2b");
    ruling!(
        "Animate Dead",
        "If Animate Dead isn't on the battlefield as its triggered ability resolves, none of its effects happen."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, bears);
    t.settle();
    // In response to the enters ability, P1 destroys Animate Dead.
    t.lands(P1, "Plains", 2);
    let d = t.hand(P1, "Disenchant");
    t.g.turn.priority = Some(P1);
    t.cast(P1, d).target(ad).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Animate Dead"));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0), "{}", t.dump_log());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn if_the_enchanted_card_leaves_the_graveyard_the_aura_is_put_into_the_graveyard() {
    cr!("704.5m", "303.4c");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, bears);
    // The card is exiled in response to the enters ability.
    let exiled = t
        .g
        .move_object(bears, Zone::Exile, events::MoveCause::Effect, Some(P1))
        .unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Animate Dead"));
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.g.current(ad) != ad);
}

#[test]
fn animate_dead_cant_be_moved_to_another_creature() {
    cr!("303.4j", "701.3b");
    ruling!(
        "Animate Dead",
        "Once the creature is returned to the battlefield, Animate Dead can't be attached to anything other than it"
    );
    ruling!(
        "Dance of the Dead",
        "Once the creature is returned to the battlefield, Dance of the Dead can't be attached to anything other than it"
    );
    let mut t = TestGame::new(2);
    let other = t.battlefield(P0, "Grizzly Bears");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let ad = cast_aura_on(&mut t, "Animate Dead", 2, bears);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(!t.g.attach(ad, Entity::Object(other)));
    assert_eq!(t.obj_now(ad).attached_to, Some(Entity::Object(now)));
}

#[test]
fn dance_of_the_dead_returns_the_creature_tapped() {
    cr!("303.4a", "702.5a", "613.1f");
    assert!(mtg_engine::card::card("Dance of the Dead")
        .unsupported_text()
        .is_empty());
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let dance = cast_aura_on(&mut t, "Dance of the Dead", 2, bears);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P0);
    assert!(t.obj_now(now).tapped);
    assert_eq!(t.obj_now(dance).attached_to, Some(Entity::Object(now)));
    // Enchanted creature gets +1/+1.
    assert_eq!(t.pt(now), (3, 3));
}

#[test]
fn necromancy_becomes_an_aura_attached_to_the_creature_it_returns() {
    cr!("303.4a", "205.3h", "611.2c", "607.2c");
    ruling!(
        "Necromancy",
        "Necromancy enters as an enchantment and then becomes an Enchant Creature Aura as a triggered ability upon entering."
    );
    ruling!(
        "Necromancy",
        "The bringing of the creature onto the battlefield and then putting Necromancy on it is all done as part of the resolution."
    );
    assert_supported("Necromancy");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let n = t.hand(P0, "Necromancy");
    t.g.turn.priority = Some(P0);
    t.cast(P0, n).go();
    t.resolve();
    let n = t.g.current(n);
    // It entered as an enchantment that's not an Aura.
    assert!(t.on_battlefield(n));
    assert!(!t.obj_now(n).chars.has_subtype("Aura"));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P0);
    assert!(t.obj_now(n).chars.has_subtype("Aura"));
    assert_eq!(t.obj_now(n).attached_to, Some(Entity::Object(now)));
    assert_eq!(
        enchant_text(&t, n),
        vec!["enchant creature put onto the battlefield with ~"]
    );
    // It stays attached (the creature is the one it put onto the battlefield).
    t.settle();
    assert!(t.on_battlefield(n));
}

#[test]
fn necromancy_cast_at_instant_speed_is_sacrificed_at_cleanup() {
    cr!("603.7a");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.lands(P0, "Swamp", 3);
    let n = t.hand(P0, "Necromancy");
    t.g.turn.priority = Some(P0);
    // "You may cast this spell as though it had flash."
    let m = t
        .g
        .cast_options(P0, n)
        .into_iter()
        .find(|o| o.flash)
        .expect("flash option")
        .method;
    t.cast(P0, n).method(m).go();
    t.resolve();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now));
    // At the beginning of the cleanup step, Necromancy is sacrificed; then the creature.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Necromancy"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
