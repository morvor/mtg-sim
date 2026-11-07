//! Value grammar II: X fixed by a payment ("you may pay {X}. When you do, ...", "X can't
//! be greater than ..."), the amount of the triggering event ("that many", "the amount of
//! damage it dealt to that player", "the amount of life you gained"), excess damage dealt
//! this way, and amounts about the spell that caused a cast trigger ("the amount of mana
//! spent to cast that spell", "the number of colors of mana spent to cast it", "the number
//! of times that spell was kicked").

use mtg_engine::decision::Answer;
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

#[test]
fn wildborn_preserver_pays_x_then_its_reflexive_ability_puts_x_counters() {
    cr!("107.3f", "603.12");
    assert_supported("Wildborn Preserver");
    ruling!(
        "Wildborn Preserver",
        "the reflexive triggered ability triggers and will resolve separately"
    );
    let mut t = TestGame::new(2);
    let wp = t.battlefield(P0, "Wildborn Preserver");
    t.lands(P0, "Forest", 4);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(wp, "+1/+1"), 3);
}

#[test]
fn jugan_defends_the_temple_reflexive_counters_go_on_that_creature() {
    cr!("107.3f", "603.12");
    let c = card("Jugan Defends the Temple // Remnant of the Rising Star");
    // The back face's ability.
    let back = &c.faces[1];
    assert!(back.unsupported.is_empty(), "{:?}", back.unsupported);
    let mut t = TestGame::new(2);
    let def = CardDef::custom(back.chars.clone());
    let remnant = t.custom(P0, def, mtg_engine::object::Zone::Battlefield);
    let _ = remnant;
    t.lands(P0, "Forest", 2);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let bears = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(bears), "+1/+1"), 2);
}

#[test]
fn shanna_cant_choose_x_greater_than_the_life_gained_this_turn() {
    cr!("107.3f", "119.3");
    assert_supported("Shanna, Purifying Blade");
    ruling!(
        "Shanna, Purifying Blade",
        "You choose the value for X as the ability resolves"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shanna, Purifying Blade");
    t.g.gain_life(P0, 2);
    t.g.flush_events();
    t.lands(P0, "Plains", 5);
    t.set_step(P0, Step::PostcombatMain);
    t.answer_yes(P0, true);
    // Asking for 5 gets the most allowed: 2.
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let before = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 2);
}

#[test]
fn treebeard_puts_that_many_counters_on_its_target() {
    cr!("603.2c", "119.3");
    assert_supported("Treebeard, Gracious Host");
    let mut t = TestGame::new(2);
    let tb = t.battlefield(P0, "Treebeard, Gracious Host");
    t.answer_targets(P0, &[Entity::Object(tb)]);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(tb, "+1/+1"), 3);
}

#[test]
fn hurska_sweet_tooth_reflexive_pump_is_the_life_gained() {
    cr!("603.12", "119.3");
    assert_supported("Hurska Sweet-Tooth");
    ruling!(
        "Hurska Sweet-Tooth",
        "The value of X is calculated only once, as the reflexive triggered ability resolves"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hurska Sweet-Tooth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.gain_life(P0, 4);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn desmond_miles_surveils_the_damage_it_dealt() {
    cr!("510.2", "701.25a");
    assert_supported("Desmond Miles");
    let mut t = TestGame::new(2);
    let dm = t.battlefield(P0, "Desmond Miles");
    // An Assassin card in the graveyard: +1/+0.
    t.graveyard(P0, "Royal Assassin");
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.g.recompute();
    let p = t.pt(dm).0;
    assert!(p >= 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(dm, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - p);
    let surveilled: Vec<usize> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::Surveil { cards } => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert_eq!(surveilled, vec![p as usize]);
}

#[test]
fn lacerate_flesh_creates_blood_for_the_excess_damage() {
    cr!("120.10", "608.2h");
    assert_supported("Lacerate Flesh");
    ruling!(
        "Lacerate Flesh",
        "deals more damage to it than the minimum amount of damage required to be lethal damage"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let lf = t.hand(P0, "Lacerate Flesh");
    t.cast(P0, lf).target(bears).go();
    t.resolve();
    // 4 damage to a 2/2: 2 excess.
    let blood =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.has_subtype("Blood"))
            .count();
    assert_eq!(blood, 2);
}

#[test]
fn goblin_negotiation_counts_excess_damage_with_damage_already_marked() {
    cr!("120.10");
    assert_supported("Goblin Negotiation");
    ruling!(
        "Goblin Negotiation",
        "although damage already marked on the creature is taken into account"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let src = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(src, Entity::Object(bears), 1, false);
    t.g.flush_events();
    t.lands(P0, "Mountain", 6);
    let gn = t.hand(P0, "Goblin Negotiation");
    t.cast(P0, gn).x(4).target(bears).go();
    t.resolve();
    // 1 more damage would be lethal: 3 excess.
    let goblins =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.has_subtype("Goblin"))
            .count();
    assert_eq!(goblins, 3);
}

#[test]
fn manaform_hellkite_token_is_the_mana_spent_on_that_spell() {
    cr!("601.2h", "608.2h");
    assert_supported("Manaform Hellkite");
    ruling!(
        "Manaform Hellkite",
        "looks at the mana that was actually spent to cast the spell"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manaform Hellkite");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    let dragon =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .find(|o| t.g.obj(*o).chars.has_subtype("Illusion"))
            .expect("Dragon Illusion token");
    assert_eq!(t.pt(dragon), (1, 1));
}

#[test]
fn wildgrowth_archaic_counts_colors_spent_on_the_creature_spell() {
    cr!("601.2h", "614.1c");
    assert_supported("Wildgrowth Archaic");
    ruling!(
        "Wildgrowth Archaic",
        "The maximum number of colors of mana you can spend to cast a spell is five"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wildgrowth Archaic");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    // Grizzly Bears ({1}{G}) paid with green and white mana.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(b, "+1/+1"), 2);
}

#[test]
fn rumbling_aftershocks_deals_damage_for_each_time_kicked() {
    cr!("702.33", "608.2h");
    assert_supported("Rumbling Aftershocks");
    ruling!(
        "Rumbling Aftershocks",
        "The ability triggers just once per kicked spell"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rumbling Aftershocks");
    t.lands(P0, "Mountain", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, burst).target(P1).kicked(true).go();
    t.resolve_all();
    // Kicked once: 1 damage from the Aftershocks, 4 from Burst Lightning.
    assert_eq!(t.life(P1), 15);
}

#[test]
fn well_of_lost_dreams_x_is_at_most_the_life_gained() {
    cr!("107.3f", "119.3");
    assert_supported("Well of Lost Dreams");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Well of Lost Dreams");
    t.lands(P0, "Plains", 6);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    t.answer_yes(P0, true);
    // Asking for 5 gets the most allowed: the 3 life gained.
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let before = t.hand_size(P0);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 3);
    // Three lands were tapped to pay {3}.
    let tapped = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).tapped)
        .count();
    assert_eq!(tapped, 3);
}
