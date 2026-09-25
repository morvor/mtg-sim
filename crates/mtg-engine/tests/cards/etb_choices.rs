//! Enters-the-battlefield replacement effects (CR 614.1c–d, 614.12) and "as this enters,
//! choose ..." abilities referred to as "the chosen [value]" (CR 607.2d).

use mtg_engine::ability::{AbilityKind, ReplacementAction, StaticEffect};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{subtype_lists, Color};
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn color_index(c: Color) -> usize {
    Color::ALL.iter().position(|x| *x == c).unwrap()
}

/// Queues the answer to a "choose a color" prompt.
fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    t.answer(p, DecisionKind::Option, Answer::Index(color_index(c)));
}

/// Queues the answer to a "choose a creature type" prompt.
fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .expect("creature type");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

// ---------------------------------------------------------------------------
// "As this land enters, you may pay 2 life. If you don't, it enters tapped."
// ---------------------------------------------------------------------------

#[test]
fn shock_lands_compile() {
    for n in [
        "Stomping Ground",
        "Breeding Pool",
        "Watery Grave",
        "Godless Shrine",
    ] {
        assert_supported(n);
    }
}

#[test]
fn shock_land_paying_life_enters_untapped() {
    cr!("614.1c", "614.12a", "119.4");
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Stomping Ground");
    t.answer_yes(P0, true);
    t.play_land(P0, land).unwrap();
    let now = t.g.current(land);
    assert!(t.on_battlefield(land));
    assert!(!t.obj_now(now).tapped);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn shock_land_not_paying_enters_tapped() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    let land = t.hand(P0, "Stomping Ground");
    t.answer_yes(P0, false);
    t.play_land(P0, land).unwrap();
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn shock_land_cant_pay_more_life_than_you_have() {
    cr!("119.4", "614.12a");
    let mut t = TestGame::new(2);
    t.g.players[0].life = 1;
    let land = t.hand(P0, "Watery Grave");
    t.answer_yes(P0, true);
    t.play_land(P0, land).unwrap();
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.life(P0), 1);
}

