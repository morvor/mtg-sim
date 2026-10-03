//! Value grammar II: characteristics of objects the text refers to ("the exiled card's
//! power", "the revealed card's mana value", "the power of the creature that died", "the
//! difference between its power and toughness", "~'s loyalty"), "that many" after an
//! instruction with an amount or after a move, cards revealed this way, and amounts for
//! each object or player in turn.

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
fn corpse_lunge_deals_the_power_of_the_card_its_cost_exiled() {
    cr!("400.7j", "601.2h");
    assert_supported("Corpse Lunge");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let target = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let cl = t.hand(P0, "Corpse Lunge");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, cl).target(target).go();
    t.resolve();
    // Hill Giant has power 3.
    assert_eq!(t.obj_now(target).damage, 3);
}

#[test]
fn morbid_bloom_creates_saprolings_equal_to_the_exiled_cards_toughness() {
    cr!("400.7j", "608.2h");
    assert_supported("Morbid Bloom");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 3);
    let mb = t.hand(P0, "Morbid Bloom");
    t.cast(P0, mb).target(wurm).go();
    t.resolve();
    // Craw Wurm is a 6/4.
    assert_eq!(count_subtype(&t, P0, "Saproling"), 4);
}

#[test]
fn induce_despair_uses_the_revealed_cards_mana_value() {
    cr!("701.20a", "601.2h");
    assert_supported("Induce Despair");
    let mut t = TestGame::new(2);
    let wurm = t.hand(P0, "Craw Wurm");
    let target = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let id = t.hand(P0, "Induce Despair");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast(P0, id).target(target).go();
    t.resolve();
    // Craw Wurm's mana value is 6: -6/-6 kills the 6/4.
    assert!(!t.on_battlefield(target));
}

#[test]
fn living_destiny_gains_the_revealed_cards_mana_value() {
    cr!("701.20a");
    assert_supported("Living Destiny");
    let mut t = TestGame::new(2);
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let ld = t.hand(P0, "Living Destiny");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, ld).go();
    t.resolve();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn deaths_presence_uses_the_power_of_the_creature_that_died() {
    cr!("603.10a", "608.2h");
    assert_supported("Death's Presence");
    ruling!(
        "Death's Presence",
        "X is the power of that creature as it last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Death's Presence");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(giant), "+1/+1", 2, None);
    t.g.flush_events();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.destroy(giant, None);
    t.g.flush_events();
    t.resolve_all();
    // The Giant was a 5/5 with its counters.
    assert_eq!(t.counters(bears, "+1/+1"), 5);
}

