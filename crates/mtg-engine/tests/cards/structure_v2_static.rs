//! Structure-coverage tests, stream 2 (item `structure-tests-2`): static and keyword
//! ability structures that alone kept their cards from being verified (see
//! `docs/CARD_COVERAGE.md`). Each test checks that a real card's ability does what its text
//! says.

use super::structure_activated_2::{act, next_turn, target_choices};
use super::structure_seed_common::*;
use super::structure_static_3::enchant;
use super::structure_triggered_7::cast_card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn silver_inlaid_dagger_gives_a_human_an_additional_plus_one() {
    cr!("301.5a", "611.3a", "613.4c");
    // "Equipped creature gets +2/+0. / As long as equipped creature is a Human, it gets an
    // additional +1/+0. / Equip {2}"
    supported("Silver-Inlaid Dagger");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Silver-Inlaid Dagger");
    let human = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    act(&mut t, P0, d, "equip", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2), "not a Human: +2/+0 only");
    assert_eq!(t.pt(human), (2, 1));
    act(&mut t, P0, d, "equip", &[obj(human)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(human), (5, 1), "a Human: +3/+0");
    assert_eq!(t.pt(bears), (2, 2), "no longer equipped");
}

#[test]
fn blade_and_plated_slivers_pump_every_sliver() {
    cr!("611.3a", "613.4c");
    // Blade Sliver: "All Sliver creatures get +1/+0." Plated Sliver: "All Sliver creatures
    // get +0/+1."
    for (name, base, bonus) in [
        ("Blade Sliver", (2, 2), (1, 0)),
        ("Plated Sliver", (1, 1), (0, 1)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, name);
        let mine = t.battlefield(P0, "Metallic Sliver");
        let theirs = t.battlefield(P1, "Metallic Sliver");
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert_eq!(
            t.pt(s),
            (base.0 + bonus.0, base.1 + bonus.1),
            "{name}: itself"
        );
        for x in [mine, theirs] {
            assert_eq!(
                t.pt(x),
                (1 + bonus.0, 1 + bonus.1),
                "{name}: every player's Slivers"
            );
        }
        assert_eq!(t.pt(bears), (2, 2), "{name}: not other creatures");
    }
}

#[test]
fn vow_of_malice_pumps_and_keeps_the_creature_from_attacking_you() {
    cr!("303.4a", "702.13a", "508.1c");
    // "Enchanted creature gets +2/+2, has intimidate, and can't attack you or
    // planeswalkers you control."
    supported("Vow of Malice");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    let walker = t.battlefield(P0, "Ajani Goldmane");
    enchant(&mut t, "Vow of Malice", "Swamp", 3, bears);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Intimidate));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Intimidate));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(t.g.can_attack(bears));
    assert!(!t.g.can_attack_target(bears, Entity::Player(P0)), "not you");
    assert!(
        !t.g.can_attack_target(bears, obj(walker)),
        "not your planeswalkers"
    );
    assert!(
        t.g.can_attack_target(bears, Entity::Player(P2)),
        "other players"
    );
    assert!(
        t.g.can_attack_target(other, Entity::Player(P0)),
        "only the enchanted creature"
    );
}

#[test]
fn hot_springs_gives_the_enchanted_land_a_prevention_ability() {
    cr!("303.4a", "615.7", "113.3b");
    // "Enchant land you control / Enchanted land has "{T}: Prevent the next 1 damage that
    // would be dealt to any target this turn.""
    supported("Hot Springs");
    let mut t = TestGame::new(2);
    // A land with no mana ability, so paying for the Aura can't tap it.
    let mine = t.battlefield(P0, "Maze of Ith");
    let theirs = t.battlefield(P1, "Maze of Ith");
    let aura = enchant(&mut t, "Hot Springs", "Forest", 2, mine);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(
        c.contains(&obj(mine)) && !c.contains(&obj(theirs)),
        "a land you control"
    );
    assert_eq!(t.obj_now(aura).attached_to, Some(obj(mine)));
    let has_prevent = |t: &TestGame, id: ObjectId| {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.to_lowercase().contains("prevent the next 1 damage"))
    };
    assert!(has_prevent(&t, mine));
    assert!(!has_prevent(&t, theirs), "only the enchanted land");
    act(
        &mut t,
        P0,
        mine,
        "prevent the next 1",
        &[Entity::Player(P0)],
    )
    .unwrap();
    assert!(t.obj_now(mine).tapped);
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "1 of the 3 damage prevented");
}