#[test]
fn shock_land_put_onto_battlefield_tapped_stays_tapped() {
    cr!("614.1c");
    ruling!(
        "Stomping Ground",
        "you may pay 2 life, but it still enters tapped"
    );
    let mut t = TestGame::new(2);
    // An effect puts the land onto the battlefield tapped; paying life doesn't untap it.
    let id =
        t.g.create_card_object(card("Stomping Ground"), P0, object::Zone::Nowhere);
    t.answer_yes(P0, true);
    let new =
        t.g.move_object_ev(replacement::MoveEv {
            obj: id,
            to: object::Zone::Battlefield,
            pos: ability::LibraryPosition::Top,
            cause: events::MoveCause::Effect,
            by: Some(P0),
            etb: replacement::EtbInfo {
                controller: Some(P0),
                tapped: true,
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    assert!(t.g.obj(new).tapped);
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------
// "enters tapped unless [condition]"
// ---------------------------------------------------------------------------

#[test]
fn check_and_fast_lands_compile() {
    for n in [
        "Glacial Fortress",
        "Sunpetal Grove",
        "Concealed Courtyard",
        "Inspiring Vantage",
        "Rockfall Vale",
        "Sodden Verdure",
        "Theorix Annex",
        "Agna Qel'a",
    ] {
        assert_supported(n);
    }
}

#[test]
fn check_land_enters_untapped_with_matching_land_type() {
    cr!("614.1d", "614.12");
    ruling!("Glacial Fortress", "not for lands named Plains or Island");
    let mut t = TestGame::new(2);
    // Hallowed Fountain has the land types Plains and Island.
    t.battlefield(P0, "Hallowed Fountain");
    let gf = t.hand(P0, "Glacial Fortress");
    t.play_land(P0, gf).unwrap();
    assert!(!t.obj_now(gf).tapped);
}

#[test]
fn check_land_enters_tapped_without_matching_land_type() {
    cr!("614.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mountain");
    // An opponent's Island doesn't count.
    t.battlefield(P1, "Island");
    let gf = t.hand(P0, "Glacial Fortress");
    t.play_land(P0, gf).unwrap();
    assert!(t.obj_now(gf).tapped);
}

#[test]
fn fast_land_counts_other_lands() {
    cr!("614.1d", "614.12");
    ruling!(
        "Concealed Courtyard",
        "If you control three or more other lands, however, it enters the battlefield tapped"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let a = t.hand(P0, "Concealed Courtyard");
    t.play_land(P0, a).unwrap();
    assert!(!t.obj_now(a).tapped, "third land enters untapped");
    let b = t.hand(P0, "Concealed Courtyard");
    t.g.players[0].lands_played_this_turn = 0;
    t.play_land(P0, b).unwrap();
    assert!(t.obj_now(b).tapped, "fourth land enters tapped");
}

#[test]
fn basic_land_check_counts_basic_lands() {
    cr!("614.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let a = t.hand(P0, "Sodden Verdure");
    t.play_land(P0, a).unwrap();
    assert!(t.obj_now(a).tapped);
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let a = t.hand(P0, "Sodden Verdure");
    t.play_land(P0, a).unwrap();
    assert!(!t.obj_now(a).tapped);
}

// ---------------------------------------------------------------------------
// "If this creature was kicked, it enters with N +1/+1 counters on it."
// ---------------------------------------------------------------------------

#[test]
fn kicked_creature_enters_with_counters() {
    cr!("614.1c", "702.33d", "122.6");
    assert_supported("Aether Figment");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let f = t.hand(P0, "Aether Figment");
    t.cast(P0, f).kicked(true).go();
    t.resolve();
    assert_eq!(t.counters(f, "+1/+1"), 2);
    assert_eq!(t.pt(f), (3, 3));
}

#[test]
fn unkicked_creature_enters_without_counters() {
    cr!("614.1c");
    ruling!(
        "Aether Figment",
        "If you put a permanent with a kicker ability onto the battlefield without casting it"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let f = t.hand(P0, "Aether Figment");
    t.cast(P0, f).kicked(false).go();
    t.resolve();
    assert_eq!(t.counters(f, "+1/+1"), 0);
    // Put onto the battlefield without being cast: not kicked.
    let g = t.enter(P0, "Aether Figment");
    assert_eq!(t.counters(g, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// Conditional and variable "enters with counters"
// ---------------------------------------------------------------------------

#[test]
fn morbid_enters_with_counters_only_if_a_creature_died() {
    cr!("614.1c", "122.6");
    assert_supported("Gravetiller Wurm");
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "Gravetiller Wurm");
    assert_eq!(t.counters(w, "+1/+1"), 0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    let w2 = t.enter(P0, "Gravetiller Wurm");
    assert_eq!(t.counters(w2, "+1/+1"), 4);
}

#[test]
fn enters_with_counters_if_cast_from_hand() {
    cr!("614.1c", "122.6");
    assert_supported("Patched Plaything");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let p = t.hand(P0, "Patched Plaything");
    t.cast(P0, p).go();
    t.resolve();
    assert_eq!(t.counters(p, "-1/-1"), 2);
    assert_eq!(t.pt(p), (2, 1));
    // Not cast: no counters.
    let q = t.enter(P0, "Patched Plaything");
    assert_eq!(t.counters(q, "-1/-1"), 0);
}

#[test]
fn converge_counts_colors_spent() {
    cr!("614.1c", "207.2c", "122.6");
    ruling!("Glinting Creeper", "Colorless is not a color");
    assert_supported("Glinting Creeper");
    let mut t = TestGame::new(2);
    for l in ["Forest", "Forest", "Mountain", "Island", "Plains"] {
        t.battlefield(P0, l);
    }
    let c = t.hand(P0, "Glinting Creeper");
    t.cast(P0, c).go();
    t.resolve();
    // Green, red, blue and white were spent: two counters for each.
    assert_eq!(t.counters(c, "+1/+1"), 8);
}

#[test]
fn choice_of_keyword_counter() {
    cr!("614.1c", "122.1b");
    assert_supported("Flycatcher Giraffid");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let g = t.enter(P0, "Flycatcher Giraffid");
    assert_eq!(t.counters(g, "vigilance"), 1);
    assert_eq!(t.counters(g, "reach"), 0);
    assert!(t.obj_now(g).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn enters_tapped_with_charge_counters() {
    cr!("614.1c", "614.1d");
    assert_supported("Vivid Marsh");
    let mut t = TestGame::new(2);
    let v = t.hand(P0, "Vivid Marsh");
    t.play_land(P0, v).unwrap();
    assert!(t.obj_now(v).tapped);
    assert_eq!(t.counters(v, "charge"), 2);
}

// ---------------------------------------------------------------------------
// "As this enters, choose a color" and "the chosen color"
// ---------------------------------------------------------------------------

#[test]
fn chosen_color_protection() {
    cr!("614.1c", "614.12a", "607.2d", "702.16b");
    assert_supported("Voice of All");
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Red);
    let v = t.enter(P0, "Voice of All");
    assert_eq!(t.obj_now(v).choices.color, Some(Color::Red));
    // Protection from red: Lightning Bolt can't target it.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    assert!(t.g.protected_from(v, bolt));
    let red_goblin = t.battlefield(P1, "Raging Goblin");
    let white_knight = t.battlefield(P1, "Elite Vanguard");
    assert!(t.g.protected_from(v, red_goblin));
    assert!(!t.g.protected_from(v, white_knight));
    let spell = t.cast(P1, bolt).target(v).go();
    let chosen = t.g.obj(spell).stack.as_ref().unwrap().chosen[0]
        .targets
        .clone();
    assert!(!chosen.iter().flatten().any(|e| *e == Entity::Object(v)));
}

#[test]
fn chosen_color_is_chosen_as_a_token_copy_is_created() {
    cr!("614.12", "607.2d", "707.2");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Voice of All");
    // A token copy of Voice of All chooses its own color as it's created.
    choose_color(&mut t, P0, Color::Green);
    let spec = mtg_engine::replacement::TokenCreate {
        chars: t.g.obj(v).copiable.clone(),
        card: t.g.obj(v).card.clone(),
        tapped: false,
        attacking: None,
        copy_of: Some(v),
        copy_exceptions: vec![],
    };
    let toks = t.g.create_tokens(P0, spec, 1, None);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.g.obj(toks[0]).choices.color, Some(Color::Green));
    assert_eq!(t.g.obj(v).choices.color, None);
}

#[test]
fn mana_of_the_chosen_color() {
    cr!("614.1c", "607.2d", "106.1");
    assert_supported("Coldsteel Heart");
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Blue);
    let h = t.enter(P0, "Coldsteel Heart");
    assert!(t.obj_now(h).tapped, "enters tapped");
    let now = t.g.current(h);
    t.g.objects[now.0 as usize].tapped = false;
    t.activate(P0, now, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::U), 1);
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
}

#[test]
fn color_other_than_excludes_that_color() {
    cr!("614.1c", "614.12a", "607.2d");
    assert_supported("Cliffgate");
    let mut t = TestGame::new(2);
    // Options exclude red: white, blue, black, green. Index 3 is green.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let c = t.hand(P0, "Cliffgate");
    t.play_land(P0, c).unwrap();
    let now = t.g.current(c);
    assert!(t.obj_now(c).tapped);
    assert_eq!(t.obj_now(c).choices.color, Some(Color::Green));
    let asked = t.asked();
    let opts = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .unwrap();
    assert!(!opts.iter().any(|o| o == "red"));
    // "{T}: Add {R} or one mana of the chosen color."
    t.g.objects[now.0 as usize].tapped = false;
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, now, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.count(ManaType::G), 1);
}

#[test]
fn chosen_color_cda() {
    cr!("607.2d", "604.3");
    assert_supported("Chameleon Spirit");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Raging Goblin");
    choose_color(&mut t, P0, Color::Green);
    let s = t.enter(P0, "Chameleon Spirit");
    // Two green permanents the opponent controls.
    assert_eq!(t.pt(s), (2, 2));
}

#[test]
fn chosen_color_is_undefined_off_the_battlefield() {
    cr!("607.5a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Chameleon Spirit");
    // In hand no color was chosen: the CDA counts nothing.
    assert_eq!(t.pt(s), (0, 0));
}

// ---------------------------------------------------------------------------
// "As this enters, choose a creature type" and "the chosen type"
// ---------------------------------------------------------------------------

#[test]
fn chosen_type_anthem() {
    cr!("614.1c", "607.2d", "613.4c");
    ruling!(
        "Etchings of the Chosen",
        "You must choose an existing creature type"
    );
    assert_supported("Shared Triumph");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let opp_elf = t.battlefield(P1, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Shared Triumph");
    // "Creatures of the chosen type get +1/+1" — all Elves, not only yours.
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(opp_elf), (2, 2));
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn chosen_type_in_addition_to_other_types() {
    cr!("607.2d", "613.1d");
    ruling!(
        "Adaptive Automaton",
        "other Construct creatures you control won't get +1/+1 unless you chose Construct"
    );
    assert_supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    let a = t.enter(P0, "Adaptive Automaton");
    let now = t.g.current(a);
    assert!(t.g.obj(now).chars.has_subtype("Elf"));
    assert!(t.g.obj(now).chars.has_subtype("Construct"));
    assert_eq!(t.pt(elf), (2, 2));
    // The lord doesn't pump itself ("other").
    assert_eq!(t.pt(a), (2, 2));
}

#[test]
fn granted_protection_uses_the_sources_choice() {
    cr!("607.2d", "702.16b", "613.1f");
    assert_supported("Riders of Gavony");
    let mut t = TestGame::new(2);
    // A Human creature you control.
    let human = t.battlefield(P0, "Elite Vanguard");
    choose_creature_type(&mut t, P0, "Goblin");
    t.enter(P0, "Riders of Gavony");
    let now = t.g.current(human);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bear = t.battlefield(P1, "Grizzly Bears");
    assert!(t.g.protected_from(now, goblin));
    assert!(!t.g.protected_from(now, bear));
}

#[test]
fn chosen_type_cost_reduction() {
    cr!("601.2f", "607.2d");
    ruling!("Urza's Incubator", "Multiple Incubators are cumulative");
    assert_supported("Urza's Incubator");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Urza's Incubator");
    // Grizzly Bears ({1}{G}) costs {G}.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn chosen_type_trigger_filter() {
    cr!("607.2d", "603.2");
    assert_supported("Species Specialist");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Species Specialist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 2);
    let b1 = t.hand(P0, "Lightning Bolt");
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(elves).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    let hand = t.hand_size(P0);
    assert_eq!(hand, 1, "an Elf died: no draw");
    t.cast(P0, b2).target(bears).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "a Bear died: may draw a card");
}

#[test]
fn spell_chooses_a_creature_type() {
    cr!("607.2d", "608.2c");
    assert_supported("And They Shall Know No Fear");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "And They Shall Know No Fear");
    choose_creature_type(&mut t, P0, "Elf");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.pt(elf), (2, 1));
    assert!(t.obj_now(elf).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.pt(bear), (2, 2));
}

// ---------------------------------------------------------------------------
// "choose A, B, or C" and anchor words
// ---------------------------------------------------------------------------

#[test]
fn choose_among_listed_card_types() {
    cr!("614.1c", "607.2d", "601.2f");
    assert_supported("Cloud Key");
    let mut t = TestGame::new(2);
    // artifact, creature, enchantment, instant, sorcery: choose creature.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.enter(P0, "Cloud Key");
    // Grizzly Bears ({1}{G}) costs {G}.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn choose_among_listed_creature_types() {
    cr!("614.1c", "607.2d");
    assert_supported("Dawn-Blessed Pennant");
    let mut t = TestGame::new(2);
    // Elemental, Elf, ...: choose Elf.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let p = t.enter(P0, "Dawn-Blessed Pennant");
    let asked = t.asked();
    let opts = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(opts.len(), 8);
    assert_eq!(t.obj_now(p).choices.creature_type.as_deref(), Some("Elf"));
    t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn anchor_word_abilities() {
    cr!("614.12c", "607.2m");
    assert_supported("Palace Siege");
    // Dragons — At the beginning of your upkeep, each opponent loses 2 life and you
    // gain 2 life.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let s = t.enter(P0, "Palace Siege");
    assert_eq!(t.obj_now(s).choices.text.as_deref(), Some("Dragons"));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 18);
    // Khans — return target creature card from your graveyard to your hand; the Dragons
    // ability isn't there.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Palace Siege");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
