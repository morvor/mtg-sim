//! Rulings batch P095 — activated abilities and spells of the "life payment" cards: what
//! a cost includes and when targets are chosen (CR 601.2c, 602.2b), where abilities work
//! (CR 113.6), permission to cast from another zone keeping the timing rules (CR 601.3),
//! total cost (CR 601.2f), dividing damage (CR 601.2d), Powerstone tokens (CR 111.10),
//! permanent types (CR 110.4), and spells whose X is 0.

use crate::r_p095_common::*;
use crate::r_p079_common::can_cast_now;
use crate::r_s01_common::{supported, tapped_lands};
use crate::r_s29_common::{choose_modes, divide, put_counters};
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::*;

/// Moves `id` to exile, as an effect unrelated to it would.
fn exile_by_other_means(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.flush_events();
}

// ---------------------------------------------------------------------------------------
// Serpent's Soul-Jar
// ---------------------------------------------------------------------------------------

#[test]
fn serpents_soul_jar_card_removed_before_the_exile_trigger_resolves() {
    cr!("607.2a", "400.7", "608.2b");
    ruling!(
        "Serpent's Soul-Jar",
        "Players can respond to the triggered ability that exiles the card from your graveyard by removing it in some other way. If the card leaves the graveyard before the ability of Serpent's Soul-Jar exiles it, Serpent's Soul-Jar won't let you cast it."
    );
    let mut t = TestGame::new(2);
    let jar = t.battlefield(P0, "Serpent's Soul-Jar");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    t.g.destroy(elf, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_from(&t, jar), 1);
    exile_by_other_means(&mut t, elf);
    t.resolve_all();
    t.activate(P0, jar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_exile("Llanowar Elves"));
    assert!(!can_cast_now(&mut t, P0, elf));
}

