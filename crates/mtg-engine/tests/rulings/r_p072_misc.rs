//! Rulings batch P072 — miscellaneous rulings on cards that give strike keywords, flying
//! or fear: X of 0, abilities that trigger an additional time on a creature dying
//! (CR 603.2d... the Teysa Karlov family), choices made on resolution, targeting rules,
//! and simple statements of what the cards do (table-driven where possible).

use crate::r_p007_common::loyalty;
use crate::r_s01_common::{attack_with, supported, tapped_lands, with_subtype};
use crate::r_s02_common::{can_activate, can_play_land, create_token, destroy, target_candidates};
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{activate_containing, attach_new, attached_to, has_kw};
use crate::r_s21_common::castable;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::{cast_new, lands_for_cost, legendary};
use crate::r_s35_common::creature_type_options;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// X = 0
// ---------------------------------------------------------------------------------------

#[test]
fn assault_on_osgiliath_with_x_of_zero() {
    cr!("701.47a", "701.47b", "107.3a");
    ruling!(
        "Assault on Osgiliath",
        "If you choose a value of 0 for X while you control a non-Orc Army, you won't put any counters on it, but it will become an Orc in addition to its other types."
    );
    ruling!(
        "Assault on Osgiliath",
        "If you choose a value of 0 for X while you don't control an Army, you will create a 0/0 Orc Army creature token and won't put any counters on it. In most cases, that 0/0 token will die immediately."
    );
    supported("Assault on Osgiliath");
    // A non-Orc Army (a 1/1 Army token).
    let mut t = TestGame::new(2);
    let army = create_token(&mut t, P0, "Army");
    lands_for_cost(&mut t, P0, "Assault on Osgiliath");
    let spell = t.hand(P0, "Assault on Osgiliath");
    t.cast(P0, spell).x(0).go();
    t.resolve_all();
    assert_eq!(t.counters(army, counters::PLUS1), 0);
    assert!(t.obj_now(army).chars.has_subtype("Orc"));
    assert!(t.obj_now(army).chars.has_subtype("Army"));
    assert_eq!(t.pt(army), (1, 1));
    // No Army: a 0/0 Orc Army that dies.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Assault on Osgiliath");
    let spell = t.hand(P0, "Assault on Osgiliath");
    t.cast(P0, spell).x(0).go();
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Army").is_empty());
    assert_eq!(crate::r_s15_common::tokens_created_by(&t, P0), 1);
}

