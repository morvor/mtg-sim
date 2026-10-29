//! Costs and activation permissions, round 4 (cards4-costs-mana-replacements,
//! `oracle/patterns/costs_5.rs`, `activation_restrictions.rs`): an escape cost exiling any
//! number of cards with four or more card types among them, granted abilities only the
//! granting player may activate, and abilities only the owner may activate.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn nethergoyf_escapes_by_exiling_cards_with_four_card_types_among_them() {
    cr!("702.138a", "205.2a", "601.2h");
    assert_supported(&["Nethergoyf"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let goyf = t.graveyard(P0, "Nethergoyf");
    let cards = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Lightning Bolt"),
        t.graveyard(P0, "Forest"),
        t.graveyard(P0, "Ornithopter"),
    ];
    let es: Vec<Entity> = cards.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    t.cast(P0, goyf)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .x(4)
        .go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(goyf)));
    for c in cards {
        assert_eq!(t.zone(t.g.current(c)), Zone::Exile);
    }
    // Three card types only (creature, instant, land): it can't escape.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let goyf = t.graveyard(P0, "Nethergoyf");
    let cards = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Lightning Bolt"),
        t.graveyard(P0, "Forest"),
    ];
    let es: Vec<Entity> = cards.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    let r = t
        .cast(P0, goyf)
        .method(CastMethod::Keyword(KeywordKind::Escape))
        .x(3)
        .try_go();
    assert!(r.is_err() || !t.on_battlefield(t.g.current(goyf)));
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(goyf)));
}

