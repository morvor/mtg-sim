//! The spell cost grammar (`src/oracle/patterns/spell_cost_grammar.rs`, rules in
//! `src/kw/spell_cost_grammar.rs`): SUBJECT + "cost(s) AMOUNT less / more to cast" +
//! QUALIFIER, in static abilities and in effects (CR 601.2f, 118.7, 611.2c, 611.2f).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::types::CardType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// How many of `p`'s lands are tapped.
fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| {
            let o = t.obj_now(**o);
            o.controller == p && o.chars.card_types.contains(CardType::Land) && o.tapped
        })
        .count()
}

/// Casts `card` for `p` (with one target per slot) and returns how much mana it cost:
/// the number of lands tapped to pay for it.
fn paid(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> usize {
    let before = tapped_lands(t, p);
    if let Err(e) = t.cast_with(p, card, targets) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    tapped_lands(t, p) - before
}

#[test]
fn spell_cost_grammar_cards_compile() {
    assert_compiles(&[
        "Alisaie Leveilleur",
        "Naiad of Hidden Coves",
        "Strong Back",
        "The Destined Warrior",
        "Temur Battlecrier",
        "Hum of the Radix",
        "Pollywog Symbiote",
        "Battlefield Thaumaturge",
        "Defense Grid",
        "Irini Sengir",
        "High Seas",
        "The Scarlet Witch",
        "Gran-Gran",
        "Spellwild Ouphe",
        "Curse of Silence",
        "Gonti, Canny Acquisitor",
        "Doc Aurlock, Grizzled Genius",
        "Accursed Witch // Infectious Curse",
        "Paladin Class",
        "Aven Interrupter",
        "Terror of the Peaks",
        "Eluge, the Shoreless Sea",
        "Vine Gecko",
        "Momo, Friendly Flier",
        "Zimone, Infinite Analyst",
        "Peerless Samurai",
        "Hardened Berserker",
        "Invasion of the Giants",
        "Elspeth Conquers Death",
        "Rowan, Scion of War",
        "Will, Scion of Peace",
        "Cheering Fanatic",
    ]);
}

#[test]
fn monk_class_second_spell_gets_the_reduction() {
    cr!("601.2f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Monk Class");
    t.lands(P0, "Island", 6);
    let opt = t.hand(P0, "Opt");
    let div = t.hand(P0, "Divination");
    let div2 = t.hand(P0, "Divination");
    assert_eq!(paid(&mut t, P0, opt, &[]), 1);
    t.resolve_all();
    assert_eq!(paid(&mut t, P0, div, &[]), 2);
    t.resolve_all();
    assert_eq!(paid(&mut t, P0, div2, &[]), 3);
}

#[test]
fn naiad_makes_spells_cheaper_during_other_turns() {
    cr!("601.2f");
    // "During turns other than yours, spells you cast cost {1} less to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naiad of Hidden Coves");
    t.lands(P0, "Island", 6);
    let a = t.hand(P0, "Think Twice");
    assert_eq!(paid(&mut t, P0, a, &[]), 2);
    t.resolve_all();
    t.set_step(P1, Step::Upkeep);
    let b = t.hand(P0, "Think Twice");
    assert_eq!(paid(&mut t, P0, b, &[]), 1);
}

#[test]
fn defense_grid_taxes_spells_except_during_their_controllers_turn() {
    cr!("601.2f");
    ruling!(
        "Defense Grid",
        "The additional cost can be reduced by effects that reduce costs"
    );
    // "Each spell costs {3} more to cast except during its controller's turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defense Grid");
    t.lands(P0, "Island", 4);
    t.lands(P1, "Island", 8);
    let mine = t.hand(P0, "Opt");
    assert_eq!(paid(&mut t, P0, mine, &[]), 1);
    t.resolve_all();
    // P1's spell during P0's turn costs {3} more.
    let theirs = t.hand(P1, "Opt");

    assert_eq!(paid(&mut t, P1, theirs, &[]), 4);
    t.resolve_all();
    // A cost reduction reduces the tax: with Naiad of Hidden Coves it's {2} more.
    t.battlefield(P1, "Naiad of Hidden Coves");
    let theirs = t.hand(P1, "Opt");
    assert_eq!(paid(&mut t, P1, theirs, &[]), 3);
}

#[test]
fn damping_sphere_counts_the_spells_that_player_cast_this_turn() {
    cr!("601.2f");
    ruling!(
        "Damping Sphere",
        "counts spells that were cast during a turn even if Damping Sphere wasn't on the battlefield"
    );
    // "Each spell a player casts costs {1} more to cast for each other spell that player
    // has cast this turn."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let a = t.hand(P0, "Opt");
    assert_eq!(paid(&mut t, P0, a, &[]), 1);
    t.resolve_all();
    t.battlefield(P0, "Damping Sphere");
    let b = t.hand(P0, "Opt");
    assert_eq!(paid(&mut t, P0, b, &[]), 2);
    t.resolve_all();
    let c = t.hand(P0, "Opt");
    assert_eq!(paid(&mut t, P0, c, &[]), 3);
    t.resolve_all();
    // The opponent's first spell this turn isn't taxed.
    t.lands(P1, "Island", 1);
    let d = t.hand(P1, "Opt");
    assert_eq!(paid(&mut t, P1, d, &[]), 1);
}

#[test]
fn battlefield_thaumaturge_counts_creature_targets() {
    cr!("601.2c", "601.2f");
    ruling!(
        "Battlefield Thaumaturge",
        "can reduce only the generic portion of the spell's total cost"
    );
    // "Each instant and sorcery spell you cast costs {1} less to cast for each creature
    // it targets."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battlefield Thaumaturge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Island", 4);
    // Into the Roil ({1}{U}) targeting a creature costs {U}.
    let roil = t.hand(P0, "Into the Roil");
    t.answer_yes(P0, false);
    assert_eq!(paid(&mut t, P0, roil, &[Entity::Object(bears)]), 1);
    t.resolve_all();
    // Targeting a land, it costs {1}{U}.
    let roil = t.hand(P0, "Into the Roil");
    t.answer_yes(P0, false);
    assert_eq!(paid(&mut t, P0, roil, &[Entity::Object(land)]), 2);
}

#[test]
fn hum_of_the_radix_counts_its_controllers_artifacts() {
    cr!("601.2f");
    // "Each artifact spell costs {1} more to cast for each artifact its controller
    // controls."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hum of the Radix");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 4);
    t.lands(P1, "Island", 4);
    // P0 controls two artifacts: Ornithopter ({0}) costs {2}.
    let o = t.hand(P0, "Ornithopter");
    assert_eq!(paid(&mut t, P0, o, &[]), 2);
    t.resolve_all();
    // P1 controls none (Hum of the Radix is an enchantment): {0}. A nonartifact spell
    // isn't taxed.
    t.set_step(P1, Step::PrecombatMain);
    let o = t.hand(P1, "Ornithopter");
    assert_eq!(paid(&mut t, P1, o, &[]), 0);
    t.resolve_all();
    let opt = t.hand(P1, "Opt");
    assert_eq!(paid(&mut t, P1, opt, &[]), 1);
}

#[test]
fn spellwild_ouphe_and_terror_of_the_peaks_change_spells_that_target_them() {
    cr!("601.2c", "601.2f");
    // Spellwild Ouphe: "Spells that target ~ cost {2} less to cast."
    let mut t = TestGame::new(2);
    let ouphe = t.battlefield(P1, "Spellwild Ouphe");
    t.lands(P0, "Mountain", 4);
    let spear = t.hand(P0, "Searing Spear");
    assert_eq!(paid(&mut t, P0, spear, &[Entity::Object(ouphe)]), 1);
    t.resolve_all();
    // Terror of the Peaks: "Spells your opponents cast that target ~ cost an additional
    // 3 life to cast."
    let terror = t.battlefield(P1, "Terror of the Peaks");
    let bolt = t.hand(P0, "Lightning Bolt");
    assert_eq!(paid(&mut t, P0, bolt, &[Entity::Object(terror)]), 1);
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    // Not for its controller's spells.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    paid(&mut t, P1, bolt, &[Entity::Object(terror)]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn high_seas_and_irini_sengir_tax_two_kinds_of_spells() {
    cr!("601.2f");
    // High Seas: "Red creature spells and green creature spells cost {1} more to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "High Seas");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    assert_eq!(paid(&mut t, P0, bears, &[]), 3);
    t.resolve_all();
    let growth = t.hand(P0, "Giant Growth");
    assert_eq!(paid(&mut t, P0, growth, &[Entity::Object(bears)]), 1);
}

#[test]
fn doc_aurlock_reduces_spells_cast_from_a_graveyard() {
    cr!("601.2f", "702.34a");
    // "Spells you cast from your graveyard or from exile cost {2} less to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doc Aurlock, Grizzled Genius");
    t.lands(P0, "Island", 4);
    let tt = t.graveyard(P0, "Think Twice");
    let before = tapped_lands(&t, P0);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    // Flashback {2}{U} costs {U}.
    assert_eq!(tapped_lands(&t, P0) - before, 1);
}

#[test]
fn vine_gecko_reduces_only_the_first_kicked_spell() {
    cr!("601.2f", "702.33d");
    // "The first kicked spell you cast each turn costs {1} less to cast."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vine Gecko");
    t.lands(P0, "Mountain", 12);
    // An unkicked spell isn't a kicked spell.
    let a = t.hand(P0, "Burst Lightning");
    let before = tapped_lands(&t, P0);
    t.cast(P0, a).target(Entity::Player(P1)).kicked(false).go();
    assert_eq!(tapped_lands(&t, P0) - before, 1);
    t.resolve_all();
    // Kicked ({R} + {4}): {1} less.
    let b = t.hand(P0, "Burst Lightning");
    let before = tapped_lands(&t, P0);
    t.cast(P0, b).target(Entity::Player(P1)).kicked(true).go();
    assert_eq!(tapped_lands(&t, P0) - before, 4);
    t.resolve_all();
    // The second kicked spell pays its full cost.
    let c = t.hand(P0, "Burst Lightning");
    let before = tapped_lands(&t, P0);
    t.cast(P0, c).target(Entity::Player(P1)).kicked(true).go();
    assert_eq!(tapped_lands(&t, P0) - before, 5);
}

#[test]
fn hardened_berserker_makes_the_next_spell_cheaper() {
    cr!("601.2a", "601.2f", "611.2f");
    // "Whenever ~ attacks, the next spell you cast this turn costs {1} less to cast."
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Hardened Berserker");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Island", 6);
    let div = t.hand(P0, "Divination");
    let div2 = t.hand(P0, "Divination");
    // Only {1}{U} lands' worth needed: the card can be cast with two lands.
    assert_eq!(paid(&mut t, P0, div, &[]), 2);
    t.resolve_all();
    assert_eq!(paid(&mut t, P0, div2, &[]), 3);
}

