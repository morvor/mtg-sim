//! Rulings batch P207 — echo with a non-mana cost (CR 702.30), embalm (CR 702.128) with
//! token-counting and copy creatures, emerge from artifact (CR 702.119), and eminence
//! (CR 113.6b, 603.4) on commanders with cast and attack triggers.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::game::{Game, GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Echo — discard a card
// ---------------------------------------------------------------------------------------

#[test]
fn an_echo_cost_of_discarding_a_card_cant_be_paid_with_an_empty_hand() {
    cr!("702.30a", "118.3");
    ruling!(
        "Deepcavern Imp",
        "If you have no cards in hand, you can't pay the echo cost and Deepcavern Imp will be sacrificed."
    );
    supported("Deepcavern Imp");
    // No cards in hand: P0 would pay, but can't.
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Deepcavern Imp");
    next_upkeep(&mut t, P0);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(triggers_on_stack(&t, "Echo"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(imp));
    assert!(t.in_graveyard(P0, "Deepcavern Imp"));
    // With a card in hand, P0 discards it and keeps the Imp.
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Deepcavern Imp");
    t.hand(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(imp));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
}

// ---------------------------------------------------------------------------------------
// Embalm
// ---------------------------------------------------------------------------------------

/// Embalms the Anointer Priest card in P0's graveyard (with the mana for {3}{W}) and
/// resolves everything. Returns P0's life gained.
fn embalm_priest(t: &mut TestGame) -> i32 {
    let priest = t.graveyard(P0, "Anointer Priest");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let life = t.life(P0);
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve_all();
    t.life(P0) - life
}

#[test]
fn an_embalmed_anointer_priest_triggers_for_itself() {
    cr!("702.128a", "603.6a", "111.1");
    ruling!(
        "Anointer Priest",
        "If Anointer Priest is a token, most likely because it's embalmed, it will cause its own ability to trigger when it enters the battlefield."
    );
    supported("Anointer Priest");
    let mut t = TestGame::new(2);
    assert_eq!(embalm_priest(&mut t), 1);
    let priests = t.named_on_battlefield("Anointer Priest");
    assert_eq!(priests.len(), 1);
    assert!(t.obj_now(priests[0]).is_token());
}

#[test]
fn anointer_priests_entering_together_trigger_for_each_other() {
    cr!("702.128a", "603.6a", "614.1a");
    ruling!(
        "Anointer Priest",
        "If Anointer Priest enters the battlefield at the same time as a creature token, its ability triggers for that other creature. For example, if you control Anointed Procession and embalm Anointer Priest, the triggered ability of each one triggers twice and you gain a total of 4 life."
    );
    supported("Anointed Procession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    assert_eq!(embalm_priest(&mut t), 4);
    assert_eq!(t.named_on_battlefield("Anointer Priest").len(), 2);
}

#[test]
fn vizier_copying_a_token_copies_what_created_it_and_isnt_a_token() {
    cr!("707.2", "111.1", "111.3");
    ruling!(
        "Vizier of Many Faces",
        "If the chosen creature is a token, Vizier of Many Faces copies the original characteristics of that token as stated by the effect that created the token. Vizier of Many Faces is not a token in this case unless it's embalmed."
    );
    supported("Vizier of Many Faces");
    // A 1/1 colorless Soldier token with a +1/+1 counter and a Giant Growth on it.
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P1, "Soldier");
    t.g.add_counters(Entity::Object(soldier), counters::PLUS1, 1, None);
    t.lands(P1, "Forest", 1);
    let gg = t.hand(P1, "Giant Growth");
    t.cast(P1, gg).target(soldier).go();
    t.resolve_all();
    assert_eq!(t.pt(soldier), (5, 5));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    let vizier = t.enter(P0, "Vizier of Many Faces");
    t.settle();
    let o = t.obj_now(vizier);
    assert_eq!(o.chars.name, "Soldier");
    assert!(o.chars.has_subtype("Soldier"));
    assert_eq!(t.pt(vizier), (1, 1));
    assert_eq!(t.counters(vizier, counters::PLUS1), 0);
    assert!(!t.obj_now(vizier).is_token());
    // Embalmed, the Vizier is a white Zombie token copy of the Soldier.
    let card_in_gy = t.graveyard(P0, "Vizier of Many Faces");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    t.activate(P0, card_in_gy, 0, &[]).unwrap();
    t.resolve_all();
    let embalmed: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(embalmed.len(), 1);
    let o = t.obj_now(embalmed[0]);
    assert_eq!(o.chars.name, "Soldier");
    assert!(o.chars.has_subtype("Zombie"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.pt(embalmed[0]), (1, 1));
}

// ---------------------------------------------------------------------------------------
// Emerge from artifact (Crabomination)
// ---------------------------------------------------------------------------------------

const EMERGE: CastMethod = CastMethod::Keyword(KeywordKind::Emerge);
const CRAB: &str = "Crabomination";

/// Casts Crabomination (emerge from artifact {5}{B}{B}) for its emerge cost sacrificing
/// `sac`, with `swamps` Swamps and `generic` Wastes for mana. Whether it could be cast.
fn emerge_crab(t: &mut TestGame, sac: ObjectId, swamps: usize, generic: usize) -> bool {
    // Its emerge ability is supported; only its enters ability (which these rulings
    // aren't about) may be unsupported.
    let def = card(CRAB);
    assert!(def
        .front()
        .chars
        .abilities
        .iter()
        .any(|a| a.text.starts_with("Emerge")));
    for text in def.unsupported_text() {
        assert!(
            text.starts_with("When ~ enters, target opponent exiles"),
            "unexpected unsupported text: {text}"
        );
    }
    t.lands(P0, "Swamp", swamps);
    t.lands(P0, "Wastes", generic);
    let crab = t.hand(P0, CRAB);
    t.answer_choose(P0, &[Entity::Object(sac)]);
    let ok = t.cast(P0, crab).method(EMERGE).try_go().is_ok();
    t.clear_answers();
    ok
}

#[test]
fn emerge_reduces_only_the_generic_component_of_the_emerge_cost() {
    cr!("702.119a", "118.7");
    ruling!(
        "Crabomination",
        "Colored mana components of emerge costs can't be reduced with emerge."
    );
    ruling!(
        "Crabomination",
        "You may sacrifice an artifact with mana value greater than or equal to the emerge cost. If you do, you'll pay only the colored mana component of the emerge cost."
    );
    supported("Darksteel Colossus");
    // Darksteel Colossus has mana value 11: {B}{B} is still needed.
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    assert!(!emerge_crab(&mut t, colossus, 1, 5));
    assert!(t.on_battlefield(colossus));
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    assert!(emerge_crab(&mut t, colossus, 2, 0));
    assert!(!t.on_battlefield(colossus));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn a_zero_mana_value_artifact_can_be_sacrificed_for_no_reduction() {
    cr!("702.119a", "202.3", "111.10b");
    ruling!(
        "Crabomination",
        "You may sacrifice an artifact with a mana value of 0, such as a Food token, to cast Crabomination for its emerge cost. You'll just pay the full emerge cost with no reduction."
    );
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    assert!(!emerge_crab(&mut t, food, 2, 4));
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    assert!(emerge_crab(&mut t, food, 2, 5));
    assert!(!t.on_battlefield(food));
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn the_mana_value_of_an_emerge_spell_is_its_mana_costs() {
    cr!("702.119a", "202.3", "118.9c");
    ruling!(
        "Crabomination",
        "The mana value of a creature spell with emerge isn't affected by whether its emerge cost is paid. For example, if you cast Crabomination for its emerge cost and sacrifice an artifact whose mana value is 3, Crabomination's mana value remains 6."
    );
    supported("Pristine Talisman");
    let mut t = TestGame::new(2);
    let talisman = t.battlefield(P0, "Pristine Talisman");
    assert_eq!(t.g.mana_value_of(talisman), 3);
    // {5}{B}{B} reduced by 3: {2}{B}{B}.
    assert!(emerge_crab(&mut t, talisman, 2, 2));
    assert!(!t.on_battlefield(talisman));
    let spell = top_of_stack(&t);
    assert_eq!(t.obj(spell).chars.name, CRAB);
    assert_eq!(t.g.mana_value_of(spell), 6);
}

#[test]
fn nothing_can_be_done_while_crabomination_is_being_cast() {
    cr!("601.2", "702.119a");
    ruling!(
        "Crabomination",
        "Once you begin to cast a spell with emerge, no player may take actions until you're done. Notably, opponents can't try to remove the permanent you wish to sacrifice."
    );
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 5);
    let crab = t.hand(P0, CRAB);
    let state = |g: &Game| {
        let food_out = g
            .battlefield
            .iter()
            .any(|o| g.obj(*o).chars.name.as_str() == "Food");
        (food_out, g.stack.len())
    };
    let seen = watch(&mut t, P1, |_| true, state);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: crab,
            method: EMERGE,
        }),
    );
    t.answer_choose(P0, &[Entity::Object(food)]);
    assert!(t.g.run_until(1000, |g| g.stack.iter().any(|o| g
        .obj(*o)
        .chars
        .name
        .as_str()
        == CRAB)));
    assert!(!t.on_battlefield(food));
    let seen = seen.lock().unwrap();
    for (food_out, stack) in seen.iter() {
        assert!(!(*food_out && *stack > 0), "P1 was asked mid-cast");
    }
}

