//! CR 801: the limited range of influence option.

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn range_of_influence_is_a_distance_in_seats_including_yourself() {
    cr!("801.1", "801.2", "801.2b");
    // Five players around the table with a range of influence of 1: each player reaches
    // the players next to them, and themselves.
    let t = ranged(5, 1);
    assert_eq!(t.g.players_in_range(P0), vec![P0, P1, P4]);
    assert_eq!(t.g.players_in_range(P2), vec![P1, P2, P3]);
    // A range of 2 reaches everyone at this table.
    let t2 = ranged(5, 2);
    assert_eq!(t2.g.players_in_range(P0).len(), 5);
    // Without the option, influence is unlimited.
    let free = TestGame::with_config(5, GameConfig::free_for_all());
    assert_eq!(free.g.range_of_influence(P0), None);
    assert_eq!(free.g.players_in_range(P0).len(), 5);
    // The option is always used in the Emperor variant.
    let emperor = TestGame::with_config(6, GameConfig::emperor(vec![0, 0, 0, 1, 1, 1]));
    assert!(emperor.g.range_of_influence(P0).is_some());
}

#[test]
fn players_may_have_different_ranges_of_influence() {
    cr!("801.2a", "801.4");
    // P0 has a range of 2; everyone else has 1.
    let mut t = TestGame::with_config(
        6,
        GameConfig {
            range_of_influence: Some(1),
            player_ranges: vec![(P0, 2)],
            ..GameConfig::free_for_all()
        },
    );
    let c = bolt_candidates(&mut t, P0);
    for p in [P1, P2, P4, P5] {
        assert!(c.contains(&Entity::Player(p)), "{p}");
    }
    assert!(!c.contains(&Entity::Player(P3)));
    // P2 has a range of 1: P0 is two seats away.
    let c = bolt_candidates(&mut t, P2);
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(P3)));
    assert!(!c.contains(&Entity::Player(P0)) && !c.contains(&Entity::Player(P4)));
}

#[test]
fn players_within_range_are_determined_as_each_turn_begins() {
    cr!("801.2c");
    let mut t = ranged(5, 1);
    let far = bear(&mut t, P3);
    // P1's turn begins: P2 and P0 are within P1's range; P3 isn't.
    to_turn_of(&mut t, P1);
    assert!(!t.g.players_in_range(P1).contains(&P3));
    // P2 leaves the game during P1's turn. P1 and P3 are now next to each other, but they
    // don't enter each other's range of influence until the next turn begins.
    concede(&mut t, P2);
    assert!(!t.g.players_in_range(P1).contains(&P3));
    assert!(!mtg_engine::multiplayer::range::object_in_range(&t.g, P1, far));
    // P3's turn (P2's is skipped).
    to_turn_of(&mut t, P3);
    assert!(t.g.players_in_range(P1).contains(&P3));
    assert!(mtg_engine::multiplayer::range::object_in_range(&t.g, P1, far));
}

#[test]
fn objects_are_within_range_by_controller_and_battles_by_protector() {
    cr!("801.2d");
    let mut t = ranged(5, 1);
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P2);
    // A battle controlled by P2 but protected by P1 is within P0's range; one controlled
    // by P1 is too, whoever protects it.
    let b1 = bf(
        &mut t,
        P2,
        with_counters_base(custom_with("Far Siege", "Battle — Siege", None, vec![]), None, Some(5)),
    );
    set_protector(&mut t, b1, P1);
    let b2 = bf(
        &mut t,
        P1,
        with_counters_base(custom_with("Near Siege", "Battle — Siege", None, vec![]), None, Some(5)),
    );
    set_protector(&mut t, b2, P3);
    let b3 = bf(
        &mut t,
        P2,
        with_counters_base(custom_with("Distant Siege", "Battle — Siege", None, vec![]), None, Some(5)),
    );
    set_protector(&mut t, b3, P3);
    let c = bolt_candidates(&mut t, P0);
    assert!(c.contains(&Entity::Object(near)));
    assert!(!c.contains(&Entity::Object(far)));
    assert!(c.contains(&Entity::Object(b1)));
    assert!(c.contains(&Entity::Object(b2)));
    assert!(!c.contains(&Entity::Object(b3)));
}

