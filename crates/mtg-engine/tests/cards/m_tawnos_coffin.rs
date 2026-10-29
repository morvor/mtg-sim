//! Tawnos's Coffin (hand-written, `src/cards/tawnos_coffin.rs`): it exiles a creature and
//! its Auras, noting the creature's counters; when the Coffin leaves the battlefield or
//! becomes untapped, the card returns tapped with those counters, and then its Auras
//! return attached to it.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Coffin exiles `target` (paying with three Swamps); returns the Coffin.
fn coffin_exiles(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let coffin = t.battlefield(P0, "Tawnos's Coffin");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, coffin, 0, &[Entity::Object(target)]).unwrap();
    t.resolve_all();
    coffin
}

#[test]
fn the_creature_returns_tapped_with_its_counters_and_its_auras() {
    cr!("607.2e", "603.7b", "303.4f");
    ruling!("Tawnos's Coffin", "The creature returns to the battlefield tapped. It does not return to the battlefield and then tap afterwards.");
    ruling!("Tawnos's Coffin", "the exiled card will suffer from summoning sickness upon its return");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    let aura = t.battlefield(P1, "Holy Strength");
    t.g.attach(aura, Entity::Object(bears));
    let coffin = coffin_exiles(&mut t, bears);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Holy Strength"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // It becomes untapped: the delayed trigger returns them.
    t.g.untap(coffin);
    t.settle();
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    let b = back[0];
    let o = t.obj_now(b);
    assert_eq!(o.controller, P1);
    assert!(o.tapped);
    assert_eq!(t.counters(b, "+1/+1"), 2);
    let auras = t.named_on_battlefield("Holy Strength");
    assert_eq!(auras.len(), 1);
    assert_eq!(t.obj_now(auras[0]).attached_to, Some(Entity::Object(b)));
    assert_eq!(t.obj_now(auras[0]).controller, P1);
    // 2/2 + two +1/+1 counters + Holy Strength's +1/+2.
    assert_eq!(t.pt(b), (5, 6));
    // A new object (CR 400.7): its controller hasn't controlled it since their turn began.
    assert!(t.obj_now(b).summoning_sick);
    // Untapping it again does nothing more (the delayed trigger triggered once).
    t.g.tap(coffin);
    t.g.untap(coffin);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn the_card_returns_when_the_coffin_leaves_the_battlefield() {
    cr!("603.7", "603.10a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let coffin = coffin_exiles(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    t.g.destroy(coffin, None);
    t.settle();
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert!(t.obj_now(back[0]).tapped);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

#[test]
fn a_token_ceases_to_exist_and_its_auras_stay_exiled() {
    cr!("111.8", "704.5d");
    ruling!("Tawnos's Coffin", "If the targeted creature is a token, it will cease to exist after being exiled. Any Auras that were attached to it will remain exiled forever.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Raise the Alarm: two 1/1 Soldier tokens.
    t.lands(P1, "Plains", 2);
    let raise = t.hand(P1, "Raise the Alarm");
    t.g.turn.priority = Some(P1);
    t.cast(P1, raise).go();
    t.resolve_all();
    let token = t.named_on_battlefield("Soldier Token");
    let token = *token.first().expect("a Soldier token");
    let aura = t.battlefield(P1, "Holy Strength");
    t.g.attach(aura, Entity::Object(token));
    let coffin = coffin_exiles(&mut t, token);
    t.g.untap(coffin);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 1);
    assert!(t.named_on_battlefield("Holy Strength").is_empty());
    assert!(t.in_exile("Holy Strength"));
}

#[test]
fn if_the_coffin_leaves_before_the_ability_resolves_the_creature_stays_exiled() {
    cr!("603.7a");
    ruling!("Tawnos's Coffin", "If Tawnos’s Coffin leaves the battlefield before its ability has resolved");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let coffin = t.battlefield(P0, "Tawnos's Coffin");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, coffin, 0, &[Entity::Object(bears)]).unwrap();
    t.g.destroy(coffin, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn an_aura_that_cant_enchant_the_returned_permanent_stays_exiled() {
    cr!("303.4f", "702.16c");
    ruling!("Tawnos's Coffin", "it now can’t be enchanted by an Aura that was also exiled by Tawnos’s Coffin, that Aura will remain exiled");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Unholy Strength");
    t.g.attach(aura, Entity::Object(bears));
    let coffin = coffin_exiles(&mut t, bears);
    // "All creatures have protection from black": the black Aura can't enchant it now.
    t.battlefield(P1, "Absolute Grace");
    t.g.untap(coffin);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.named_on_battlefield("Unholy Strength").is_empty());
    assert!(t.in_exile("Unholy Strength"));
}
