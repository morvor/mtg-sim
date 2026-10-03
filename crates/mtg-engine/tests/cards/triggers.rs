//! Triggered ability conditions (CR 603) compiled from oracle text: event triggers,
//! batched "one or more" triggers, "for the first time each turn", state triggers,
//! beginning-of-step triggers, delayed triggers, and what "it"/"that creature"/"that
//! player" refer to for each kind of trigger.

use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

/// Puts a card onto the battlefield from its owner's hand with a real zone change, so
/// enters-the-battlefield abilities trigger.
fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.hand(p, name);
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(p),
    )
    .expect("failed to enter the battlefield")
}

/// Permanents `p` controls with the given subtype (tokens made from oracle text have
/// no name).
fn count_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.has_subtype(subtype)
        })
        .count()
}

/// Number of triggered abilities from `source` currently on the stack.
fn triggers_on_stack(t: &TestGame, source: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| match t.g.obj(**id).stack.as_deref().map(|s| &s.kind) {
            Some(mtg_engine::object::StackKind::Triggered { source: s, .. }) => {
                t.g.current(*s) == t.g.current(source)
            }
            _ => false,
        })
        .count()
}

// ---------------------------------------------------------------------------
// Zone-change triggers: "it" is the new object for actions, LKI for information.
// ---------------------------------------------------------------------------

#[test]
fn dies_trigger_finds_the_card_in_the_graveyard() {
    cr!("400.7e", "603.10a");
    assert_supported(&["Mortus Strider"]);
    let mut t = TestGame::new(2);
    let strider = t.battlefield(P0, "Mortus Strider");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(strider).go();
    t.resolve_all();
    // "When this creature dies, return it to its owner's hand."
    assert!(t.in_hand(P0, "Mortus Strider"));
    assert!(!t.in_graveyard(P0, "Mortus Strider"));
}

#[test]
fn dies_trigger_uses_last_known_power() {
    cr!("608.2h", "603.10a");
    assert_supported(&["Goblin Fireleaper", "Giant Growth", "Murder", "Hill Giant"]);
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Goblin Fireleaper");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 3);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(leaper).go();
    t.resolve_all();
    assert_eq!(t.pt(leaper), (4, 4));
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(leaper).go();
    // The dies trigger targets the Hill Giant; it deals damage equal to the power the
    // Fireleaper had on the battlefield (4), not its printed power (1).
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Goblin Fireleaper"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn leaves_the_battlefield_trigger_player_target_does_not_become_it() {
    cr!("603.10a", "608.2h");
    assert_supported(&["Rapacious Guest"]);
    let mut t = TestGame::new(2);
    let guest = t.battlefield(P0, "Rapacious Guest");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(guest).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // "target opponent loses life equal to its power": its = the Guest (2 power).
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// Cast triggers.
// ---------------------------------------------------------------------------

#[test]
fn heroic_triggers_for_spells_that_target_it() {
    cr!("601.2c", "603.3");
    ruling!("Akroan Line Breaker", "resolve before the spell");
    assert_supported(&["Akroan Line Breaker"]);
    let mut t = TestGame::new(2);
    let breaker = t.battlefield(P0, "Akroan Line Breaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    // A spell targeting another creature doesn't trigger it.
    let g1 = t.hand(P0, "Giant Growth");
    t.cast(P0, g1).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(breaker), (2, 1));
    // Targeting it: the heroic trigger goes on the stack above the spell.
    let g2 = t.hand(P0, "Giant Growth");
    t.cast(P0, g2).target(breaker).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, breaker), 1);
    t.resolve();
    assert_eq!(t.pt(breaker), (4, 1));
    t.resolve_all();
    assert_eq!(t.pt(breaker), (7, 4));
}

#[test]
fn noncreature_spell_trigger() {
    cr!("603.2");
    assert_supported(&["Crackling Cyclops"]);
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Crackling Cyclops");
    let (p, tough) = t.pt(cyclops);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.pt(cyclops), (p + 3, tough));
    // A creature spell doesn't trigger it.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, cyclops), 0);
}

