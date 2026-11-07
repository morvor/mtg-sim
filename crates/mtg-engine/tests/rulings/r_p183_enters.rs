//! Rulings batch P183 — "whenever [a creature of a type] enters/dies" triggers: permanents
//! entering or dying at the same time see each other (CR 603.6a, 603.10a), type-changing
//! effects (CR 205.1b), Katara's additional triggers (CR 603.2d), the Allies' targeted
//! and optional triggers (CR 603.3d, 608.2d), Harabaz Druid's mana ability (CR 605.3a),
//! and Angelic Captain's count (CR 608.2h).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p189_common::{stack_triggers_from, triggers_of};
use crate::r_s01_common::{attack_with, supported, tokens, with_subtype};
use crate::r_s02_common::can_cast;
use crate::r_s04_common::run_with;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts real cards onto the battlefield under P0's control at the same time (one
/// simultaneous zone change), each transformed if marked. Returns the new objects.
fn enter_all(t: &mut TestGame, cards: &[(&str, bool)]) -> Vec<ObjectId> {
    let moves = cards
        .iter()
        .map(|(n, transformed)| MoveEv {
            obj: t.g.create_card_object(card(n), P0, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo {
                controller: Some(P0),
                transformed: *transformed,
                ..Default::default()
            },
            source: None,
        })
        .collect();
    let ids: Vec<ObjectId> = t.g.move_objects(moves).into_iter().flatten().collect();
    t.g.flush_events();
    t.settle();
    ids
}

/// Puts the real card onto the battlefield for P0 through a real zone change and puts
/// the triggers on the stack.
fn arrive(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.enter(P0, name);
    t.g.flush_events();
    t.settle();
    id
}

fn together(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    let cards: Vec<(&str, bool)> = names.iter().map(|n| (*n, false)).collect();
    enter_all(t, &cards)
}

#[test]
fn a_permanent_entering_with_others_triggers_for_each_of_them() {
    cr!("603.6a", "603.2c");
    ruling!(
        "Valley Mightcaller",
        "If Valley Mightcaller enters at the same time as one or more other Frogs, Rabbits, Raccoons, or Squirrels you control, its second ability will trigger for each of them."
    );
    ruling!(
        "Woodland Liege",
        "If Woodland Liege enters at the same time as one or more Beasts you control, its ability triggers for each of those Beasts."
    );
    ruling!(
        "Arahbo, the First Fang",
        "If Arahbo enters at the same time as one or more nontoken Cats you control, its last ability will trigger once for each of those Cats (as well as itself)."
    );
    ruling!(
        "Earth King's Lieutenant",
        "If Earth King's Lieutenant enters at the same time as another Ally, each of Earth King's Lieutenant's abilities will trigger. You'll put a +1/+1 counter on both creatures."
    );
    for c in [
        "Valley Mightcaller",
        "Woodland Liege",
        "Arahbo, the First Fang",
        "Earth King's Lieutenant",
    ] {
        supported(c);
    }
    // Valley Mightcaller with two changelings (Frogs, Rabbits, ...): two counters.
    let mut t = TestGame::new(2);
    let ids = together(
        &mut t,
        &["Valley Mightcaller", "Woodland Changeling", "Woodland Changeling"],
    );
    t.resolve_all();
    assert_eq!(t.counters(ids[0], counters::PLUS1), 2);
    // Woodland Liege with two Beasts: two cards.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let ids = together(&mut t, &["Woodland Liege", "Coeurl", "Coeurl"]);
    assert_eq!(triggers_of(&t, ids[0]), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Arahbo with two nontoken Cats: three Cat tokens.
    let mut t = TestGame::new(2);
    together(
        &mut t,
        &["Arahbo, the First Fang", "Savannah Lions", "Savannah Lions"],
    );
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    // Earth King's Lieutenant with another Ally: a counter on each.
    let mut t = TestGame::new(2);
    let ids = together(
        &mut t,
        &["Earth King's Lieutenant", "Kyoshi Warrior Guard"],
    );
    assert_eq!(triggers_of(&t, ids[0]), 2);
    t.resolve_all();
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1);
    assert_eq!(t.counters(ids[1], counters::PLUS1), 1);
}

#[test]
fn ephara_entering_with_other_enchantments_triggers_for_each() {
    cr!("603.6a");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "If Ephara enters the battlefield under your control at the same time as other enchantments, its last ability will trigger for each of those other enchantments."
    );
    supported("Invasion of Theros // Ephara, Ever-Sheltering");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    let ids = enter_all(
        &mut t,
        &[
            ("Invasion of Theros // Ephara, Ever-Sheltering", true),
            ("Propaganda", false),
            ("Propaganda", false),
        ],
    );
    assert_eq!(t.obj_now(ids[0]).chars.name, "Ephara, Ever-Sheltering");
    assert_eq!(triggers_of(&t, ids[0]), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn ephara_keeps_marked_damage_while_indestructible_but_deathtouch_is_checked_once() {
    cr!("120.6", "704.5g", "704.5h", "702.12b");
    ruling!(
        "Invasion of Theros // Ephara, Ever-Sheltering",
        "Damage dealt to Ephara is tracked even if Ephara has indestructible."
    );
    let setup = |t: &mut TestGame| {
        let ids = enter_all(
            t,
            &[
                ("Invasion of Theros // Ephara, Ever-Sheltering", true),
                ("Propaganda", false),
                ("Propaganda", false),
                ("Propaganda", false),
            ],
        );
        t.resolve_all();
        assert!(t
            .obj_now(ids[0])
            .has_keyword(KeywordKind::Indestructible));
        ids
    };
    // Lethal damage while indestructible: destroyed once it loses indestructible.
    let mut t = TestGame::new(2);
    let ids = setup(&mut t);
    let wurm = t.battlefield(P1, "Craw Wurm");
    crate::r_p194_common::damage(&mut t, wurm, obj(ids[0]), 5);
    assert!(t.on_battlefield(ids[0]));
    assert_eq!(t.obj_now(ids[0]).damage, 5);
    kill(&mut t, ids[1]);
    assert!(!t.on_battlefield(ids[0]));
    // 1 damage from a deathtouch source while indestructible: not destroyed later.
    let mut t = TestGame::new(2);
    let ids = setup(&mut t);
    let viper = t.battlefield(P1, "Typhoid Rats");
    crate::r_p194_common::damage(&mut t, viper, obj(ids[0]), 1);
    assert!(t.on_battlefield(ids[0]));
    kill(&mut t, ids[1]);
    assert!(t.on_battlefield(ids[0]));
    assert_eq!(t.obj_now(ids[0]).damage, 1);
}

#[test]
fn a_type_changing_effect_lets_the_permanent_trigger_for_itself() {
    cr!("613.1d", "603.6a", "603.10a");
    ruling!(
        "Woodland Liege",
        "If Woodland Liege is itself a Beast when it enters (probably because of a static ability that applies to it changing its creature types), its ability will trigger when it enters."
    );
    ruling!(
        "Bishop of Wings",
        "If an effect (such as that of Arcane Adaptation) causes Bishop of Wings to be an Angel, its first ability triggers as it enters the battlefield and its last ability triggers as it dies."
    );
    supported("Arcane Adaptation");
    supported("Bishop of Wings");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Beast");
    arrive(&mut t, "Arcane Adaptation");
    let hand = t.hand_size(P0);
    arrive(&mut t, "Woodland Liege");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Angel");
    arrive(&mut t, "Arcane Adaptation");
    let bishop = arrive(&mut t, "Bishop of Wings");
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    kill(&mut t, bishop);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
}

#[test]
fn bishop_of_wings_dying_with_angels_triggers_for_each() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Bishop of Wings",
        "If Bishop of Wings dies at the same time as one or more Angels you control, its last ability triggers for each of those Angels."
    );
    let mut t = TestGame::new(2);
    let bishop = t.battlefield(P0, "Bishop of Wings");
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, bishop), 2);
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 2);
}