// ---------------------------------------------------------------------------------------
// Eminence
// ---------------------------------------------------------------------------------------

/// A Commander game with `commander` in P0's command zone as their commander, in P0's
/// precombat main phase.
fn commander_game(commander: &str) -> (TestGame, ObjectId) {
    supported(commander);
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let c = t.command(P0, commander);
    t.g.objects[c.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push(commander.into());
    (t, c)
}

const EDGAR: &str = "Edgar Markov";

/// Casts Vampire Nighthawk (a Vampire spell) for P0; the eminence trigger, if any, is put
/// on the stack above it.
fn cast_vampire(t: &mut TestGame) {
    let hawk = in_hand_with_mana(t, P0, "Vampire Nighthawk");
    t.cast(P0, hawk).go();
    t.settle();
}

#[test]
fn edgars_eminence_triggers_from_the_command_zone_and_needs_edgar_there_as_it_resolves() {
    cr!("113.6b", "603.4", "400.7");
    ruling!(
        "Edgar Markov",
        "Edgar Markov’s eminence ability is a triggered ability. Edgar must be on the battlefield or in the command zone when you cast another Vampire spell and also as the triggered ability resolves. If it’s on the battlefield or in the command zone when you cast another Vampire spell but leaves that zone before the ability resolves, the ability won’t do anything as it resolves."
    );
    // From the command zone: a 1/1 black Vampire token.
    let (mut t, _edgar) = commander_game(EDGAR);
    cast_vampire(&mut t);
    assert_eq!(triggers_on_stack(&t, "command zone"), 1);
    t.resolve_all();
    let vamps = tokens_with_subtype(&t, P0, "Vampire");
    assert_eq!(vamps.len(), 1);
    assert_eq!(t.pt(vamps[0]), (1, 1));
    // From the battlefield.
    let (mut t, edgar) = commander_game(EDGAR);
    move_to(&mut t, edgar, Zone::Battlefield);
    cast_vampire(&mut t);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Vampire").len(), 1);
    // Not from P0's hand.
    let (mut t, edgar) = commander_game(EDGAR);
    t.answer_yes(P0, false);
    move_to(&mut t, edgar, Zone::Hand(P0));
    assert_eq!(t.zone(edgar), Zone::Hand(P0));
    cast_vampire(&mut t);
    assert_eq!(triggers_on_stack(&t, "command zone"), 0);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Edgar leaves the command zone (to P0's hand) with the ability on the stack.
    let (mut t, edgar) = commander_game(EDGAR);
    cast_vampire(&mut t);
    assert_eq!(triggers_on_stack(&t, "command zone"), 1);
    t.answer_yes(P0, false);
    move_to(&mut t, edgar, Zone::Hand(P0));
    assert_eq!(t.zone(edgar), Zone::Hand(P0));
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.named_on_battlefield("Vampire Nighthawk").len(), 1);
}

#[test]
fn edgars_eminence_does_nothing_once_edgar_goes_from_the_battlefield_to_the_command_zone() {
    cr!("603.4", "400.7", "903.9a");
    ruling!(
        "Edgar Markov",
        "Notably, if Edgar Markov is on the battlefield and its eminence ability triggers, but it’s put into the command zone before that ability resolves, that ability won’t do anything as it resolves. This is because an object that changes zones is considered a new object."
    );
    let (mut t, edgar) = commander_game(EDGAR);
    let edgar = move_to(&mut t, edgar, Zone::Battlefield).unwrap();
    cast_vampire(&mut t);
    assert_eq!(triggers_on_stack(&t, "command zone"), 1);
    // Edgar dies and its owner puts it into the command zone.
    t.answer_yes(P0, true);
    t.g.destroy(edgar, None);
    t.settle();
    assert_eq!(t.zone(edgar), Zone::Command);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // Casting Edgar itself from the command zone doesn't trigger it ("another").
    let (mut t, edgar) = commander_game(EDGAR);
    add_mana(&mut t, P0, ManaType::R, 2);
    add_mana(&mut t, P0, ManaType::W, 2);
    add_mana(&mut t, P0, ManaType::B, 2);
    t.cast(P0, edgar).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "command zone"), 0);
}

