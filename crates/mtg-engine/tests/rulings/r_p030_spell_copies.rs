//! Rulings batch P030 — copying instant and sorcery spells: a copy is created on the stack,
//! so it isn't cast and doesn't trigger "whenever you cast" abilities; it resolves like a
//! normal spell, before the original (CR 707.10, 707.10c); the copy copies the choices
//! made for the original, including any additional costs paid for it (CR 707.10); the
//! "you may pay" of a copying trigger is chosen on resolution (CR 603.5), its modes as it's
//! put on the stack (CR 700.2b); copies of permanent spells (CR 707.10f).

use crate::r_p030_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::*;
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// --- A copy isn't cast -------------------------------------------------------------------

#[test]
fn reverberate_s_copy_isnt_cast_and_resolves_like_a_normal_spell() {
    cr!("707.10", "707.10c", "601.2");
    ruling!(
        "Reverberate",
        "When Reverberate resolves, it creates a copy of a spell. You control the copy. That copy is created on the stack, so it's not \"cast.\""
    );
    supported("Reverberate");
    supported("Young Pyromancer");
    // P1 casts Divination ("Draw two cards."); P0, who controls Young Pyromancer, copies it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    let div = cast_new(&mut t, P1, "Divination", &[]);
    cast_new(&mut t, P0, "Reverberate", &[obj(div)]);
    // Young Pyromancer's trigger for Reverberate, then Reverberate.
    t.resolve();
    t.resolve();
    assert_eq!(elementals(&t, P0), 1);
    // The copy is on the stack above Divination, controlled by P0; no new trigger.
    let copy = spell_copies(&t)[0];
    assert_eq!(*t.g.stack.last().unwrap(), copy);
    assert_eq!(t.obj(copy).controller, P0);
    assert_eq!(t.stack_len(), 2);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1 + 2);
}

#[test]
fn howl_of_the_horde_s_copy_isnt_cast() {
    cr!("707.10", "707.10c", "603.7");
    ruling!(
        "Howl of the Horde",
        "When either ability resolves, it creates a copy of the instant or sorcery spell. You control each copy. Each copy is created on the stack, so it's not \"cast.\""
    );
    supported("Howl of the Horde");
    // "When you next cast an instant or sorcery spell this turn, copy that spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    cast_new(&mut t, P0, "Howl of the Horde", &[]);
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    // Young Pyromancer's trigger and the delayed trigger; resolving them copies Divination.
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    t.resolve_all();
    // Two spells were cast (Howl and Divination); Divination resolved twice.
    assert_eq!(elementals(&t, P0), 2);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn meletis_charlatan_s_copy_is_controlled_by_the_spell_s_controller() {
    cr!("707.10", "707.10c");
    ruling!(
        "Meletis Charlatan",
        "When the ability resolves, it creates a copy of the spell. The controller of the original spell controls the copy. That copy is created on the stack, so it's not “cast.”"
    );
    supported("Meletis Charlatan");
    // "{2}{U}, {T}: The controller of target instant or sorcery spell copies it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let charlatan = t.battlefield(P1, "Meletis Charlatan");
    let div = cast_new(&mut t, P0, "Divination", &[]);
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 2);
    t.answer_targets(P1, &[obj(div)]);
    activate_containing(&mut t, P1, charlatan, "copies it").unwrap();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    assert_eq!(t.hand_size(P0), h0 + 4);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn chandra_s_delayed_trigger_creates_a_copy_that_isnt_cast() {
    cr!("707.10", "707.10c", "603.7");
    ruling!(
        "Chandra, the Firebrand",
        "When the delayed trigger resolves, it creates a copy of a spell. You control the copy. That copy is created on the stack, so it's not \"cast.\""
    );
    supported("Chandra, the Firebrand");
    // "−2: When you next cast an instant or sorcery spell this turn, copy that spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let chandra = t.battlefield(P0, "Chandra, the Firebrand");
    activate_containing(&mut t, P0, chandra, "When you next cast").unwrap();
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 1);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn storm_king_s_thunder_creates_x_copies_that_arent_cast() {
    cr!("707.10", "707.10c", "603.7");
    ruling!(
        "Storm King's Thunder",
        "When the delayed triggered ability of Storm King's Thunder resolves, it creates X copies of a spell. You control each of the copies. Those copies are created on the stack, so they're not \"cast.\""
    );
    supported("Storm King's Thunder");
    // "When you next cast an instant or sorcery spell this turn, copy that spell X times."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 2);
    let skt = t.hand(P0, "Storm King's Thunder");
    t.cast(P0, skt).x(2).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| t.obj(*c).controller == P0));
    t.resolve_all();
    assert_eq!(elementals(&t, P0), 2);
    assert_eq!(t.hand_size(P0), hand + 6);
}

// --- Additional costs paid for the original are copied -----------------------------------