#[test]
fn bloodline_pretender_and_the_chosen_type() {
    cr!("603.6a", "614.12", "702.37c");
    ruling!(
        "Bloodline Pretender",
        "If Bloodline Pretender enters the battlefield at the same time as another creature, you can choose one of that creature's creature types and have Bloodline Pretender's last ability trigger for that other creature."
    );
    ruling!(
        "Bloodline Pretender",
        "If Bloodline Pretender is somehow on the battlefield without a chosen creature type, its last ability can't trigger, even if a creature with no creature types enters the battlefield."
    );
    supported("Bloodline Pretender");
    // Entering with Llanowar Elves, choosing Elf.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    let ids = together(&mut t, &["Bloodline Pretender", "Llanowar Elves"]);
    t.resolve_all();
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1);
    // No chosen type: neither Grizzly Bears nor a face-down creature (no creature types)
    // triggers it.
    let mut t = TestGame::new(2);
    let bp = t.battlefield(P0, "Bloodline Pretender");
    arrive(&mut t, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    let angel = t.hand(P0, "Exalted Angel");
    t.cast(P0, angel)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let fd = t.g.current(angel);
    assert!(t.on_battlefield(fd) && t.obj_now(fd).face_down);
    assert!(t.obj_now(fd).chars.subtypes.is_empty());
    assert_eq!(triggers_of(&t, bp), 0);
    assert_eq!(t.counters(bp, counters::PLUS1), 0);
}

