//! Referent and value grammar III (CR 107, 603.4, 603.10a, 608.2c, 608.2h): "that many" /
//! "that much" bound to an intervening "if" clause's count, to a trigger's "one or more"
//! qualifier, or to the trigger event's amount; "remove all of them" after a condition
//! about counters; "the counters removed this way"; "the difference" after a comparison.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = t.g.obj(*id);
            o.controller == p && o.kind == object::ObjKind::Token
        })
        .collect()
}

#[test]
fn desert_warfare_creates_as_many_tokens_as_deserts() {
    cr!("603.4");
    compiles("Desert Warfare");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Desert Warfare");
    for _ in 0..6 {
        t.battlefield(P0, "Desert of the True");
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let made = tokens(&t, P0);
    assert_eq!(made.len(), 6, "one token per Desert you control");
    // They gain haste.
    t.g.recompute();
    assert!(made
        .iter()
        .all(|id| t.g.obj(*id).chars.has_keyword(keywords::KeywordKind::Haste)));
}

#[test]
fn desert_warfare_with_fewer_than_five_deserts_does_nothing() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Desert Warfare");
    for _ in 0..4 {
        t.battlefield(P0, "Desert of the True");
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn reyhan_counts_the_counters_the_creature_had() {
    cr!("603.10a", "603.4");
    ruling!(
        "Reyhan, Last of the Abzan",
        "the number of +1/+1 counters on it before it got any -1/-1 counters will be used"
    );
    compiles("Reyhan, Last of the Abzan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reyhan, Last of the Abzan");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Llanowar Elves");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 3, None);
    // Six -1/-1 counters at once: it dies with three +1/+1 counters it had.
    t.g.add_counters(Entity::Object(bear), "-1/-1", 6, None);
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.counters(other, "+1/+1"), 3);
}

#[test]
fn lightning_coils_removes_its_counters_and_creates_that_many() {
    cr!("603.4", "122.8");
    compiles("Lightning Coils");
    let mut t = TestGame::new(2);
    let coils = t.battlefield(P0, "Lightning Coils");
    t.g.add_counters(Entity::Object(coils), "charge", 6, None);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(coils, "charge"), 0);
    assert_eq!(tokens(&t, P0).len(), 6, "as many tokens as counters removed");
}

#[test]
fn sarulf_exiles_by_the_number_of_counters_removed() {
    cr!("608.2c", "603.4");
    ruling!(
        "Sarulf, Realm Eater",
        "If you do remove +1/+1 counters, you must remove all of them"
    );
    compiles("Sarulf, Realm Eater");
    let mut t = TestGame::new(2);
    let sarulf = t.battlefield(P0, "Sarulf, Realm Eater");
    t.g.add_counters(Entity::Object(sarulf), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(sarulf), "charge", 1, None);
    let cheap = t.battlefield(P1, "Grizzly Bears");
    let pricey = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(sarulf, "+1/+1"), 0);
    // "won't affect any other counters it may have"
    assert_eq!(t.counters(sarulf, "charge"), 1);
    assert!(!t.on_battlefield(cheap), "mana value 2 <= 2 counters removed");
    assert!(t.on_battlefield(pricey), "mana value 4 > 2");
    assert!(t.on_battlefield(land), "nonland only");
    assert!(t.on_battlefield(sarulf), "other permanents only");
}

#[test]
fn smoldering_egg_removes_its_ember_counters_and_transforms() {
    cr!("122.8");
    compiles("Smoldering Egg // Ashmouth Dragon");
    let mut t = TestGame::new(2);
    let egg = t.battlefield(P0, "Smoldering Egg // Ashmouth Dragon");
    t.g.add_counters(Entity::Object(egg), "ember", 5, None);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lava Spike");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    // 5 + 1 = 6: not yet.
    assert_eq!(t.counters(egg, "ember"), 6);
    let spike = t.hand(P0, "Lava Spike");
    t.cast(P0, spike).target(Entity::Player(P1)).go();
    t.resolve_all();
    let now = t.g.current(egg);
    assert_eq!(t.counters(now, "ember"), 0);
    assert_eq!(t.g.obj(now).chars.name.as_str(), "Ashmouth Dragon");
}