#[test]
fn second_spell_each_turn() {
    cr!("603.2");
    assert_supported(&["Sunstar Lightsmith"]);
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P0, "Sunstar Lightsmith");
    t.lands(P0, "Mountain", 3);
    let hand = t.hand_size(P0);
    for i in 0..3 {
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.resolve_all();
        // Only the second spell triggers it.
        let expected = if i >= 1 { 1 } else { 0 };
        assert_eq!(
            t.counters(smith, "+1/+1"),
            expected,
            "after spell {}",
            i + 1
        );
    }
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn when_you_cast_this_spell_triggers_from_the_stack() {
    cr!("603.2", "113.6k");
    assert_supported(&["Artisan of Kozilek"]);
    let c = card("Artisan of Kozilek");
    let ab = c.faces[0]
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Triggered(t) => Some(t.clone()),
            _ => None,
        })
        .unwrap();
    assert!(matches!(ab.zone, FunctionZone::Stack));
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 10);
    let artisan = t.hand(P0, "Artisan of Kozilek");
    t.cast(P0, artisan).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Artisan of Kozilek").len(), 1);
}

// ---------------------------------------------------------------------------
// Player event triggers.
// ---------------------------------------------------------------------------

#[test]
fn second_card_drawn_each_turn() {
    cr!("603.2");
    assert_supported(&["Lat-Nam Adept"]);
    let mut t = TestGame::new(2);
    let adept = t.battlefield(P0, "Lat-Nam Adept");
    for (i, expected) in [(1, 0), (2, 1), (3, 1)] {
        t.g.draw_cards(P0, 1);
        t.resolve_all();
        assert_eq!(t.counters(adept, "+1/+1"), expected, "after draw {i}");
    }
}

#[test]
fn first_life_gain_each_turn_counts_gains_before_it_existed() {
    cr!("603.2");
    ruling!(
        "Vanguard Seraph",
        "before Vanguard Seraph is on the battlefield"
    );
    assert_supported(&["Vanguard Seraph"]);
    let mut t = TestGame::new(2);
    let seraph = t.battlefield(P0, "Vanguard Seraph");
    t.g.gain_life(P0, 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, seraph), 1);
    t.resolve_all();
    t.g.gain_life(P0, 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, seraph), 0);
    // Next turn: gaining life before a new Seraph enters means it won't trigger.
    t.advance_to(P1, Step::PrecombatMain);
    t.g.gain_life(P0, 1);
    t.settle();
    t.resolve_all();
    let late = t.battlefield(P0, "Vanguard Seraph");
    t.g.gain_life(P0, 1);
    t.settle();
    assert_eq!(triggers_on_stack(&t, late), 0);
}