#[test]
fn realmwalker_without_a_choice_still_shows_the_top_card() {
    cr!("601.3", "401.5", "702.37c");
    ruling!(
        "Realmwalker",
        "If Realmwalker is somehow on the battlefield with no choice having been made for it, you may still look at the top card of your library any time. You won't be able to cast any creature spells from the top of your library, even if they have no creature types."
    );
    supported("Realmwalker");
    for top in ["Woodland Changeling", "Exalted Angel"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Realmwalker");
        t.lands(P0, "Plains", 4);
        t.lands(P0, "Forest", 2);
        let c = t.library_top(P0, top);
        assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, c));
        assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, c));
        assert!(!can_cast(&mut t, P0, c, CastMethod::Normal), "{top}");
        // Cast face down, it would be a creature spell with no creature types.
        assert!(
            !can_cast(&mut t, P0, c, CastMethod::FaceDown(KeywordKind::Morph)),
            "{top}"
        );
    }
}

#[test]
fn katara_makes_ally_triggers_trigger_an_additional_time() {
    cr!("603.2d");
    ruling!(
        "Katara, the Fearless",
        "If an Ally entering at the same time as Katara (including Katara herself) causes a triggered ability of an Ally you control to trigger, that ability triggers an additional time."
    );
    ruling!(
        "Katara, the Fearless",
        "If you somehow control two copies of Katara, the Fearless, an Ally's triggered ability will trigger three times, not four."
    );
    ruling!(
        "Katara, the Fearless",
        "Replacement effects are unaffected by Katara's ability. For example, an Ally that enters with one +1/+1 counter on it won't receive an additional +1/+1 counter."
    );
    supported("Katara, the Fearless");
    supported("Mirror Gallery");
    supported("The Duke, Rebel Sentry");
    // Katara herself entering: Hada Freeblade's ability triggers twice.
    let mut t = TestGame::new(2);
    let hada = t.battlefield(P0, "Hada Freeblade");
    arrive(&mut t, "Katara, the Fearless");
    assert_eq!(triggers_of(&t, hada), 2);
    // Katara and another Ally entering together: twice for each.
    let mut t = TestGame::new(2);
    let hada = t.battlefield(P0, "Hada Freeblade");
    together(&mut t, &["Katara, the Fearless", "Kyoshi Warrior Guard"]);
    assert_eq!(triggers_of(&t, hada), 4);
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    assert_eq!(t.counters(hada, counters::PLUS1), 4);
    // Two Kataras: three times.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Katara, the Fearless");
    t.battlefield(P0, "Katara, the Fearless");
    let hada = t.battlefield(P0, "Hada Freeblade");
    t.settle();
    assert_eq!(t.named_on_battlefield("Katara, the Fearless").len(), 2);
    arrive(&mut t, "Kyoshi Warrior Guard");
    assert_eq!(triggers_of(&t, hada), 3);
    // An Ally entering with a +1/+1 counter gets just the one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Katara, the Fearless");
    let duke = arrive(&mut t, "The Duke, Rebel Sentry");
    t.resolve_all();
    assert_eq!(t.counters(duke, counters::PLUS1), 1);
}