#[test]
fn serpents_soul_jar_doesnt_change_when_the_creature_can_be_cast() {
    cr!("601.3", "307.1");
    ruling!(
        "Serpent's Soul-Jar",
        "The activated ability doesn't change when you can cast the creature spell. In most cases, this means during your main phase while the stack is empty, although flash may change this."
    );
    let mut t = TestGame::new(2);
    let jar = t.battlefield(P0, "Serpent's Soul-Jar");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    t.g.destroy(elf, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_exile("Llanowar Elves"));
    t.set_step(P0, Step::BeginningOfCombat);
    t.activate(P0, jar, 0, &[]).unwrap();
    t.resolve_all();
    // During combat: no.
    assert!(!can_cast_now(&mut t, P0, elf));
    // In the main phase with something on the stack: no.
    t.set_step(P0, Step::PostcombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    assert!(!can_cast_now(&mut t, P0, elf));
    t.resolve_all();
    // Main phase, empty stack: yes.
    assert!(can_cast_now(&mut t, P0, elf));
}

// ---------------------------------------------------------------------------------------
// Powerstones
// ---------------------------------------------------------------------------------------

#[test]
fn powerstone_tokens_are_predefined_artifacts_with_a_restricted_mana_ability() {
    cr!("111.10", "111.10k", "106.6");
    ruling!(
        "Terisiare's Devastation",
        "Powerstone tokens are a kind of predefined token. Each one has the artifact subtype “Powerstone” and the ability “{T}: Add {C}. This mana can’t be spent to cast a nonartifact spell.”"
    );
    supported("Terisiare's Devastation");
    // X = 1: lose 1 life, one tapped Powerstone.
    let mut t = TestGame::new(2);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Terisiare's Devastation");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Terisiare's Devastation");
    t.cast(P0, card).x(1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    let stones = crate::r_s01_common::with_subtype(&t, P0, "Powerstone");
    assert_eq!(stones.len(), 1);
    let stone = t.obj_now(stones[0]);
    assert!(stone.is_token() && stone.is(CardType::Artifact) && stone.tapped);
    assert!(stone
        .chars
        .abilities
        .iter()
        .any(|a| a.is_mana_ability()));
}

#[test]
fn powerstone_mana_pays_for_anything_but_nonartifact_spells() {
    cr!("106.6", "111.10k", "602.2b");
    ruling!(
        "Karn, Living Legacy",
        "You can use the {C} added by a Powerstone token on anything that isn’t a nonartifact spell. This includes paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    ruling!(
        "Terisiare's Devastation",
        "You can use the {C} added by a Powerstone token on anything that isn’t a nonartifact spell. This includes paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    ruling!(
        "The Mana Rig",
        "You can use the {C} added by a Powerstone token on anything that isn’t a nonartifact spell. This includes paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    ruling!(
        "Wreck Hunter",
        "You can use the {C} added by a Powerstone token on anything that isn’t a nonartifact spell. This includes paying costs to activate abilities of both artifact and nonartifact permanents, paying ward costs, and so on."
    );
    supported("Karn, Living Legacy");
    // Karn's +1 makes a tapped Powerstone.
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Living Legacy");
    t.activate(P0, karn, 0, &[]).unwrap();
    t.resolve_all();
    let stone = crate::r_s01_common::with_subtype(&t, P0, "Powerstone")[0];
    t.g.untap(stone);
    // Not a nonartifact spell: Raging Goblin ({R}) can't be cast with a Mountain's mana
    // plus the Powerstone's for Grizzly Bears ({1}{G}) with a Forest.
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // An artifact spell: Sol Ring ({1}).
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    t.resolve_all();
    assert!(!t.named_on_battlefield("Sol Ring").is_empty());
    // An ability of a nonartifact permanent: Mishra's Factory's "{1}: ... becomes a 2/2
    // Assembly-Worker artifact creature ...", with the Powerstone's mana.
    let mut t = TestGame::new(2);
    let stone = crate::r_s02_common::create_token(&mut t, P0, "Powerstone");
    let factory = t.battlefield(P0, "Mishra's Factory");
    // (Tapped, so it can't pay for its own ability.)
    t.g.tap(factory);
    crate::r_s06_common::activate_containing(&mut t, P0, factory, "becomes").unwrap();
    assert!(t.obj_now(stone).tapped);
    t.resolve_all();
    assert!(t.obj_now(factory).is(CardType::Creature));
}

// ---------------------------------------------------------------------------------------
// Activated abilities: costs, timing, targets, zones
// ---------------------------------------------------------------------------------------

#[test]
fn netherborn_altar_counts_soul_counters_as_it_resolves() {
    cr!("602.2b", "608.2h", "903.9b");
    ruling!(
        "Netherborn Altar",
        "Putting a soul counter onto Netherborn Altar is part of the cost to activate its ability, and counting the number of soul counters is done as the ability resolves. Players may take actions between those times to change the number of soul counters, perhaps by removing them all so that you lose no life."
    );
    let mut t = crate::r_s13_common::commander_game();
    let isamaru = crate::r_s13_common::commander(&mut t, P0, "Isamaru, Hound of Konda");
    let altar = t.battlefield(P0, "Netherborn Altar");
    t.answer_yes(P0, false);
    crate::r_s06_common::activate_containing(&mut t, P0, altar, "soul").unwrap();
    assert_eq!(t.counters(altar, "soul"), 1);
    // A response removes the counter.
    t.g.remove_counters(Entity::Object(altar), "soul", 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(isamaru), Zone::Hand(P0));
    assert_eq!(t.life(P0), 40);
}

#[test]
fn cleaving_reaper_any_angel_or_berserker_that_entered_this_turn() {
    cr!("602.5b", "113.6");
    ruling!(
        "Cleaving Reaper",
        "The Angel or Berserker doesn't have to still be under your control in order to activate Cleaving Reaper's ability. Additionally, the Angel or Berserker could have entered the battlefield under your control at any point in the turn, even before Cleaving Reaper was put into the graveyard."
    );
    supported("Cleaving Reaper");
    // No Angel or Berserker entered: no.
    let mut t = TestGame::new(2);
    let reaper = t.graveyard(P0, "Cleaving Reaper");
    assert!(t.activate(P0, reaper, 0, &[]).is_err());
    // Serra Angel entered, then left; then Cleaving Reaper went to the graveyard.
    let mut t = TestGame::new(2);
    let angel = t.enter(P0, "Serra Angel");
    bounce(&mut t, angel);
    let reaper = t.battlefield(P0, "Cleaving Reaper");
    t.g.destroy(reaper, None);
    t.g.flush_events();
    t.settle();
    let reaper = t.g.current(reaper);
    assert_eq!(t.zone(reaper), Zone::Graveyard(P0));
    t.activate(P0, reaper, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.in_hand(P0, "Cleaving Reaper"));
}

#[test]
fn foul_familiar_ability_works_only_on_the_battlefield() {
    cr!("113.6", "602.2");
    ruling!(
        "Foul Familiar",
        "The ability can only be activated while this card is on the battlefield."
    );
    supported("Foul Familiar");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    for zone in [Zone::Hand(P0), Zone::Graveyard(P0)] {
        let ff = crate::r_s01_common::place(&mut t, P0, "Foul Familiar", zone);
        assert!(t.activate(P0, ff, 0, &[]).is_err());
    }
    assert_eq!(t.life(P0), 20);
    let ff = t.battlefield(P0, "Foul Familiar");
    t.activate(P0, ff, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.zone(ff), Zone::Hand(P0));
}

#[test]
fn vona_activated_while_attacking_stays_in_combat() {
    cr!("506.4", "602.5d");
    ruling!(
        "Vona, Butcher of Magan",
        "Vona's last ability can be activated during any step or phase of your turn, including the combat phase. It's possible to attack with Vona and then activate its ability before blockers are declared. Doing so won't remove Vona from combat."
    );
    supported("Vona, Butcher of Magan");
    let mut t = TestGame::new(2);
    let vona = t.battlefield(P0, "Vona, Butcher of Magan");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(vona, Entity::Player(P1))]);
    t.activate(P0, vona, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.g.is_attacking(vona));
    crate::r_s01_common::block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 20 - 7 + 4);
}