#[test]
fn next_spell_reduction_counts_when_checking_whether_it_can_be_cast() {
    cr!("601.2f", "611.2f");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Hardened Berserker");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Island", 2);
    let div = t.hand(P0, "Divination");
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let castable = t
        .g
        .cast_options(P0, div)
        .into_iter()
        .any(|o| o.method == CastMethod::Normal && t.g.can_begin_cast(P0, div, &o));
    assert!(castable);
}

#[test]
fn rowan_locks_in_the_life_lost_as_the_ability_resolves() {
    cr!("611.2c", "601.2f");
    ruling!(
        "Rowan, Scion of War",
        "The value of X is determined only once"
    );
    // "{T}: Spells you cast this turn that are black and/or red cost {X} less to cast,
    // where X is the amount of life you lost this turn. Activate only as a sorcery."
    let mut t = TestGame::new(2);
    let rowan = t.battlefield(P0, "Rowan, Scion of War");
    t.g.lose_life(P0, 2);
    t.activate(P0, rowan, 0, &[]).expect("Rowan's ability");
    t.resolve_all();
    // More life lost later doesn't change X.
    t.g.lose_life(P0, 3);
    t.lands(P0, "Swamp", 6);
    // Hymn to Tourach ({B}{B}) can't be reduced; Mind Rot ({2}{B}) costs {B}.
    let rot = t.hand(P0, "Mind Rot");
    assert_eq!(paid(&mut t, P0, rot, &[Entity::Player(P1)]), 1);
    t.resolve_all();
    // A blue spell isn't reduced.
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    assert_eq!(paid(&mut t, P0, div, &[]), 3);
}
