//! Trigger conditions on player actions and spell properties: cycling (CR 702.29),
//! magecraft "cast or copy" (CR 707.10), keyword-action events, kicked spells, "your first
//! [kind of] spell each turn", battalion, counters being put on permanents, and tokens
//! leaving the battlefield.

use mtg_engine::testing::*;
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

/// Index of the ability named `text` among the object's activated abilities.
fn activated_index(t: &TestGame, id: ObjectId, text: &str) -> usize {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text == text)
        .expect("no such activated ability")
}

#[test]
fn when_you_cycle_this_card() {
    cr!("702.29a", "702.29c");
    assert_supported(&["Windcaller Aven"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    let aven = t.hand(P0, "Windcaller Aven");
    let hand = t.hand_size(P0);
    let i = activated_index(&t, aven, "Cycling");
    // The cycling trigger targets the Bears (it triggers from the graveyard).
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.activate(P0, aven, i, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Windcaller Aven"));
    // Discarded the Aven, drew a card.
    assert_eq!(t.hand_size(P0), hand);
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn whenever_a_player_cycles_a_card() {
    cr!("702.29c");
    assert_supported(&["Stoic Champion", "Windcaller Aven"]);
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Stoic Champion");
    let (p, tough) = t.pt(champ);
    t.lands(P1, "Island", 3);
    let aven = t.hand(P1, "Windcaller Aven");
    let i = activated_index(&t, aven, "Cycling");
    t.answer_targets(P1, &[Entity::Object(champ)]);
    t.activate(P1, aven, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(champ), (p + 2, tough + 2));
}

#[test]
fn magecraft_triggers_on_cast_and_on_copy() {
    cr!("707.10", "603.2");
    assert_supported(&["Lorehold Pledgemage", "Opt"]);
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Lorehold Pledgemage");
    let (p, _) = t.pt(mage);
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    let spell = t.cast(P0, opt).go();
    t.settle();
    // Cast: one trigger.
    assert_eq!(t.stack_len(), 2);
    // A copy of the spell triggers it too (a copy isn't cast, CR 707.10).
    mtg_engine::copy::copy_spell(&mut t.g, spell, P0, false);
    t.resolve_all();
    assert_eq!(t.pt(mage).0, p + 2);
}

#[test]
fn whenever_you_scry_once_each_turn() {
    cr!("701.22a", "701.22d");
    assert_supported(&["Chance-Met Elves", "Opt"]);
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Chance-Met Elves");
    t.lands(P0, "Island", 2);
    for _ in 0..2 {
        let opt = t.hand(P0, "Opt");
        t.cast(P0, opt).go();
        t.resolve_all();
    }
    // "This ability triggers only once each turn."
    assert_eq!(t.counters(elves, "+1/+1"), 1);
}

#[test]
fn scry_zero_is_no_scry_event() {
    cr!("701.22b");
    assert_supported(&["Chance-Met Elves"]);
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Chance-Met Elves");
    mtg_engine::library::scry(&mut t.g, P0, 0);
    t.resolve_all();
    assert_eq!(t.counters(elves, "+1/+1"), 0);
    mtg_engine::library::scry(&mut t.g, P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(elves, "+1/+1"), 1);
}

#[test]
fn whenever_you_surveil() {
    cr!("701.25a", "701.25d");
    assert_supported(&["Dimir Spybug"]);
    let mut t = TestGame::new(2);
    let bug = t.battlefield(P0, "Dimir Spybug");
    mtg_engine::library::surveil(&mut t.g, P0, 2);
    t.resolve_all();
    assert_eq!(t.counters(bug, "+1/+1"), 1);
}

#[test]
fn kicked_spell_trigger() {
    cr!("702.33d");
    assert_supported(&["Lullmage's Familiar", "Tempest Owl"]);
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lullmage's Familiar");
        t.lands(P0, "Island", 7);
        let owl = t.hand(P0, "Tempest Owl");
        t.cast(P0, owl).kicked(kicked).go();
        t.resolve_all();
        let expected = if kicked { 22 } else { 20 };
        assert_eq!(t.life(P0), expected, "kicked: {kicked}");
    }
}

#[test]
fn first_noncreature_spell_each_turn() {
    cr!("603.2");
    assert_supported(&["Valeria Richards, Precocious"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valeria Richards, Precocious");
    t.lands(P0, "Forest", 2);
    let hand = t.hand_size(P0);
    // A creature spell first: doesn't count.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    t.lands(P0, "Mountain", 2);
    for _ in 0..2 {
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(P1).go();
        t.resolve_all();
    }
    // Only the first Lightning Bolt drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn battalion() {
    cr!("508.1");
    assert_supported(&["Haazda Marshal"]);
    for n in [2usize, 3] {
        let mut t = TestGame::new(2);
        let marshal = t.battlefield(P0, "Haazda Marshal");
        let mut attackers = vec![(marshal, Entity::Player(P1))];
        for _ in 1..n {
            let b = t.battlefield(P0, "Grizzly Bears");
            attackers.push((b, Entity::Player(P1)));
        }
        t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
        t.attack(&attackers, &[]);
        let soldiers =
            t.g.battlefield
                .iter()
                .filter(|id| t.g.obj(**id).chars.has_subtype("Soldier"))
                .count();
        // Haazda Marshal is itself a Soldier.
        let expected = if n == 3 { 2 } else { 1 };
        assert_eq!(soldiers, expected, "{n} attackers");
    }
}

#[test]
fn counters_put_on_another_creature_you_control() {
    cr!("122.1", "603.2");
    assert_supported(&["Enduring Scalelord"]);
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Enduring Scalelord");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.add_counters(Entity::Object(bears), "+1/+1", 2, None);
    t.resolve_all();
    // One trigger for the one event; the Scalelord's own counter doesn't retrigger it.
    assert_eq!(t.counters(lord, "+1/+1"), 1);
}

#[test]
fn sacrificing_a_food_to_its_own_ability() {
    cr!("603.10a", "701.21a");
    assert_supported(&["Rapacious Guest"]);
    let mut t = TestGame::new(2);
    let guest = t.battlefield(P0, "Rapacious Guest");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(guest, Entity::Player(P1))], &[]);
    // "Whenever one or more creatures you control deal combat damage to a player, create
    // a Food token."
    let food =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).chars.has_subtype("Food"))
            .expect("a Food token");
    t.set_step(P0, mtg_engine::turn::Step::PostcombatMain);
    t.lands(P0, "Plains", 2);
    // "Whenever you sacrifice a Food, put a +1/+1 counter on ~": the Food is sacrificed to
    // pay its own ability's cost.
    t.activate(P0, food, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(guest, "+1/+1"), 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn token_you_control_leaves_the_battlefield() {
    cr!("603.6c", "603.10a");
    assert_supported(&["Boomer Scrapper", "Raise the Alarm"]);
    let mut t = TestGame::new(2);
    let scrapper = t.battlefield(P0, "Boomer Scrapper");
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    let token =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).kind == mtg_engine::object::ObjKind::Token)
            .unwrap();
    t.g.destroy(token, None);
    t.resolve_all();
    assert_eq!(t.counters(scrapper, "+1/+1"), 1);
    // A nontoken creature leaving doesn't trigger it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.counters(scrapper, "+1/+1"), 1);
}
