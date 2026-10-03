//! Value grammar II, results of earlier instructions (CR 608.2c, 608.2h): "the number of
//! creatures you controlled that were destroyed this way", "the total power of the
//! creatures sacrificed this way", "the mana value of the permanent exiled this way",
//! "for each spell countered this way", "that creature's power" after a sacrifice, "that
//! card's mana value" after a discard, "the greatest number of cards a player discarded
//! this way", "for each 1 life lost this way".

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

fn count_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    t.g.permanents_controlled_by(p)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype(sub))
        .count()
}

#[test]
fn kaya_s_wrath_counts_creatures_you_controlled_as_they_last_existed() {
    cr!("608.2h", "400.7");
    assert_supported("Kaya's Wrath");
    ruling!(
        "Kaya's Wrath",
        "If a creature you control has indestructible, it isn't destroyed this way"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    // An opponent's creature you control counts (it goes to its owner's graveyard).
    let stolen = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(stolen).controller = P0;
    t.g.obj_mut(stolen).base_controller = P0;
    // An indestructible creature isn't destroyed and doesn't count.
    t.battlefield(P0, "Darksteel Myr");
    t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 2);
    let kw = t.hand(P0, "Kaya's Wrath");
    t.cast(P0, kw).go();
    t.resolve();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn phyrexian_rebirth_counts_only_creatures_actually_destroyed() {
    cr!("608.2h");
    assert_supported("Phyrexian Rebirth");
    ruling!(
        "Phyrexian Rebirth",
        "If a creature regenerates or has indestructible, it won't be counted"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Darksteel Myr");
    t.lands(P0, "Plains", 6);
    let pr = t.hand(P0, "Phyrexian Rebirth");
    t.cast(P0, pr).go();
    t.resolve();
    let horror =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .find(|o| t.g.obj(*o).chars.has_subtype("Horror"))
            .expect("Horror token");
    assert_eq!(t.pt(horror), (2, 2));
}

#[test]
fn deadly_tempest_each_player_loses_life_for_their_destroyed_creatures() {
    cr!("608.2h");
    assert_supported("Deadly Tempest");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    let dt = t.hand(P0, "Deadly Tempest");
    t.cast(P0, dt).go();
    t.resolve();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn ruinous_intrusion_uses_the_exiled_permanents_last_known_mana_value() {
    cr!("608.2h", "400.7");
    assert_supported("Ruinous Intrusion");
    ruling!(
        "Ruinous Intrusion",
        "Use the permanent's characteristics as it last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Ornithopter");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let enchantment = t.battlefield(P1, "Glorious Anthem");
    let _ = target;
    t.lands(P0, "Forest", 4);
    let ri = t.hand(P0, "Ruinous Intrusion");
    t.cast(P0, ri).target(enchantment).target(mine).go();
    t.resolve();
    // Glorious Anthem has mana value 3.
    assert_eq!(t.counters(mine, "+1/+1"), 3);
}

#[test]
fn swift_silence_draws_for_each_spell_countered() {
    cr!("608.2c", "701.6a");
    assert_supported("Swift Silence");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 2);
    let b1 = t.hand(P1, "Lightning Bolt");
    let b2 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b1).target(P0).go();
    t.cast(P1, b2).target(P0).go();
    t.lands(P0, "Island", 2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Forest", 2);
    let ss = t.hand(P0, "Swift Silence");
    let before = t.hand_size(P0);
    t.cast(P0, ss).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), before - 1 + 2);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn syphon_flesh_creates_a_zombie_for_each_creature_sacrificed() {
    cr!("701.21a", "608.2c");
    assert_supported("Syphon Flesh");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    let sf = t.hand(P0, "Syphon Flesh");
    t.cast(P0, sf).go();
    t.resolve();
    assert_eq!(count_subtype(&t, P0, "Zombie"), 2);
}

#[test]
fn reign_of_the_pit_uses_the_total_power_of_the_sacrificed_creatures() {
    cr!("608.2h", "701.21a");
    assert_supported("Reign of the Pit");
    ruling!(
        "Reign of the Pit",
        "Use the powers of the creatures as they last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 6);
    let rp = t.hand(P0, "Reign of the Pit");
    t.cast(P0, rp).go();
    t.resolve();
    let demon =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .find(|o| t.g.obj(*o).chars.has_subtype("Demon"))
            .expect("Demon token");
    assert_eq!(t.pt(demon), (5, 5));
}

#[test]
fn windfall_draws_the_greatest_number_discarded_by_one_player() {
    cr!("608.2c", "101.4");
    assert_supported("Windfall");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.hand(P0, "Grizzly Bears");
    }
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    t.lands(P0, "Island", 3);
    let wf = t.hand(P0, "Windfall");
    t.cast(P0, wf).go();
    t.resolve();
    // Both draw four: the most cards one player discarded.
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.hand_size(P1), 4);
}