#[test]
fn messenger_hawk_gets_plus_two_after_two_draws_this_turn() {
    cr!("611.3a", "121.1");
    // "This creature gets +2/+0 as long as you've drawn two or more cards this turn."
    supported("Messenger Hawk");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Messenger Hawk");
    assert_eq!(t.pt(h), (1, 2));
    // The opponent's draws don't count.
    cast_card(&mut t, P1, "Think Twice", "Island", 2, &[]);
    assert_eq!(t.pt(h), (1, 2));
    cast_card(&mut t, P0, "Think Twice", "Island", 2, &[]);
    assert_eq!(t.pt(h), (1, 2), "one card drawn");
    cast_card(&mut t, P0, "Think Twice", "Island", 2, &[]);
    assert_eq!(t.pt(h), (3, 2), "two cards drawn");
    next_turn(&mut t);
    assert_eq!(t.pt(h), (1, 2), "a new turn");
}

#[test]
fn tenured_concocter_gets_plus_two_after_you_gain_life() {
    cr!("611.3a", "119.3");
    // "Infusion — This creature gets +2/+0 as long as you gained life this turn."
    supported("Tenured Concocter");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Tenured Concocter");
    assert_eq!(t.pt(c), (4, 5));
    cast_card(&mut t, P1, "Revitalize", "Plains", 2, &[]);
    assert_eq!(t.life(P1), 23);
    assert_eq!(t.pt(c), (4, 5), "an opponent's life gain doesn't count");
    cast_card(&mut t, P0, "Revitalize", "Plains", 2, &[]);
    assert_eq!(t.pt(c), (6, 5));
    next_turn(&mut t);
    assert_eq!(t.pt(c), (4, 5));
}

#[test]
fn strength_of_isolation_gives_plus_one_plus_two_and_protection_from_black() {
    cr!("303.4a", "702.16b");
    // "Enchanted creature gets +1/+2 and has protection from black."
    supported("Strength of Isolation");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    enchant(&mut t, "Strength of Isolation", "Plains", 2, bears);
    assert_eq!(t.pt(bears), (3, 4));
    assert_eq!(t.pt(other), (3, 3));
    // A black spell can't target it.
    t.lands(P1, "Swamp", 1);
    let d = t.hand(P1, "Disfigure");
    t.answer_targets(P1, &[obj(other)]);
    t.cast(P1, d).go();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&obj(other)));
    assert!(!c.contains(&obj(bears)), "protection from black");
    t.resolve_all();
    assert_eq!(t.pt(other), (1, 1));
}

#[test]
fn lightwalker_flies_with_a_counter_on_it() {
    cr!("611.3a", "613.1f", "122.1a");
    // "This creature has flying as long as it has a +1/+1 counter on it."
    supported("Lightwalker");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lightwalker");
    assert!(!t.obj_now(l).has_keyword(KeywordKind::Flying));
    t.g.add_counters(obj(l), "-1/-1", 1, None);
    t.g.recompute();
    assert!(
        !t.obj_now(l).has_keyword(KeywordKind::Flying),
        "a -1/-1 counter isn't one"
    );
    let l = t.battlefield(P0, "Lightwalker");
    t.g.add_counters(obj(l), "+1/+1", 1, None);
    t.g.recompute();
    assert!(t.obj_now(l).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(l), (3, 2));
}

#[test]
fn giant_tortoise_gets_plus_zero_three_while_untapped() {
    cr!("611.3a", "613.4c");
    // "This creature gets +0/+3 as long as it's untapped."
    supported("Giant Tortoise");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Giant Tortoise");
    assert_eq!(t.pt(g), (1, 4));
    t.g.tap(g);
    t.g.recompute();
    assert_eq!(t.pt(g), (1, 1));
}

