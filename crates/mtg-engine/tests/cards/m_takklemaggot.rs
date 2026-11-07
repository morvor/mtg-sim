//! Takklemaggot (hand-written, `src/cards/takklemaggot.rs`): when the enchanted creature
//! dies, its controller chooses a creature for it to enchant; with none, it returns as a
//! non-Aura enchantment that damages that player each upkeep.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn takk_now(t: &TestGame) -> ObjectId {
    let v = t.named_on_battlefield("Takklemaggot");
    assert_eq!(v.len(), 1, "Takklemaggot on the battlefield");
    v[0]
}

fn kill(t: &mut TestGame, creature: ObjectId) {
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(creature).go();
    t.resolve_all();
}

#[test]
fn it_returns_attached_to_a_creature_its_controller_chooses() {
    cr!("303.4f");
    ruling!("Takklemaggot", "The player who controlled the creature that went to the graveyard");
    ruling!("Takklemaggot", "The player who controlled it when it left the battlefield is the same player that controls it");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Listed first, so a default (or the wrong player's) choice would pick it.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let takk = t.battlefield(P0, "Takklemaggot");
    t.g.attach(takk, Entity::Object(bears));
    t.answer_choose(P1, &[Entity::Object(giant)]);
    kill(&mut t, bears);
    let now = takk_now(&t);
    assert_eq!(t.obj_now(now).controller, P0);
    assert_eq!(t.obj_now(now).attached_to, Some(Entity::Object(giant)));
    assert!(t.on_battlefield(elves));
}

#[test]
fn with_no_creature_it_returns_as_a_non_aura_enchantment_that_damages_that_player() {
    cr!("303.4i");
    ruling!("Takklemaggot", "Takklemaggot returns to the battlefield as a non-Aura enchantment");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let takk = t.battlefield(P0, "Takklemaggot");
    t.g.attach(takk, Entity::Object(bears));
    kill(&mut t, bears);
    let now = takk_now(&t);
    let o = t.obj_now(now);
    assert_eq!(o.controller, P0);
    assert_eq!(o.attached_to, None);
    assert!(!o.chars.subtypes.iter().any(|s| s.as_str() == "Aura"));
    // At the beginning of that player's upkeep it deals them 1 damage; not on the
    // controller's.
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.life(P1), 19);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(now));
}

#[test]
fn if_the_creature_is_exiled_it_just_goes_to_the_graveyard() {
    cr!("303.4c", "704.5m");
    ruling!("Takklemaggot", "If the creature leaves the battlefield without going to the graveyard, Takklemaggot is simply put into its owner's graveyard.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let takk = t.battlefield(P0, "Takklemaggot");
    t.g.attach(takk, Entity::Object(bears));
    t.lands(P0, "Plains", 1);
    let path = t.hand(P0, "Swords to Plowshares");
    t.cast(P0, path).target(bears).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Takklemaggot").is_empty());
    assert!(t.in_graveyard(P0, "Takklemaggot"));
}

#[test]
fn it_returns_only_if_still_in_the_graveyard() {
    cr!("603.10a", "400.7");
    ruling!("Takklemaggot", "When the ability resolves, if Takklemaggot is still in that graveyard, it's returned to the battlefield.");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let takk = t.battlefield(P0, "Takklemaggot");
    t.g.attach(takk, Entity::Object(bears));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    t.settle();
    // The trigger is on the stack and Takklemaggot is in the graveyard: exile it first.
    assert_eq!(t.stack_len(), 1);
    let in_gy = t.g.current(takk);
    assert!(matches!(t.zone(in_gy), Zone::Graveyard(_)));
    t.g.exile_object(in_gy, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Takklemaggot").is_empty());
    assert!(t.in_exile("Takklemaggot"));
}
