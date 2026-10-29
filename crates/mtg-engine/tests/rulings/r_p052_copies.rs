//! Rulings batch P052 — token copies of artifacts (Esoteric Duplicator, Schema Thief):
//! the copy's enters abilities (CR 707.4, 614.1c), {X} in a copied mana cost (CR 707.2,
//! 202.3e), copying a token (CR 707.2, 111.10) and copying an artifact that was itself
//! copying something (CR 707.3), using the sacrificed artifact's last known information
//! (CR 608.2h).

use crate::r_p046_common::*;
use crate::r_p052_common::*;
use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls Esoteric Duplicator (and Krark-Clan Ironworks: "Sacrifice an artifact: Add
/// {C}{C}."); returns the Ironworks.
fn duplicator(t: &mut TestGame) -> ObjectId {
    supported("Esoteric Duplicator");
    t.battlefield(P0, "Esoteric Duplicator");
    t.battlefield(P0, "Krark-Clan Ironworks")
}

/// P0 sacrifices `artifact` to the Ironworks, pays {2} for the Duplicator's trigger, and
/// the game goes to the end step, where the copy is created. Returns P0's tokens.
fn sacrifice_and_copy(t: &mut TestGame, ironworks: ObjectId, artifact: ObjectId) -> Vec<ObjectId> {
    t.answer_choose(P0, &[obj(artifact)]);
    assert!(tap_for_mana(t, P0, ironworks, "Add {C}{C}"));
    copy_at_end_step(t)
}

/// Pays {2} for the Duplicator's trigger and goes to the end step.
fn copy_at_end_step(t: &mut TestGame) -> Vec<ObjectId> {
    t.settle();
    assert_eq!(triggers_on_stack(t, "you may pay {2}"), 1);
    t.answer_yes(P0, true);
    fill_pool(t, P0, 2);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    tokens(t, P0)
}

#[test]
fn a_token_copy_of_an_artifact_has_its_enters_abilities() {
    cr!("707.4", "614.1c", "603.6a");
    ruling!(
        "Esoteric Duplicator",
        "Any enters-the-battlefield abilities of the copied artifact will trigger when the token enters the battlefield. Any “as [this artifact] enters the battlefield” or “[this artifact] enters the battlefield with” abilities of the copied artifact will also work."
    );
    // Shriekhorn: "This artifact enters with three charge counters on it."
    let mut t = TestGame::new(2);
    let ironworks = duplicator(&mut t);
    let horn = t.battlefield(P0, "Shriekhorn");
    let toks = sacrifice_and_copy(&mut t, ironworks, horn);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Shriekhorn");
    assert_eq!(t.counters(toks[0], "charge"), 3);
    // Elsewhere Flask: "When this artifact enters, draw a card." (sacrificed by its own
    // ability).
    let mut t = TestGame::new(2);
    duplicator(&mut t);
    let flask = t.battlefield(P0, "Elsewhere Flask");
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    activate_containing(&mut t, P0, flask, "Choose a basic land type").expect("flask");
    let hand = t.hand_size(P0);
    let toks = copy_at_end_step(&mut t);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Elsewhere Flask");
    assert_eq!(t.hand_size(P0), hand + 1);
    // Schema Thief: "Whenever this creature deals combat damage to a player, create a
    // token that's a copy of target artifact that player controls."
    supported("Schema Thief");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Schema Thief");
    old(&mut t, thief);
    let horn = t.battlefield(P1, "Shriekhorn");
    t.g.objects[horn.0 as usize].counters.clear();
    t.answer_targets(P0, &[obj(horn)]);
    attack_with(&mut t, &[(thief, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Shriekhorn");
    assert_eq!(t.counters(toks[0], "charge"), 3);
}

#[test]
fn a_copied_x_in_the_mana_cost_is_zero() {
    cr!("707.2", "202.3e", "107.3f");
    ruling!(
        "Esoteric Duplicator",
        "If the copied artifact had {X} in its mana cost, X is 0."
    );
    // Chalice of the Void ({X}{X}): "This artifact enters with X charge counters on it."
    let mut t = TestGame::new(2);
    let ironworks = duplicator(&mut t);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    t.g.objects[chalice.0 as usize]
        .counters
        .insert("charge".into(), 2);
    let toks = sacrifice_and_copy(&mut t, ironworks, chalice);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Chalice of the Void");
    assert_eq!(t.counters(toks[0], "charge"), 0);
    assert_eq!(t.g.mana_value_of(toks[0]), 0);
}

#[test]
fn a_copy_of_a_token_uses_the_tokens_original_characteristics() {
    cr!("707.2", "111.10a");
    ruling!(
        "Esoteric Duplicator",
        "If the copied artifact was a token, the token that’s created copies the original characteristics of that token as stated by the effect that created that token."
    );
    supported("Ensoul Artifact");
    let mut t = TestGame::new(2);
    let ironworks = duplicator(&mut t);
    let swarming = t.hand(P0, "Swarming of Moria");
    fill_pool(&mut t, P0, 3);
    t.cast(P0, swarming).go();
    t.resolve_all();
    let treasure = with_subtype(&t, P0, "Treasure")[0];
    // Ensoul Artifact: the Treasure is a 5/5 artifact creature.
    let ensoul = t.battlefield(P0, "Ensoul Artifact");
    t.g.attach(ensoul, obj(treasure));
    t.g.recompute();
    assert!(t.obj_now(treasure).is(CardType::Creature));
    let before: Vec<ObjectId> = tokens(&t, P0);
    let toks = sacrifice_and_copy(&mut t, ironworks, treasure);
    let new: Vec<ObjectId> = toks.into_iter().filter(|x| !before.contains(x)).collect();
    assert_eq!(new.len(), 1);
    let copy = t.obj_now(new[0]);
    // (A token's name is its subtype plus "Token", CR 111.4.)
    assert_eq!(copy.chars.name, "Treasure Token");
    assert!(copy.chars.has_subtype("Treasure") && copy.is(CardType::Artifact));
    assert!(!copy.is(CardType::Creature));
    assert!(tap_for_mana(&mut t, P0, new[0], "Add one mana"));
}

#[test]
fn a_copy_of_a_copying_artifact_copies_what_it_copied() {
    cr!("707.3", "707.2", "608.2h");
    ruling!(
        "Esoteric Duplicator",
        "If the copied artifact was copying something else when it was last on the battlefield, then the token enters the battlefield as whatever that artifact copied."
    );
    supported("Sculpting Steel");
    let mut t = TestGame::new(2);
    let ironworks = duplicator(&mut t);
    let horn = t.battlefield(P1, "Shriekhorn");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(horn)]);
    let steel = t.enter(P0, "Sculpting Steel");
    t.settle();
    assert_eq!(t.obj_now(steel).chars.name, "Shriekhorn");
    let toks = sacrifice_and_copy(&mut t, ironworks, steel);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Shriekhorn");
    assert_eq!(t.counters(toks[0], "charge"), 3);
}

#[test]
fn three_dogs_copy_of_a_sacrificed_copying_aura_copies_what_it_copied() {
    cr!("707.3", "608.2h", "603.12");
    ruling!(
        "Three Dog, Galaxy News DJ",
        "If the copied Aura was copying something else, the tokens enter the battlefield as whatever that Aura was copying."
    );
    supported("Three Dog, Galaxy News DJ");
    supported("Copy Enchantment");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Three Dog, Galaxy News DJ");
    let bears = t.battlefield(P0, "Grizzly Bears");
    old(&mut t, dog);
    old(&mut t, bears);
    let wall = t.battlefield(P1, "Wall of Wood");
    // Holy Strength ("Enchanted creature gets +1/+2.") on P1's Wall; Copy Enchantment
    // enters as a copy of it, attached to Three Dog.
    let holy = t.battlefield(P1, "Holy Strength");
    t.g.attach(holy, obj(wall));
    t.answer_choose(P0, &[obj(holy)]);
    t.answer_choose(P0, &[obj(dog)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.settle();
    assert_eq!(t.obj_now(copy).chars.name, "Holy Strength");
    assert_eq!(t.obj_now(copy).attached_to, Some(obj(dog)));
    // Three Dog's attack trigger: pay {2} and sacrifice the copy.
    attack_with(&mut t, &[(dog, pl(P1)), (bears, pl(P1))]);
    fill_pool(&mut t, P0, 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(copy)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Copy Enchantment"));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Holy Strength");
    assert_eq!(t.obj_now(toks[0]).attached_to, Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn a_dies_trigger_copies_a_dead_clone_as_it_last_existed() {
    cr!("608.2h", "707.3");
    // The same rule as Esoteric Duplicator's: Ochre Jelly's "When ~ dies, if it had two or
    // more +1/+1 counters on it, create a token that's a copy of it at the beginning of
    // the next end step" on a Clone copying Ochre Jelly makes an Ochre Jelly (the Clone as
    // it last existed), not a copy of the Clone card in the graveyard.
    supported("Ochre Jelly");
    let mut t = TestGame::new(2);
    let jelly = t.battlefield(P1, "Ochre Jelly");
    t.g.objects[jelly.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.answer_choose(P0, &[obj(jelly)]);
    let clone = t.enter(P0, "Clone");
    // (A 0/0 copy: give it counters before state-based actions.)
    let now = t.g.current(clone);
    t.g.objects[now.0 as usize]
        .counters
        .insert("+1/+1".into(), 4);
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Ochre Jelly");
    t.g.destroy(clone, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Clone"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.obj_now(toks[0]).chars.name, "Ochre Jelly");
    assert_eq!(t.counters(toks[0], "+1/+1"), 2);
}