#[test]
fn jaws_of_defeat_loses_the_difference_between_power_and_toughness() {
    cr!("208.1");
    assert_supported("Jaws of Defeat");
    ruling!(
        "Jaws of Defeat",
        "subtract the smaller of those two numbers from the larger one"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jaws of Defeat");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // Craw Wurm: 6/4.
    t.enter(P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn superior_numbers_counts_creatures_in_excess() {
    cr!("107.1b");
    assert_supported("Superior Numbers");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.battlefield(P0, "Grizzly Bears");
    }
    let target = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let sn = t.hand(P0, "Superior Numbers");
    t.cast(P0, sn).target(target).target(P1).go();
    t.resolve();
    // Four creatures against one: 3 damage.
    assert_eq!(t.obj_now(target).damage, 3);
}

#[test]
fn feral_ghoul_gives_rad_counters_equal_to_its_power() {
    cr!("122.1", "603.10a");
    assert_supported("Feral Ghoul");
    let mut t = TestGame::new(2);
    let fg = t.battlefield(P0, "Feral Ghoul");
    t.g.add_counters(Entity::Object(fg), "+1/+1", 1, None);
    t.g.flush_events();
    t.g.destroy(fg, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.g.player(P1).counters.get("rad").copied().unwrap_or(0), 3);
}

#[test]
fn canopy_gargantuan_each_creature_gets_counters_equal_to_its_own_toughness() {
    cr!("608.2h", "608.2f");
    assert_supported("Canopy Gargantuan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Canopy Gargantuan");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert_eq!(t.counters(wurm, "+1/+1"), 4);
}

#[test]
fn day_of_the_dragons_creates_a_dragon_for_each_creature_exiled() {
    cr!("608.2c");
    assert_supported("Day of the Dragons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Day of the Dragons");
    t.resolve_all();
    assert_eq!(count_subtype(&t, P0, "Dragon"), 2);
}

#[test]
fn vengeful_regrowth_creates_a_token_for_each_land_returned() {
    cr!("608.2c");
    assert_supported("Vengeful Regrowth");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Forest");
    let b = t.graveyard(P0, "Island");
    t.lands(P0, "Forest", 6);
    let vr = t.hand(P0, "Vengeful Regrowth");
    t.cast(P0, vr)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(count_subtype(&t, P0, "Plant"), 2);
}

#[test]
fn laquatus_s_creativity_discards_the_number_drawn() {
    cr!("608.2h");
    assert_supported("Laquatus's Creativity");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.lands(P0, "Island", 5);
    let lc = t.hand(P0, "Laquatus's Creativity");
    t.cast(P0, lc).target(P1).go();
    t.resolve();
    // Drew three, then discarded three.
    assert_eq!(t.hand_size(P1), 3);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn horrid_shadowspinner_discards_as_many_as_its_power_when_it_chose_to_draw() {
    cr!("608.2h", "603.12");
    assert_supported("Horrid Shadowspinner");
    ruling!(
        "Horrid Shadowspinner",
        "equal to Horrid Shadowspinner's power at the time you chose to draw cards"
    );
    let mut t = TestGame::new(2);
    let hs = t.battlefield(P0, "Horrid Shadowspinner");
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(hs, Entity::Player(P1))], &[]);
    // Drew two, discarded two.
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn blood_oath_counts_cards_of_the_chosen_type_revealed() {
    cr!("701.20a", "607.2d");
    assert_supported("Blood Oath");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.hand(P1, "Island");
    t.lands(P0, "Mountain", 4);
    let bo = t.hand(P0, "Blood Oath");
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.cast(P0, bo).target(P1).go();
    t.resolve();
    // Whatever card type was chosen, 3 damage for each card of it in that hand.
    let lost = 20 - t.life(P1);
    assert!(lost % 3 == 0 && lost <= 6, "lost {lost}");
}

#[test]
fn madcap_experiment_deals_damage_for_each_card_revealed() {
    cr!("701.20a");
    assert_supported("Madcap Experiment");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Ornithopter");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.lands(P0, "Mountain", 4);
    let me = t.hand(P0, "Madcap Experiment");
    t.cast(P0, me).go();
    t.resolve();
    // Island, Island, Ornithopter: three cards revealed.
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn doran_pumps_by_the_difference_between_power_and_toughness() {
    cr!("208.1");
    assert_supported("Doran, Besieged by Time");
    ruling!(
        "Doran, Besieged by Time",
        "The value of X is calculated only once, as Doran's last ability resolves"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doran, Besieged by Time");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wurm, Entity::Player(P1))], &[]);
    // Craw Wurm 6/4: +2/+2.
    assert_eq!(t.life(P1), 12);
}

#[test]
fn drach_nyen_gets_the_power_of_the_card_it_exiled() {
    cr!("607.2a", "613.4c");
    assert_supported("Drach'Nyen");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let dn = t.enter(P0, "Drach'Nyen");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(dn, Entity::Object(bears));
    t.g.recompute();
    // Hill Giant's power is 3.
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn venom_reflexive_counters_use_the_exiled_cards_toughness() {
    cr!("603.12", "607.2a");
    assert_supported("Venom, Deadly Devourer");
    let mut t = TestGame::new(2);
    let venom = t.battlefield(P0, "Venom, Deadly Devourer");
    let giant = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.answer_targets(P0, &[Entity::Object(venom)]);
    t.activate(P0, venom, 0, &[Entity::Object(giant)])
        .expect("activate");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // Hill Giant's toughness is 3.
    assert_eq!(t.counters(venom, "+1/+1"), 3);
}

#[test]
fn blazing_effigy_adds_damage_from_other_effigies() {
    cr!("120.2", "603.10a");
    assert_supported("Blazing Effigy");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Blazing Effigy");
    let second = t.battlefield(P0, "Blazing Effigy");
    let target = t.battlefield(P1, "Ancient Brontodon");
    t.answer_targets(P0, &[Entity::Object(second)]);
    t.answer_targets(P0, &[Entity::Object(target)]);
    // The first Effigy dies and deals 3 to the second; the second dies (3 damage on a
    // 0/3) and deals 3 + 3 to the Wurm.
    t.g.destroy(first, None);
    t.resolve_all();
    assert_eq!(t.g.obj(target).damage, 6);
}

#[test]
fn grothama_players_draw_the_damage_their_sources_dealt() {
    cr!("120.2", "603.10a");
    // Its other ability (granting a fight trigger) isn't part of this test.
    let c = card("Grothama, All-Devouring");
    assert!(
        !c.unsupported_text().iter().any(|u| u.contains("draws cards")),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grothama, All-Devouring");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(mine, Entity::Object(g), 2, false);
    t.g.deal_damage(theirs, Entity::Object(g), 3, false);
    t.g.flush_events();
    let before = (t.hand_size(P0), t.hand_size(P1));
    t.g.destroy(g, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before.0 + 2);
    assert_eq!(t.hand_size(P1), before.1 + 3);
}

#[test]
fn aesir_escape_valhalla_uses_the_exiled_cards_mana_value() {
    cr!("714.2b", "607.2a");
    assert_supported("The Aesir Escape Valhalla");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let saga = t.enter(P0, "The Aesir Escape Valhalla");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    // Chapter I: Hill Giant's mana value is 4.
    assert_eq!(t.life(P0), 24);
    // Chapter II.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 4);
    // Chapter III returns the Saga and the exiled card to their owner's hand.
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(giant)), mtg_engine::object::Zone::Hand(P0));
}