#[test]
fn cruel_sadist_removes_any_counters() {
    cr!("602.2b", "122.1");
    ruling!(
        "Cruel Sadist",
        "You can remove any +1/+1 counters from Cruel Sadist to activate its second ability, not just those put on Cruel Sadist by its first ability."
    );
    supported("Cruel Sadist");
    let mut t = TestGame::new(2);
    let sadist = t.battlefield(P0, "Cruel Sadist");
    put_counters(&mut t, sadist, "+1/+1", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    answer_x(&mut t, P0, 2);
    t.activate(P0, sadist, 1, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.counters(sadist, "+1/+1"), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn priest_of_fell_rites_cant_target_itself() {
    cr!("602.2b", "601.2c", "601.2h");
    ruling!(
        "Priest of Fell Rites",
        "You choose the target of the first ability before paying any of that ability's costs. In other words, you can't choose Priest of Fell Rites itself as the target to bring it back from the graveyard."
    );
    supported("Priest of Fell Rites");
    // No creature card in the graveyard: it can't be activated.
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Priest of Fell Rites");
    assert!(t.activate(P0, priest, 0, &[]).is_err());
    assert!(t.on_battlefield(priest));
    // With Grizzly Bears there, the Priest isn't among the candidates.
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, priest, 0, &[Entity::Object(bears)]).unwrap();
    let candidates: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(candidates, vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Priest of Fell Rites"));
    assert!(!t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn chainer_returns_the_creature_black_and_a_nightmare_as_it_enters() {
    cr!("614.12", "613.1d", "613.1e", "105.3");
    ruling!(
        "Chainer, Dementia Master",
        "The creature put onto the battlefield by Chainer's activated ability is black only, not black in addition to its other colors. It is, however, a Nightmare in addition to its other creature types. As it enters the battlefield, it is already a black Nightmare (and perhaps some other creature types); it doesn't enter and then become a black Nightmare."
    );
    supported("Chainer, Dementia Master");
    // Shadow Alley Denizen: "Whenever another black creature you control enters, target
    // creature gains intimidate until end of turn."
    let mut t = TestGame::new(2);
    let chainer = t.battlefield(P0, "Chainer, Dementia Master");
    let denizen = t.battlefield(P0, "Shadow Alley Denizen");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, chainer, 0, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.life(P0), 17);
    t.answer_targets(P0, &[Entity::Object(denizen)]);
    t.resolve();
    let now = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj_now(now).controller, P0);
    assert_eq!(t.obj_now(now).chars.colors, ColorSet::single(Color::Black));
    assert!(t.obj_now(now).chars.has_subtype("Nightmare"));
    assert!(t.obj_now(now).chars.has_subtype("Bear"));
    assert_eq!(t.pt(now), (3, 3));
    assert_eq!(abilities_from(&t, denizen), 1);
}

// ---------------------------------------------------------------------------------------
// Casting: permissions, total cost
// ---------------------------------------------------------------------------------------

#[test]
fn noctis_artifact_spells_from_the_graveyard_follow_costs_and_timing() {
    cr!("601.3", "601.2f", "307.1");
    ruling!(
        "Noctis, Prince of Lucis",
        "You pay all costs and follow all timing rules for artifact spells cast with the permission granted by Noctis's last ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Noctis, Prince of Lucis");
    let thopter = t.graveyard(P0, "Ornithopter");
    let ring = t.graveyard(P0, "Sol Ring");
    // Timing: not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast_now(&mut t, P0, thopter));
    t.set_step(P0, Step::PostcombatMain);
    // Costs: Sol Ring needs its {1} as well as 3 life.
    assert!(!can_cast_now(&mut t, P0, ring));
    assert!(can_cast_now(&mut t, P0, thopter));
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    t.lands(P0, "Wastes", 1);
    assert!(can_cast_now(&mut t, P0, ring));
}

#[test]
fn licia_cost_increases_apply_before_reductions() {
    cr!("601.2f", "202.3");
    ruling!(
        "Licia, Sanguine Tribune",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as the \"commander tax\"), then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    // Lodestone Golem: "Nonartifact spells cost {1} more to cast." P0 gained 6 life this
    // turn: {5}{R}{W}{B} + {1} - {6} = {R}{W}{B}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Lodestone Golem");
    gain(&mut t, P0, 6);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    let licia = t.hand(P0, "Licia, Sanguine Tribune");
    let spell = t.cast(P0, licia).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    assert_eq!(t.g.mana_value_of(spell), 8);
}

// ---------------------------------------------------------------------------------------
// Spells
// ---------------------------------------------------------------------------------------

#[test]
fn fire_covenant_divides_as_cast_with_at_least_one_each() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Fire Covenant",
        "You divide the damage as you cast the spell. You can't redistribute the damage if any of the target creatures becomes illegal before the spell resolves."
    );
    ruling!(
        "Fire Covenant",
        "You must distribute at least 1 damage to each target."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Fire Covenant");
    let card = t.hand(P0, "Fire Covenant");
    divide(&mut t, P0, &[1, 4]);
    let from = t.asked().len();
    t.cast(P0, card)
        .x(5)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    assert_eq!(t.life(P0), 15);
    let min = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(*min_each),
            _ => None,
        })
        .unwrap();
    assert_eq!(min, 1);
    // The Giant leaves: the Bears still get only their 1.
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(crate::r_s29_common::damage_marked(&t, bears), 1);
}

#[test]
fn grim_tutor_doesnt_reveal_the_card() {
    cr!("701.23a", "701.20a");
    ruling!("Grim Tutor", "You don't reveal the card you search for.");
    supported("Grim Tutor");
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    cast_new(&mut t, P0, "Grim Tutor", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(crate::r_p181_common::revealed(&t).is_empty());
    assert_eq!(t.life(P0), 17);
}

#[test]
fn the_cruelty_of_gix_chapter_ii_searches_and_loses_life() {
    cr!("714.2b", "701.23a");
    ruling!(
        "The Cruelty of Gix",
        "You must search your library as the chapter II ability resolves, and you will lose 3 life."
    );
    supported("The Cruelty of Gix");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.enter(P0, "The Cruelty of Gix");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn agonizing_remorse_must_exile_a_card_if_able() {
    cr!("608.2c", "609.3");
    ruling!(
        "Agonizing Remorse",
        "You must choose a card to exile if able. If there are no nonland cards in the opponent's hand, you must exile a card from their graveyard, and vice versa."
    );
    // Only a land in hand: the graveyard's card.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    t.graveyard(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Agonizing Remorse", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P1, "Forest"));
    assert_eq!(t.life(P0), 19);
    // An empty graveyard: the hand's nonland card.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Agonizing Remorse", &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn orcus_with_x_zero_does_nothing() {
    cr!("107.3", "107.3m");
    ruling!(
        "Orcus, Prince of Undeath",
        "You may cast Orcus, Prince of Undeath with X equal to 0. If you do, the triggered ability will do nothing as it resolves."
    );
    supported("Orcus, Prince of Undeath");
    for mode in [0usize, 1] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.graveyard(P0, "Raging Goblin");
        crate::r_s25_common::lands_for_cost(&mut t, P0, "Orcus, Prince of Undeath");
        let card = t.hand(P0, "Orcus, Prince of Undeath");
        t.cast(P0, card).x(0).go();
        choose_modes(&mut t, P0, &[mode]);
        t.resolve_all();
        assert!(!t.named_on_battlefield("Orcus, Prince of Undeath").is_empty());
        assert_eq!(t.pt(bears), (2, 2));
        assert_eq!(t.life(P0), 20);
        assert!(t.in_graveyard(P0, "Raging Goblin"));
    }
}

#[test]
fn revival_experiment_one_card_per_permanent_type() {
    cr!("110.4", "205.4a");
    ruling!(
        "Revival Experiment",
        "The permanent types are artifact, creature, enchantment, land, and planeswalker. Things like “legendary” or “snow” are supertypes, not types."
    );
    supported("Revival Experiment");
    let mut t = TestGame::new(2);
    let ids: Vec<ObjectId> = [
        "Grizzly Bears",
        "Isamaru, Hound of Konda",
        "Forest",
        "Snow-Covered Forest",
        "Sol Ring",
        "Glorious Anthem",
        "Jace Beleren",
    ]
    .iter()
    .map(|n| t.graveyard(P0, n))
    .collect();
    let [bears, isamaru, forest, snow, ring, anthem, jace] = ids[..] else {
        unreachable!()
    };
    // One choice per permanent type that has cards: artifact, creature, enchantment,
    // land, planeswalker. The legendary and the snow cards are among the creature and land
    // choices, not choices of their own.
    for pick in [ring, isamaru, anthem, snow, jace] {
        t.answer_choose(P0, &[Entity::Object(pick)]);
    }
    cast_new(&mut t, P0, "Revival Experiment", &[]);
    let from = t.asked().len();
    t.resolve_all();
    let choices: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    let o = Entity::Object;
    assert_eq!(
        choices,
        vec![
            vec![o(ring)],
            vec![o(bears), o(isamaru)],
            vec![o(anthem)],
            vec![o(forest), o(snow)],
            vec![o(jace)],
        ]
    );
    for id in [ring, isamaru, anthem, snow, jace] {
        assert!(t.on_battlefield(id));
    }
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(forest));
    assert_eq!(t.life(P0), 20 - 15);
    assert!(t.in_exile("Revival Experiment"));
}

#[test]
fn nameless_race_has_no_creature_type() {
    cr!("205.3d", "205.3m");
    ruling!(
        "Nameless Race",
        "This currently has no creature type. It's Nameless."
    );
    let c = card("Nameless Race");
    let chars = &c.front().chars;
    assert!(chars.card_types.contains(CardType::Creature));
    assert!(chars.subtypes.is_empty());
}