#[test]
fn shadow_urchin_exiles_as_many_cards_as_counters() {
    cr!("603.10a");
    ruling!(
        "Shadow Urchin",
        "effects that refer to the counters on that creature when it died will see all of those"
    );
    compiles("Shadow Urchin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Urchin");
    let bear = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.g.add_counters(Entity::Object(bear), "+1/+1", 1, None);
    // Three -1/-1 counters: one of each annihilates, it dies with two -1/-1 counters, but
    // the ability sees all four counters it had.
    t.g.add_counters(Entity::Object(bear), "-1/-1", 3, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    let exiled = t
        .g
        .exile
        .iter()
        .filter(|id| t.g.obj(**id).chars.name.as_str() == "Island")
        .count();
    assert_eq!(exiled, 4);
}

#[test]
fn namor_counts_blue_mana_symbols() {
    cr!("107.4e");
    compiles("Namor the Sub-Mariner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Namor the Sub-Mariner");
    t.lands(P0, "Island", 7);
    // Divination and Deep Analysis each have one blue mana symbol in their mana costs.
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve_all();
    assert_eq!(merfolk_tokens(&t), 1);
    let opt = t.hand(P0, "Deep Analysis");
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(merfolk_tokens(&t), 2, "Deep Analysis has one blue mana symbol");
}

fn merfolk_tokens(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.kind == object::ObjKind::Token && o.chars.has_subtype("Merfolk")
        })
        .count()
}

#[test]
fn basri_ket_soldiers_count_the_attacking_nontoken_creatures() {
    cr!("603.7b", "603.2c");
    compiles("Basri Ket");
    let mut t = TestGame::new(2);
    let basri = t.battlefield(P0, "Basri Ket");
    t.g.add_counters(Entity::Object(basri), "loyalty", 3, None);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, basri, 1, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1)), (b, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let soldiers: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    assert!(soldiers.iter().all(|s| t.g.obj(*s).tapped));
}

#[test]
fn heroic_feast_targets_up_to_that_many_creatures() {
    cr!("603.2");
    compiles("Heroic Feast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heroic Feast");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Object(a), Entity::Object(b)]),
    );
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
}

#[test]
fn jagged_poppet_discards_equal_to_the_damage_if_hellbent() {
    cr!("603.4");
    compiles("Jagged Poppet");
    let mut t = TestGame::new(2);
    let poppet = t.battlefield(P0, "Jagged Poppet");
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(poppet, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P1), 1, "three cards for three damage");
}

#[test]
fn fear_of_burning_alive_deals_that_amount() {
    cr!("603.4");
    compiles("Fear of Burning Alive");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fear of Burning Alive");
    for n in ["Lightning Bolt", "Grizzly Bears", "Forest", "Bonesplitter"] {
        t.graveyard(P0, n);
    }
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(giant), "3 damage to the 3/3");
}

#[test]
fn drizzt_gets_the_difference_in_power() {
    cr!("603.4", "603.10a");
    compiles("Drizzt Do'Urden");
    let mut t = TestGame::new(2);
    let drizzt = t.battlefield(P0, "Drizzt Do'Urden");
    let colossus = t.battlefield(P1, "Craw Wurm");
    t.g.destroy(colossus, None);
    t.resolve_all();
    // Craw Wurm 6/4 vs Drizzt 3/5: three counters.
    assert_eq!(t.counters(drizzt, "+1/+1"), 3);
    // A smaller creature dying doesn't trigger.
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(t.counters(drizzt, "+1/+1"), 3);
}

