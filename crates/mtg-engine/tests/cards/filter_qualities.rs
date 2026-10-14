//! Object qualities and relational qualifiers in object phrases (`oracle/patterns/
//! filter_qualities.rs`, `filters_relational.rs`, `conditions_referents.rs`): face-up and
//! double-faced objects, nameless objects, abilities with {T} in their cost, negated
//! counter and history clauses, attached Equipment, "whose controller controls", "with
//! equal or lesser [stat]" against a referent, same-name conditions and the most common
//! color among permanents.

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Whether the `index`th activated ability of `src` can be activated targeting `target`.
fn can_target(t: &mut TestGame, src: ObjectId, index: usize, target: Entity) -> bool {
    t.g.recompute();
    let r = t.activate(P0, src, index, &[target]);
    t.clear_answers();
    match r {
        Ok(Some(ab)) => t
            .g
            .obj(ab)
            .stack
            .as_ref()
            .is_some_and(|s| s.chosen.iter().flat_map(|c| c.targets.iter().flatten()).any(|e| *e == target)),
        _ => false,
    }
}

#[test]
fn creatures_without_a_name_cant_block_pompous_gadabout() {
    cr!("201.2a", "708.2a", "509.1b");
    ruling!(
        "Pompous Gadabout",
        "Face-down creatures don't have names unless an effect says otherwise"
    );
    compiles("Pompous Gadabout");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Pompous Gadabout");
    let named = t.battlefield(P1, "Grizzly Bears");
    let hidden = t.battlefield(P1, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, hidden));
    t.set_step(P0, Step::BeginningOfCombat);
    // The face-down blocker isn't allowed; the named one is.
    t.attack(&[(g, Entity::Player(P1))], &[(hidden, g)]);
    assert_eq!(t.life(P1), 20 - t.pt(g).0);
    let mut t2 = TestGame::new(2);
    let g = t2.battlefield(P0, "Pompous Gadabout");
    let named2 = t2.battlefield(P1, "Grizzly Bears");
    t2.set_step(P0, Step::BeginningOfCombat);
    t2.attack(&[(g, Entity::Player(P1))], &[(named2, g)]);
    assert_eq!(t2.life(P1), 20);
    let _ = named;
}

#[test]
fn activated_ability_with_tap_in_its_cost_magewrights_stone() {
    cr!("602.1", "107.5");
    compiles("Magewright's Stone");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Magewright's Stone");
    t.lands(P0, "Plains", 2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(elves);
    t.g.tap(bears);
    assert!(!can_target(&mut t, stone, 0, Entity::Object(bears)));
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Magewright's Stone");
    t.lands(P0, "Plains", 2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.tap(elves);
    t.activate(P0, stone, 0, &[Entity::Object(elves)]).unwrap();
    t.resolve();
    assert!(!t.obj_now(elves).tapped);
}

#[test]
fn target_without_a_counter_is_rechecked_coiling_stalker() {
    cr!("608.2b");
    ruling!(
        "Coiling Stalker",
        "will become an illegal target if a +1/+1 counter is placed on it"
    );
    compiles("Coiling Stalker");
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Coiling Stalker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(stalker, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // A creature with a +1/+1 counter can't be chosen: the stalker itself is the only
    // other one, so it gets the counter.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(stalker, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(stalker, "+1/+1"), 1);
}

#[test]
fn didnt_attack_or_enter_this_turn_the_fifth_doctor() {
    compiles("The Fifth Doctor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fifth Doctor");
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    // On a later turn, so that they didn't enter this turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    let newcomer = t.enter(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(idle, "+1/+1"), 1);
    assert_eq!(t.counters(attacker, "+1/+1"), 0);
    assert_eq!(t.counters(newcomer, "+1/+1"), 0);
}

#[test]
fn whose_controller_controls_an_island_dwarven_sea_clan() {
    compiles("Dwarven Sea Clan");
    for island in [false, true] {
        let mut t = TestGame::new(2);
        let clan = t.battlefield(P0, "Dwarven Sea Clan");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if island {
            t.battlefield(P0, "Island");
        }
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(bears, Entity::Player(P1))]),
        );
        t.advance_to(P0, Step::DeclareAttackers);
        assert_eq!(can_target(&mut t, clan, 0, Entity::Object(bears)), island);
    }
}

