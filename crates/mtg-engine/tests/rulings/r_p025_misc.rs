//! Rulings batch P025 — the rest: costs and the mana spent to cast a spell (CR 601.2f,
//! 118.7, 202.3), base power (CR 208.4b, 613.4b), Two-Headed Giant (CR 810), searches
//! for a creature type (CR 205.3), and copy abilities whose source card left the
//! graveyard (CR 608.2h).

use crate::r_p017_common::choice_candidates;
use crate::r_p025_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::events::MoveCause;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

// --- Mana spent and costs -------------------------------------------------------------------

#[test]
fn mockingbird_counts_all_the_mana_spent_to_cast_it() {
    cr!("601.2f", "107.3a", "614.1c");
    ruling!(
        "Mockingbird",
        "Mockingbird’s second ability counts all mana spent to cast Mockingbird, not just mana spent on {X}. If there are any additional costs to cast it, the mana spent to pay those costs counts as well."
    );
    supported("Mockingbird");
    let mut t = TestGame::new(2);
    // Sphere of Resistance: Mockingbird with X = 0 costs {1}{U}: 2 mana.
    t.battlefield(P1, "Sphere of Resistance");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{U}");
    let bird = t.hand(P0, "Mockingbird");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, bird).x(0).go();
    t.resolve_all();
    let offered = choice_candidates(&t, from);
    assert!(!offered.is_empty());
    assert!(offered[0].contains(&obj(bears)) && !offered[0].contains(&obj(giant)));
    assert_eq!(name_of(&t, bird), "Grizzly Bears");
    assert!(has_kw(&t, bird, KeywordKind::Flying) && has_subtype(&t, bird, "Bird"));
}

#[test]
fn gyrus_counts_the_commander_tax_in_the_mana_spent() {
    cr!("601.2f", "903.8", "614.1c");
    ruling!(
        "Gyrus, Waker of Corpses",
        "The amount of mana you spent to cast Gyrus is usually equal to its mana value, normally 3 plus the value chosen for X. However, you also include any additional costs you pay, including the commander tax."
    );
    let mut t = crate::r_s13_common::commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let gyrus = crate::r_s13_common::commander(&mut t, P0, "Gyrus, Waker of Corpses");
    t.g.players[0]
        .commander_casts
        .insert("Gyrus, Waker of Corpses".into(), 1);
    // X = 1 and {2} of commander tax: {1}{B}{R}{G} + {2} = 6 mana.
    lands_for(&mut t, P0, "{3}{B}{R}{G}");
    t.cast(P0, gyrus).x(1).go();
    t.resolve_all();
    let g = t.named_on_battlefield("Gyrus, Waker of Corpses")[0];
    assert_eq!(t.counters(g, counters::PLUS1), 6);
    assert_eq!(mv_now(&mut t, g), 3);
}

#[test]
fn sarkhans_reduction_doesnt_change_mana_value_or_colored_mana() {
    cr!("601.2f", "202.3", "118.7c");
    ruling!(
        "Sarkhan, Soul Aflame",
        "Sarkhan's first ability doesn't change the mana cost or mana value of any Dragon spell. It changes only the total cost you pay."
    );
    ruling!(
        "Sarkhan, Soul Aflame",
        "That ability can't reduce the amount of colored mana you pay for a Dragon spell. It reduces only the generic component of that cost."
    );
    supported("Sarkhan, Soul Aflame");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarkhan, Soul Aflame");
    // Shivan Dragon ({4}{R}{R}) costs {3}{R}{R} but its mana value is still 6.
    lands_for(&mut t, P0, "{3}{R}{R}");
    let dragon = t.hand(P0, "Shivan Dragon");
    let spell = t.cast(P0, dragon).go();
    assert_eq!(mv_now(&mut t, spell), 6);
    t.resolve_all();
    // Scion of the Ur-Dragon ({W}{U}{B}{R}{G}) can't be cast with four colored mana.
    lands_for(&mut t, P0, "{W}{U}{B}{R}");
    let scion = t.hand(P0, "Scion of the Ur-Dragon");
    assert!(t.cast(P0, scion).try_go().is_err());
    lands_for(&mut t, P0, "{G}");
    assert!(t.cast(P0, scion).try_go().is_ok());
}