#[test]
fn first_life_gain_returns_card_from_graveyard() {
    cr!("113.6m", "603.2");
    assert_supported(&["Deathless Knight"]);
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Deathless Knight");
    t.g.gain_life(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Deathless Knight"));
}

#[test]
fn first_life_loss_each_turn() {
    cr!("603.2");
    assert_supported(&["Vengeful Warchief"]);
    let mut t = TestGame::new(2);
    let chief = t.battlefield(P0, "Vengeful Warchief");
    t.g.lose_life(P0, 1);
    t.resolve_all();
    t.g.lose_life(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(chief, "+1/+1"), 1);
}

#[test]
fn you_gain_life_that_much() {
    cr!("603.2");
    assert_supported(&["Sanguine Bond"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Bond");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.gain_life(P0, 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sacrifice_another_permanent() {
    cr!("603.10a");
    assert_supported(&["Gixian Infiltrator"]);
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Gixian Infiltrator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bears, P0);
    t.resolve_all();
    assert_eq!(t.counters(inf, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------
// Object triggers with compound subjects and verbs.
// ---------------------------------------------------------------------------

#[test]
fn enters_or_attacks() {
    cr!("603.2", "603.6a");
    assert_supported(&["Stadium Tidalmage"]);
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    let mage = enter(&mut t, P0, "Stadium Tidalmage");
    t.answer_yes(P0, true);
    t.resolve_all();
    // Drew a card and discarded a card.
    assert_eq!(t.library_size(P0), lib - 1);
    assert_eq!(t.graveyard_size(P0), 1);
    t.g.objects[mage.0 as usize].summoning_sick = false;
    t.answer_yes(P0, true);
    t.attack(&[(mage, Entity::Player(P1))], &[]);
    assert_eq!(t.library_size(P0), lib - 2);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn enters_or_dies() {
    cr!("603.2", "603.6a", "603.10a");
    assert_supported(&["Thawbringer"]);
    let mut t = TestGame::new(2);
    let tb = enter(&mut t, P0, "Thawbringer");
    t.settle();
    assert_eq!(triggers_on_stack(&t, tb), 1);
    t.resolve_all();
    t.g.destroy(tb, None);
    t.settle();
    assert_eq!(triggers_on_stack(&t, tb), 1);
}

#[test]
fn this_or_another_ally_enters() {
    cr!("603.6a");
    assert_supported(&["Kazandu Blademaster"]);
    let mut t = TestGame::new(2);
    let bm = t.battlefield(P0, "Kazandu Blademaster");
    enter(&mut t, P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, bm), 0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Kazandu Blademaster");
    t.resolve_all();
    // The other Blademaster entering triggers both (each puts a counter on itself).
    assert_eq!(t.counters(bm, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------
// Combat triggers.
// ---------------------------------------------------------------------------

#[test]
fn becomes_blocked_by_a_creature_triggers_per_blocker() {
    cr!("509.3d");
    assert_supported(&["Pygmy Troll"]);
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Pygmy Troll");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(troll, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, troll), (b2, troll)]),
    );
    let (p, tough) = t.pt(troll);
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_on_stack(&t, troll), 2);
    t.resolve_all();
    assert_eq!(t.pt(troll), (p + 2, tough + 2));
}

#[test]
fn blocks_a_creature_with_flying() {
    cr!("509.3b");
    assert_supported(&["Netcaster Spider"]);
    for (attacker, pumped) in [("Wind Drake", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, attacker);
        let spider = t.battlefield(P1, "Netcaster Spider");
        t.set_step(P0, Step::BeginningOfCombat);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(a, Entity::Player(P1))]),
        );
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(spider, a)]),
        );
        let base = t.pt(spider).0;
        t.advance_to(P0, Step::DeclareBlockers);
        t.resolve_all();
        let expected = if pumped { base + 2 } else { base };
        assert_eq!(t.pt(spider).0, expected, "{attacker}");
    }
}

#[test]
fn one_or_more_creatures_deal_combat_damage_triggers_once_per_player() {
    cr!("603.2c", "510.2");
    assert_supported(&["Ongoing Investigation"]);
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Ongoing Investigation");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
        &[],
    );
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P2), 18);
    // Two players were dealt damage: two Clues, not three.
    assert_eq!(count_subtype(&t, P0, "Clue"), 2);
}

