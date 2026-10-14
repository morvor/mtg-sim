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

#[test]
fn fangorn_adds_twice_as_much_green_as_treefolk_attacking() {
    cr!("603.2c", "508.3c");
    compiles("Fangorn, Tree Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fangorn, Tree Shepherd");
    let a = t.battlefield(P0, "Ironroot Treefolk");
    let b = t.battlefield(P0, "Ironroot Treefolk");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (bear, Entity::Player(P1)),
        ]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(mana::ManaType::G), 4);
}

#[test]
fn swallow_whole_counters_the_creature_tapped_for_its_cost() {
    cr!("601.2b", "118.8");
    ruling!(
        "Swallow Whole",
        "The target tapped creature will be exiled even if the creature you tapped leaves the battlefield"
    );
    compiles("Swallow Whole");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Llanowar Elves");
    let victim = t.battlefield(P1, "Hill Giant");
    t.g.tap(victim);
    let spell = t.hand(P0, "Swallow Whole");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.cast(P0, spell).target(Entity::Object(victim)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(victim));
    assert!(t.g.obj(bear).tapped);
    assert_eq!(t.counters(bear, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 0);
}

#[test]
fn fists_of_flame_counts_the_card_it_drew() {
    cr!("608.2h");
    ruling!(
        "Fists of Flame",
        "The bonus that the target creature receives is locked in as Fists of Flame resolves"
    );
    compiles("Fists of Flame");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let spell = t.hand(P0, "Fists of Flame");
    t.advance_to(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(bear)).go();
    t.resolve_all();
    t.g.recompute();
    // The card drawn in the draw step and the one Fists of Flame drew.
    assert_eq!(t.pt(bear), (4, 2));
    assert!(t
        .g
        .obj(bear)
        .chars
        .has_keyword(keywords::KeywordKind::Trample));
}

#[test]
fn jared_carthalion_counts_the_colors_of_each_creature() {
    cr!("105.2");
    compiles("Jared Carthalion");
    let mut t = TestGame::new(2);
    let jared = t.battlefield(P0, "Jared Carthalion");
    t.g.add_counters(Entity::Object(jared), "loyalty", 3, None);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let charm = t.battlefield(P0, "Kitchen Finks");
    t.advance_to(P0, Step::PrecombatMain);
    // Both creatures for the one "up to two targets" slot.
    t.answer_targets(P0, &[Entity::Object(bear), Entity::Object(charm)]);
    t.activate(P0, jared, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 1);
    assert_eq!(t.counters(charm, "+1/+1"), 2, "green and white");
}

#[test]
fn debt_to_the_deathless_gains_the_total_life_lost() {
    cr!("119.3", "608.2c");
    compiles("Debt to the Deathless");
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Debt to the Deathless");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(PlayerId(2)), 16);
    assert_eq!(t.life(P0), 28);
}

#[test]
fn niv_mizzet_guildpact_counts_different_color_pairs() {
    cr!("105.5");
    ruling!(
        "Niv-Mizzet, Guildpact",
        "A \"color pair\" is exactly two colors."
    );
    compiles("Niv-Mizzet, Guildpact");
    let mut t = TestGame::new(2);
    // Niv-Mizzet (white, blue, black, red, green: five colors) isn't a color pair.
    let niv = t.battlefield(P0, "Niv-Mizzet, Guildpact");
    t.battlefield(P0, "Azorius Guildmage");
    t.battlefield(P0, "Azorius Guildmage");
    t.battlefield(P0, "Kitchen Finks");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let hand = t.hand_size(P0);
    t.attack(&[(niv, Entity::Player(P1))], &[]);
    t.resolve_all();
    // Two different pairs (white-blue, green-white): 2 damage, 2 cards, 2 life.
    assert_eq!(t.life(P1), 20 - 6 - 2);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn blademane_baku_gets_a_bonus_for_each_counter_removed() {
    cr!("107.3a", "602.2b");
    compiles("Blademane Baku");
    let mut t = TestGame::new(2);
    let baku = t.battlefield(P0, "Blademane Baku");
    t.g.add_counters(Entity::Object(baku), "ki", 3, None);
    t.lands(P0, "Mountain", 1);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let idx = 0;
    t.activate(P0, baku, idx, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.counters(baku, "ki"), 1);
    assert_eq!(t.pt(baku), (1 + 4, 1));
}

#[test]
fn stabwhisker_drains_for_each_card_fewer_than_three() {
    cr!("710.2");
    compiles("Nezumi Shortfang // Stabwhisker the Odious");
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Nezumi Shortfang // Stabwhisker the Odious");
    assert!(flip::flip(&mut t.g, rat));
    t.g.recompute();
    t.hand(P1, "Island");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "one card: two fewer than three");
    // Three or more cards: no life loss.
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn roiling_terrain_counts_the_lands_controllers_graveyard() {
    cr!("608.2h");
    compiles("Roiling Terrain");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let land = t.battlefield(P1, "Forest");
    t.graveyard(P1, "Plains");
    t.graveyard(P1, "Island");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Swamp");
    let spell = t.hand(P0, "Roiling Terrain");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(land)).go();
    t.resolve_all();
    // Plains, Island and the destroyed Forest.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn isildurs_fateful_strike_exiles_cards_down_to_four() {
    cr!("608.2h", "107.1b");
    compiles("Isildur's Fateful Strike");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Swamp", 4);
    let bear = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..6 {
        t.hand(P1, "Island");
    }
    let spell = t.hand(P0, "Isildur's Fateful Strike");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).target(Entity::Object(bear)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.hand_size(P1), 4);
    assert_eq!(
        t.g.exile
            .iter()
            .filter(|id| t.g.obj(**id).chars.name.as_str() == "Island")
            .count(),
        2
    );
}

#[test]
fn mister_negative_draws_the_life_it_lost_in_the_exchange() {
    cr!("119.5", "608.2c");
    compiles("Mister Negative");
    for (mine, theirs, drawn) in [(20, 15, 5), (10, 15, 0)] {
        let mut t = TestGame::new(2);
        t.g.players[0].life = mine;
        t.g.players[1].life = theirs;
        for _ in 0..8 {
            t.library_top(P0, "Island");
        }
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Mister Negative");
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (theirs, mine));
        assert_eq!(t.hand_size(P0), hand + drawn);
    }
}

#[test]
fn river_songs_diary_casts_one_of_its_exiled_cards_at_random() {
    cr!("607.2a", "603.4");
    compiles("River Song's Diary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Song's Diary");
    t.lands(P0, "Mountain", 4);
    t.advance_to(P0, Step::PrecombatMain);
    for _ in 0..4 {
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(Entity::Player(P1)).go();
        t.resolve_all();
    }
    assert_eq!(t.life(P1), 8);
    assert_eq!(t.g.exile.len(), 4);
    t.advance_to(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 5, "one of them cast for free");
}
