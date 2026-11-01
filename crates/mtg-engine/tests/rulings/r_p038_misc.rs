//! Rulings batch P038 — the rest: the legend rule and Mirror Box (CR 704.5j), tokens
//! named after cards (CR 111.4), counting objects of several types once, sacrificing
//! simultaneously (CR 101.4), APNAP order (CR 603.3b), and other card-specific rulings.

use crate::r_p038_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, creatures, stack_library, supported};
use crate::r_s02_common::create_token;
use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` creates a token with these types (as a resolving effect would).
fn token_with(t: &mut TestGame, p: PlayerId, types: &[CardType], subtypes: &[&str]) -> ObjectId {
    let spec = TokenSpec {
        name: subtypes.join(" ").into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: types.to_vec(),
        subtypes: subtypes.iter().map(|s| (*s).into()).collect(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    let id = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .unwrap();
    t.g.flush_events();
    id
}

// ---------------------------------------------------------------------------
// Mirror Box and the legend rule
// ---------------------------------------------------------------------------

#[test]
fn mirror_box_turns_off_the_legend_rule_until_it_leaves() {
    cr!("704.5j", "704.3");
    ruling!(
        "Mirror Box",
        "While the “legend rule” doesn't apply to permanents you control, you can control any number of legendary permanents with the same name and none of them will be put into the graveyard."
    );
    ruling!(
        "Mirror Box",
        "If you control more than one legendary permanent with the same name and the “legend rule” begins applying again (perhaps because Mirror Box leaves the battlefield), you'll immediately have to comply with the rule and put all but one of those permanents into the graveyard."
    );
    ruling!(
        "Mirror Box",
        "The “legend rule” is the rule that states that if a player controls two or more legendary permanents with the same name, that player chooses one of them, and the rest are put into their owners' graveyards."
    );
    supported("Mirror Box");
    // Without Mirror Box: the legend rule applies.
    let mut t = TestGame::new(2);
    add(&mut t, P0, "Isamaru, Hound of Konda");
    add(&mut t, P0, "Isamaru, Hound of Konda");
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
    // With it: three of them stay (each +1/+1 legendary, +2/+2 for the two others).
    let mut t = TestGame::new(2);
    let mbox = t.battlefield(P0, "Mirror Box");
    for _ in 0..3 {
        add(&mut t, P0, "Isamaru, Hound of Konda");
    }
    let isamarus = t.named_on_battlefield("Isamaru, Hound of Konda");
    assert_eq!(isamarus.len(), 3);
    assert_eq!(t.pt(isamarus[0]), (5, 5));
    // Mirror Box leaves: all but one go to the graveyard right away.
    kill(&mut t, mbox);
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn mirror_box_token_copies_improve_the_nontoken_bonus() {
    cr!("707.2", "111.1");
    ruling!(
        "Mirror Box",
        "Although token creatures won't get a bonus from Mirror Box's last ability, tokens that are copies of nontoken creatures improve the bonus those creatures get."
    );
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Box");
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Cackling Counterpart", &[obj(bear)]);
    let copy = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(copy.len(), 1);
    assert_eq!(t.pt(bear), (3, 3));
    assert_eq!(t.pt(copy[0]), (2, 2));
}

// ---------------------------------------------------------------------------
// Card-specific counts
// ---------------------------------------------------------------------------

#[test]
fn tokens_named_after_cards_have_no_mana_cost() {
    cr!("111.4", "202.3a");
    ruling!(
        "Cloudseeder",
        "Unlike the card Cloud Sprite, the token has no mana cost. Its mana value is 0."
    );
    ruling!(
        "Llanowar Mentor",
        "Unlike the card Llanowar Elves, the token has no mana cost. Its mana value is 0."
    );
    for (card, token) in [("Cloudseeder", "Cloud Sprite"), ("Llanowar Mentor", "Llanowar Elves")] {
        supported(card);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        t.hand(P0, "Grizzly Bears");
        mana_all(&mut t, P0, 1);
        t.answer_choose(P0, &[]);
        t.activate(P0, src, 0, &[]).unwrap();
        t.resolve_all();
        let made = t.named_on_battlefield(token);
        assert_eq!(made.len(), 1, "{card}");
        assert!(t.obj_now(made[0]).is_token());
        assert_eq!(t.g.mana_value_of(made[0]), 0, "{card}");
        assert!(t.obj_now(made[0]).chars.mana_cost.is_none(), "{card}");
    }
}

#[test]
fn counting_cards_by_card_specific_rules() {
    cr!("608.2h", "208.1");
    ruling!(
        "Golden Ratio",
        "Creatures have different power if their powers are not equal. For example, if you have a creature with 2 power, two creatures with 1 power, and a creature with -1 power, you will draw three cards as Golden Ratio resolves."
    );
    ruling!(
        "War Report",
        "Each artifact creature on the battlefield is counted twice."
    );
    ruling!(
        "Honored Dreyleader",
        "If a permanent you control is both a Squirrel and a Food, count it only once when determining how many counters to put on Honored Dreyleader with its second ability."
    );
    ruling!(
        "Honored Dreyleader",
        "Similarly, if a permanent you control that is both a Squirrel and a Food enters, Honored Dreyleader's last ability will trigger only once."
    );
    ruling!(
        "Hollowhenge Overlord",
        "If you control a creature that is both a Wolf and a Werewolf, it's only counted once for Hollowhenge Overlord's triggered ability."
    );
    // Golden Ratio: powers 2, 1, 1 and -1: three cards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    let wall = t.battlefield(P0, "Wall of Wood");
    t.g.add_counters(obj(wall), counters::MINUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(wall).0, -1);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Golden Ratio", &[]);
    assert_eq!(t.hand_size(P0), hand + 3);

    // War Report: Ornithopter counts as a creature and as an artifact.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "War Report", &[]);
    assert_eq!(t.life(P0), 23);

    // Honored Dreyleader: a Squirrel Food counts once and triggers once.
    supported("Honored Dreyleader");
    let mut t = TestGame::new(2);
    token_with(&mut t, P0, &[CardType::Artifact, CardType::Creature], &["Squirrel", "Food"]);
    let d = enter(&mut t, P0, "Honored Dreyleader");
    t.resolve_all();
    assert_eq!(t.counters(d, counters::PLUS1), 1);
    token_with(&mut t, P0, &[CardType::Artifact, CardType::Creature], &["Squirrel", "Food"]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(d, counters::PLUS1), 2);

    // Hollowhenge Overlord (a Wolf) and a Wolf Werewolf: two Wolves.
    supported("Hollowhenge Overlord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hollowhenge Overlord");
    token_with(&mut t, P0, &[CardType::Creature], &["Wolf", "Werewolf"]);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(creature_count(&t, P0), 4);
}

#[test]
fn coat_of_arms_counts_creatures_sharing_any_creature_type() {
    cr!("613.4c", "205.3");
    ruling!(
        "Coat of Arms",
        "If a creature has more than one creature type, and one of those types matches the creature you are calculating for, then count that creature. Only one type needs to match in order to get counted."
    );
    ruling!(
        "Coat of Arms",
        "If you have a creature with more than one creature type, count all creatures which have either creature type."
    );
    supported("Coat of Arms");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coat of Arms");
    let visionary = t.battlefield(P0, "Elvish Visionary"); // Elf Shaman 1/1
    let elves = t.battlefield(P0, "Llanowar Elves"); // Elf Druid 1/1
    let shaman = t.battlefield(P1, "Shaman of Forgotten Ways"); // Human Shaman 2/3
    let bear = t.battlefield(P1, "Grizzly Bears"); // Bear 2/2
    t.g.recompute();
    assert_eq!(t.pt(visionary), (3, 3), "the Elf and the Shaman");
    assert_eq!(t.pt(elves), (2, 2));
    assert_eq!(t.pt(shaman), (3, 4));
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn deathbringer_regent_destroys_all_others_and_copies_trigger() {
    cr!("603.4", "707.2", "603.6a");
    ruling!(
        "Deathbringer Regent",
        "Deathbringer Regent's last ability destroys all creatures except Deathbringer Regent, including the five other creatures required for the ability to have an effect."
    );
    ruling!(
        "Deathbringer Regent",
        "If you cast a creature spell that enters the battlefield as a copy of Deathbringer Regent, the enters-the-battlefield ability will trigger (assuming the “five or more other creatures” requirement is also met)."
    );
    supported("Deathbringer Regent");
    let mut t = TestGame::new(2);
    bears(&mut t, P0, 2);
    bears(&mut t, P1, 3);
    cast_resolve(&mut t, P0, "Deathbringer Regent", &[]);
    assert_eq!(creature_count(&t, P0) + creature_count(&t, P1), 1);
    assert_eq!(t.named_on_battlefield("Deathbringer Regent").len(), 1);

    // Clone cast from hand, copying a Regent with five other creatures around.
    let mut t = TestGame::new(2);
    let regent = t.battlefield(P1, "Deathbringer Regent");
    bears(&mut t, P0, 2);
    bears(&mut t, P1, 2);
    t.answer_choose(P0, &[obj(regent)]);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Clone", &[]);
    // The copy survives; everything else (the original Regent too) is destroyed.
    assert_eq!(creature_count(&t, P1), 0);
    let left = creatures(&t, P0);
    assert_eq!(left.len(), 1);
    assert_eq!(t.obj_now(left[0]).chars.name.as_str(), "Deathbringer Regent");
}

// ---------------------------------------------------------------------------
// Sacrifices
// ---------------------------------------------------------------------------

#[test]
fn edicts_sacrifice_everything_short_and_all_at_once_in_apnap_choice_order() {
    cr!("101.4", "101.4a", "603.10a", "701.21a");
    ruling!(
        "Blasphemous Edict",
        "If a player controls fewer than thirteen creatures when Blasphemous Edict resolves, they will sacrifice all of their creatures."
    );
    ruling!(
        "Blasphemous Edict",
        "Starting with the player whose turn it is, each player in turn order chooses which creatures they will sacrifice, then all the creatures chosen by all players are sacrificed at the same time. Players get to know the choices made by players who chose before them."
    );
    ruling!(
        "By Invitation Only",
        "Starting with the player whose turn it is, each player in turn order chooses which creatures they will sacrifice, then all the creatures chosen by all players are sacrificed at the same time. Players get to know the choices made by players who chose before them."
    );
    supported("Blasphemous Edict");
    let mut t = TestGame::new(2);
    bears(&mut t, P0, 2);
    bears(&mut t, P1, 5);
    cast_resolve(&mut t, P0, "Blasphemous Edict", &[]);
    assert_eq!(creature_count(&t, P0) + creature_count(&t, P1), 0);

    // By Invitation Only, 1: P0 (the active player) chooses first, then P1; Blood Artist
    // (P0's choice) sees P1's creature die at the same time.
    supported("By Invitation Only");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    let theirs = bears(&mut t, P1, 2);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_choose(P0, &[obj(artist)]);
    t.answer_choose(P1, &[obj(theirs[0])]);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[pl(P1)]);
    let from = t.asked().len();
    cast_only(&mut t, P0, "By Invitation Only", &[]);
    t.resolve();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert!(t.in_graveyard(P0, "Blood Artist"));
    assert_eq!(creature_count(&t, P1), 1);
    // Two triggers: Blood Artist itself and P1's Bears.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// End step timing
// ---------------------------------------------------------------------------

#[test]
fn throne_resolves_after_the_opponents_end_step_triggers() {
    cr!("603.3b", "101.4");
    ruling!(
        "Throne of the God-Pharaoh",
        "If other abilities trigger at the beginning of your end step, first those controlled by your opponents resolve, then yours resolve. Each player orders their triggers in any order."
    );
    supported("Wound Reflection");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Throne of the God-Pharaoh");
    let b = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, b);
    t.battlefield(P1, "Wound Reflection");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).controller, P1, "the opponent's trigger resolves first");
    // Wound Reflection resolves first (P0 lost no life), then Throne: P1 loses 1.
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn growing_rites_and_creatures_leaving_or_lent_until_end_of_turn() {
    cr!("603.3b", "514.2", "603.4");
    ruling!(
        "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun",
        "If you control a creature that will be removed from the battlefield \"at the beginning of the next end step,\" you can resolve the ability of Growing Rites of Itlimoc before that creature leaves the battlefield."
    );
    ruling!(
        "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun",
        "If you gain control of a creature \"until end of turn,\" or if a noncreature permanent you control becomes a creature \"until end of turn,\" it'll be a creature under your control through the entire end step."
    );
    let rites_name = "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun";
    let transformed = |t: &TestGame, r: ObjectId| t.obj_now(r).chars.name.as_str() == "Itlimoc, Cradle of the Sun";
    // Ball Lightning (sacrificed at the beginning of the end step) is the fourth creature:
    // ordering Growing Rites' trigger to resolve first transforms it.
    supported("Ball Lightning");
    let mut results = vec![];
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let rites = t.battlefield(P0, rites_name);
        bears(&mut t, P0, 3);
        t.battlefield(P0, "Ball Lightning");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Ball Lightning"));
        results.push(transformed(&t, rites));
    }
    assert!(results.contains(&true), "{results:?}");

    // A creature P0 gained control of until end of turn is the fourth creature.
    supported("Threaten");
    let mut t = TestGame::new(2);
    let rites = t.battlefield(P0, rites_name);
    bears(&mut t, P0, 3);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Threaten", &[obj(theirs)]);
    assert_eq!(t.obj_now(theirs).controller, P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(transformed(&t, rites));
}

// ---------------------------------------------------------------------------
// Others
// ---------------------------------------------------------------------------

#[test]
fn starlight_spectaculars_each_line_up_the_creatures_separately() {
    cr!("608.2c", "613.4c");
    ruling!(
        "Starlight Spectacular",
        "If you control more than one Starlight Spectacular, each one calculates its bonuses separately."
    );
    supported("Starlight Spectacular");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Starlight Spectacular");
    t.battlefield(P0, "Starlight Spectacular");
    let b = bears(&mut t, P0, 3);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let mut bonus: Vec<i32> = b.iter().map(|c| t.pt(*c).0 - 2).collect();
    bonus.sort();
    let total: i32 = bonus.iter().sum();
    assert_eq!(total, 6, "{bonus:?}: 0+1+2 from each");
    assert!(bonus.iter().all(|x| (0..=4).contains(x)));
}

#[test]
fn kindred_summons_puts_what_it_found_when_the_library_runs_out() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Kindred Summons",
        "If you reveal your entire library and reveal fewer than X creature cards of the chosen type, you'll put the cards of the chosen type that you did reveal onto the battlefield and shuffle your library."
    );
    supported("Kindred Summons");
    let mut t = TestGame::new(2);
    bears(&mut t, P0, 3);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Bear");
    let lib = t.library_size(P0);
    cast_resolve(&mut t, P0, "Kindred Summons", &[]);
    assert_eq!(creature_count(&t, P0), 4);
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn plagon_changes_only_the_combat_damage_assigned() {
    cr!("510.1a", "208.3");
    ruling!(
        "Plagon, Lord of the Beach",
        "Plagon's last ability doesn't actually change the target creature's power. It changes only the amount of combat damage the creature assigns."
    );
    supported("Plagon, Lord of the Beach");
    let mut t = TestGame::new(2);
    let plagon = t.battlefield(P0, "Plagon, Lord of the Beach");
    mana_all(&mut t, P0, 1);
    t.activate(P0, plagon, 0, &[obj(plagon)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(plagon), (0, 3));
    attack_with(&mut t, &[(plagon, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(plagon).0, 0);
    let _ = create_token;
}
