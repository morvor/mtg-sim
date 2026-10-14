//! What a reflexive triggered ability's pronouns refer to (CR 603.12): after "create a
//! token", "it" / "that token" is the token ("When you do, attach it to target creature
//! you control"); after "sacrifice this enchantment", "it" is the enchantment; a
//! character's "he" / "she" is the card itself (pattern in
//! `src/oracle/patterns/r600_triggers.rs`).

use mtg_engine::ability::{Destination, Effect, Sel};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";

#[test]
fn ajani_nacatl_avenger_he_deals_the_damage_not_the_token() {
    cr!("603.12", "702.16e");
    assert!(card(AJANI).unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    // Ajani, Nacatl Avenger, put onto the battlefield transformed.
    let card = t.graveyard(P0, AJANI);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(card)]];
    let mut to = Destination::battlefield();
    to.transformed = true;
    t.g.exec(
        &Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.resolve_all();
    let ajani = t.g.current(card);
    assert_eq!(t.obj(ajani).chars.name, "Ajani, Nacatl Avenger");
    // A red permanent other than Ajani.
    t.battlefield(P0, "Hill Giant");
    // Protection from creatures: damage from the Cat Warrior token would be prevented
    // (CR 702.16e), damage from Ajani isn't.
    let chaplain = t.battlefield(P1, "Beloved Chaplain");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(chaplain)]);
    t.activate(P0, ajani, 1, &[]).expect("0 ability");
    t.resolve_all();
    assert!(!t.on_battlefield(chaplain));
}

#[test]
fn stensia_uprising_it_is_the_sacrificed_enchantment() {
    cr!("603.12", "702.16e");
    assert!(card("Stensia Uprising").unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stensia Uprising");
    // Twelve permanents; the Human token makes thirteen.
    t.lands(P0, "Mountain", 11);
    let chaplain = t.battlefield(P1, "Beloved Chaplain");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(chaplain)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Stensia Uprising"));
    // The enchantment (not the 1/1 Human token, a creature) dealt the 7 damage.
    assert!(!t.on_battlefield(chaplain));
}

#[test]
fn dain_ironfoot_attaches_the_axe_token_it_created() {
    cr!("603.12", "111.1");
    assert!(card("Dáin Ironfoot").unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Dáin Ironfoot");
    t.resolve_all();
    let axe = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|o| t.obj(*o).is_token() && t.obj(*o).chars.name == "Axe")
        .expect("Axe token");
    assert_eq!(t.obj(axe).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (3, 2));
}