#[test]
fn katara_extra_trigger_is_a_separate_instance_with_its_own_choices() {
    cr!("603.2d", "603.3d", "608.2d");
    ruling!(
        "Katara, the Fearless",
        "Katara's effect doesn't copy the triggered ability; it just causes the ability to trigger twice. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability. Any choices made on resolution, such as whether to put counters on a permanent, are also made individually."
    );
    supported("Murasa Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Katara, the Fearless");
    let murasa = t.battlefield(P0, "Murasa Pyromancer");
    let giant = t.battlefield(P1, "Colossal Dreadmaw");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(wurm)]);
    arrive(&mut t, "Kyoshi Warrior Guard");
    let trig = stack_triggers_from(&t, murasa);
    assert_eq!(trig.len(), 2);
    let mut targets: Vec<Entity> = trig.iter().flat_map(|x| targets_of(&t, *x)).collect();
    targets.sort();
    let mut want = vec![obj(giant), obj(wurm)];
    want.sort();
    assert_eq!(targets, want);
    // Neither is a copy: both triggered (were put on the stack as triggered abilities).
    assert_eq!(triggers_of(&t, murasa), 2);
    // The top one: yes (3 damage: three Allies). The other: no.
    let top = targets_of(&t, trig[1])[0];
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    for c in [giant, wurm] {
        let want = if obj(c) == top { 3 } else { 0 };
        assert_eq!(t.obj_now(c).damage, want);
    }
}

#[test]
fn murasa_pyromancer_targets_on_triggering_and_decides_on_resolution() {
    cr!("603.3d", "608.2d");
    ruling!(
        "Murasa Pyromancer",
        "You target a creature when the ability triggers. You decide whether to have Murasa Pyromancer deal damage to that creature as the ability resolves."
    );
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let murasa = t.battlefield(P0, "Murasa Pyromancer");
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.answer_targets(P0, &[obj(wurm)]);
        arrive(&mut t, "Kyoshi Warrior Guard");
        let trig = stack_triggers_from(&t, murasa);
        assert_eq!(targets_of(&t, trig[0]), vec![obj(wurm)]);
        let asked_yes = |t: &TestGame| {
            t.asked()
                .iter()
                .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
                .count()
        };
        assert_eq!(asked_yes(&t), 0, "nothing decided yet");
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(asked_yes(&t), 1);
        // Two Allies: Murasa Pyromancer and Kyoshi Warrior Guard.
        assert_eq!(t.obj_now(wurm).damage, if yes { 2 } else { 0 });
    }
}