#[test]
fn creature_attacks_alone() {
    cr!("506.5");
    assert_supported(&["Agents of S.H.I.E.L.D."]);
    // "that creature gets +1/+1": the creature attacking alone.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agents of S.H.I.E.L.D.");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    // Two attackers: neither attacks alone.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agents of S.H.I.E.L.D.");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn attack_trigger_targets_a_creature_defending_player_controls() {
    cr!("508.5", "603.3d");
    assert_supported(&["Fiend Binder"]);
    let mut t = TestGame::new(2);
    let binder = t.battlefield(P0, "Fiend Binder");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    // An illegal choice (our own creature) is rejected; the defending player's creature
    // is the only legal target.
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(binder, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    assert!(!t.obj_now(mine).tapped);
}

#[test]
fn attack_with_two_or_more_creatures() {
    cr!("508.1");
    assert_supported(&["Military Intelligence"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Military Intelligence");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Military Intelligence");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    let _ = b;
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b2, Entity::Player(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn creature_deals_combat_damage_to_you_that_creature() {
    cr!("510.2");
    assert_supported(&["Teysa, Envoy of Ghosts"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa, Envoy of Ghosts");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    // "destroy that creature": the creature that dealt the damage.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(count_subtype(&t, P0, "Spirit"), 1);
}

// ---------------------------------------------------------------------------
// Damage triggers batched per simultaneous event.
// ---------------------------------------------------------------------------

#[test]
fn is_dealt_damage_by_several_sources_triggers_once() {
    cr!("603.2c", "510.2");
    ruling!(
        "Boros Reckoner",
        "its ability triggers once and one target is dealt that much damage"
    );
    assert_supported(&["Boros Reckoner"]);
    let mut t = TestGame::new(2);
    let reckoner = t.battlefield(P0, "Boros Reckoner");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(reckoner, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, reckoner), (b2, reckoner)]),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::EndOfCombat);
    // One trigger dealing 4 (2 + 2) damage to the target.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn deals_damage_gain_that_much_life() {
    cr!("603.2");
    assert_supported(&["Mourning Thrull"]);
    let mut t = TestGame::new(2);
    let thrull = t.battlefield(P0, "Mourning Thrull");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(thrull, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn enchanted_creature_is_dealt_damage() {
    cr!("603.2");
    assert_supported(&["Sleep Magic"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let magic = t.battlefield(P0, "Sleep Magic");
    t.g.attach(magic, Entity::Object(giant));
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Sleep Magic"));
    assert!(t.on_battlefield(giant));
}

// ---------------------------------------------------------------------------
// Batched "one or more" triggers.
// ---------------------------------------------------------------------------

#[test]
fn one_or_more_tokens_enter_triggers_once() {
    cr!("603.2c");
    assert_supported(&["Cloakwood Swarmkeeper", "Raise the Alarm"]);
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Cloakwood Swarmkeeper");
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Soldier"), 2);
    assert_eq!(t.counters(keeper, "+1/+1"), 1);
}

#[test]
fn one_or_more_other_creatures_die_once_each_turn() {
    cr!("603.2c", "603.10a");
    assert_supported(&["Morbid Opportunist", "Pyroclasm"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Morbid Opportunist");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let hand = t.hand_size(P0);
    let clasm = t.hand(P0, "Pyroclasm");
    t.cast(P0, clasm).go();
    t.resolve_all();
    // The Opportunist (2/1) dies too; it still sees the others die (look back in time).
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn one_or_more_cards_leave_your_graveyard() {
    cr!("603.2c", "603.10a");
    assert_supported(&["Spirit Mascot"]);
    let mut t = TestGame::new(2);
    let mascot = t.battlefield(P0, "Spirit Mascot");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let moves = [a, b]
        .iter()
        .map(|o| mtg_engine::replacement::MoveEv {
            obj: *o,
            to: mtg_engine::object::Zone::Exile,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: Default::default(),
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.resolve_all();
    assert_eq!(t.counters(mascot, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------
// State triggers (CR 603.8).
// ---------------------------------------------------------------------------

#[test]
fn state_trigger_control_no_islands() {
    cr!("603.8");
    assert_supported(&["Skeleton Ship"]);
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Skeleton Ship");
    let island = t.battlefield(P0, "Island");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.destroy(island, None);
    t.resolve_all();
    assert!(!t.on_battlefield(ship));
    assert!(t.in_graveyard(P0, "Skeleton Ship"));
}

#[test]
fn state_trigger_control_no_other_creatures() {
    cr!("603.8");
    assert_supported(&["Emperor Crocodile"]);
    let mut t = TestGame::new(2);
    let croc = t.battlefield(P0, "Emperor Crocodile");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(!t.on_battlefield(croc));
}

// ---------------------------------------------------------------------------
// Beginning-of-step triggers.
// ---------------------------------------------------------------------------

#[test]
fn beginning_of_the_end_step_is_every_end_step() {
    cr!("603.2b", "513.1");
    assert_supported(&["Mark of Fury"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mark = t.battlefield(P0, "Mark of Fury");
    t.g.attach(mark, Entity::Object(bears));
    // On the opponent's turn as well as ours.
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mark of Fury"));
}

#[test]
fn upkeep_of_enchanted_creatures_controller() {
    cr!("603.2b");
    assert_supported(&["Unstable Mutation"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mutation = t.battlefield(P0, "Unstable Mutation");
    t.g.attach(mutation, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(bears), (5, 5));
    // Not during its enchanting player's upkeep.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Draw);
    // "put a -1/-1 counter on that creature": the enchanted creature.
    assert_eq!(t.counters(bears, "-1/-1"), 1);
    assert_eq!(t.pt(bears), (4, 4));
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.counters(bears, "-1/-1"), 1);
}

#[test]
fn enchanted_players_upkeep() {
    cr!("603.2b");
    assert_supported(&["Curse of the Bloody Tome"]);
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of the Bloody Tome");
    t.g.attach(curse, Entity::Player(P1));
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.graveyard_size(P1), 2);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn triggers_only_once_each_turn() {
    cr!("603.2");
    assert_supported(&["Nanoform Sentinel"]);
    let mut t = TestGame::new(2);
    let sentinel = t.battlefield(P0, "Nanoform Sentinel");
    let land = t.battlefield(P0, "Forest");
    t.g.tap(land);
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.g.tap(sentinel);
    t.settle();
    assert_eq!(triggers_on_stack(&t, sentinel), 1);
    t.resolve_all();
    assert!(!t.obj_now(land).tapped);
    t.g.untap(sentinel);
    t.g.tap(land);
    t.g.tap(sentinel);
    t.settle();
    assert_eq!(triggers_on_stack(&t, sentinel), 0);
}

// ---------------------------------------------------------------------------
// Self-targeting triggers and delayed triggers.
// ---------------------------------------------------------------------------

#[test]
fn becomes_the_target_sacrifice_it() {
    cr!("603.2");
    assert_supported(&["Tar Pit Warrior"]);
    let mut t = TestGame::new(2);
    let warrior = t.battlefield(P0, "Tar Pit Warrior");
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(warrior).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tar Pit Warrior"));
}

#[test]
fn draw_at_the_beginning_of_the_next_turns_upkeep() {
    cr!("603.7a", "603.7b");
    assert_supported(&["Fevered Strength"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let fs = t.hand(P0, "Fevered Strength");
    t.cast(P0, fs).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    assert_eq!(t.hand_size(P0), hand);
    // The next turn is the opponent's: the delayed trigger draws at their upkeep, once.
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P0, Step::PrecombatMain);
    // Our own draw step draw only.
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn pact_delayed_upkeep_payment() {
    cr!("603.7a", "603.7b", "118.12");
    assert_supported(&["Slaughter Pact"]);
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if pay {
            t.lands(P0, "Swamp", 3);
        }
        let pact = t.hand(P0, "Slaughter Pact");
        t.cast(P0, pact).target(bears).go();
        t.resolve_all();
        assert!(!t.on_battlefield(bears));
        // Nothing happens at the opponent's upkeep; at ours we pay or lose.
        t.advance_to(P1, Step::Draw);
        assert!(!t.has_lost(P0));
        t.answer_yes(P0, true);
        if pay {
            t.advance_to(P0, Step::Draw);
        } else {
            let _ = t.g.run_until(10_000, |g| {
                g.result.is_some() || (g.turn.active == P0 && g.turn.step == Step::Draw)
            });
        }
        assert_eq!(t.has_lost(P0), !pay, "paid: {pay}");
        if pay {
            assert!(t.g.battlefield.iter().all(|id| {
                let o = t.g.obj(*id);
                o.controller != P0 || o.tapped
            }));
        }
    }
}

#[test]
fn enters_then_draw_at_next_upkeep() {
    cr!("603.7a", "603.7e");
    assert_supported(&["Ritual of Steel"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ritual = enter(&mut t, P0, "Ritual of Steel");
    t.g.attach(ritual, Entity::Object(bears));
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 1);
}