#[test]
fn nahiri_enters_with_loyalty_doubled_by_doubling_season() {
    cr!("614.1c", "616.1", "306.5b");
    ruling!(
        "Nahiri, the Unforgiving",
        "Other replacement effects that would change the number of loyalty counters Nahiri enters with will apply as normal."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    // {1}{R}{R/W/P}{W}, paying the Phyrexian symbol with {R}.
    lands_for(&mut t, P0, "{1}{R}{R}{W}");
    let nahiri = t.hand(P0, "Nahiri, the Unforgiving");
    t.cast(P0, nahiri).go();
    t.resolve_all();
    assert_eq!(t.counters(nahiri, counters::LOYALTY), 10);
}

#[test]
fn cursecloth_wrappings_embalm_exiles_the_card_as_a_cost() {
    cr!("702.128a", "602.2b", "118.3");
    ruling!(
        "Cursecloth Wrappings",
        "Once you’ve activated an embalm ability, the card is immediately exiled. Opponents can’t try to stop the ability by exiling the card."
    );
    supported("Cursecloth Wrappings");
    let mut t = TestGame::new(2);
    let wraps = t.battlefield(P0, "Cursecloth Wrappings");
    let card = t.graveyard(P0, "Hill Giant");
    t.activate(P0, wraps, 0, &[obj(card)]).expect("activation");
    t.resolve_all();
    lands_for(&mut t, P0, "{3}{R}");
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, card, "mbalm").expect("embalm");
    // The card is in exile while the ability is on the stack.
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert!(has_subtype(&t, toks[0], "Zombie"));
}

// --- Base power -----------------------------------------------------------------------------

#[test]
fn base_power_counts_setting_effects_but_not_modifying_ones() {
    cr!("208.4b", "613.4b", "613.4c", "604.3");
    ruling!(
        "Zinnia, Valley's Voice",
        "Normally, a creature's base power is the power printed on the card or, for a token, the power set by the effect that created it. If another effect sets a creature's power to a specific number or value, those become its base power. If an effect modifies a creature's power without setting it, that is not included when determining its base power."
    );
    ruling!(
        "Zinnia, Valley's Voice",
        "Some creatures have base power and toughness 0/0 and an ability that gives them a bonus based on some criteria. Those are not characteristic-defining abilities, and that ability doesn't change its base power."
    );
    supported("Zinnia, Valley's Voice");
    let mut t = TestGame::new(2);
    let zinnia = t.battlefield(P0, "Zinnia, Valley's Voice");
    // Llanowar Elves (1/1) with +3/+3: base power 1, counts.
    let elves = t.battlefield(P0, "Llanowar Elves");
    giant_growth(&mut t, P0, elves);
    // Hill Giant set to 1/1: base power 1, counts.
    let giant = t.battlefield(P0, "Hill Giant");
    modify(
        &mut t,
        giant,
        vec![Modification::SetPT(Some(Value::c(1)), Some(Value::c(1)))],
    );
    // Grizzly Bears with -1/-0: power 1 but base power 2, doesn't count.
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(-1), Value::c(0))],
    );
    // Nighthowler with one creature card in a graveyard: a 1/1 with base power 0.
    t.graveyard(P1, "Grizzly Bears");
    let howler = t.battlefield(P0, "Nighthowler");
    assert_eq!(t.pt(howler), (1, 1));
    assert_eq!(t.pt(bears).0, 1);
    assert_eq!(t.pt(zinnia), (1 + 2, 3));
}

#[test]
fn rapid_augmenter_doesnt_give_haste_to_a_0_0_with_a_bonus() {
    cr!("208.4b", "604.3");
    ruling!(
        "Rapid Augmenter",
        "Some creatures have base power and toughness 0/0 and an ability that gives them a bonus based on some criteria. Those are not characteristic-defining abilities, and that ability doesn't change its base power."
    );
    supported("Rapid Augmenter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rapid Augmenter");
    t.graveyard(P1, "Grizzly Bears");
    let howler = t.enter(P0, "Nighthowler");
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(howler), (1, 1));
    assert!(!has_kw(&t, howler, KeywordKind::Haste));
    // A 1/1 does get haste.
    let elves = t.enter(P0, "Llanowar Elves");
    t.settle();
    t.resolve_all();
    assert!(has_kw(&t, elves, KeywordKind::Haste));
}

