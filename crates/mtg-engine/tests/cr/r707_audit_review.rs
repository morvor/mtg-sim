//! CR 707.10 audit (review): a copy of a spell copies the decisions made for the spell
//! (X, alternative and additional costs) but isn't cast, so what happened as the original
//! was cast doesn't carry over: no mana was spent on the copy's X ("the amount of {B}
//! spent on X" is 0 for a copy of Soul Burn), and the copy wasn't cast "any time a sorcery
//! couldn't have been cast" (the token a copy of Spider Climb becomes stays).

use crate::r603_audit_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_soul_burn_had_no_black_mana_spent_on_its_x() {
    cr!("707.10", "107.3a");
    ruling!(
        "Soul Burn",
        "Will give 1 life for each black mana used, but not more life than the amount of unprevented damage that is dealt."
    );
    supported("Soul Burn");
    supported("Reverberate");
    // Soul Burn with X = 3, paid with three Swamps for X; Reverberate copies it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.lands(P0, "Mountain", 2);
    let burn = t.hand(P0, "Soul Burn");
    let spell = t.cast(P0, burn).target(P1).x(3).go();
    let rev = t.hand(P0, "Reverberate");
    t.answer_yes(P0, false);
    t.cast(P0, rev).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(copy).kind, ObjKind::SpellCopy);
    // The copy has the same X (it deals 3 damage) but no mana was spent on it: no life.
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20, "no {{B}} was spent on the copy's X");
    // The original: 3 damage, 3 life for the three {B} spent on its X.
    t.resolve();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 23);
}

/// Spider Climb's "as though it had flash" casting method.
fn own_flash(t: &TestGame, card: ObjectId) -> CastMethod {
    t.g.cast_options(P0, card)
        .into_iter()
        .find(|o| o.flash)
        .expect("flash option")
        .method
}

#[test]
fn the_token_a_copy_of_spider_climb_cast_at_instant_speed_becomes_isnt_sacrificed() {
    cr!("707.10", "707.10f", "307.5a");
    ruling!(
        "Spider Climb",
        "The sacrifice occurs only if you cast it using its own ability"
    );
    supported("Spider Climb");
    supported("Lithoform Engine");
    // Spider Climb cast with its own ability during the opponent's turn; Lithoform Engine
    // copies it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Forest", 1);
    let climb = t.hand(P0, "Spider Climb");
    t.set_step(P1, Step::DeclareAttackers);
    let m = own_flash(&t, climb);
    let spell = t.cast(P0, climb).method(m).target(bears).go();
    t.g.players[0].mana_pool.add_type(ManaType::C, 4);
    t.activate(P0, engine, 2, &[Entity::Object(spell)])
        .expect("Lithoform Engine");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spider Climb").len(), 2);
    assert_eq!(t.pt(bears), (2, 8));
    // At the next cleanup step the card is sacrificed; the copy wasn't cast at all, so
    // the token it became stays.
    t.advance_to(P0, Step::Upkeep);
    let left = t.named_on_battlefield("Spider Climb");
    assert_eq!(left.len(), 1);
    assert_eq!(t.g.obj(left[0]).kind, ObjKind::Token);
    assert!(t.in_graveyard(P0, "Spider Climb"));
    assert_eq!(t.pt(bears), (2, 5));
}

#[test]
fn a_ward_creature_becomes_the_target_of_a_copy_once_whether_or_not_the_target_changed() {
    cr!("707.10c", "115.7d", "702.21a");
    supported("Rimeshield Frost Giant");
    supported("Reverberate");
    // P1's Lightning Bolt targets P0; P1 copies it with Reverberate and chooses P0's Giant
    // (ward {3}) as the copy's new target: the Giant becomes the target of the copy once.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Rimeshield Frost Giant");
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    t.settle();
    assert_eq!(triggers_of(&t, "Rimeshield Frost Giant"), 0);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    t.resolve();
    t.settle();
    let copy = t.g.stack[1];
    assert_eq!(
        mtg_engine::copy::chosen_targets(&t.g, copy),
        vec![Entity::Object(giant)]
    );
    assert_eq!(triggers_of(&t, "Rimeshield Frost Giant"), 1);
    // A copy whose target is left unchanged makes the Giant its target as it's created:
    // ward triggers for the Bolt and once more for the copy.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Rimeshield Frost Giant");
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(giant).go();
    t.settle();
    assert_eq!(triggers_of(&t, "Rimeshield Frost Giant"), 1);
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    t.answer_yes(P1, false);
    t.resolve();
    t.settle();
    assert_eq!(triggers_of(&t, "Rimeshield Frost Giant"), 2);
}
