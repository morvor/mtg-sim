//! CR 704.3 / 704.7 / 704.8 audit: all applicable state-based actions are performed at
//! once, from one check of the game state. A second legend entering makes creatures die
//! at the same time the legend rule applies; annihilating counters and dying in the same
//! check use the information from before either; dies triggers look back at every
//! permanent that left in the same check; and triggered abilities wait until no more
//! state-based actions happen.

use crate::r603_audit_common::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_second_kaervek_kills_creatures_at_the_same_time_as_the_legend_rule() {
    cr!("704.3", "704.5f", "704.5j", "704.5q", "704.8");
    ruling!(
        "Kaervek, the Spiteful",
        "those creatures die at the same time that the “legend rule” remove your excess Kaerveks"
    );
    supported("Kaervek, the Spiteful");
    supported("Kitchen Finks");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Kaervek, the Spiteful");
    // A 2/2: 1/1 under one Kaervek, 0/0 under two.
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A 3/2 with a +1/+1 and a -1/-1 counter: 1/0 under two Kaerveks. It dies with its
    // -1/-1 counter (the counters are removed in the same check), so persist doesn't
    // return it (CR 704.8).
    let finks = t.battlefield(P1, "Kitchen Finks");
    // A 3/3 with a +1/+1 and a -1/-1 counter: 1/1 under two Kaerveks, it survives and its
    // counters annihilate.
    let giant = t.battlefield(P1, "Hill Giant");
    t.settle();
    for id in [finks, giant] {
        t.g.add_counters(Entity::Object(id), counters::MINUS1, 1, None);
        t.g.add_counters(Entity::Object(id), counters::PLUS1, 1, None);
    }
    assert_eq!(t.counters(finks, counters::MINUS1), 1);
    assert_eq!(t.pt(bears), (1, 1));
    // The second Kaervek: the player keeps the first one.
    t.answer_choose(P0, &[Entity::Object(first)]);
    let second = t.enter(P0, "Kaervek, the Spiteful");
    assert_eq!(t.pt(bears), (0, 0));
    t.settle();
    assert!(t.on_battlefield(first));
    assert!(!t.on_battlefield(second));
    assert!(t.in_graveyard(P0, "Kaervek, the Spiteful"));
    // Had the legend rule been performed first, the Bears would be a 1/1 again.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Kitchen Finks"));
    // It died with a -1/-1 counter: persist doesn't trigger (a returned Finks would be a
    // 1/0 under the remaining Kaervek and die again, so look at the stack).
    assert_eq!(
        t.g.obj(finks).counters.get(counters::MINUS1).copied(),
        Some(1)
    );
    assert_eq!(triggers_of(&t, "Kitchen Finks"), 0, "no persist");
    let life = t.life(P1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Kitchen Finks").is_empty());
    assert_eq!(t.life(P1), life, "Kitchen Finks never returned");
    assert!(t.on_battlefield(giant));
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
    assert_eq!(t.counters(giant, counters::MINUS1), 0);
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn a_teysa_put_into_the_graveyard_by_the_legend_rule_triggers_dies_abilities_three_times() {
    cr!("704.5j", "704.8", "603.10a");
    ruling!(
        "Teysa Karlov",
        "if you control Teysa and cast a second one, an ability that triggers when it dies due to the “legend rule” triggers three times"
    );
    supported("Teysa Karlov");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Teysa Karlov");
    t.battlefield(P0, "Blood Artist");
    t.answer_choose(P0, &[Entity::Object(first)]);
    targets_n(&mut t, P0, Entity::Player(P1), 3);
    t.enter(P0, "Teysa Karlov");
    t.settle();
    assert_eq!(controlled_named(&t, P0, "Teysa Karlov"), 1);
    // Blood Artist looks back at the game before the legend rule: two Teysas.
    assert_eq!(triggers_of(&t, "Blood Artist"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sharuum_put_into_the_graveyard_by_the_legend_rule_can_return_itself() {
    cr!("704.3", "704.5j", "603.3");
    ruling!(
        "Sharuum the Hegemon",
        "it can be the artifact card targeted by its own ability"
    );
    supported("Sharuum the Hegemon");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Sharuum the Hegemon");
    t.answer_choose(P0, &[Entity::Object(first)]);
    let second = t.enter(P0, "Sharuum the Hegemon");
    // State-based actions are performed before the enters ability is put on the stack:
    // its target is chosen with the second Sharuum already in the graveyard.
    t.g.flush_events();
    assert_eq!(t.g.pending_triggers.len(), 1);
    assert!(t.g.check_sbas());
    assert!(t.g.stack.is_empty());
    let card = t.g.current(second);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.settle();
    assert_eq!(triggers_of(&t, "Sharuum the Hegemon"), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(first)]);
    t.g.resolve_top();
    assert!(t.on_battlefield(t.g.current(card)));
}

#[test]
fn leyline_of_singularity_makes_the_legend_rule_apply_to_tokens_with_the_same_name() {
    cr!("704.5j", "704.5d", "205.4d");
    ruling!(
        "Leyline of Singularity",
        "only one of the tokens will be allowed to remain on the battlefield"
    );
    supported("Leyline of Singularity");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Singularity");
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve();
    assert_eq!(tokens_named(&t, P0, "Soldier"), 1);
    // The other token went to the graveyard and ceased to exist.
    assert_eq!(t.graveyard_size(P0), 1, "only Raise the Alarm");
}

#[test]
fn the_legend_rule_isnt_a_sacrifice_but_a_saga_is_sacrificed() {
    cr!("704.5j", "704.5s", "701.21a");
    ruling!(
        "Mayhem Devil",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    supported("Mayhem Devil");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayhem Devil");
    let first = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[Entity::Object(first)]);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(controlled_named(&t, P0, "Isamaru, Hound of Konda"), 1);
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 0);
    // A Saga past its last chapter is sacrificed (by its controller).
    supported("The Eldest Reborn");
    let saga = t.battlefield(P0, "The Eldest Reborn");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    t.g.dirty = true;
    t.settle();
    assert!(t.in_graveyard(P0, "The Eldest Reborn"));
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 1);
}

#[test]
fn umbra_armor_replaces_lethal_damage_and_deathtouch_together() {
    cr!("704.5g", "704.5h", "704.7", "702.89a");
    ruling!(
        "Felidar Umbra",
        "umbra armor’s effect will replace all of them and save the creature"
    );
    supported("Felidar Umbra");
    supported("Typhoid Rats");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = t.battlefield(P0, "Felidar Umbra");
    assert!(t.g.attach(umbra, Entity::Object(bears)));
    let rats = t.battlefield(P1, "Typhoid Rats");
    // 2 damage from a deathtouch source: lethal damage and deathtouch damage at once.
    t.g.deal_damage(rats, Entity::Object(bears), 2, false);
    t.settle();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_graveyard(P0, "Felidar Umbra"));
    // Nothing else is left to replace: the next check does nothing more.
    t.settle();
    assert!(t.on_battlefield(bears));
}

#[test]
fn the_legend_rule_and_zero_toughness_ignore_indestructible() {
    cr!("704.5f", "704.5j", "702.12b");
    ruling!(
        "Sapling of Colfenor",
        "could be put into the graveyard if its toughness becomes 0 or less, if it’s sacrificed, or if you control another Sapling of Colfenor"
    );
    supported("Sapling of Colfenor");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Sapling of Colfenor");
    t.answer_choose(P0, &[Entity::Object(first)]);
    let second = t.battlefield(P0, "Sapling of Colfenor");
    t.settle();
    assert!(t.on_battlefield(first));
    assert!(!t.on_battlefield(second));
    // Lethal damage doesn't destroy it; 0 toughness puts it into the graveyard.
    let rats = t.battlefield(P1, "Typhoid Rats");
    t.g.deal_damage(rats, Entity::Object(first), 5, false);
    t.settle();
    assert!(t.on_battlefield(first));
    // A 2/5: five -1/-1 counters.
    t.g.add_counters(Entity::Object(first), counters::MINUS1, 5, None);
    t.settle();
    assert!(!t.on_battlefield(first));
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn a_third_brothers_yamazaki_brings_back_the_legend_rule() {
    cr!("704.5j");
    ruling!(
        "Brothers Yamazaki",
        "If a third Brothers Yamazaki enters the battlefield, the legend rule will apply."
    );
    supported("Brothers Yamazaki");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Brothers Yamazaki");
    let b = t.battlefield(P0, "Brothers Yamazaki");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // Each gets +2/+2 from the other.
    assert_eq!(t.pt(a), (4, 3));
    // A third one, controlled by the other player: the legend rule applies again, to the
    // two P0 controls (P1 controls only one).
    t.answer_choose(P0, &[Entity::Object(b)]);
    let c = t.battlefield(P1, "Brothers Yamazaki");
    t.settle();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    assert!(t.on_battlefield(c));
    // Two remain: the exemption applies again.
    t.settle();
    assert!(t.on_battlefield(b) && t.on_battlefield(c));
}

#[test]
fn the_legend_rule_applies_as_soon_as_mirror_gallery_leaves() {
    cr!("704.5j", "704.3");
    ruling!(
        "Mirror Gallery",
        "If all Mirror Galleries leave the battlefield, the “Legend rule” will apply the next time state-based actions are performed."
    );
    supported("Mirror Gallery");
    let mut t = TestGame::new(2);
    let gallery = t.battlefield(P0, "Mirror Gallery");
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    let b = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    t.g.destroy(gallery, None);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.settle();
    assert!(!t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}

#[test]
fn counters_annihilating_are_removed_counters_for_protean_hydra() {
    cr!("704.5q", "122.3");
    ruling!(
        "Protean Hydra",
        "that -1/-1 counter and a +1/+1 counter will be removed from it as a state-based action. This will cause Protean Hydra’s last ability to trigger."
    );
    supported("Protean Hydra");
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Protean Hydra");
    t.g.add_counters(Entity::Object(hydra), counters::PLUS1, 3, None);
    t.settle();
    t.g.add_counters(Entity::Object(hydra), counters::MINUS1, 1, None);
    t.settle();
    assert_eq!(t.counters(hydra, counters::PLUS1), 2);
    assert_eq!(t.counters(hydra, counters::MINUS1), 0);
    assert_eq!(triggers_of(&t, "Protean Hydra"), 1);
}

#[test]
fn persist_returns_a_creature_again_after_its_minus_one_counter_annihilated() {
    cr!("704.5q", "702.79a");
    ruling!(
        "Putrid Goblin",
        "Putrid Goblin’s persist ability can bring it back again if its -1/-1 counters are removed this way."
    );
    supported("Putrid Goblin");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Putrid Goblin");
    t.g.add_counters(Entity::Object(goblin), counters::MINUS1, 1, None);
    t.settle();
    t.g.add_counters(Entity::Object(goblin), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.counters(goblin, counters::MINUS1), 0);
    t.g.destroy(goblin, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Putrid Goblin");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], counters::MINUS1), 1);
}