const SIDAR: &str = "Sidar Jabari of Zhalfir";

#[test]
fn sidar_jabaris_eminence_needs_it_in_its_zone_as_knights_attack_and_as_it_resolves() {
    cr!("113.6b", "603.4", "400.7", "508.1", "903.9a");
    ruling!(
        "Sidar Jabari of Zhalfir",
        "Sidar Jabari must be on the battlefield or in the command zone at the moment you attack with one or more Knights and also as the triggered ability tries to resolve. Notably, if Sidar Jabari is on the battlefield and its eminence ability triggers, but it's put into the command zone before that ability resolves, that ability won't do anything as it resolves. This is because an object that changes zones is considered a new object."
    );
    supported("Silver Knight");
    // From the command zone, attacking with Silver Knight (a Knight): loot.
    let (mut t, _sidar) = commander_game(SIDAR);
    let knight = t.battlefield(P0, "Silver Knight");
    t.hand(P0, "Grizzly Bears");
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "command zone"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    // A non-Knight attacking doesn't trigger it.
    let (mut t, _sidar) = commander_game(SIDAR);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "command zone"), 0);
    // On the battlefield, Sidar Jabari (a Knight) attacks; it's put into the command zone
    // before the ability resolves: nothing happens.
    let (mut t, sidar) = commander_game(SIDAR);
    let sidar = move_to(&mut t, sidar, Zone::Battlefield).unwrap();
    t.g.objects[sidar.0 as usize].summoning_sick = false;
    t.hand(P0, "Grizzly Bears");
    attack_with(&mut t, &[(sidar, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "command zone"), 1);
    t.answer_yes(P0, true);
    t.g.destroy(sidar, None);
    t.settle();
    assert_eq!(t.zone(sidar), Zone::Command);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 0);
}