#[test]
fn creatures_can_attack_only_opponents_within_range() {
    cr!("801.3");
    let mut t = ranged(5, 1);
    let bear0 = bear(&mut t, P0);
    let pw_near = t.battlefield(P1, "Jace Beleren");
    let pw_far = t.battlefield(P2, "Jace Beleren");
    let opts = attack_choices(&mut t, P0);
    let targets = targets_of(&opts, bear0);
    assert!(targets.contains(&Entity::Player(P1)));
    assert!(targets.contains(&Entity::Player(P4)));
    assert!(targets.contains(&Entity::Object(pw_near)));
    assert!(!targets.contains(&Entity::Player(P2)));
    assert!(!targets.contains(&Entity::Player(P3)));
    assert!(!targets.contains(&Entity::Object(pw_far)));
    // The players next to P0 are its teammates: no opponent is within range, so P0's
    // creatures can't attack.
    let mut t = with_teams(
        &[0, 0, 1, 1, 0],
        GameConfig {
            range_of_influence: Some(1),
            ..GameConfig::team_vs_team(vec![])
        },
    );
    let lonely = bear(&mut t, P0);
    let opts = attack_choices(&mut t, P0);
    assert!(targets_of(&opts, lonely).is_empty());
    assert!(t.g.combat.as_ref().unwrap().defending_players.is_empty());
}

#[test]
fn spells_and_abilities_cant_target_outside_range() {
    cr!("801.4");
    let mut t = ranged(5, 1);
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P3);
    let c = bolt_candidates(&mut t, P0);
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(P4)));
    assert!(c.contains(&Entity::Object(near)));
    assert!(!c.contains(&Entity::Player(P2)) && !c.contains(&Entity::Player(P3)));
    assert!(!c.contains(&Entity::Object(far)));
    // Abilities too: "{T}: This creature deals 1 damage to any target."
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, pyro, 0, &[]).unwrap();
    let c = last_target_candidates(&t, P0);
    assert!(c.contains(&Entity::Object(near)));
    assert!(!c.contains(&Entity::Object(far)) && !c.contains(&Entity::Player(P3)));
}

#[test]
fn a_chosen_object_or_player_must_be_within_range() {
    cr!("801.5", "801.5a");
    let mut t = ranged(5, 1);
    // "As this creature enters, choose an opponent." P0 tries to choose P2.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let nyx = t.enter(P0, "Nyxathid");
    let offered = last_entity_candidates(&t, P0);
    assert!(offered.contains(&Entity::Player(P1)) && offered.contains(&Entity::Player(P4)));
    assert!(!offered.contains(&Entity::Player(P2)) && !offered.contains(&Entity::Player(P3)));
    let chosen = t.obj_now(nyx).choices.player;
    assert!(matches!(chosen, Some(p) if p == P1 || p == P4), "{chosen:?}");
}

#[test]
fn a_choice_between_options_can_refer_to_things_outside_range() {
    cr!("801.5b", "801.10");
    let mut t = ranged(5, 1);
    // Pithing Needle: "As this artifact enters, choose a card name." P0 may name a card
    // that only players outside their range of influence have.
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Prodigal Pyromancer".into()),
    );
    t.enter(P0, "Pithing Needle");
    let near = t.battlefield(P1, "Prodigal Pyromancer");
    let far = t.battlefield(P2, "Prodigal Pyromancer");
    // The name was chosen, and the Needle stops the one within P0's range; it can't affect
    // the one outside it.
    assert!(t.activate(P1, near, 0, &[Entity::Player(P0)]).is_err());
    assert!(t.activate(P2, far, 0, &[Entity::Player(P3)]).is_ok());
}

