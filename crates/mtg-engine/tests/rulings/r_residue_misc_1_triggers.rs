//! Item `rulings-residue-misc-1` — last known information, combat damage triggers, and
//! activated abilities' costs and timing: Vincent Valentine, Susur Secundi (filed under
//! Survey Mechan), Hordewing Skaab, Nettling Nuisance, Zaffai, Alluring Suitor, Battle Cry
//! Goblin, Ghired, Seafloor Stalker, Rift Elemental, Hex Parasite, Insatiable Frugivore,
//! Avatar of Fury, Flamescroll Celebrant, Mage il-Vec and Ennis.

use crate::r_residue_misc_1_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Pumps the creature +3/+3 with a resolved Giant Growth of its controller's.
fn giant_growth(t: &mut TestGame, id: ObjectId) {
    let c = t.obj_now(id).controller;
    cast_resolve(t, c, "Giant Growth", &[obj(id)]);
}

/// How many times an ability of the card named `name` resolved so far.
fn resolutions(t: &TestGame, name: &str) -> usize {
    t.dump_log()
        .lines()
        .filter(|l| l.contains(&format!("Resolving {name} ability")))
        .count()
}

// ---------------------------------------------------------------------------------------
// Last known information
// ---------------------------------------------------------------------------------------

#[test]
fn vincent_valentine_counts_the_dead_creatures_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Vincent Valentine // Galian Beast",
        "Use the creature's power as it last existed on the battlefield to determine how many +1/+1 counters to put on Vincent Valentine."
    );
    supported("Vincent Valentine // Galian Beast");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vincent Valentine // Galian Beast");
    let bears = t.battlefield(P1, "Grizzly Bears");
    giant_growth(&mut t, bears);
    assert_eq!(pt_now(&mut t, bears), (5, 5));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(p1(&t, v), 5);
}

