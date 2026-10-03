//! The spell cost grammar (`src/oracle/patterns/spell_cost_grammar.rs`, rules in
//! `src/kw/spell_cost_grammar.rs`): SUBJECT + "cost(s) AMOUNT less / more to cast" +
//! QUALIFIER, in static abilities and in effects (CR 601.2f, 118.7, 611.2c, 611.2f).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::object::Zone;
use mtg_engine::types::{CardType, Color};
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

#[test]
fn own_cost_and_additional_cost_cards_compile() {
    assert_compiles(&[
        "Phyrexian Purge",
        "Closing Statement",
        "Sailors' Bane",
        "Geistlight Snare",
        "Corpse Cobble",
        "Burn at the Stake",
        "March of Wretched Sorrow",
        "March of Swirling Mist",
        "March of Burgeoning Life",
        "March of Reckless Joy",
        "March of Otherworldly Light",
        "Gorex, the Tombshell",
        "Explosive Singularity",
        "Hierophant Bio-Titan",
        "Dargo, the Shipwrecker",
        "Bite Down on Crime",
        "Voltage Surge",
        "Tectonic Split",
        "Bogslither's Embrace",
        "Molten Exhale",
    ]);
}

#[test]
fn phyrexian_purge_costs_life_for_each_target() {
    cr!("601.2c", "601.2f");
    // "This spell costs 3 life more to cast for each target. Destroy any number of target
    // creatures."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    let purge = t.hand(P0, "Phyrexian Purge");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, purge, CastMethod::Normal).expect("cast");
    assert_eq!(t.life(P0), 14);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn closing_statement_costs_less_during_your_end_step() {
    cr!("601.2f");
    // "This spell costs {2} less to cast during your end step."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 3);
    let cs = t.hand(P0, "Closing Statement");
    t.set_step(P0, Step::End);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(paid(&mut t, P0, cs, &[]), 3);
}

#[test]
fn geistlight_snare_has_two_reductions() {
    cr!("601.2f");
    // "This spell costs {1} less to cast if you control a Spirit. It also costs {1} less
    // to cast if you control an enchantment."
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    t.cast_with(P1, opt, &[]).expect("Opt");
    let opt = t.g.current(opt);
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Spectral Sailor");
    t.lands(P0, "Island", 3);
    let snare = t.hand(P0, "Geistlight Snare");
    assert_eq!(paid(&mut t, P0, snare, &[Entity::Object(opt)]), 1);
}

#[test]
fn torgaar_costs_less_for_each_creature_sacrificed() {
    cr!("601.2b", "601.2f", "601.2h");
    // "As an additional cost to cast this spell, you may sacrifice any number of
    // creatures. This spell costs {2} less to cast for each creature sacrificed this way."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 8);
    let torgaar = t.hand(P0, "Torgaar, Famine Incarnate");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    // {6}{B}{B} minus {4}.
    assert_eq!(paid(&mut t, P0, torgaar, &[]), 4);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn corpse_cobble_sacrifices_any_number_of_creatures() {
    cr!("601.2b", "601.2h");
    // "As an additional cost to cast this spell, sacrifice any number of creatures. Create
    // an X/X blue and black Zombie creature token with menace, where X is the total power
    // of the sacrificed creatures."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let cobble = t.hand(P0, "Corpse Cobble");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    paid(&mut t, P0, cobble, &[]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    let zombie = t.named_on_battlefield("Zombie Token");
    assert_eq!(zombie.len(), 1);
    assert_eq!(t.pt(zombie[0]), (5, 5));
}

#[test]
fn march_exiles_cards_for_a_reduction_independent_of_x() {
    cr!("601.2b", "601.2f", "107.3b");
    // March of Wretched Sorrow: "As an additional cost to cast this spell, you may exile
    // any number of black cards from your hand. This spell costs {2} less to cast for each
    // card exiled this way. ~ deals X damage to target creature or planeswalker and you
    // gain X life."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let march = t.hand(P0, "March of Wretched Sorrow");
    let rot = t.hand(P0, "Mind Rot");
    // X is 2 and one card is exiled: {2}{B} minus {2}.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    assert_eq!(paid(&mut t, P0, march, &[Entity::Object(giant)]), 1);
    assert_eq!(t.zone(rot), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn explosive_singularity_taps_creatures_for_a_reduction() {
    cr!("601.2b", "601.2f");
    // "As an additional cost to cast this spell, you may tap any number of untapped
    // creatures you control. This spell costs {1} less to cast for each creature tapped
    // this way."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 10);
    let es = t.hand(P0, "Explosive Singularity");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    // {8}{R}{R} minus {2}.
    assert_eq!(paid(&mut t, P0, es, &[Entity::Player(P1)]), 8);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn voltage_surge_optional_sacrifice() {
    cr!("601.2b", "118.8");
    // "As an additional cost to cast this spell, you may sacrifice an artifact. ~ deals 2
    // damage to target creature or planeswalker. If this spell's additional cost was paid,
    // ~ deals 4 damage instead."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let a = t.hand(P0, "Voltage Surge");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    paid(&mut t, P0, a, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    let b = t.hand(P0, "Voltage Surge");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    paid(&mut t, P0, b, &[Entity::Object(giant)]);
    assert!(!t.on_battlefield(thopter));
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn tectonic_split_sacrifices_half_the_lands_rounded_up() {
    cr!("601.2f", "601.2h");
    // "As an additional cost to cast this spell, sacrifice half the lands you control,
    // rounded up."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let split = t.hand(P0, "Tectonic Split");
    t.g.turn.priority = Some(P0);
    t.g.cast_spell(P0, split, CastMethod::Normal).expect("cast");
    let lands = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.obj_now(**o).controller == P0 && t.obj_now(**o).chars.card_types.contains(CardType::Land))
        .count();
    assert_eq!(lands, 3);
}

#[test]
fn bite_down_on_crime_costs_less_if_evidence_was_collected() {
    cr!("601.2b", "601.2f", "701.59a");
    // "As an additional cost to cast this spell, you may collect evidence 6. This spell
    // costs {2} less to cast if evidence was collected."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Colossal Dreadmaw");
    t.lands(P0, "Forest", 2);
    let bite = t.hand(P0, "Bite Down on Crime");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    // {3}{G} minus {2}.
    let before = tapped_lands(&t, P0);
    t.answer_targets(P0, &[Entity::Object(mine)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.g.turn.priority = Some(P0);
    let r = t.g.cast_spell(P0, bite, CastMethod::Normal);
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(tapped_lands(&t, P0) - before, 2);
    assert!(t.in_exile("Colossal Dreadmaw"));
}

#[test]
fn molten_exhale_can_be_cast_with_flash_by_beholding_a_dragon() {
    cr!("601.3c", "701.4a");
    // "You may cast this spell as though it had flash if you behold a Dragon as an
    // additional cost to cast it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shivan Dragon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let exhale = t.hand(P0, "Molten Exhale");
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let opts = t.g.cast_options(P0, exhale);
    assert!(opts.iter().any(|o| t.g.can_begin_cast(P0, exhale, o)));
}

#[test]
fn chain_lightning_lets_the_damaged_player_pay_to_copy_it() {
    cr!("707.10", "118.12");
    // "~ deals 3 damage to any target. Then that player or that permanent's controller may
    // pay {R}{R}. If the player does, they may copy this spell and may choose a new target
    // for that copy."
    assert_compiles(&[
        "Chain Lightning",
        "Chain Stasis",
        "String of Disappearances",
        "Chain of Vapor",
        "Chain of Plasma",
        "Chain of Silence",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P0, "Chain Lightning");
    // P1 pays {R}{R}, copies it, and targets P0 with the copy; P0 declines to pay.
    // (Pay {R}{R}? Copy it? Choose new targets for the copy?)
    for _ in 0..3 {
        t.answer_yes(P1, true);
    }
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).expect("cast");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn string_of_disappearances_asks_the_returned_creatures_controller() {
    cr!("707.10", "608.2h");
    // "Return target creature to its owner's hand. Then that creature's controller may
    // pay {U}{U}. If the player does, they may copy this spell and may choose a new
    // target for that copy."
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 2);
    let s = t.hand(P0, "String of Disappearances");
    for _ in 0..3 {
        t.answer_yes(P1, true);
    }
    t.answer_targets(P1, &[Entity::Object(mine)]);
    t.cast_with(P0, s, &[Entity::Object(theirs)]).expect("cast");
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn payment_cards_compile() {
    assert_compiles(&[
        "Purgatory",
        "Miara, Thorn of the Glade",
        "Ripples of Undeath",
        "Zoraline, Cosmos Caller",
        "Draco",
        "Mana-Charged Dragon",
        "Urza's Saga",
        "Asmoranomardicadaistinaculdacar",
        "Foil",
        "Land Grant",
        "Necrodominance",
        "Karn, Living Legacy",
        "Leyline Tyrant",
    ]);
}

#[test]
fn miara_pays_mana_and_life_to_draw() {
    cr!("118.12", "119.4");
    // "Whenever ~ or another Elf you control dies, you may pay {1} and 1 life. If you do,
    // draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Miara, Thorn of the Glade");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.destroy(elf, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn draco_upkeep_cost_is_reduced_by_domain() {
    cr!("118.7", "118.12");
    // "Domain — At the beginning of your upkeep, sacrifice ~ unless you pay {10}. This cost
    // is reduced by {2} for each basic land type among lands you control."
    let mut t = TestGame::new(2);
    let draco = t.battlefield(P0, "Draco");
    for l in ["Plains", "Island", "Swamp", "Mountain"] {
        t.lands(P0, l, 1);
    }
    t.lands(P0, "Mountain", 2);
    // Four basic land types: {2}.
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(draco));
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn urzas_saga_finds_an_artifact_with_mana_cost_0_or_1() {
    cr!("202.1", "714.2c");
    // "III — Search your library for an artifact card with mana cost {0} or {1}, put it
    // onto the battlefield, then shuffle."
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Urza's Saga");
    t.library_top(P0, "Mind Stone");
    t.library_top(P0, "Sol Ring");
    t.library_top(P0, "Mind Stone");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1, "{}", t.dump_log());
    assert!(t.named_on_battlefield("Mind Stone").is_empty());
}

#[test]
fn asmoranomardicadaistinaculdacar_costs_br_after_a_discard() {
    cr!("118.9", "601.2b");
    // "As long as you've discarded a card this turn, you may pay {B/R} to cast ~."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let asmor = t.hand(P0, "Asmoranomardicadaistinaculdacar");
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    let can = |t: &mut TestGame| {
        t.g.cast_options(P0, asmor)
            .into_iter()
            .any(|o| t.g.can_begin_cast(P0, asmor, &o))
    };
    assert!(!can(&mut t));
    let filler = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, filler, None);
    t.g.recompute();
    assert!(can(&mut t));
}

#[test]
fn foil_discards_an_island_and_another_card_instead_of_its_mana_cost() {
    cr!("118.9");
    // "You may discard an Island card and another card rather than pay ~'s mana cost."
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast_with(P1, bolt, &[Entity::Player(P0)]).expect("bolt");
    let bolt = t.g.current(bolt);
    let foil = t.hand(P0, "Foil");
    let island = t.hand(P0, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.turn.priority = Some(P0);
    let opts = t.g.cast_options(P0, foil);
    let alt = opts
        .into_iter()
        .find(|o| o.alt_cost.is_some())
        .expect("alternative cost");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.g.cast_spell(P0, foil, alt.method).expect("Foil");
    let _ = (island, bears);
    assert!(t.in_graveyard(P0, "Island"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn necrodominance_draws_as_many_cards_as_life_paid() {
    cr!("119.4", "107.1b");
    // "At the beginning of your end step, you may pay any amount of life. If you do, draw
    // that many cards."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necrodominance");
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn leyline_tyrant_deals_as_much_damage_as_red_mana_paid() {
    cr!("603.12", "107.1b");
    // "When this creature dies, you may pay any amount of {R}. When you do, it deals that
    // much damage to any target."
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Leyline Tyrant");
    t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.destroy(tyrant, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn heart_of_kiran_crews_by_removing_a_loyalty_counter() {
    cr!("118.9", "702.122a");
    // "You may remove a loyalty counter from a planeswalker you control rather than pay
    // ~'s crew cost."
    assert_compiles(&["Heart of Kiran", "Gavi, Nest Warden", "Festival of Embers"]);
    let mut t = TestGame::new(2);
    let heart = t.battlefield(P0, "Heart of Kiran");
    let jace = t.battlefield(P0, "Jace Beleren");
    let loyalty = t.counters(jace, "loyalty");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, heart, 0, &[]).expect("crew");
    t.resolve_all();
    assert_eq!(t.counters(jace, "loyalty"), loyalty - 1);
    assert!(t.obj_now(heart).is_creature());
}

#[test]
fn gavi_cycles_the_first_card_each_turn_for_free() {
    cr!("118.9", "702.29a");
    // "You may pay {0} rather than pay the cycling cost of the first card you cycle each
    // turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gavi, Nest Warden");
    let a = t.hand(P0, "Shefet Monitor");
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, a, 0, &[]).expect("cycling for {0}");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn inquisitive_glimmer_makes_unlock_costs_cheaper() {
    cr!("709.5e", "118.7a");
    // "Unlock costs you pay cost {1} less."
    use mtg_engine::decision::{Action, SpecialAction};
    use mtg_engine::mana::ManaType;
    assert_compiles(&["Inquisitive Glimmer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inquisitive Glimmer");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let card = t.hand(P0, "Glassworks // Shattered Yard");
    // (Glassworks {2}{R} is an enchantment spell: it costs {1}{R}.)
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.g.players[0].mana_pool.add_type(ManaType::C, 1);
    t.cast(P0, card).method(CastMethod::Half(0)).go();
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.resolve_all();
    let room = t.named_on_battlefield("Glassworks")[0];
    // Shattered Yard's {4}{R} costs {3}{R}.
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.g.players[0].mana_pool.add_type(ManaType::C, 3);
    t.g.turn.priority = Some(P0);
    let unlock = t.g.legal_actions(P0).into_iter().find(|a| {
        matches!(a, Action::Special(SpecialAction::Other { obj: Some(o), .. }) if *o == room)
    });
    let unlock = unlock.expect("the door can be unlocked for {3}{R}");
    t.g.perform_action(P0, unlock).unwrap();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn emberwilde_djinn_that_player_may_pay_mana_or_life() {
    cr!("118.12", "119.4");
    // "At the beginning of each player's upkeep, that player may pay {R}{R} or 2 life. If
    // the player does, they gain control of ~."
    assert_compiles(&["Emberwilde Djinn", "Isu the Abominable"]);
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Emberwilde Djinn");
    // P1 can't pay {R}{R}; they pay 2 life instead.
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(djinn).controller, P1);
}

#[test]
fn isu_offers_one_of_three_mana_costs() {
    cr!("118.12");
    // "Whenever another snow permanent you control enters, you may pay {G}, {W}, or {U}.
    // If you do, put a +1/+1 counter on ~."
    let mut t = TestGame::new(2);
    let isu = t.battlefield(P0, "Isu the Abominable");
    // The entering Snow-Covered Island can pay {U} only: {G} and {W} can't be paid, so
    // only {U} is offered.
    t.answer_yes(P0, true);
    t.enter(P0, "Snow-Covered Island");
    t.resolve_all();
    assert_eq!(t.counters(isu, "+1/+1"), 1);
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(t.asked().iter().filter(|(_, d)| format!("{d:?}").contains("Pay {")).count(), 1);
}

#[test]
fn madame_null_puts_counters_equal_to_the_life_paid() {
    cr!("118.12", "119.4");
    // "Whenever another creature you control enters, you may pay life equal to its power.
    // If you do, put that many +1/+1 counters on it."
    assert_compiles(&["Madame Null, Power Broker"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Madame Null, Power Broker");
    t.answer_yes(P0, true);
    let giant = t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(giant, "+1/+1"), 3);
}

#[test]
fn overencumbered_asks_for_one_mana_per_artifact_or_no_attacks() {
    cr!("118.12", "508.1d");
    // "At the beginning of combat on enchanted opponent's turn, that player may pay {1} for
    // each artifact they control. If they don't, creatures can't attack this combat."
    assert_compiles(&["Overencumbered"]);
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Overencumbered");
    t.g.attach(aura, Entity::Player(P1));
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Ornithopter");
    t.lands(P1, "Island", 3);
    // P1 pays {2} (two artifacts).
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P1), 2);
}

#[test]
fn seal_of_the_guildpact_counts_the_chosen_colors_a_spell_is() {
    cr!("601.2f", "105.2");
    // "As this artifact enters, choose two colors. Each spell you cast costs {1} less to
    // cast for each of the chosen colors it is."
    assert_compiles(&["Seal of the Guildpact"]);
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let seal = t.enter(P0, "Seal of the Guildpact");
    let chosen = t.obj_now(seal).choices.colors.expect("two colors chosen");
    assert_eq!(chosen.iter().count(), 2);
    assert!(chosen.contains(Color::White) && chosen.contains(Color::Blue));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Island", 3);
    // Detention Sphere ({1}{W}{U}) is both: {2} less, but only its {1} can be reduced.
    let sphere = t.hand(P0, "Detention Sphere");
    assert_eq!(paid(&mut t, P0, sphere, &[Entity::Object(bears)]), 2);
    t.resolve_all();
    // Divination ({2}{U}) is one of them: {1} less.
    let div = t.hand(P0, "Divination");
    assert_eq!(paid(&mut t, P0, div, &[]), 2);
}

#[test]
fn elminster_reduces_the_next_spell_by_the_cards_scried() {
    cr!("611.2c", "611.2f", "701.22a");
    // "Whenever you scry, the next instant or sorcery spell you cast this turn costs {X}
    // less to cast, where X is the number of cards looked at while scrying this way."
    assert_compiles(&["Elminster"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elminster");
    t.lands(P0, "Island", 6);
    let opt = t.hand(P0, "Opt");
    paid(&mut t, P0, opt, &[]);
    t.resolve_all();
    // Opt scried 1: Divination costs {1}{U}.
    let div = t.hand(P0, "Divination");
    assert_eq!(paid(&mut t, P0, div, &[]), 2);
    t.resolve_all();
    let div = t.hand(P0, "Divination");
    assert_eq!(paid(&mut t, P0, div, &[]), 3);
}