#[test]
fn hellhole_rats_deals_the_discarded_cards_mana_value() {
    cr!("400.7j", "608.2h");
    assert_supported("Hellhole Rats");
    let mut t = TestGame::new(2);
    t.hand(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    let rats = t.hand(P0, "Hellhole Rats");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, rats).go();
    t.resolve_all();
    // Hill Giant has mana value 4.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn doomgape_gains_the_sacrificed_creatures_toughness() {
    cr!("608.2h", "701.21a");
    assert_supported("Doomgape");
    let mut t = TestGame::new(2);
    let dg = t.battlefield(P0, "Doomgape");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(dg));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn shadowheart_draws_cards_equal_to_the_sacrificed_creatures_power() {
    cr!("602.2", "608.2h");
    assert_supported("Shadowheart, Dark Justiciar");
    let mut t = TestGame::new(2);
    let sh = t.battlefield(P0, "Shadowheart, Dark Justiciar");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let before = t.hand_size(P0);
    t.activate(P0, sh, 0, &[]).expect("activate");
    t.resolve();
    assert_eq!(t.hand_size(P0), before + 3);
}

#[test]
fn fiery_bombardment_counts_red_symbols_of_the_sacrificed_creature() {
    cr!("700.5", "608.2h");
    assert_supported("Fiery Bombardment");
    ruling!(
        "Fiery Bombardment",
        "Chroma abilities count hybrid mana symbols of the appropriate color"
    );
    let mut t = TestGame::new(2);
    let fb = t.battlefield(P0, "Fiery Bombardment");
    // {1}{R}{R}.
    let c = t.battlefield(P0, "Hellhole Rats");
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.activate(P0, fb, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn occult_epiphany_creates_a_spirit_for_each_card_type_discarded() {
    cr!("205.2a", "608.2h");
    assert_supported("Occult Epiphany");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let island = t.hand(P0, "Island");
    t.hand(P0, "Grizzly Bears");
    let oe = t.hand(P0, "Occult Epiphany");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(island)]);
    t.cast(P0, oe).x(2).go();
    t.resolve();
    // A creature card and a land card discarded: two Spirits.
    assert_eq!(count_subtype(&t, P0, "Spirit"), 2);
}

#[test]
fn liliana_s_indignation_counts_creature_cards_milled() {
    cr!("701.17a", "608.2h");
    assert_supported("Liliana's Indignation");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let li = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, li).x(3).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn paradoxical_outcome_draws_only_for_cards_you_own() {
    cr!("400.7", "608.2h");
    assert_supported("Paradoxical Outcome");
    ruling!(
        "Paradoxical Outcome",
        "If you control but don't own some of the target permanents, they won't count"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(b).controller = P0;
    t.g.obj_mut(b).base_controller = P0;
    t.g.recompute();
    t.lands(P0, "Island", 4);
    let po = t.hand(P0, "Paradoxical Outcome");
    let before = t.hand_size(P0);
    t.cast(P0, po)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    // The returned Bears of yours, plus one card drawn for it.
    assert_eq!(t.hand_size(P0), before - 1 + 1 + 1);
}

#[test]
fn blood_tyrant_counts_all_the_life_lost_this_way() {
    cr!("119.3", "608.2c");
    assert_supported("Blood Tyrant");
    ruling!(
        "Blood Tyrant",
        "Blood Tyrant will count each player’s life loss, including yours"
    );
    let mut t = TestGame::new(3);
    let bt = t.battlefield(P0, "Blood Tyrant");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bt, "+1/+1"), 3);
    assert_eq!(t.life(P0), 19);
}