#[test]
fn susur_secundi_draws_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h", "118.3", "602.2b");
    // Misattached: the data files this ruling under Survey Mechan; it describes Susur
    // Secundi, Void Altar ("12+ | {1}{B}, {T}, Pay 2 life, Sacrifice a creature: Draw cards
    // equal to the sacrificed creature's power."). Survey Mechan takes part as the
    // creature sacrificed.
    ruling!(
        "Survey Mechan",
        "Use the sacrificed creature’s power as it last existed on the battlefield to determine how many cards to draw with Susur Secundi’s last ability."
    );
    supported("Susur Secundi, Void Altar");
    supported("Survey Mechan");
    let mut t = TestGame::new(2);
    let susur = t.battlefield(P0, "Susur Secundi, Void Altar");
    crate::r_p108_common::put_counters(&mut t, susur, counters::CHARGE, 12);
    let mechan = t.battlefield(P0, "Survey Mechan");
    giant_growth(&mut t, mechan);
    assert_eq!(pt_now(&mut t, mechan), (4, 6));
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(mechan)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, susur, "Draw cards").unwrap();
    assert!(t.in_graveyard(P0, "Survey Mechan"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------------------
// Combat damage triggers in each combat damage step
// ---------------------------------------------------------------------------------------

#[test]
fn hordewing_skaab_triggers_in_each_combat_damage_step() {
    cr!("510.4", "603.2c");
    ruling!(
        "Hordewing Skaab",
        "If some of your Zombies have first strike but others don't, or if you have a Zombie with double strike, the ability will trigger twice, the first time counting the number of opponents who were dealt damage in the first strike combat damage step, and the second time counting the ones who were dealt damage in the second combat damage step."
    );
    supported("Hordewing Skaab");
    // A first-striking Zombie attacks P1 and a regular one P2: one trigger in each step,
    // each counting one opponent; then a double-striking Skaab: two triggers.
    for double in [false, true] {
        let mut t = TestGame::new(3);
        let skaab = t.battlefield(P0, "Hordewing Skaab");
        let mut attackers = vec![];
        if double {
            grant_kw(
                &mut t,
                skaab,
                mtg_engine::keywords::KeywordKind::DoubleStrike,
            );
            attackers.push((skaab, Entity::Player(P1)));
        } else {
            let corpse = t.battlefield(P0, "Walking Corpse");
            grant_kw(
                &mut t,
                corpse,
                mtg_engine::keywords::KeywordKind::FirstStrike,
            );
            attackers.push((corpse, Entity::Player(P1)));
            attackers.push((skaab, Entity::Player(P2)));
        }
        for _ in 0..2 {
            t.answer_yes(P0, true);
        }
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        t.attack(&attackers, &[]);
        t.resolve_all();
        assert_eq!(resolutions(&t, "Hordewing Skaab"), 2, "double: {double}");
        // Each draws one (one opponent dealt damage that step), then discards one.
        let draws = count_asked(&t, from, |d| matches!(d, Decision::YesNo { .. }));
        assert_eq!(draws, 2);
        assert_eq!(t.hand_size(P0), hand);
        assert_eq!(t.graveyard_size(P0), 2);
    }
}

#[test]
fn nettling_nuisance_triggers_once_per_player_per_combat_damage_step() {
    cr!("510.2", "510.4", "603.2c");
    ruling!(
        "Nettling Nuisance",
        "Normally, combat damage is dealt all at the same time. In that case, Nettling Nuisance's ability triggers once for each player that Faeries you control dealt combat damage to, regardless of how many creatures were dealing that damage. If any of those Faeries have double strike or first strike, the ability will trigger once for each player dealt damage in each combat damage step."
    );
    supported("Nettling Nuisance");
    let mut t = TestGame::new(3);
    let nuisance = t.battlefield(P0, "Nettling Nuisance");
    let miscreant = t.battlefield(P0, "Faerie Miscreant");
    let sprite = t.battlefield(P0, "Spellstutter Sprite");
    grant_kw(
        &mut t,
        sprite,
        mtg_engine::keywords::KeywordKind::DoubleStrike,
    );
    // Two Faeries at P1 (at once): one token; a double striker at P2: two.
    t.attack(
        &[
            (nuisance, Entity::Player(P1)),
            (miscreant, Entity::Player(P1)),
            (sprite, Entity::Player(P2)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1, "Pirate"), 1);
    assert_eq!(tokens_of(&t, P2, "Pirate"), 2);
}

// ---------------------------------------------------------------------------------------
// Zaffai, Thunder Conductor: "If that spell's mana value is 5 or greater, create a 4/4
// blue and red Elemental creature token."
// ---------------------------------------------------------------------------------------

#[test]
fn zaffai_counts_x_in_the_spells_mana_value() {
    cr!("202.3e", "107.3f");
    ruling!(
        "Zaffai, Thunder Conductor",
        "If the instant or sorcery spell has {X} in its mana cost, X is the value chosen for X as the spell was cast."
    );
    supported("Zaffai, Thunder Conductor");
    for (x, elementals) in [(4, 1), (3, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Zaffai, Thunder Conductor");
        t.lands(P0, "Mountain", 1 + x as usize);
        let blaze = t.hand(P0, "Blaze");
        t.cast(P0, blaze).x(x).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(tokens_of(&t, P0, "Elemental"), elementals, "X = {x}");
        assert_eq!(t.life(P1), 20 - x as i32);
    }
}

// ---------------------------------------------------------------------------------------
// Creatures put onto the battlefield attacking weren't declared as attackers (CR 508.3a,
// 508.4)
// ---------------------------------------------------------------------------------------

#[test]
fn alluring_suitor_doesnt_count_a_creature_put_onto_the_battlefield_attacking() {
    cr!("508.4", "508.3a");
    ruling!(
        "Alluring Suitor // Deadly Dancer",
        "A creature put onto the battlefield attacking didn't \"attack\" and won't be counted to determine whether Alluring Suitor's ability should trigger."
    );
    supported("Alluring Suitor // Deadly Dancer");
    // Suitor and Battle Cry Goblin (total power 6 with the Goblin's pump) attack: the
    // Goblin token put onto the battlefield attacking makes three attacking creatures, but
    // P0 attacked with exactly two.
    let mut t = TestGame::new(2);
    let suitor = t.battlefield(P0, "Alluring Suitor // Deadly Dancer");
    let goblin = t.battlefield(P0, "Battle Cry Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    giant_growth(&mut t, goblin);
    t.attack(
        &[(suitor, Entity::Player(P1)), (goblin, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    let _ = giant;
    assert_eq!(tokens_of(&t, P0, "Goblin"), 1);
    assert_eq!(name_now(&t, suitor), "Deadly Dancer");
}

#[test]
fn battle_cry_goblins_pump_misses_the_goblin_token_created_later() {
    cr!("611.2c", "508.4");
    ruling!(
        "Battle Cry Goblin",
        "Battle Cry Goblin's activated ability will affect only Goblins you control as it resolves. If you activate it before attacking and then create a Goblin with its triggered ability during combat, the Goblin you create will not get +1/+0."
    );
    supported("Battle Cry Goblin");
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Battle Cry Goblin");
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, goblin, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, goblin), (3, 2));
    // 3 + 3 + 2 = 8 power: pack tactics creates a tapped and attacking 1/1 Goblin.
    attack_with(
        &mut t,
        &[
            (goblin, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
            (other, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert!(is_attacking(&t, token));
    assert_eq!(pt_now(&mut t, token), (1, 1));
    assert_eq!(pt_now(&mut t, other), (2, 1));
}

#[test]
fn ghireds_populated_token_attacks_without_triggering_attack_abilities() {
    cr!("508.4", "508.3a", "701.36a");
    ruling!(
        "Ghired, Conclave Exile",
        "Although the token is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    supported("Ghired, Conclave Exile");
    supported("Hellrider");
    let mut t = TestGame::new(2);
    let ghired = t.battlefield(P0, "Ghired, Conclave Exile");
    t.battlefield(P0, "Hellrider");
    let rhino = create_token(&mut t, P0, "Rhino");
    t.answer_choose(P0, &[obj(rhino)]);
    attack_with(&mut t, &[(ghired, Entity::Player(P1))]);
    // Hellrider's "Whenever a creature you control attacks" triggers once (Ghired).
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    let rhinos = tokens_of(&t, P0, "Rhino");
    assert_eq!(rhinos, 2);
    assert_eq!(triggers_on_stack(&t, "deals 1 damage"), 0);
}

#[test]
fn ghired_copying_a_token_thats_not_normally_a_creature_doesnt_attack() {
    cr!("707.2", "506.3", "508.4");
    ruling!(
        "Ghired, Conclave Exile",
        "In some rare cases, you might copy a creature token that isn't normally a creature. This token enters the battlefield as a noncreature permanent. It will be tapped but it won't be attacking."
    );
    let mut t = TestGame::new(2);
    let ghired = t.battlefield(P0, "Ghired, Conclave Exile");
    let clue = create_token(&mut t, P0, "Clue");
    animate(&mut t, clue, 2, 2);
    t.answer_choose(P0, &[obj(clue)]);
    attack_with(&mut t, &[(ghired, Entity::Player(P1))]);
    t.resolve_all();
    let clues: Vec<ObjectId> = with_subtype(&t, P0, "Clue");
    assert_eq!(clues.len(), 2);
    let copy = *clues.iter().find(|c| **c != clue).unwrap();
    assert!(!t.g.obj(copy).is(CardType::Creature));
    assert!(tapped(&t, copy));
    assert!(!is_attacking(&t, copy));
}

// ---------------------------------------------------------------------------------------
// Seafloor Stalker: "{4}{U}: This creature gets +1/+0 until end of turn and can't be
// blocked this turn. This ability costs {1} less to activate for each creature in your
// party."
// ---------------------------------------------------------------------------------------

#[test]
fn seafloor_stalker_already_blocked_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Seafloor Stalker",
        "Once a creature has blocked Seafloor Stalker, activating its ability won't cause it to become unblocked or remove that blocking creature from combat."
    );
    supported("Seafloor Stalker");
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Seafloor Stalker");
    let wall = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    attack_with(&mut t, &[(stalker, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(wall, stalker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.activate(P0, stalker, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, stalker), (3, 3));
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Seafloor Stalker"));
}

/// Activates Seafloor Stalker's ability paying from a pool of plenty of mana; the mana
/// spent.
fn stalker_cost(t: &mut TestGame, stalker: ObjectId) -> usize {
    tap_lands(t, P0);
    fill_pool(t, P0, 6);
    let before = pool(t, P0);
    t.activate(P0, stalker, 0, &[]).unwrap();
    before - pool(t, P0)
}

#[test]
fn seafloor_stalkers_cost_is_locked_in_once_activation_begins() {
    cr!("602.2", "601.2f", "700.8");
    ruling!(
        "Seafloor Stalker",
        "Once you declare that you're activating Seafloor Stalker's ability, no player can take actions to try and change the number of creatures in your party and change the activation cost."
    );
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Seafloor Stalker");
    t.battlefield(P0, "Thelonite Druid");
    t.lands(P1, "Mountain", 1);
    t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    // A Rogue (itself) and a Cleric: {2}{U}.
    assert_eq!(stalker_cost(&mut t, stalker), 3);
    assert!(others_asked_since(&t, from, P0).is_empty());
}

#[test]
fn seafloor_stalker_can_activate_again_when_already_unblockable() {
    cr!("602.1", "611.2a");
    ruling!(
        "Seafloor Stalker",
        "You can activate Seafloor Stalker's ability more than once, even if it already can't be blocked this turn."
    );
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Seafloor Stalker");
    for _ in 0..2 {
        assert_eq!(stalker_cost(&mut t, stalker), 4);
        t.resolve_all();
    }
    assert_eq!(pt_now(&mut t, stalker), (4, 3));
}

// ---------------------------------------------------------------------------------------
// Rift Elemental: "{1}{R}, Remove a time counter from a permanent you control or suspended
// card you own: This creature gets +2/+0 until end of turn."
// ---------------------------------------------------------------------------------------

#[test]
fn rift_elemental_removing_the_last_time_counter_triggers_above_its_ability() {
    cr!("702.62a", "702.63a", "603.3", "117.5");
    ruling!(
        "Rift Elemental",
        "If an suspended card's last time counter is removed this way, the suspend triggered ability is put on the stack on top of the Rift Elemental ability. The same is true for the vanishing ability of a permanent that loses its last time counter."
    );
    ruling!(
        "Rift Elemental",
        "The counter is removed as a cost, so it can't be responded to."
    );
    supported("Rift Elemental");
    for vanishing in [false, true] {
        let mut t = TestGame::new(2);
        let rift = t.battlefield(P0, "Rift Elemental");
        t.lands(P0, "Mountain", 2);
        let with = if vanishing {
            let c = t.battlefield(P0, "Calciderm");
            t.g.objects[c.0 as usize]
                .counters
                .insert(counters::TIME.into(), 1);
            c
        } else {
            let c = t.exile(P0, "Rift Bolt");
            t.g.add_counters(Entity::Object(c), counters::TIME, 1, None);
            c
        };
        t.g.recompute();
        t.answer_choose(P0, &[obj(with)]);
        let ability = t.activate(P0, rift, 0, &[]).unwrap().unwrap();
        // Gone as the ability is activated: nothing to respond to.
        assert_eq!(t.counters(with, counters::TIME), 0);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert_eq!(t.g.stack[0], ability);
        let top = *t.g.stack.last().unwrap();
        assert!(matches!(
            t.g.obj(top).stack.as_ref().unwrap().kind,
            mtg_engine::object::StackKind::Triggered { source, .. } if source == t.g.current(with)
        ));
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(pt_now(&mut t, rift), (3, 1));
        if vanishing {
            assert!(t.in_graveyard(P0, "Calciderm"));
        }
    }
}

// ---------------------------------------------------------------------------------------
// Hex Parasite: "{X}{B/P}: Remove up to X counters from target permanent. For each
// counter removed this way, this creature gets +1/+0 until end of turn."
// ---------------------------------------------------------------------------------------

#[test]
fn hex_parasite_chooses_how_many_of_each_kind_on_resolution() {
    cr!("608.2c", "122.8");
    ruling!(
        "Hex Parasite",
        "If the permanent has multiple kinds of counters, you choose how many of each to remove when the ability resolves."
    );
    supported("Hex Parasite");
    let mut t = TestGame::new(2);
    let parasite = t.battlefield(P0, "Hex Parasite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p108_common::put_counters(&mut t, bears, counters::PLUS1, 2);
    crate::r_p108_common::put_counters(&mut t, bears, counters::CHARGE, 2);
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, parasite, 0, &[obj(bears)]).unwrap();
    // Nothing chosen yet on activation.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    // Both counters removed are charge counters, chosen one at a time on resolution.
    let labels =
        t.g.obj(bears)
            .counters
            .keys()
            .map(|k| k.to_string())
            .collect::<Vec<_>>();
    let ci = labels.iter().position(|k| k == counters::CHARGE).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(ci));
    t.answer(P0, DecisionKind::Option, Answer::Index(ci));
    t.resolve_all();
    assert!(count_asked(&t, from, |d| matches!(d, Decision::ChooseOption { .. })) >= 2);
    assert_eq!(t.counters(bears, counters::CHARGE), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(pt_now(&mut t, parasite), (3, 1));
}

// ---------------------------------------------------------------------------------------
// Insatiable Frugivore: "When this creature enters, create a Food token, then you may
// exile three cards from your graveyard. If you do, repeat this process. / {3}{B},
// Sacrifice X Foods: Creatures you control get +X/+0 and gain menace until end of turn."
// ---------------------------------------------------------------------------------------

#[test]
fn insatiable_frugivore_repeats_until_declined_or_unable() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Insatiable Frugivore",
        "The process in Insatiable Frugivore's first ability is repeated until you decline to exile three cards from your graveyard. You can't try to exile three cards from your graveyard if you don't have at least three cards in your graveyard."
    );
    ruling!(
        "Insatiable Frugivore",
        "Players can't take other actions during the resolution of Insatiable Frugivore's first ability. Notably, they can't try to remove cards from your graveyard between repetitions of this process."
    );
    supported("Insatiable Frugivore");
    // Seven cards: two repetitions exile six; with one left, no third offer.
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[0..3].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &cards[3..6].iter().map(|c| obj(*c)).collect::<Vec<_>>());
    enter(&mut t, P0, "Insatiable Frugivore");
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Food"), 3);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::YesNo { .. })),
        2
    );
    // No priority for anyone while it resolved (only P0's choices were asked).
    assert!(others_asked_since(&t, from, P0).is_empty());
    // Declining stops it: one more Food, the graveyard untouched.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.answer_yes(P0, false);
    enter(&mut t, P0, "Insatiable Frugivore");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Food"), 1);
    assert_eq!(t.graveyard_size(P0), 4);
}

#[test]
fn insatiable_frugivore_x_of_zero_just_grants_menace() {
    cr!("107.3", "702.111a");
    ruling!(
        "Insatiable Frugivore",
        "You can choose 0 for the value of X in the cost of Insatiable Frugivore's last ability. If you do, creatures you control will just gain menace until end of turn. (The mere idea of someone eating berries like that is scary enough, apparently.)"
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Insatiable Frugivore");
    let bears = t.battlefield(P0, "Grizzly Bears");
    create_token(&mut t, P0, "Food");
    t.lands(P0, "Swamp", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer_choose(P0, &[]);
    activate_containing(&mut t, P0, f, "Sacrifice X Foods").unwrap();
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Food"), 1);
    assert_eq!(pt_now(&mut t, bears), (2, 2));
    assert!(has(
        &mut t,
        bears,
        mtg_engine::keywords::KeywordKind::Menace
    ));
    assert!(has(&mut t, f, mtg_engine::keywords::KeywordKind::Menace));
}

// ---------------------------------------------------------------------------------------
// Avatar of Fury: "If an opponent controls seven or more lands, this spell costs {6} less
// to cast."
// ---------------------------------------------------------------------------------------

#[test]
fn avatar_of_fury_mana_value_stays_eight_when_reduced() {
    cr!("202.3", "601.2f");
    ruling!(
        "Avatar of Fury",
        "The mana value of this card is still 8, even if you only pay {R}{R} to cast it."
    );
    supported("Avatar of Fury");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 7);
    let avatar = t.hand(P0, "Avatar of Fury");
    assert_eq!(cost_to_cast(&mut t, P0, avatar, &[]), Some(2));
    let spell = t.g.current(avatar);
    assert_eq!(t.g.mana_value_of(spell), 8);
    t.resolve_all();
    let a = t.named_on_battlefield("Avatar of Fury")[0];
    assert_eq!(t.g.mana_value_of(a), 8);
}

// ---------------------------------------------------------------------------------------
// Flamescroll Celebrant // Revel in Silence: "Whenever an opponent activates an ability
// that isn't a mana ability, this creature deals 1 damage to that player." // "Your
// opponents can't cast spells or activate planeswalkers' loyalty abilities this turn."
// ---------------------------------------------------------------------------------------

const CELEBRANT: &str = "Flamescroll Celebrant // Revel in Silence";

#[test]
fn flamescroll_celebrant_a_mana_cost_doesnt_make_a_mana_ability() {
    cr!("605.1a", "602.1");
    ruling!(
        "Flamescroll Celebrant // Revel in Silence",
        "An activated ability that costs mana to activate is not a “mana ability” unless it could also produce mana and has no targets."
    );
    supported(CELEBRANT);
    let mut t = TestGame::new(2);
    t.battlefield(P0, CELEBRANT);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    // A mana ability: no trigger.
    activate_containing(&mut t, P1, elves, "Add").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // {R}: +1/+0 costs mana but makes none: it triggers.
    t.lands(P1, "Mountain", 1);
    activate_containing(&mut t, P1, dragon, "+1/+0").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn revel_in_silence_in_response_doesnt_stop_the_spell() {
    cr!("101.2", "608.1");
    ruling!(
        "Flamescroll Celebrant // Revel in Silence",
        "Casting Revel in Silence in response to a spell or ability won’t affect that spell or ability. That is, it’s not countermagic."
    );
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    lands_for_cost(&mut t, P0, "Revel in Silence");
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, CELEBRANT);
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // P1 can't cast another spell this turn.
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(!can_cast(&mut t, P1, bolt, CastMethod::Normal));
}

// ---------------------------------------------------------------------------------------
// Mage il-Vec: "{T}, Discard a card at random: This creature deals 1 damage to any target."
// ---------------------------------------------------------------------------------------

#[test]
fn mage_il_vec_needs_a_card_to_discard() {
    cr!("602.2b", "118.3", "601.2h");
    ruling!(
        "Mage il-Vec",
        "You must discard a card as part of the activation cost. You can't announce the ability unless you have a card in hand."
    );
    supported("Mage il-Vec");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Mage il-Vec");
    assert!(!can_activate(&mut t, P0, mage));
    t.hand(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, mage));
    t.activate(P0, mage, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------------------
// Ennis, Debate Moderator: "At the beginning of your end step, if one or more cards were
// put into exile this turn, put a +1/+1 counter on Ennis."
// ---------------------------------------------------------------------------------------

#[test]
fn ennis_doesnt_count_the_copy_of_a_prepare_spell() {
    cr!("722.3c", "707.10", "603.4");
    ruling!(
        "Ennis, Debate Moderator",
        "Since copies of prepare spells aren't cards, they won't count for Ennis's last ability."
    );
    supported("Ennis, Debate Moderator");
    for exile_a_card in [false, true] {
        let mut t = TestGame::new(2);
        let ennis = t.battlefield(P0, "Ennis, Debate Moderator");
        // Goblin Glasswright enters prepared: a copy of its prepare spell is created in
        // exile.
        let g = t.enter(P0, "Goblin Glasswright // Craft with Pride");
        assert!(t.obj_now(g).prepared.is_some());
        if exile_a_card {
            let c = t.graveyard(P1, "Grizzly Bears");
            exile_card(&mut t, c);
        }
        through_end_step(&mut t, P0);
        assert_eq!(p1(&t, ennis), u32::from(exile_a_card));
    }
}