#[test]
fn shadow_of_mortality_costs_less_by_the_life_lost() {
    cr!("601.2f", "119.1");
    compiles("Shadow of Mortality");
    let mut t = TestGame::new(2);
    t.g.lose_life(P0, 11);
    t.lands(P0, "Swamp", 4);
    let shadow = t.hand(P0, "Shadow of Mortality");
    // {13}{B}{B} minus 11: {2}{B}{B}.
    t.cast(P0, shadow).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(shadow)));
}

#[test]
fn conformer_shuriken_compares_with_the_tapped_creature() {
    cr!("608.2h");
    compiles("Conformer Shuriken");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let shuriken = t.battlefield(P0, "Conformer Shuriken");
    t.g.attach(shuriken, Entity::Object(bear));
    let giant = t.battlefield(P1, "Craw Wurm");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.g.obj(giant).tapped);
    // 6 - 2 = 4 counters.
    assert_eq!(t.counters(bear, "+1/+1"), 4);
}

#[test]
fn oath_of_lim_dul_sacrifices_a_permanent_for_each_life_lost() {
    cr!("119.3", "608.2c");
    compiles("Oath of Lim-Dûl");
    let mut t = TestGame::new(2);
    let oath = t.battlefield(P0, "Oath of Lim-Dûl");
    let forest = t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    // No cards in hand to discard instead: two permanents for two life.
    t.g.lose_life(P0, 2);
    t.resolve_all();
    assert!(t.on_battlefield(oath), "other than ~");
    let left = [forest, bear, elf]
        .iter()
        .filter(|id| t.on_battlefield(**id))
        .count();
    assert_eq!(left, 1);
}

#[test]
fn discordant_spirit_counts_the_damage_dealt_to_you_this_turn() {
    cr!("603.4", "120.3");
    compiles("Discordant Spirit");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Discordant Spirit");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.advance_to(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(spirit, "+1/+1"), 3);
}

#[test]
fn tobias_creates_a_zombie_for_each_nontoken_creature_that_died_under_your_control() {
    cr!("700.4", "603.10a");
    compiles("Tobias, Doomed Conqueror");
    let mut t = TestGame::new(2);
    let tobias = t.battlefield(P0, "Tobias, Doomed Conqueror");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.g.destroy(bear, None);
    t.g.destroy(theirs, None);
    t.resolve_all();
    t.g.destroy(tobias, None);
    t.resolve_all();
    // The Bears and Tobias itself; not the opponent's creature.
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn angrath_deals_damage_equal_to_the_total_power_of_the_creatures() {
    cr!("608.2h");
    ruling!(
        "Angrath, Minotaur Pirate",
        "uses the power of those creatures as they last existed on the battlefield to determine their total power"
    );
    ruling!(
        "Angrath, Minotaur Pirate",
        "its power still contributes to the amount of damage dealt"
    );
    compiles("Angrath, Minotaur Pirate");
    let mut t = TestGame::new(2);
    let angrath = t.battlefield(P0, "Angrath, Minotaur Pirate");
    t.g.add_counters(Entity::Object(angrath), "loyalty", 6, None);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let gargoyle = t.battlefield(P1, "Darksteel Gargoyle");
    t.g.add_counters(Entity::Object(bear), "+1/+1", 1, None);
    let mine = t.battlefield(P0, "Hill Giant");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, angrath, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(wurm) && !t.on_battlefield(bear));
    assert!(t.on_battlefield(gargoyle), "indestructible");
    assert!(t.on_battlefield(mine));
    // 6 (Craw Wurm) + 3 (Grizzly Bears with its counter) + 3 (Darksteel Gargoyle).
    assert_eq!(t.life(P1), 8);
}

#[test]
fn unquenchable_fury_deals_damage_equal_to_the_defending_players_hand() {
    cr!("506.2", "107.3c");
    compiles("Unquenchable Fury");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let fury = t.battlefield(P0, "Unquenchable Fury");
    t.g.attach(fury, Entity::Object(bear));
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    t.resolve_all();
    // 4 from the trigger, 2 combat damage.
    assert_eq!(t.life(P1), 14);
}
