//! CR 704.5 audit, with copies: the state-based actions look at what a permanent is now,
//! copy effects included. A copy of a legend is a legend with the same name (704.5j)
//! unless the copy effect changes that; a copy of a planeswalker or battle that entered as
//! one has the printed counters, one that became one has none (704.5i, 704.5v); a copy of
//! a world enchantment is the newest one (704.5k); a permanent turned into a copy of a
//! noncreature loses its Auras and Equipment (704.5m, 704.5n); and a copy of a Saga is a
//! Saga of its own (704.5s).

use crate::r603_audit_common::*;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` enters `name` (a "you may have this enter as a copy" card) copying `what`.
fn enter_copying(t: &mut TestGame, p: PlayerId, name: &str, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, name);
    t.g.current(c)
}

/// `obj` becomes a copy of `of` (a copy effect from a resolving ability).
fn become_copy(t: &mut TestGame, obj: ObjectId, of: ObjectId) {
    let mut ctx = Ctx::new(Some(obj), t.g.obj(obj).controller);
    ctx.targets = vec![vec![Entity::Object(obj)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn a_copy_of_a_used_planeswalker_has_its_printed_loyalty_and_the_legend_rule_applies() {
    cr!("704.5j", "306.5b", "707.2");
    ruling!(
        "Clever Impersonator",
        "it will enter the battlefield with a number of loyalty counters on it equal to the loyalty printed in the lower right corner of the planeswalker card"
    );
    ruling!(
        "Clever Impersonator",
        "if you control more than one legendary permanent with the same name, you'll choose one to remain on the battlefield"
    );
    supported("Clever Impersonator");
    supported("Garruk Wildspeaker");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk Wildspeaker");
    t.g.add_counters(Entity::Object(garruk), counters::LOYALTY, 2, None);
    assert_eq!(t.counters(garruk, counters::LOYALTY), 5);
    let copy = enter_copying(&mut t, P0, "Clever Impersonator", garruk);
    assert_eq!(t.counters(copy, counters::LOYALTY), 3);
    assert!(t.obj_now(copy).chars.is_legendary());
    // The legend rule: the player keeps the copy.
    t.answer_choose(P0, &[Entity::Object(copy)]);
    t.settle();
    assert!(t.on_battlefield(copy));
    assert!(!t.on_battlefield(garruk));
    assert!(t.in_graveyard(P0, "Garruk Wildspeaker"));
}

#[test]
fn spark_double_copying_a_planeswalker_isnt_legendary_and_has_an_extra_loyalty_counter() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Spark Double",
        "If it copies a planeswalker, Spark Double enters with the appropriate number of loyalty counters as printed on the card plus one."
    );
    supported("Spark Double");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk Wildspeaker");
    let copy = enter_copying(&mut t, P0, "Spark Double", garruk);
    t.settle();
    assert!(t.on_battlefield(copy) && t.on_battlefield(garruk));
    assert!(!t.obj_now(copy).chars.is_legendary());
    assert_eq!(t.counters(copy, counters::LOYALTY), 4);
    // A second Spark Double copying the first isn't legendary either.
    let copy2 = enter_copying(&mut t, P0, "Spark Double", copy);
    t.settle();
    assert!(t.on_battlefield(copy) && t.on_battlefield(copy2) && t.on_battlefield(garruk));
    assert_eq!(t.counters(copy2, counters::LOYALTY), 4);
}

#[test]
fn a_permanent_that_becomes_a_copy_of_a_planeswalker_has_no_loyalty_and_dies() {
    cr!("704.5i", "707.4", "306.5b");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of a planeswalker, it won’t receive loyalty counters"
    );
    supported("Crystalline Resonance");
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, "Crystalline Resonance");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    become_copy(&mut t, res, garruk);
    assert!(t.obj_now(res).is(CardType::Planeswalker));
    assert_eq!(t.counters(res, counters::LOYALTY), 0);
    t.settle();
    assert!(t.in_graveyard(P0, "Crystalline Resonance"));
    assert!(t.on_battlefield(garruk));
}

#[test]
fn a_permanent_that_becomes_a_copy_of_an_unattached_aura_or_a_legend_you_control_goes() {
    cr!("704.5m", "704.5j", "707.4");
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of an Aura, it’s put into its owner’s graveyard unless it’s somehow attached"
    );
    ruling!(
        "Crystalline Resonance",
        "If Crystalline Resonance becomes a copy of a legendary permanent you control, you’ll put one of them into its owner’s graveyard."
    );
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, "Crystalline Resonance");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    become_copy(&mut t, res, strength);
    assert!(t.obj_now(res).chars.has_subtype("Aura"));
    t.settle();
    assert!(t.in_graveyard(P0, "Crystalline Resonance"));
    assert!(t.on_battlefield(strength));

    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, "Crystalline Resonance");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    become_copy(&mut t, res, isamaru);
    t.answer_choose(P0, &[Entity::Object(res)]);
    t.settle();
    assert!(t.on_battlefield(res));
    assert!(!t.on_battlefield(isamaru));
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
}

#[test]
fn oko_copying_a_creature_isnt_a_planeswalker_and_damage_leaves_his_loyalty() {
    cr!("704.5i", "120.3e", "613.1a");
    ruling!(
        "Oko, the Ringleader",
        "While Oko is a copy of a creature, he isn't a planeswalker. He won't have his loyalty abilities, and damage dealt to him won't cause him to lose loyalty counters."
    );
    supported("Oko, the Ringleader");
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, the Ringleader");
    let giant = t.battlefield(P0, "Hill Giant");
    let loyalty = t.counters(oko, counters::LOYALTY);
    assert_eq!(loyalty, 3);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.settle();
    t.resolve();
    let o = t.obj_now(oko);
    assert!(o.is_creature() && !o.is(CardType::Planeswalker));
    assert!(o.has_keyword(KeywordKind::Hexproof));
    assert_eq!(t.counters(oko, counters::LOYALTY), loyalty);
    let rats = t.battlefield(P1, "Typhoid Rats");
    // Damage to a creature that isn't a planeswalker is marked on it.
    t.g.objects[rats.0 as usize].chars.abilities.clear();
    t.g.deal_damage(rats, Entity::Object(oko), 2, false);
    t.settle();
    assert!(t.on_battlefield(oko));
    assert_eq!(t.counters(oko, counters::LOYALTY), loyalty);
    assert_eq!(t.obj_now(oko).damage, 2);
}

#[test]
fn a_permanent_that_becomes_a_copy_of_a_siege_has_no_defense_and_goes() {
    cr!("704.5v", "707.4", "310.4b");
    ruling!(
        "Invasion of Tarkir // Defiant Thundermaw",
        "it will most likely be put into its owner’s graveyard because it has no defense counters"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let siege = t.battlefield(P0, "Invasion of Tarkir // Defiant Thundermaw");
    t.settle();
    assert!(t.on_battlefield(siege));
    become_copy(&mut t, bears, siege);
    assert!(t.obj_now(bears).is(CardType::Battle));
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(siege));
}

#[test]
fn a_permanent_entering_as_a_copy_of_a_battle_enters_with_its_defense_counters() {
    cr!("310.4b", "707.5", "120.3h");
    ruling!(
        "Invasion of Tarkir // Defiant Thundermaw",
        "If another permanent enters the battlefield as a copy of a battle, it also enters with that number of defense counters."
    );
    let mut t = TestGame::new(2);
    let siege = t.battlefield(P0, "Invasion of Tarkir // Defiant Thundermaw");
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.g.objects[rats.0 as usize].chars.abilities.clear();
    // Two damage: three defense counters left.
    t.g.deal_damage(rats, Entity::Object(siege), 2, false);
    t.settle();
    assert_eq!(t.counters(siege, counters::DEFENSE), 3);
    let copy = enter_copying(&mut t, P0, "Clever Impersonator", siege);
    t.settle();
    assert!(t.on_battlefield(copy));
    assert_eq!(t.counters(copy, counters::DEFENSE), 5);
}

#[test]
fn a_copy_of_a_world_enchantment_is_the_newest_world_permanent() {
    cr!("704.5k", "707.5");
    supported("Copy Enchantment");
    supported("Concordant Crossroads");
    let mut t = TestGame::new(2);
    let world = t.battlefield(P1, "Concordant Crossroads");
    t.settle();
    let copy = enter_copying(&mut t, P0, "Copy Enchantment", world);
    t.settle();
    assert!(t.on_battlefield(copy));
    assert!(!t.on_battlefield(world));
    assert!(t.in_graveyard(P1, "Concordant Crossroads"));
}

#[test]
fn a_permanent_that_becomes_a_copy_of_a_world_enchantment_is_the_newest_one() {
    cr!("704.5k", "707.4");
    let mut t = TestGame::new(2);
    let older = t.battlefield(P0, "Concordant Crossroads");
    let res = t.battlefield(P1, "Crystalline Resonance");
    t.settle();
    t.g.next_timestamp += 10;
    become_copy(&mut t, res, older);
    t.settle();
    assert!(t.on_battlefield(res));
    assert!(!t.on_battlefield(older));
}

#[test]
fn a_creature_that_becomes_a_copy_of_a_noncreature_loses_its_aura_and_equipment() {
    cr!("704.5m", "704.5n", "707.4", "301.5a", "303.4c");
    supported("Holy Strength");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.battlefield(P0, "Holy Strength");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    let ring = t.battlefield(P1, "Sol Ring");
    t.settle();
    assert_eq!(t.pt(bears), (5, 4));
    become_copy(&mut t, bears, ring);
    assert!(!t.obj_now(bears).is_creature());
    t.settle();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert!(t.on_battlefield(bears));
}

#[test]
fn copy_enchantment_copying_an_aura_enters_attached_to_a_chosen_creature() {
    cr!("704.5m", "303.4f", "707.5");
    ruling!(
        "Copy Enchantment",
        "If Copy Enchantment copies an Aura this way, you choose what the Aura will enchant just before it enters the battlefield."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    t.answer_choose(P0, &[Entity::Object(strength)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let copy = t.enter(P0, "Copy Enchantment");
    let copy = t.g.current(copy);
    t.settle();
    assert!(t.on_battlefield(copy));
    assert_eq!(t.obj_now(copy).attached_to, Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (4, 5));
}

#[test]
fn a_copy_of_a_saga_is_a_saga_with_its_own_lore_counters() {
    cr!("704.5s", "714.3a", "707.2");
    supported("The Eldest Reborn");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Eldest Reborn");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    t.g.dirty = true;
    t.settle();
    let copy = enter_copying(&mut t, P0, "Copy Enchantment", saga);
    // It enters with one lore counter (it's a Saga), not the original's two, and its
    // chapter I ability triggers.
    assert_eq!(t.counters(copy, counters::LORE), 1);
    t.settle();
    assert_eq!(triggers_of(&t, "The Eldest Reborn"), 1);
    t.resolve_all();
    assert!(t.on_battlefield(copy) && t.on_battlefield(saga));
}

#[test]
fn machine_gods_effigy_copying_a_legend_you_control_has_its_name_and_the_legend_rule_applies() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Machine God's Effigy",
        "so if you copy a legendary creature you control, you will need to choose one of them"
    );
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let effigy = enter_copying(&mut t, P0, "Machine God's Effigy", isamaru);
    assert!(!t.obj_now(effigy).is_creature());
    assert!(t.obj_now(effigy).chars.is_legendary());
    t.answer_choose(P0, &[Entity::Object(isamaru)]);
    t.settle();
    assert!(t.on_battlefield(isamaru));
    assert!(!t.on_battlefield(effigy));
}

#[test]
fn sunfrill_imitator_copying_a_legend_keeps_its_name_and_no_legend_rule_applies() {
    cr!("704.5j", "707.9b");
    ruling!(
        "Sunfrill Imitator",
        "because its name remains Sunfrill Imitator, the \"legend rule\" won't cause either of them to be put into the graveyard"
    );
    supported("Sunfrill Imitator");
    supported("Ghalta, Primal Hunger");
    let mut t = TestGame::new(2);
    let imitator = t.battlefield(P0, "Sunfrill Imitator");
    let ghalta = t.battlefield(P0, "Ghalta, Primal Hunger");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(ghalta)]);
    // The attack trigger resolves during combat.
    t.attack(&[(imitator, Entity::Player(P1))], &[]);
    let o = t.obj_now(imitator);
    assert_eq!(o.chars.name.as_str(), "Sunfrill Imitator");
    assert!(o.chars.is_legendary());
    assert_eq!(t.pt(imitator), (12, 12));
    t.settle();
    assert!(t.on_battlefield(imitator) && t.on_battlefield(ghalta));
}