#[test]
fn martyrdoms_granted_ability_can_be_activated_only_by_its_caster() {
    cr!("602.2", "614.9");
    assert_supported(&["Martyrdom"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let spell = t.hand(P0, "Martyrdom");
    t.cast(P0, spell).target(bears).go();
    t.resolve();
    // Another player gains control of the creature: they can't activate it; P0 can.
    t.g.objects[bears.0 as usize].controller = P1;
    t.g.dirty = true;
    t.g.recompute();
    assert!(t.activate(P1, bears, 0, &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    t.activate(P0, bears, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    // The next 1 damage to P0 is dealt to the Bears instead.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.obj(bears).damage, 1);
}

#[test]
fn personal_incarnation_only_its_owner_activates_and_damage_goes_to_its_owner() {
    cr!("602.2", "614.9");
    assert_supported(&["Personal Incarnation"]);
    let mut t = TestGame::new(2);
    let inc = t.battlefield(P0, "Personal Incarnation");
    t.g.objects[inc.0 as usize].controller = P1;
    t.g.dirty = true;
    t.g.recompute();
    // Its controller (not its owner) can't activate it.
    assert!(t.activate(P1, inc, 0, &[]).is_err());
    t.activate(P0, inc, 0, &[]).unwrap();
    t.resolve();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(inc).go();
    t.resolve();
    assert_eq!(t.life(P0), 19, "1 damage dealt to its owner");
    assert_eq!(t.g.obj(inc).damage, 2);
}

#[test]
fn flash_keeps_the_creature_for_its_mana_cost_reduced_by_two() {
    cr!("118.7d", "118.12a");
    ruling!("Flash", "Only the generic mana in that creature's mana cost is reduced");
    assert_supported(&["Flash"]);
    // Hill Giant {3}{R}: {1}{R} keeps it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Hill Giant");
    let flash = t.hand(P0, "Flash");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.cast(P0, flash).go();
    t.resolve();
    assert!(t.on_battlefield(t.g.current(giant)));
    assert!(t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .all(|o| !t.g.obj(o).chars.is(mtg_engine::types::CardType::Land) || t.g.obj(o).tapped));
    // Without the mana for {1}{R}: sacrificed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    let giant = t.hand(P0, "Hill Giant");
    let flash = t.hand(P0, "Flash");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.cast(P0, flash).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn convergence_of_dominion_reduces_graveyard_abilities_with_your_commander() {
    cr!("602.2b", "601.2f", "903.3");
    assert_supported(&["Convergence of Dominion"]);
    let commander_game = || {
        TestGame::with_config(
            2,
            mtg_engine::GameConfig {
                variant: mtg_engine::game::Variant::Commander,
                ..Default::default()
            },
        )
    };
    // Scrapwork Cohort's unearth {2}{W} costs {W}.
    let mut t = commander_game();
    t.battlefield(P0, "Convergence of Dominion");
    let cmd = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmd.0 as usize].is_commander = true;
    let name = t.g.obj(cmd).chars.name.clone();
    t.g.players[0].commander_names.push(name);
    t.g.recompute();
    t.battlefield(P0, "Plains");
    let cohort = t.graveyard(P0, "Scrapwork Cohort");
    t.activate(P0, cohort, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(cohort)));
    // Without your commander: full cost, can't pay with one Plains.
    let mut t = commander_game();
    t.battlefield(P0, "Convergence of Dominion");
    t.battlefield(P0, "Plains");
    let cohort = t.graveyard(P0, "Scrapwork Cohort");
    assert!(t.activate(P0, cohort, 0, &[]).is_err());
}

#[test]
fn vault_112_exiles_as_many_cards_as_the_energy_paid() {
    cr!("107.14", "714.2b");
    assert_supported(&["Vault 112: Sadistic Simulation"]);
    for paid in [2i64, 0] {
        let mut t = TestGame::new(2);
        t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
        t.g.add_counters(Entity::Player(P0), "energy", 3, None);
        let saga = t.battlefield(P0, "Vault 112: Sadistic Simulation");
        t.g.add_counters(Entity::Object(saga), "lore", 2, None);
        t.resolve_all();
        let energy = t.g.player(P0).counter("energy");
        let lib = t.library_size(P0);
        let exiled = |t: &TestGame| {
            t.g.objects
                .iter()
                .filter(|o| o.zone == Zone::Exile && o.owner == P0)
                .count()
        };
        let before = exiled(&t);
        t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(paid));
        t.g.add_counters(Entity::Object(t.g.current(saga)), "lore", 1, None);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        assert_eq!(t.g.player(P0).counter("energy"), energy - paid as u32);
        assert_eq!(t.library_size(P0), lib - paid as usize);
        assert_eq!(exiled(&t) - before, paid as usize);
    }
}

#[test]
fn phyrexian_dreadnought_stays_if_creatures_with_total_power_12_are_sacrificed() {
    cr!("118.12a", "107.3");
    ruling!("Phyrexian Dreadnought", "The sacrifices are simultaneous");
    assert_supported(&["Phyrexian Dreadnought"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Colossal Dreadmaw");
    let b = t.battlefield(P0, "Colossal Dreadmaw");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let dn = t.enter(P0, "Phyrexian Dreadnought");
    t.resolve_all();
    assert!(t.on_battlefield(dn));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    // Not paying: it's sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.enter(P0, "Phyrexian Dreadnought");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phyrexian Dreadnought"));
    assert!(t.named_on_battlefield("Grizzly Bears").len() == 1);
}

#[test]
fn plumb_the_forbidden_copies_itself_for_each_creature_sacrificed() {
    cr!("601.2b", "603.12", "707.10");
    ruling!("Plumb the Forbidden", "If you copy Plumb the Forbidden multiple times");
    assert_supported(&["Plumb the Forbidden"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    let hand = t.hand_size(P0);
    let spell = t.hand(P0, "Plumb the Forbidden");
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    // The spell and two copies: three cards, three life.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.life(P0), 17);
    // Without sacrificing: no copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let hand = t.hand_size(P0);
    let spell = t.hand(P0, "Plumb the Forbidden");
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Number(0));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn jotun_grunt_cumulative_upkeep_puts_cards_from_a_single_graveyard_on_the_bottom() {
    cr!("702.24a", "118.3");
    assert_supported(&["Jötun Grunt"]);
    let mut t = TestGame::new(2);
    let grunt = t.battlefield(P0, "Jötun Grunt");
    let x = t.graveyard(P1, "Lightning Bolt");
    let y = t.graveyard(P1, "Shock");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(x), Entity::Object(y)]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(grunt), "{}", t.dump_log());
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.graveyard_size(P0), 1);
    // One card in each graveyard: two from a single graveyard can't be chosen, so the cost
    // can't be paid and the Grunt is sacrificed.
    let mut t = TestGame::new(2);
    let grunt = t.battlefield(P0, "Jötun Grunt");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(!t.on_battlefield(grunt), "{}", t.dump_log());
}

#[test]
fn rift_elemental_removes_a_time_counter_from_a_suspended_card() {
    cr!("702.62b", "118.3");
    assert_supported(&["Rift Elemental"]);
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, "Rift Elemental");
    t.lands(P0, "Mountain", 2);
    let bloom = t.exile(P0, "Lotus Bloom");
    t.g.add_counters(Entity::Object(bloom), "time", 3, None);
    t.answer_choose(P0, &[Entity::Object(bloom)]);
    t.activate(P0, rift, 0, &[]).unwrap();
    assert_eq!(t.counters(bloom, "time"), 2);
    t.resolve();
    assert_eq!(t.pt(rift), (3, 1));
    // Nothing with a time counter: it can't be activated.
    let mut t = TestGame::new(2);
    let rift = t.battlefield(P0, "Rift Elemental");
    t.lands(P0, "Mountain", 2);
    t.exile(P0, "Lotus Bloom");
    assert!(t.activate(P0, rift, 0, &[]).is_err());
}

#[test]
fn cemetery_prowler_reduces_spells_by_card_types_shared_with_its_exiled_cards() {
    cr!("601.2f", "205.2a", "607.2a");
    ruling!(
        "Cemetery Prowler",
        "checks the number of card types among cards exiled with it"
    );
    assert_supported(&["Cemetery Prowler"]);
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(dead)]);
    t.enter(P0, "Cemetery Prowler");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // A creature spell shares one card type: {1}{G} costs {G}.
    t.battlefield(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
    // A sorcery shares none: {1}{G} with one Forest can't be cast.
    t.battlefield(P0, "Forest");
    let growth = t.hand(P0, "Rampant Growth");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, growth).try_go().is_err());
}