/// P1's Treasure token.
fn treasure(t: &mut TestGame) -> ObjectId {
    run_with(
        t,
        P1,
        Effect::CreateToken {
            spec: mtg_engine::tokens::predefined("Treasure").expect("Treasure"),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &[],
    );
    with_subtype(t, P1, "Treasure")[0]
}

#[test]
fn tuktuk_scrapper_damages_the_controller_of_a_destroyed_token() {
    cr!("111.7", "704.5d", "701.8a");
    ruling!(
        "Tuktuk Scrapper",
        "A token permanent that’s destroyed is put into its owner’s graveyard before it ceases to exist. If a token is destroyed by Tuktuk Scrapper’s ability, Tuktuk Scrapper deals damage to that token’s controller."
    );
    supported("Tuktuk Scrapper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kyoshi Warrior Guard");
    let tr = treasure(&mut t);
    t.answer_targets(P0, &[obj(tr)]);
    t.answer_yes(P0, true);
    arrive(&mut t, "Tuktuk Scrapper");
    t.resolve_all();
    assert!(!t.g.is_live(tr) || !t.on_battlefield(tr));
    assert!(with_subtype(&t, P1, "Treasure").is_empty());
    // Two Allies.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tuktuk_scrapper_deals_no_damage_unless_the_artifact_reaches_a_graveyard() {
    cr!("702.12b", "701.19a", "614.1a");
    ruling!(
        "Tuktuk Scrapper",
        "If the targeted artifact hass indestructible or regenerates (or you choose not to destroy it), Tuktuk Scrapper doesn’t deal damage to that artifact’s controller. Similarly, if the targeted artifact is destroyed but a replacement effect moves it to a different zone instead of its owner’s graveyard, Tuktuk Scrapper doesn’t deal damage to that artifact’s controller."
    );
    supported("Welding Jar");
    supported("Rest in Peace");
    // (the artifact, P0 destroys it, P1 regenerates it with Welding Jar, Rest in Peace)
    let cases: &[(&str, bool, bool, bool)] = &[
        ("Darksteel Relic", true, false, false),
        ("Mind Stone", true, true, false),
        ("Mind Stone", false, false, false),
        ("Mind Stone", true, false, true),
        ("Mind Stone", true, false, false),
    ];
    for (i, (art, destroy, regen, rip)) in cases.iter().enumerate() {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, art);
        let jar = t.battlefield(P1, "Welding Jar");
        if *rip {
            t.battlefield(P1, "Rest in Peace");
        }
        t.answer_targets(P0, &[obj(a)]);
        t.answer_yes(P0, *destroy);
        arrive(&mut t, "Tuktuk Scrapper");
        if *regen {
            t.answer(
                P1,
                DecisionKind::Targets,
                mtg_engine::decision::Answer::Entities(vec![obj(a)]),
            );
            t.activate(P1, jar, 0, &[]).unwrap();
        }
        t.resolve_all();
        let damaged = t.life(P1) < 20;
        // Only the last case (destroyed into the graveyard) deals damage.
        assert_eq!(damaged, i == cases.len() - 1, "case {i}");
        if *regen {
            assert!(t.on_battlefield(a), "regenerated");
        }
        if *rip {
            assert_eq!(t.zone(a), Zone::Exile);
        }
    }
}

#[test]
fn harabaz_druids_ability_doesnt_use_the_stack() {
    cr!("605.3b");
    ruling!(
        "Harabaz Druid",
        "Harabaz Druid’s activated ability is a mana ability, so it doesn’t use the stack and players can’t respond to it."
    );
    supported("Harabaz Druid");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Harabaz Druid");
    t.battlefield(P0, "Kyoshi Warrior Guard");
    t.activate(P0, druid, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn angelic_captain_counts_attacking_allies_on_resolution_including_a_teammates() {
    cr!("608.2h", "810.7", "805.10a");
    ruling!(
        "Angelic Captain",
        "Count the number of other attacking Allies as the ability resolves to determine the size of the bonus. Attacking Allies controlled by a teammate will count toward this total."
    );
    ruling!(
        "Angelic Captain",
        "Once the ability resolves, the bonus won’t change later in the turn, even if the number of other attacking Allies does."
    );
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let captain = t.battlefield(P0, "Angelic Captain");
    let mine = t.battlefield(P0, "Kyoshi Warrior Guard");
    let theirs = t.battlefield(P1, "Kyoshi Warrior Guard");
    let envoy = t.battlefield(P1, "Expedition Envoy");
    attack_with(
        &mut t,
        &[
            (captain, pl(P2)),
            (mine, pl(P2)),
            (theirs, pl(P2)),
            (envoy, pl(P2)),
        ],
    );
    assert_eq!(stack_triggers_from(&t, captain).len(), 1);
    // In response, P0's attacking Ally leaves: the teammate's two remain at resolution.
    kill(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.pt(captain), (4 + 2, 3 + 2));
    // Later another attacking Ally leaves: the bonus stays.
    kill(&mut t, theirs);
    assert_eq!(t.pt(captain), (6, 5));
}