#[test]
fn storm_king_s_thunder_copies_of_fling_deal_the_sacrificed_power() {
    cr!("707.10", "118.8");
    ruling!(
        "Storm King's Thunder",
        "if you sacrifice a 3/3 creature to cast Fling and then copy it with Storm King's Thunder, each copy of Fling will also deal 3 damage to its target."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 2);
    let skt = t.hand(P0, "Storm King's Thunder");
    t.cast(P0, skt).x(2).go();
    t.resolve_all();
    let creatures = crate::r_s01_common::creatures(&t, P0).len();
    fling_giant(&mut t, P0, Entity::Player(P1));
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    // Three Flings, 3 damage each; no other creature was sacrificed for the copies.
    assert_eq!(t.life(P1), 11);
    assert_eq!(crate::r_s01_common::creatures(&t, P0).len(), creatures);
}

#[test]
fn reverberate_s_copy_of_fling_deals_the_sacrificed_power() {
    cr!("707.10", "118.8", "115.7");
    ruling!(
        "Reverberate",
        "if a player sacrifices a 3/3 creature to cast Fling, and you copy it with Reverberate, the copy of Fling will also deal 3 damage to its target."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let fling = fling_giant(&mut t, P1, Entity::Player(P0));
    cast_new(&mut t, P0, "Reverberate", &[obj(fling)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn chandra_s_copy_of_fling_deals_the_sacrificed_power() {
    cr!("707.10", "118.8");
    ruling!(
        "Chandra, the Firebrand",
        "if you sacrifice a 3/3 creature to cast Fling after activating Chandra's second ability, the copy of Fling will also deal 3 damage."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, the Firebrand");
    activate_containing(&mut t, P0, chandra, "When you next cast").unwrap();
    t.resolve_all();
    fling_giant(&mut t, P0, Entity::Player(P1));
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn dualcaster_mage_s_copy_of_fling_deals_the_sacrificed_power() {
    cr!("707.10", "118.8");
    ruling!(
        "Dualcaster Mage",
        "effects based on any additional or alternative costs paid for the targeted spell are copied as though those same costs were paid for the copy."
    );
    supported("Dualcaster Mage");
    // "Flash. When this creature enters, copy target instant or sorcery spell."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let fling = fling_giant(&mut t, P1, Entity::Player(P0));
    cast_new(&mut t, P0, "Dualcaster Mage", &[]);
    t.answer_targets(P0, &[obj(fling)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn jin_gitaxias_s_copy_of_fling_deals_the_sacrificed_power() {
    cr!("707.10", "118.8");
    ruling!(
        "Jin-Gitaxias, Progress Tyrant",
        "You can't choose to pay any additional costs for the copy created by Jin-Gitaxias's first ability."
    );
    supported("Jin-Gitaxias, Progress Tyrant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jin-Gitaxias, Progress Tyrant");
    fling_giant(&mut t, P0, Entity::Player(P1));
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn najal_s_copy_of_fling_deals_the_sacrificed_power() {
    cr!("707.10", "118.8", "603.7");
    ruling!(
        "Najal, the Storm Runner",
        "You can't choose to pay any additional costs for the copy created by Najal's delayed triggered ability."
    );
    supported("Najal, the Storm Runner");
    // "Whenever Najal attacks, you may pay {2}. If you do, when you next cast an instant or
    // sorcery spell this turn, copy it."
    let mut t = TestGame::new(2);
    let najal = t.battlefield(P0, "Najal, the Storm Runner");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(najal, Entity::Player(P1))]);
    t.resolve_all();
    fling_giant(&mut t, P0, Entity::Player(P1));
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn lithoform_engine_copies_a_kicked_spell_as_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Lithoform Engine",
        "Most notably, if the original spell was kicked, the copy is kicked."
    );
    supported("Lithoform Engine");
    supported("Burst Lightning");
    // Burst Lightning: "Kicker {4}. Burst Lightning deals 2 damage to any target. If this
    // spell was kicked, it deals 4 damage instead."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let bl = t.hand(P0, "Burst Lightning");
    let bl = t.cast(P0, bl).target(Entity::Player(P1)).kicked(true).go();
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[obj(bl)]);
    activate_containing(&mut t, P0, engine, "instant or sorcery spell").unwrap();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

// --- Chandra's ultimate -----------------------------------------------------------------

#[test]
fn chandra_deals_6_damage_to_each_still_legal_target() {
    cr!("115.1c", "608.2b");
    ruling!(
        "Chandra, the Firebrand",
        "No damage is divided, and Chandra can't deal more than 6 damage to any one target. Chandra deals 6 damage to each target that is still legal when the ability resolves."
    );
    // "−6: Chandra deals 6 damage to each of up to six targets."
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, the Firebrand");
    t.g.objects[chandra.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(
        P0,
        &[Entity::Player(P1), obj(giant), obj(bears)],
    );
    activate_containing(&mut t, P0, chandra, "each of up to six").unwrap();
    // The Bears leave before the ability resolves; the other targets still take 6.
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

// --- Mirari and Cloven Casting -------------------------------------------------------------

#[test]
fn mirari_pays_on_resolution() {
    cr!("603.5", "707.10");
    ruling!(
        "Mirari",
        "You choose whether or not to pay when the triggered ability resolves."
    );
    supported("Mirari");
    // "Whenever you cast an instant or sorcery spell, you may pay {3}. If you do, copy that
    // spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirari");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let yes_no = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count()
    };
    assert_eq!(yes_no(&t), 0, "not asked as the trigger is put on the stack");
    // Lands for {3} become available only now; P0 pays as the trigger resolves.
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(yes_no(&t), 1);
    assert_eq!(spell_copies(&t).len(), 1);
}

#[test]
fn mirari_s_copy_with_unchanged_illegal_targets_is_still_put_on_the_stack() {
    cr!("707.10c", "608.2b");
    ruling!(
        "Mirari",
        "If you don’t change them, the spell is placed on the stack whether or not the targets are legal."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirari");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
    t.settle();
    // The Bears die before Mirari's trigger resolves.
    kill(&mut t, bears);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    // The copy and Bolt are countered on resolution for having no legal target.
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn cloven_casting_copies_a_spell_countered_in_response() {
    cr!("707.10", "608.2h", "113.7a");
    ruling!(
        "Cloven Casting",
        "You can copy the spell that caused Cloven Casting’s ability to trigger even if it’s been countered by the time that ability resolves."
    );
    supported("Cloven Casting");
    // "Whenever you cast a multicolored instant or sorcery spell, you may pay {1}. If you
    // do, copy that spell."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cloven Casting");
    let helix = cast_new(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    t.settle();
    cast_new(&mut t, P1, "Counterspell", &[obj(helix)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

// --- Demonic Pact -------------------------------------------------------------------------

#[test]
fn demonic_pact_s_mode_is_chosen_as_the_trigger_is_put_on_the_stack() {
    cr!("700.2b", "603.3c", "115.1");
    ruling!(
        "Demonic Pact",
        "You choose the mode as the triggered ability goes on the stack. You can choose a mode that requires targets only if there are legal targets available."
    );
    supported("Demonic Pact");
    supported("Leyline of Sanctity");
    // P1 has hexproof, so "Target opponent discards two cards" can't be chosen.
    let mut t = TestGame::new(2);
    let pact = t.battlefield(P0, "Demonic Pact");
    t.battlefield(P1, "Leyline of Sanctity");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let trig = crate::r_s25_common::abilities_from(&t, pact);
    assert_eq!(trig.len(), 1);
    let modes = modes_on_stack(&t, trig[0]);
    assert_eq!(modes.len(), 1, "the mode is chosen before the trigger resolves");
    assert_ne!(modes, vec![1]);
}

// --- Double Down (copies of permanent spells) ------------------------------------------------

#[test]
fn double_down_s_copy_resolves_even_if_the_spell_is_countered() {
    cr!("707.10", "707.10f", "608.3f");
    ruling!(
        "Double Down",
        "Double Down's ability and the copy it creates will resolve before the spell that caused it to trigger. They resolve even if that spell is countered."
    );
    supported("Double Down");
    // "Whenever you cast an outlaw spell, copy that spell." Bane Alley Blackguard is a 1/3
    // Human Rogue.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Down");
    let rogue = cast_new(&mut t, P0, "Bane Alley Blackguard", &[]);
    t.settle();
    cast_new(&mut t, P1, "Counterspell", &[obj(rogue)]);
    t.resolve();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bane Alley Blackguard"));
    assert_eq!(tokens_named(&t, P0, "Bane Alley Blackguard").len(), 1);
}

#[test]
fn double_down_doesnt_trigger_for_an_outlaw_put_onto_the_battlefield() {
    cr!("603.2", "601.2");
    ruling!(
        "Double Down",
        "Double Down's ability doesn't trigger if an outlaw permanent is put onto the battlefield without being cast."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Down");
    t.enter(P0, "Bane Alley Blackguard");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(tokens_named(&t, P0, "Bane Alley Blackguard").is_empty());
}

#[test]
fn double_down_copies_a_kindred_rogue_spell() {
    cr!("308.1", "205.3m");
    ruling!(
        "Double Down",
        "If a spell has the kindred card type and one of the outlaw creature types, Double Down will copy it."
    );
    supported("Morsel Theft");
    // Morsel Theft (Kindred Sorcery — Rogue): "Target player loses 3 life and you gain 3
    // life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Down");
    cast_new(&mut t, P0, "Morsel Theft", &[Entity::Player(P1)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}