#[test]
fn sokrates_prevents_combat_damage_and_both_players_draw_half_of_it() {
    cr!("615.5", "107.1a");
    assert_supported(&["Sokrates, Athenian Teacher"]);
    let mut t = TestGame::new(2);
    let sok = t.battlefield(P0, "Sokrates, Athenian Teacher");
    let giant = t.battlefield(P0, "Hill Giant");
    t.activate(P0, sok, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve();
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // 3 damage: each draws one (half, rounded down).
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 1);
}

#[test]
fn urgent_necropsy_collects_evidence_equal_to_its_targets_total_mana_value() {
    cr!("701.59a", "601.2c", "601.2f");
    ruling!("Urgent Necropsy", "The value of X is locked in after you choose targets");
    assert_supported(&["Urgent Necropsy"]);
    // Targets of total mana value 6 (Mind Stone 2, Hill Giant 4): cards with a total mana
    // value of 6 or more exiled from the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let ring = t.battlefield(P1, "Mind Stone");
    let a = t.graveyard(P0, "Serra Angel");
    let b = t.graveyard(P0, "Shock");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    let spell = t.hand(P0, "Urgent Necropsy");
    t.cast(P0, spell)
        .target(Entity::Object(ring))
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Mind Stone"));
    assert!(t.in_exile("Serra Angel") && t.in_exile("Shock"));
    // Not enough mana value in the graveyard: it can't be cast with those targets.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.graveyard(P0, "Shock");
    let spell = t.hand(P0, "Urgent Necropsy");
    let r = t.cast(P0, spell).target(Entity::Object(giant)).try_go();
    assert!(r.is_err());
    assert!(t.on_battlefield(giant));
}