#[test]
fn equal_or_lesser_power_uses_last_known_power_body_launderer() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Body Launderer",
        "Use Body Launderer's power as it last existed on the battlefield"
    );
    compiles("Body Launderer");
    let mut t = TestGame::new(2);
    let bl = t.battlefield(P0, "Body Launderer");
    // 3/3 with two +1/+1 counters: a 5-power creature card can be returned.
    t.g.add_counters(Entity::Object(bl), "+1/+1", 2, None);
    let big = t.graveyard(P0, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(big)]);
    t.g.destroy(bl, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 0);
    let mut t = TestGame::new(2);
    let bl = t.battlefield(P0, "Body Launderer");
    t.g.add_counters(Entity::Object(bl), "+1/+1", 3, None);
    let big = t.graveyard(P0, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(big)]);
    t.g.destroy(bl, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn equal_or_lesser_toughness_of_the_creature_that_died_bane() {
    ruling!(
        "Bane, Lord of Darkness",
        "looks at the toughness of the creature that died"
    );
    compiles("Bane, Lord of Darkness");
    compiles("Jailbreak");
    compiles("Kellan, the Kid");
    compiles("Puca's Mischief");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bane, Lord of Darkness");
    let wall = t.battlefield(P0, "Wall of Stone"); // 0/8
    t.hand(P0, "Craw Wurm"); // 6/4
    // The opponent declines to have you draw.
    t.answer_yes(P1, false);
    t.answer_yes(P0, true);
    t.g.destroy(wall, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn another_permanent_with_the_same_name_winnow() {
    cr!("201.2");
    compiles("Winnow");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let w = t.hand(P0, "Winnow");
    t.cast(P0, w).target(a).go();
    t.resolve();
    assert!(t.on_battlefield(a));
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let w = t.hand(P0, "Winnow");
    t.cast(P0, w).target(a).go();
    t.resolve();
    assert!(!t.on_battlefield(a));
}

#[test]
fn same_name_as_a_card_in_your_graveyard_candles_of_leng() {
    compiles("Candles of Leng");
    compiles("Retraced Image");
    compiles("Bazaar of Wonders");
    let mut t = TestGame::new(2);
    let candles = t.battlefield(P0, "Candles of Leng");
    t.lands(P0, "Plains", 4);
    t.graveyard(P0, "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    t.activate(P0, candles, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 2);
    let hand = t.hand_size(P0);
    let mut t2 = TestGame::new(2);
    let candles = t2.battlefield(P0, "Candles of Leng");
    t2.lands(P0, "Plains", 4);
    t2.graveyard(P0, "Hill Giant");
    t2.library_top(P0, "Grizzly Bears");
    t2.activate(P0, candles, 0, &[]).unwrap();
    t2.resolve();
    assert!(t2.in_hand(P0, "Grizzly Bears"));
    let _ = hand;
}

#[test]
fn spell_with_the_same_name_as_a_card_in_a_graveyard_is_countered_bazaar() {
    compiles("Bazaar of Wonders");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Bazaar of Wonders");
    t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    let b = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let h = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, h).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn most_common_color_among_permanents_with_ties_tsabos_assassin() {
    cr!("105.4");
    compiles("Tsabo's Assassin");
    compiles("Barrin's Unmaking");
    compiles("Heroic Defiance");
    // Green (two) is the most common; the black Assassin isn't destroyed... the target
    // is.
    let run = |extra_red: usize| {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Tsabo's Assassin"); // black
        let target = t.battlefield(P1, "Grizzly Bears"); // green
        t.battlefield(P1, "Grizzly Bears");
        for _ in 0..extra_red {
            t.battlefield(P1, "Hill Giant"); // red
        }
        t.activate(P0, a, 0, &[Entity::Object(target)]).unwrap();
        t.resolve();
        t.on_battlefield(target)
    };
    // Green most common: destroyed.
    assert!(!run(0));
    // Red and green tied: still destroyed.
    assert!(!run(2));
    // Red more common: not destroyed.
    assert!(run(3));
}

#[test]
fn unless_it_shares_the_most_common_color_heroic_defiance() {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hd = t.battlefield(P0, "Heroic Defiance");
    t.g.attach(hd, Entity::Object(bears));
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    // Green and red tied.
    assert_eq!(t.pt(bears), (2, 2));
    let _ = KeywordKind::Flying;
}
