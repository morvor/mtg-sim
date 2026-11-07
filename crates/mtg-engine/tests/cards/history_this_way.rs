//! "If [objects] [verb] this way, [effect]" after an optional instruction, after "exile
//! ..., then return that card", and with "you put [a land] onto the battlefield this
//! way" (`oracle/patterns/conditions_this_way.rs`); "If excess damage was dealt to a
//! creature / that creature this way, ..." (`r120_excess_this_way.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn count_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.name == name)
        .count()
}

#[test]
fn sirens_ruse_draws_only_for_a_pirate() {
    cr!("608.2c", "400.7");
    assert_supported("Siren's Ruse");
    for (name, draws) in [("Fathom Fleet Captain", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let c = t.battlefield(P0, name);
        t.lands(P0, "Island", 2);
        let ruse = t.hand(P0, "Siren's Ruse");
        let hand = t.hand_size(P0);
        t.cast(P0, ruse).target(c).go();
        t.resolve_all();
        assert_eq!(count_named(&t, P0, name), 1, "{}", t.dump_log());
        // The Ruse left the hand; a Pirate draws one card.
        assert_eq!(
            t.hand_size(P0),
            hand - 1 + draws,
            "{name}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn spelunking_gains_life_only_for_a_cave() {
    cr!("608.2c");
    assert_supported("Spelunking");
    for (land, life) in [("Captivating Cave", 24), ("Forest", 20)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.library_top(P0, "Island");
        let l = t.hand(P0, land);
        t.lands(P0, "Forest", 3);
        let sp = t.hand(P0, "Spelunking");
        t.cast(P0, sp).go();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(l)]);
        t.resolve_all();
        assert_eq!(
            count_named(&t, P0, land),
            1 + (land == "Forest") as usize * 3
        );
        assert_eq!(t.life(P0), life, "{land}: {}", t.dump_log());
    }
    // Declining to put a land onto the battlefield: no life.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    t.hand(P0, "Captivating Cave");
    t.lands(P0, "Forest", 3);
    let sp = t.hand(P0, "Spelunking");
    t.cast(P0, sp).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn oviya_counters_only_on_an_artifact() {
    cr!("608.2c");
    for (name, artifact) in [("Alpha Myr", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let oviya = t.battlefield(P0, "Oviya, Automech Artisan");
        t.lands(P0, "Forest", 1);
        let c = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.activate(P0, oviya, 0, &[]).unwrap();
        t.resolve_all();
        let p = t.named_on_battlefield(name);
        assert_eq!(p.len(), 1, "{}", t.dump_log());
        let n = if artifact { 2 } else { 0 };
        assert_eq!(t.counters(p[0], "+1/+1"), n, "{name}: {}", t.dump_log());
    }
}

#[test]
fn lazav_becomes_a_copy_of_an_exiled_creature_card() {
    cr!("608.2c", "707.2");
    assert_supported("Lazav, Familiar Stranger");
    for (exiled, copies) in [("Hill Giant", true), ("Forest", false)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let lazav = t.battlefield(P0, "Lazav, Familiar Stranger");
        let card = t.graveyard(P1, exiled);
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        // Targeting an opponent is a crime.
        t.cast(P0, bolt).target(Entity::Player(P1)).go();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(card)]);
        t.answer_yes(P0, true);
        t.resolve_all();
        let name = t.g.obj(t.g.current(lazav)).chars.name.clone();
        assert_eq!(name == "Hill Giant", copies, "{exiled}: {}", t.dump_log());
        assert_eq!(t.g.exile.len(), 1, "{}", t.dump_log());
    }
}

#[test]
fn torch_the_witness_investigates_only_with_excess_damage() {
    cr!("120.10");
    assert_supported("Torch the Witness");
    for (victim, clue) in [("Llanowar Elves", 1), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let v = t.battlefield(P1, victim);
        t.lands(P0, "Mountain", 2);
        let torch = t.hand(P0, "Torch the Witness");
        // X = 1: 2 damage.
        t.cast(P0, torch).x(1).target(v).go();
        t.resolve_all();
        assert_eq!(
            count_named(&t, P0, "Clue Token"),
            clue,
            "{victim}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn vikya_draws_for_excess_damage_to_a_creature_only() {
    cr!("120.10");
    let player = Entity::Player(P1);
    for (victim, draws) in [
        (Some("Llanowar Elves"), 1),
        (Some("Hill Giant"), 0),
        (None, 0),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let vikya = t.battlefield(P0, "Vikya, Scorching Stalwart");
        t.g.tap(vikya);
        t.lands(P0, "Mountain", 5);
        t.hand(P0, "Island");
        let target = victim.map_or(player, |v| Entity::Object(t.battlefield(P1, v)));
        t.library_top(P0, "Island");
        let hand = t.hand_size(P0);
        t.activate(P0, vikya, 0, &[target]).unwrap();
        t.resolve_all();
        // A card discarded for the cost.
        assert_eq!(
            t.hand_size(P0),
            hand - 1 + draws,
            "{victim:?}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn break_the_spell_draws_for_your_permanent_or_a_token() {
    cr!("608.2c", "608.2h");
    assert_supported("Break the Spell");
    for (whose, token, draws) in [(P0, false, 1), (P1, false, 0), (P1, true, 1)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let e = if token {
            let spec = mtg_engine::tokens::predefined("Shard").expect("Shard token");
            let tc = mtg_engine::replacement::TokenCreate {
                chars: mtg_engine::tokens::token_characteristics(&spec),
                card: None,
                tapped: false,
                attacking: None,
                copy_of: None,
                copy_exceptions: vec![],
            };
            t.g.create_tokens(whose, tc, 1, None)[0]
        } else {
            t.battlefield(whose, "Glorious Anthem")
        };
        t.lands(P0, "Plains", 1);
        let bts = t.hand(P0, "Break the Spell");
        let hand = t.hand_size(P0);
        t.cast(P0, bts).target(e).go();
        t.resolve_all();
        assert!(!t.on_battlefield(e));
        assert_eq!(
            t.hand_size(P0),
            hand - 1 + draws,
            "{whose:?} {token}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn shilgengar_makes_blood_equal_to_a_sacrificed_angels_toughness() {
    cr!("608.2h", "608.2c");
    assert_supported("Shilgengar, Sire of Famine");
    for (victim, bloods) in [("Serra Angel", 4), ("Grizzly Bears", 1)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let sh = t.battlefield(P0, "Shilgengar, Sire of Famine");
        let v = t.battlefield(P0, victim);
        t.answer_choose(P0, &[Entity::Object(v)]);
        t.activate(P0, sh, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(
            count_named(&t, P0, "Blood Token"),
            bloods,
            "{victim}: {}",
            t.dump_log()
        );
    }
}

#[test]
fn arid_archway_surveils_only_for_another_desert() {
    cr!("608.2c");
    assert_supported("Arid Archway");
    // Returning a Desert other than itself, a non-Desert, and itself.
    for (other, surveils) in [
        (Some("Desert"), true),
        (Some("Forest"), false),
        (None, false),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let o = other.map(|n| t.battlefield(P0, n));
        t.library_top(P0, "Island");
        let aa = t.hand(P0, "Arid Archway");
        // The only choice is the land returned.
        if let Some(o) = o {
            t.answer_choose(P0, &[Entity::Object(o)]);
        }
        t.play_land(P0, aa).unwrap();
        t.resolve_all();
        let asked = t
            .asked()
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Surveil { .. }));
        assert_eq!(asked, surveils, "{other:?}: {}", t.dump_log());
    }
}