#[test]
fn profane_command_one_x_for_every_mode() {
    cr!("107.3a", "700.2");
    ruling!(
        "Profane Command",
        "The value chosen for X applies to each X in the spell's effect. You pay {X} only once."
    );
    supported("Profane Command");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Profane Command");
    t.cast(P0, spell)
        .modes(&[0, 2])
        .x(2)
        .target(Entity::Player(P1))
        .target(giant)
        .go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(giant), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Triggering an additional time on a creature dying
// ---------------------------------------------------------------------------------------

#[test]
fn dying_triggers_twice_but_sacrifice_triggers_once() {
    cr!("603.2d", "700.4");
    ruling!(
        "Drivnod, Carnage Dominus",
        "An ability that triggers on an event that causes a creature to die doesn't trigger twice. For example, an ability that triggers \"whenever you sacrifice a creature\" triggers only once."
    );
    ruling!(
        "The Masamune",
        "An ability that triggers on an event that causes a creature to die doesn't trigger twice. For example, an ability that triggers \"whenever you sacrifice a creature\" triggers only once."
    );
    supported("Drivnod, Carnage Dominus");
    // Mayhem Devil ("Whenever a player sacrifices a permanent, ...") triggers once; Blood
    // Artist ("Whenever this creature or another creature dies, ...") twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drivnod, Carnage Dominus");
    let devil = t.battlefield(P0, "Mayhem Devil");
    let artist = t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    let from = |src: ObjectId| crate::r_s14_common::triggers_from(&t, src);
    assert_eq!(from(devil), 1);
    assert_eq!(from(artist), 2);
    // The Masamune: only the equipped creature's own abilities; Mayhem Devil equipped.
    let mut t = TestGame::new(2);
    let devil = t.battlefield(P0, "Mayhem Devil");
    attach_new(&mut t, P0, "The Masamune", devil);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(crate::r_s14_common::triggers_from(&t, devil), 1);
}

#[test]
fn an_animated_land_dying_is_a_creature_dying() {
    cr!("603.2d", "603.10a", "700.4");
    ruling!(
        "Drivnod, Carnage Dominus",
        "Look at each creature as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times. For example, if a land that has become a creature dies, an ability that triggers when it dies triggers twice."
    );
    ruling!(
        "Teysa Karlov",
        "Look at each creature as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times. For example, if a land that has become a creature dies, an ability that triggers when it dies triggers twice."
    );
    ruling!(
        "The Masamune",
        "Look at each creature as it exists on the battlefield, taking into account continuous effects, to determine whether any triggered abilities will trigger multiple times. For example, if a land that has become a creature dies, an ability that triggers when it dies triggers twice."
    );
    supported("Teysa Karlov");
    supported("Natural Emergence");
    for doubler in ["Drivnod, Carnage Dominus", "Teysa Karlov", "The Masamune"] {
        let mut t = TestGame::new(2);
        // Natural Emergence: "Lands you control are 2/2 creatures with first strike."
        t.battlefield(P0, "Natural Emergence");
        let artist = t.battlefield(P0, "Blood Artist");
        if doubler == "The Masamune" {
            attach_new(&mut t, P0, doubler, artist);
        } else {
            t.battlefield(P0, doubler);
        }
        let forest = t.lands(P0, "Forest", 1)[0];
        assert!(t.obj_now(forest).is(mtg_engine::types::CardType::Creature));
        destroy(&mut t, forest);
        assert_eq!(
            crate::r_s14_common::triggers_from(&t, artist),
            2,
            "{doubler}"
        );
    }
}

#[test]
fn the_masamune_choices_are_made_separately_for_each_instance() {
    cr!("603.2d", "603.3d");
    ruling!(
        "The Masamune",
        "The granted ability's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to pay a cost for that triggered ability, are also made separately."
    );
    // Blood Artist equipped: "target player loses 1 life and you gain 1 life", once
    // targeting P1, once targeting P0.
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    attach_new(&mut t, P0, "The Masamune", artist);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

// ---------------------------------------------------------------------------------------
// Chronicle of Victory
// ---------------------------------------------------------------------------------------

#[test]
fn chronicle_of_victory_choice_and_cast_trigger() {
    cr!("607.2d", "603.2", "205.3m");
    ruling!(
        "Chronicle of Victory",
        "The choice of creature type is made as Chronicle of Victory enters. Players can't take any actions between the time the choice is made and the time the appropriate creatures begin to get +2/+2."
    );
    ruling!(
        "Chronicle of Victory",
        "You must choose an existing creature type, such as Goblin or Elemental. Card types such as artifact and supertypes such as legendary can't be chosen."
    );
    ruling!(
        "Chronicle of Victory",
        "Chronicle of Victory's last ability resolves before the spell that causes it to trigger. The ability will resolve even if that spell is countered or otherwise leaves the stack."
    );
    supported("Chronicle of Victory");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Bear");
    enter(&mut t, P0, "Chronicle of Victory");
    let options = creature_type_options(&t, P0, from);
    assert_eq!(options.len(), 1);
    for word in ["Artifact", "Legendary", "Creature"] {
        assert!(!options[0].iter().any(|o| o == word), "{word}");
    }
    assert_eq!(t.pt(bears), (4, 4));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    // A Bear spell, countered: the draw still happens.
    let hand = t.hand_size(P0);
    let spell = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.lands(P1, "Island", 2);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Choices on resolution, one or two targets
// ---------------------------------------------------------------------------------------

#[test]
fn heroes_hangout_modes() {
    cr!("700.2", "115.1d", "608.2d");
    ruling!(
        "Heroes' Hangout",
        "For the first mode, you choose which card you'll be allowed to play as the ability resolves. The other card will remain in exile indefinitely."
    );
    ruling!(
        "Heroes' Hangout",
        "If you choose the second mode, you choose whether Heroes' Hangout has one or two targets at the time you cast it. If you choose two targets, each of them get +1/+0 and gain first strike until end of turn."
    );
    // Date Night: Grizzly Bears and an Island exiled; P0 chooses the Bears.
    let mut t = TestGame::new(2);
    let island = t.library_top(P0, "Island");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose_moved(P0, &[Entity::Object(bears)]);
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Heroes' Hangout");
    t.cast(P0, spell).modes(&[0]).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    let island = t.g.current(island);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(island), Zone::Exile);
    t.lands(P0, "Forest", 2);
    assert!(castable(&mut t, P0, bears));
    assert!(!can_play_land(&mut t, P0, island));
    // Patrol Night with one target, then with two.
    for n in [1, 2] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        t.lands(P0, "Mountain", 1);
        let spell = t.hand(P0, "Heroes' Hangout");
        let targets: Vec<Entity> = [a, b][..n].iter().map(|x| Entity::Object(*x)).collect();
        t.answer_targets(P0, &targets);
        t.cast(P0, spell).modes(&[1]).go();
        t.resolve_all();
        assert_eq!(t.pt(a), (3, 2));
        assert!(has_kw(&t, a, KeywordKind::FirstStrike));
        assert_eq!(t.pt(b), if n == 2 { (4, 3) } else { (3, 3) });
        assert_eq!(has_kw(&t, b, KeywordKind::FirstStrike), n == 2);
    }
}

#[test]
fn natural_emergence_returns_an_enchantment_without_targeting() {
    cr!("603.3d", "115.1");
    ruling!(
        "Natural Emergence",
        "You choose the enchantment to return on resolution of the triggered ability. This is not targeted."
    );
    let mut t = TestGame::new(2);
    let rage = t.battlefield(P0, "Rage Reflection");
    t.answer_choose(P0, &[Entity::Object(rage)]);
    let from = t.asked().len();
    enter(&mut t, P0, "Natural Emergence");
    let targets_on_trigger = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    t.resolve_all();
    assert_eq!(targets_on_trigger, 0);
    assert!(t.in_hand(P0, "Rage Reflection"));
    assert_eq!(t.named_on_battlefield("Natural Emergence").len(), 1);
}

#[test]
fn rise_of_the_hobgoblins_pays_x_on_resolution() {
    cr!("107.3", "608.2d", "605.3a");
    ruling!(
        "Rise of the Hobgoblins",
        "When the \"enters\" ability resolves, you choose the value of X, pay for it, and put the tokens onto the battlefield. You can activate mana abilities to pay for {X}, but no one can cast spells or activate abilities in response to this choice."
    );
    supported("Rise of the Hobgoblins");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    enter(&mut t, P0, "Rise of the Hobgoblins");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Goblin").len(), 2);
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn battleflight_eagle_must_target_but_may_target_itself() {
    cr!("603.3d", "115.1");
    ruling!(
        "Battleflight Eagle",
        "Battleflight Eagle’s ability is mandatory, although you can choose Battleflight Eagle as the target."
    );
    // The only creature: the Eagle targets itself.
    let mut t = TestGame::new(2);
    let eagle = enter(&mut t, P0, "Battleflight Eagle");
    t.resolve_all();
    assert_eq!(t.pt(eagle), (4, 4));
    // With another creature, choosing itself.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let eagle = enter(&mut t, P0, "Battleflight Eagle");
    let cands = target_candidates(&t, P0, from);
    assert!(cands[0].contains(&Entity::Object(eagle)));
    assert!(cands[0].contains(&Entity::Object(bears)));
    t.resolve_all();
    let boosted = [eagle, bears]
        .iter()
        .filter(|c| t.pt(**c) == (4, 4) || t.pt(**c) == (6, 4))
        .count();
    assert_eq!(boosted, 1);
}

// ---------------------------------------------------------------------------------------
// Targets: the same creature twice, a creature you control
// ---------------------------------------------------------------------------------------

#[test]
fn rally_maneuver_and_abandon_reason_target_different_creatures() {
    cr!("115.3", "601.2c");
    ruling!(
        "Rally Maneuver",
        "The first target will always get +2/+0 and first strike. If you target only one creature, you cannot choose to give it +0/+2 and lifelink."
    );
    ruling!("Rally Maneuver", "You cannot target the same creature twice.");
    ruling!(
        "Abandon Reason",
        "You can't target the same creature twice to have it get +2/+0."
    );
    supported("Rally Maneuver");
    supported("Abandon Reason");
    // One target: +2/+0 and first strike.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[]);
    cast_new(&mut t, P0, "Rally Maneuver", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert!(!has_kw(&t, bears, KeywordKind::Lifelink));
    // The second target can't be the first one.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    cast_new(&mut t, P0, "Rally Maneuver", &[]);
    let cands = target_candidates(&t, P0, from);
    assert!(!cands[1].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 5));
    // Abandon Reason: an answer naming the same creature twice is invalid; the default
    // (the first `min` = 0 candidates: no targets) is used, so it never gets +2/+0.
    interp!("default-first-listed-choice");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A second candidate, so two targets may be chosen.
    let wurm = t.battlefield(P0, "Craw Wurm");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(bears)]);
    lands_for_cost(&mut t, P0, "Abandon Reason");
    let spell = t.hand(P0, "Abandon Reason");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { max: 2, .. })));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(wurm), (6, 4));
    // Control: two different creatures each get +1/+0 and first strike.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    lands_for_cost(&mut t, P0, "Abandon Reason");
    let spell = t.hand(P0, "Abandon Reason");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(giant), (4, 3));
    assert!(has_kw(&t, giant, KeywordKind::FirstStrike));
}