#[test]
fn with_no_opponent_in_range_the_closest_one_to_the_left_chooses() {
    cr!("801.5c");
    // P0's neighbors are teammates. "An opponent chooses one of those piles."
    let mut t = with_teams(
        &[0, 0, 1, 1, 0],
        GameConfig {
            range_of_influence: Some(1),
            ..GameConfig::team_vs_team(vec![])
        },
    );
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 1);
    let augury = t.hand(P0, "Steam Augury");
    t.cast(P0, augury).go();
    t.resolve();
    // P2 (two seats to the left) is the closest opponent to P0's left; P3 isn't asked.
    let choosers: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert!(choosers.contains(&P2), "{choosers:?}");
    assert!(!choosers.contains(&P3));
}

#[test]
fn abilities_of_objects_outside_range_cant_be_activated() {
    cr!("801.6");
    // Lethal Vapors: "{0}: Destroy this enchantment. You skip your next turn. Any player
    // may activate this ability."
    let mut t = ranged(5, 1);
    let vapors = t.battlefield(P0, "Lethal Vapors");
    t.g.turn.priority = Some(P2);
    assert!(t.activate(P2, vapors, 0, &[]).is_err());
    t.g.turn.priority = Some(P1);
    assert!(t.activate(P1, vapors, 0, &[]).is_ok());
}

#[test]
fn triggers_need_the_event_entirely_within_range() {
    cr!("801.7", "801.7a");
    // Blood Artist: "Whenever this creature or another creature dies, target player loses
    // 1 life and you gain 1 life."
    let mut t = ranged(5, 1);
    t.battlefield(P0, "Blood Artist");
    // P2 (outside P0's range) owns this creature, but P1 (within range) controls it.
    let near = bear(&mut t, P2);
    apply(
        &mut t,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[near],
    );
    let far = bear(&mut t, P2);
    let before = t.life(P0);
    // It dies, going to P2's graveyard, outside P0's range. The event is judged by the game
    // state before it: the creature was controlled by P1 on the battlefield, within range,
    // so the ability triggers.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(near, None);
    t.settle();
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), before + 1);
    // A creature controlled by a player outside range dies: no trigger.
    t.g.destroy(far, None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), before + 1);
}

#[test]
fn auras_cant_enchant_outside_range() {
    cr!("801.8");
    let mut t = ranged(5, 1);
    // P0's Pacifism enchants P1's creature. P2 gains control of it: it's now outside P0's
    // range of influence, so Pacifism is put into its owner's graveyard.
    let target = bear(&mut t, P1);
    t.lands(P0, "Plains", 2);
    let pac = t.hand(P0, "Pacifism");
    t.cast(P0, pac).target(target).go();
    t.resolve();
    assert!(on_bf(&t, pac));
    apply(
        &mut t,
        P2,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[target],
    );
    assert_eq!(t.obj_now(target).controller, P2);
    t.settle();
    assert!(t.in_graveyard(P0, "Pacifism"));
}

#[test]
fn equipment_becomes_unattached_outside_range() {
    cr!("801.9");
    let mut t = ranged(5, 1);
    let own = bear(&mut t, P0);
    let split = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(split, Entity::Object(own)));
    t.g.recompute();
    assert_eq!(t.pt(own), (4, 2));
    // P2 gains control of the equipped creature: outside P0's range, so the Equipment
    // becomes unattached but stays on the battlefield.
    apply(
        &mut t,
        P2,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[own],
    );
    t.settle();
    assert!(on_bf(&t, split));
    assert_eq!(t.obj_now(split).attached_to, None);
    assert_eq!(t.pt(own), (2, 2));
}

#[test]
fn spells_affect_only_what_is_within_range() {
    cr!("801.10");
    let mut t = ranged(5, 1);
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P2);
    let own = bear(&mut t, P0);
    // Wrath of God: "Destroy all creatures. They can't be regenerated."
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    assert!(!on_bf(&t, near) && !on_bf(&t, own));
    assert!(on_bf(&t, far), "a creature outside P0's range isn't destroyed");
    // Static abilities: Bad Moon ("Black creatures get +1/+1") affects only black
    // creatures within its controller's range.
    let near_black = t.battlefield(P4, "Vampire Nighthawk");
    let far_black = t.battlefield(P3, "Vampire Nighthawk");
    t.battlefield(P0, "Bad Moon");
    assert_eq!(t.pt(near_black), (3, 4));
    assert_eq!(t.pt(far_black), (2, 3));
}