#[test]
fn curie_draws_cards_equal_to_its_base_power() {
    cr!("208.4b", "613.4c", "510.2");
    ruling!(
        "Curie, Emergent Intelligence",
        "Normally, a creature's base power is the power printed on the card, or, for a token, the power set by the effect that created it. If another effect sets a creature's power to a specific number or value, that becomes its base power. If an effect modifies a creature's power without setting it, that is not included when determining its base power."
    );
    supported("Curie, Emergent Intelligence");
    let mut t = TestGame::new(2);
    let curie = t.battlefield(P0, "Curie, Emergent Intelligence");
    // +3/+3: a 4/6 with base power 1.
    giant_growth(&mut t, P0, curie);
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(curie, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand + 1);
}

// --- Two-Headed Giant ---------------------------------------------------------------------

#[test]
fn saheeli_rai_deals_2_damage_to_the_opposing_team() {
    cr!("810.4", "810.9");
    ruling!(
        "Saheeli Rai",
        "In a Two-Headed Giant game, Saheeli's first ability causes 2 damage total to be dealt to the opposing team."
    );
    supported("Saheeli Rai");
    let mut t = two_headed_giant();
    let saheeli = t.battlefield(P0, "Saheeli Rai");
    let (ours, theirs) = (t.life(P0), t.life(PlayerId(2)));
    activate_containing(&mut t, P0, saheeli, "Scry 1").expect("activation");
    t.resolve_all();
    assert_eq!(t.life(PlayerId(2)), theirs - 2);
    assert_eq!(t.life(PlayerId(3)), theirs - 2);
    assert_eq!(t.life(P0), ours);
}

#[test]
fn shapeshifters_marrow_triggers_for_each_opponent_in_two_headed_giant() {
    cr!("810.2", "805.4d", "707.2");
    ruling!(
        "Shapeshifter's Marrow",
        "In a Two-Headed Giant game, this ability triggers separately for each player at the start of the opposing team’s turn. If the first ability to resolve reveals a creature card, Shapeshifter’s Marrow becomes a creature. If the second ability to resolve also reveals a creature card, the ex-Shapeshifter’s Marrow then becomes a copy of that creature."
    );
    supported("Shapeshifter's Marrow");
    let mut t = two_headed_giant();
    let marrow = t.battlefield(P0, "Shapeshifter's Marrow");
    t.library_top(PlayerId(2), "Hill Giant");
    t.library_top(PlayerId(3), "Grizzly Bears");
    // The opposing team's turn begins.
    t.set_step(P1, Step::End);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.active != P0
            && g.turn.active != P1
            && g.turn.step == Step::Upkeep
            && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    assert!(ok);
    t.settle();
    assert_eq!(triggers_on_stack(&t).len(), 2);
    t.resolve();
    t.settle();
    let first = name_of(&t, marrow);
    assert!(first == "Hill Giant" || first == "Grizzly Bears");
    assert!(is_creature(&t, marrow));
    t.resolve_all();
    let second = name_of(&t, marrow);
    assert!(second == "Hill Giant" || second == "Grizzly Bears");
    assert_ne!(first, second);
    assert!(t.in_graveyard(PlayerId(2), "Hill Giant"));
    assert!(t.in_graveyard(PlayerId(3), "Grizzly Bears"));
}

// --- Searching, choosing and leaving the graveyard ---------------------------------------------

#[test]
fn the_eleventh_hour_finds_a_doctor_by_creature_type() {
    cr!("205.3m", "701.23a", "714.2b");
    ruling!(
        "The Eleventh Hour",
        "The Eleventh Hour's first ability can find any card with the Doctor creature type, not cards with the word \"Doctor\" in the name."
    );
    supported("The Eleventh Hour");
    let mut t = TestGame::new(2);
    let healer = t.library_top(P0, "Cult Healer");
    let named = t.library_top(P0, "Trait Doctoring");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(healer)]);
    t.enter(P0, "The Eleventh Hour");
    t.settle();
    t.resolve_all();
    let offered = choice_candidates(&t, from);
    assert!(!offered.is_empty());
    assert!(offered[0].contains(&obj(healer)) && !offered[0].contains(&obj(named)));
    assert!(t.in_hand(P0, "Cult Healer"));
}