#[test]
fn quick_draw_needs_a_creature_you_control_to_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Quick Draw",
        "You can’t cast Quick Draw unless you control a creature that’s a legal target for Quick Draw."
    );
    supported("Quick Draw");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Quick Draw");
    lands_for_cost(&mut t, P0, "Quick Draw");
    assert!(!castable(&mut t, P0, spell));
    t.battlefield(P0, "Hill Giant");
    assert!(castable(&mut t, P0, spell));
}

#[test]
fn warlords_fury_with_no_creatures_still_draws() {
    cr!("608.2c");
    ruling!(
        "Warlord's Fury",
        "You can cast Warlord’s Fury even if you control no creatures. You’ll still draw a card."
    );
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Warlord's Fury", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Statics and the cards' own text
// ---------------------------------------------------------------------------------------

#[test]
fn fencers_magemark_only_when_you_control_the_creature() {
    cr!("611.3a", "109.4");
    ruling!(
        "Fencer's Magemark",
        "Each Magemark gives its bonus to the creature it's attached to, as long as the same player controls both the creature and the Aura."
    );
    supported("Fencer's Magemark");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Fencer's Magemark", theirs);
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(!has_kw(&t, theirs, KeywordKind::FirstStrike));
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Fencer's Magemark", mine);
    assert_eq!(t.pt(mine), (3, 3));
    assert!(has_kw(&t, mine, KeywordKind::FirstStrike));
}

#[test]
fn neheb_the_worthy_pumps_itself_and_discards_in_apnap_order() {
    cr!("101.4", "611.3a");
    ruling!(
        "Neheb, the Worthy",
        "Neheb is affected by its own power-modifying ability once you have one or zero cards in hand."
    );
    ruling!(
        "Neheb, the Worthy",
        "When Neheb's triggered ability resolves, first the player whose turn it is chooses a card to discard, then each other player in turn order chooses a card to discard, then all of those cards are discarded simultaneously. No one sees what the other players are discarding before deciding which card to discard."
    );
    supported("Neheb, the Worthy");
    let mut t = TestGame::new(2);
    let neheb = t.battlefield(P0, "Neheb, the Worthy");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    assert_eq!(t.pt(neheb), (2, 2));
    t.hand(P1, "Island");
    t.hand(P1, "Forest");
    // Discard one: now one card in hand.
    let first = t.g.player(P0).hand[0];
    move_to(&mut t, first, Zone::Graveyard(P0));
    assert_eq!(t.pt(neheb), (4, 2));
    // Combat damage: P0 chooses, then P1, then both discard.
    attack_with(&mut t, &[(neheb, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    let from = t.asked().len();
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn justiciars_portal_returns_a_new_untapped_object() {
    cr!("400.7", "608.2b", "110.5b");
    ruling!(
        "Justiciar's Portal",
        "The creature returns untapped unless another effect causes it to enter the battlefield tapped."
    );
    ruling!(
        "Justiciar's Portal",
        "The returned card won’t be the target of any spells or abilities that targeted it before. Spells that don’t target, such as Kaya’s Wrath, may still affect it."
    );
    supported("Justiciar's Portal");
    // A tapped Hill Giant, targeted by Lightning Bolt, with Pyroclasm below on the
    // stack.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.tap(giant);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Pyroclasm", &[]);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(giant)]);
    cast_new(&mut t, P0, "Justiciar's Portal", &[Entity::Object(giant)]);
    t.resolve();
    let new_giant = t.g.current(giant);
    assert_ne!(new_giant, giant);
    assert!(!t.obj_now(new_giant).tapped);
    assert!(has_kw(&t, new_giant, KeywordKind::FirstStrike));
    // Bolt fizzles; Pyroclasm (2 to each creature) still hits the returned Giant.
    t.resolve();
    assert_eq!(t.obj_now(new_giant).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    t.resolve_all();
    assert_eq!(t.obj_now(new_giant).damage, 2);
}

#[test]
fn helm_of_kaldra_creates_kaldra() {
    cr!("111.4");
    ruling!(
        "Helm of Kaldra",
        "The token is named Kaldra, has the supertype legendary, and has creature type Avatar."
    );
    supported("Helm of Kaldra");
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Helm of Kaldra");
    let sword = t.battlefield(P0, "Sword of Kaldra");
    let shield = t.battlefield(P0, "Shield of Kaldra");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, helm, "Kaldra").unwrap();
    t.resolve_all();
    let kaldra = t.named_on_battlefield("Kaldra");
    assert_eq!(kaldra.len(), 1);
    let k = kaldra[0];
    assert!(legendary(&t, k));
    assert!(t.obj_now(k).chars.has_subtype("Avatar"));
    for e in [helm, sword, shield] {
        assert_eq!(attached_to(&t, e), Some(Entity::Object(k)));
    }
}

#[test]
fn runechanters_pike_counts_continuously() {
    cr!("611.3a", "604.3");
    ruling!(
        "Runechanter's Pike",
        "The value of X is constantly updated as instant cards and sorcery cards are put into or removed from your graveyard."
    );
    supported("Runechanter's Pike");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Runechanter's Pike", bears);
    assert_eq!(t.pt(bears), (2, 2));
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    move_to(&mut t, bolt, Zone::Exile);
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn rowans_talent_one_loyalty_ability_per_turn() {
    cr!("606.3");
    ruling!(
        "Rowan's Talent",
        "You can activate one loyalty ability of each planeswalker you control during each of your turns, whether that ability is one it always has or the one it gains because of Rowan's Talent."
    );
    supported("Rowan's Talent");
    // Ajani, Inspiring Leader with Rowan's Talent: the granted +1, then nothing else.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    attach_new(&mut t, P0, "Rowan's Talent", ajani);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, ajani, "first strike and trample").unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert!(!can_activate(&mut t, P0, ajani));
    // Or one of Ajani's own, then not the granted one.
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani, Inspiring Leader");
    attach_new(&mut t, P0, "Rowan's Talent", ajani);
    loyalty(&mut t, P0, ajani, 0, &[]);
    t.resolve_all();
    assert!(!can_activate(&mut t, P0, ajani));
}