#[test]
fn abilities_see_only_information_within_range() {
    cr!("801.11");
    let mut t = ranged(5, 1);
    // Lord of Extinction: "power and toughness are each equal to the number of cards in
    // all graveyards" — all graveyards it can see.
    let lord = t.battlefield(P0, "Lord of Extinction");
    for _ in 0..2 {
        t.graveyard(P1, "Grizzly Bears");
    }
    for _ in 0..3 {
        t.graveyard(P2, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.pt(lord), (2, 2), "P2's graveyard is outside P0's range");
}

#[test]
fn the_world_rule_considers_only_worlds_within_range() {
    cr!("801.12");
    let mut t = ranged(5, 1);
    // Concordant Crossroads is a world enchantment. P0's and P2's are outside each other's
    // range, so both stay; P1's is within range of both and is newer, so the older ones
    // within its range go.
    let a = t.enter(P0, "Concordant Crossroads");
    let b = t.enter(P2, "Concordant Crossroads");
    t.settle();
    assert!(on_bf(&t, a) && on_bf(&t, b));
    let c = t.enter(P1, "Concordant Crossroads");
    t.settle();
    assert!(on_bf(&t, c));
    assert!(!on_bf(&t, a) && !on_bf(&t, b));
}

#[test]
fn replacement_effects_dont_apply_outside_range() {
    cr!("801.13");
    // Leyline of the Void: "If a card would be put into an opponent's graveyard from
    // anywhere, exile it instead."
    let mut t = ranged(5, 1);
    t.battlefield(P0, "Leyline of the Void");
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P2);
    t.g.destroy(near, None);
    t.g.destroy(far, None);
    t.settle();
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
}

#[test]
fn a_replacement_cant_make_a_spell_affect_something_outside_its_range() {
    cr!("801.13a");
    let mut t = ranged(5, 1);
    // P1's Pariah enchants P2's creature: "All damage that would be dealt to you is dealt
    // to enchanted creature instead."
    let victim = bear(&mut t, P2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 3);
    let pariah = t.hand(P1, "Pariah");
    t.cast(P1, pariah).target(victim).go();
    t.resolve();
    assert!(on_bf(&t, pariah));
    // P0 Bolts P1. Pariah would have the Bolt deal its damage to P2's creature, outside
    // P0's range of influence: that portion of the event does nothing.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert!(on_bf(&t, victim));
    assert_eq!(t.obj_now(victim).damage, 0);
    // A Bolt from P2, for whom P2's creature is within range, is redirected to it.
    let bolt = t.hand(P2, "Lightning Bolt");
    t.lands(P2, "Mountain", 1);
    t.cast(P2, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert!(!on_bf(&t, victim), "redirected to the enchanted creature");
}

#[test]
fn prevention_depends_on_what_the_effect_specifies() {
    cr!("801.13b");
    // P1 attacks P2 (both within P1's range; P2 is outside P0's).
    let setup = || {
        let mut t = ranged(5, 1);
        let attacker = bear(&mut t, P1);
        t.set_step(P1, Step::BeginningOfCombat);
        declare(&mut t, &[(attacker, Entity::Player(P2))]);
        go_to(&mut t, Step::DeclareAttackers);
        (t, attacker)
    };
    // Fog: "Prevent all combat damage that would be dealt this turn." Neither the source
    // nor the recipient is specified: it prevents damage only if both are within P0's
    // range, and P2 isn't.
    let (mut t, _) = setup();
    t.lands(P0, "Forest", 1);
    let fog = t.hand(P0, "Fog");
    t.cast(P0, fog).go();
    t.resolve();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P2), 18);
    // Maze of Ith: "Untap target attacking creature. Prevent all combat damage that would
    // be dealt to and dealt by that creature this turn." It specifies the source, which is
    // within P0's range: the damage it deals to P2 is prevented.
    let (mut t, attacker) = setup();
    let maze = t.battlefield(P0, "Maze of Ith");
    t.activate(P0, maze, 0, &[Entity::Object(attacker)]).unwrap();
    t.resolve();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn winning_makes_opponents_within_range_lose() {
    cr!("801.14");
    let mut t = ranged(5, 1);
    t.battlefield(P0, "Felidar Sovereign");
    t.g.players[0].life = 40;
    t.set_step(P4, Step::End);
    to_step(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P1) && t.has_lost(P4));
    assert!(!t.has_lost(P2) && !t.has_lost(P3) && !t.has_lost(P0));
    assert_eq!(t.g.result, None);
}