#[test]
fn scion_of_the_ur_dragon_stays_a_copy_after_the_card_leaves_the_graveyard() {
    cr!("707.2", "707.2b", "611.2c");
    ruling!(
        "Scion of the Ur-Dragon",
        "Removing the other Dragon card from your graveyard has no effect on Scion of the Ur-Dragon."
    );
    supported("Scion of the Ur-Dragon");
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of the Ur-Dragon");
    let dragon = t.library_top(P0, "Shivan Dragon");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(dragon)]);
    activate_containing(&mut t, P0, scion, "Search").expect("activation");
    t.resolve_all();
    assert_eq!(name_of(&t, scion), "Shivan Dragon");
    let card = t.g.current(dragon);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, Some(P1));
    t.g.recompute();
    assert_eq!(name_of(&t, scion), "Shivan Dragon");
    assert_eq!(t.pt(scion), (5, 5));
}

#[test]
fn sakashimas_will_copies_printed_values_and_an_animated_land_makes_lands() {
    cr!("707.2", "613.1a", "700.2");
    ruling!(
        "Sakashima's Will",
        "Sakashima's Will copies the printed values of the chosen creature. It won't copy any counters on that creature or any effects that have changed its power, toughness, types, color, and so on. Notably, it won't copy effects that made the chosen creature become a creature if it isn't normally a creature. For example, if you choose a land that's become a creature, all your other creatures will become noncreature lands until end of turn."
    );
    supported("Sakashima's Will");
    let will = |t: &mut TestGame, chosen: ObjectId| {
        lands_for(t, P0, "{3}{U}");
        let c = t.hand(P0, "Sakashima's Will");
        t.answer_choose(P0, &[obj(chosen)]);
        t.cast(P0, c).modes(&[1]).go();
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    will(&mut t, giant);
    assert_printed_giant(&t, bears, Some((3, 3)), true);
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    animate_land(&mut t, land);
    let bears = t.battlefield(P0, "Grizzly Bears");
    will(&mut t, land);
    assert_eq!(name_of(&t, bears), "Forest");
    assert!(t.obj_now(bears).is(CardType::Land) && !is_creature(&t, bears));
}

#[test]
fn mirror_of_the_forebears_with_no_chosen_type_cant_be_activated() {
    cr!("602.2", "707.2", "607.2d");
    ruling!(
        "Mirror of the Forebears",
        "If you somehow control a Mirror of the Forebears with no chosen creature type, its activated ability can’t be activated."
    );
    supported("Mirror of the Forebears");
    supported("Mirage Mirror");
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Giant");
    let forebears = t.enter(P0, "Mirror of the Forebears");
    t.settle();
    let giant = t.battlefield(P0, "Hill Giant");
    // A Mirage Mirror becomes a copy of the Mirror of the Forebears: no type was chosen
    // for it.
    let mirage = t.battlefield(P0, "Mirage Mirror");
    lands_for(&mut t, P0, "{2}");
    t.activate(P0, mirage, 0, &[obj(forebears)]).expect("activate");
    t.resolve_all();
    assert_eq!(name_of(&t, mirage), "Mirror of the Forebears");
    lands_for(&mut t, P0, "{1}");
    assert!(activate_containing(&mut t, P0, mirage, "becomes a copy")
        .ok()
        .flatten()
        .is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, mirage), "Mirror of the Forebears");
    let _ = giant;
}


#[test]
fn the_mimeoplasm_copying_a_card_that_enters_with_counters_gets_both() {
    cr!("707.9b", "614.1c", "614.12");
    ruling!(
        "The Mimeoplasm",
        "Specifically, if The Mimeoplasm as a copy of a creature card that enters the battlefield with a number of +1/+1 counters, The Mimeoplasm will enter with those +1/+1 counters and +1/+1 counters equal to the power of the other exiled card."
    );
    let mut t = TestGame::new(2);
    // Kalonian Hydra: a 0/0 that "enters with four +1/+1 counters"; Hill Giant: power 3.
    let hydra = t.graveyard(P1, "Kalonian Hydra");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(hydra), obj(giant)]);
    t.answer_choose(P0, &[obj(hydra)]);
    let m = t.enter(P0, "The Mimeoplasm");
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Kalonian Hydra");
    assert_eq!(t.counters(m, counters::PLUS1), 4 + 3);
    assert_eq!(t.pt(m), (7, 7));
}