#[test]
fn river_serpent_attacks_only_with_five_cards_in_your_graveyard() {
    cr!("508.1c");
    // "This creature can't attack unless there are five or more cards in your graveyard."
    supported("River Serpent");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "River Serpent");
    for _ in 0..4 {
        t.graveyard(P0, "Forest");
        t.graveyard(P1, "Forest");
    }
    t.graveyard(P1, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(
        !t.g.can_attack(s),
        "four cards (the opponent's don't count)"
    );
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert!(t.g.can_attack(s), "five cards");
}

#[test]
fn leech_fanatic_has_lifelink_during_your_turn() {
    cr!("611.3a", "702.15b");
    // "During your turn, this creature has lifelink."
    supported("Leech Fanatic");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Leech Fanatic");
    let theirs = t.battlefield(P1, "Leech Fanatic");
    assert!(t.obj_now(l).has_keyword(KeywordKind::Lifelink));
    assert!(
        !t.obj_now(theirs).has_keyword(KeywordKind::Lifelink),
        "not its controller's turn"
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(l, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!t.obj_now(l).has_keyword(KeywordKind::Lifelink));
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Lifelink));
}

#[test]
fn treefolk_seedlings_toughness_counts_your_forests() {
    cr!("604.3", "613.4a");
    // "Treefolk Seedlings's toughness is equal to the number of Forests you control."
    supported("Treefolk Seedlings");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Forest", 3);
    t.lands(P0, "Island", 1);
    let s = t.battlefield(P0, "Treefolk Seedlings");
    assert_eq!(t.pt(s), (2, 2));
    t.lands(P0, "Forest", 1);
    assert_eq!(t.pt(s), (2, 3));
}

#[test]
fn meekstone_keeps_power_three_creatures_tapped() {
    cr!("502.3", "611.3a");
    // "Creatures with power 3 or greater don't untap during their controllers' untap
    // steps."
    supported("Meekstone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Meekstone");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    for id in [giant, bears, mine] {
        t.g.tap(id);
    }
    next_turn(&mut t);
    assert!(t.obj_now(giant).tapped, "power 3: stays tapped");
    assert!(!t.obj_now(bears).tapped, "power 2 untaps");
    assert!(t.obj_now(mine).tapped, "not its controller's untap step");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(mine).tapped);
}

#[test]
fn cloudreach_cavalry_gets_plus_two_and_flying_with_a_bird() {
    cr!("611.3a", "613.4c", "613.1f");
    // "As long as you control a Bird, this creature gets +2/+2 and has flying."
    supported("Cloudreach Cavalry");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cloudreach Cavalry");
    t.battlefield(P1, "Birds of Paradise");
    assert_eq!(t.pt(c), (1, 1), "the opponent's Bird doesn't count");
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Flying));
    t.battlefield(P0, "Birds of Paradise");
    assert_eq!(t.pt(c), (3, 3));
    assert!(t.obj_now(c).has_keyword(KeywordKind::Flying));
}

#[test]
fn doorkeeper_thrull_stops_artifacts_and_creatures_entering_from_triggering() {
    cr!("603.6a");
    // "Artifacts and creatures entering don't cause abilities to trigger."
    supported("Doorkeeper Thrull");
    let mut t = TestGame::new(2);
    let thrull = t.battlefield(P0, "Doorkeeper Thrull");
    t.battlefield(P0, "Soul Warden");
    t.enter(P1, "Elvish Visionary");
    t.enter(P1, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "Soul Warden didn't trigger");
    assert_eq!(t.hand_size(P1), 0, "the Visionary's own ETB didn't trigger");
    // A land entering still triggers.
    t.enter(P0, "Scoured Barrens");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    // Without the Thrull, creatures entering trigger again.
    t.g.destroy(thrull, None);
    t.resolve_all();
    t.enter(P1, "Elvish Visionary");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn brine_giant_costs_one_less_for_each_enchantment_you_control() {
    cr!("702.41a", "601.2f");
    // "Affinity for enchantments"
    supported("Brine Giant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Island", 4);
    let g = t.hand(P0, "Brine Giant");
    assert!(
        t.cast(P0, g).try_go().is_err(),
        "{{4}}{{U}} with two enchantments"
    );
    t.clear_answers();
    t.lands(P0, "Island", 1);
    t.cast(P0, g).go();
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Brine Giant"), 1);
}