#[test]
fn a_draw_applies_to_the_controller_and_players_within_range() {
    cr!("801.15");
    let mut t = ranged(5, 1);
    let di = t.battlefield(P2, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    t.set_step(P1, Step::End);
    to_step(&mut t, P2, Step::Upkeep);
    t.resolve_all();
    for p in [P1, P2, P3] {
        assert!(t.g.drew_game(p) && t.g.player(p).left_game, "{p}");
    }
    assert_eq!(t.g.players_in_game(), vec![P0, P4]);
    assert_eq!(t.g.result, None, "the remaining players continue");
}

#[test]
fn a_loop_is_a_draw_for_players_involved_and_within_their_range() {
    cr!("801.16");
    let mut t = ranged(5, 1);
    // P2 controls a loop of mandatory actions: the game is a draw for P2 and the players
    // within P2's range; P0 and P4 keep playing.
    let e = super::r105_util::card_from_text(
        "Endless Return",
        "",
        "Enchantment",
        None,
        "Whenever a creature dies, return that card to the battlefield under its owner's control.",
    );
    t.custom(P2, e, Zone::Battlefield);
    let z = super::r105_util::card_from_text(
        "Hollow Husk",
        "",
        "Creature — Construct",
        Some((0, 0)),
        "",
    );
    t.custom(P2, z, Zone::Graveyard(P2));
    let husk = t.g.find_in_zone(Zone::Graveyard(P2), "Hollow Husk")[0];
    t.g.move_object(
        husk,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P2),
    );
    t.g.run_until(4000, |g| g.is_over() || !g.player(P2).in_game());
    for p in [P1, P2, P3] {
        assert!(t.g.drew_game(p), "{p}");
    }
    assert!(t.g.player(P0).in_game() && t.g.player(P4).in_game());
    assert_eq!(t.g.result, None);
}

#[test]
fn restarting_the_game_involves_every_player() {
    cr!("801.17");
    // Karn Liberated's last ability restarts the game: every player is in the new game,
    // whatever the ranges of influence.
    let mut t = ranged(5, 1);
    let karn = t.battlefield(P0, "Karn Liberated");
    t.g.objects[karn.0 as usize]
        .counters
        .insert("loyalty".into(), 14);
    t.activate(P0, karn, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.players_in_game().len(), 5);
    assert_eq!(t.g.turn.number, 1, "a new game began");
}

#[test]
fn planes_are_exempt_from_the_range_of_influence_option() {
    cr!("801.18");
    // A Planechase game with a range of influence of 1: Krosa ("All creatures get +2/+2")
    // affects every creature in the game, even ones outside the planar controller's range.
    let mut t = TestGame::with_config(
        5,
        GameConfig {
            variant: mtg_engine::game::Variant::Planechase,
            range_of_influence: Some(1),
            ..Default::default()
        },
    );
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P2);
    super::r107_planechase::add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    mtg_engine::planechase::set_starting_plane(&mut t.g);
    t.g.recompute();
    assert_eq!(mtg_engine::planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.pt(near), (4, 4));
    assert_eq!(t.pt(far), (4, 4));
    // An ordinary permanent with the same ability affects only creatures within range.
    let mut t = ranged(5, 1);
    let near = bear(&mut t, P1);
    let far = bear(&mut t, P2);
    bf(
        &mut t,
        P0,
        custom_card("Krosan Banner", "Enchantment", None, "All creatures get +2/+2."),
    );
    assert_eq!(t.pt(near), (4, 4));
    assert_eq!(t.pt(far), (2, 2));
}
