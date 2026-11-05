//! Rulings batch P026 — the rest of the copy-creature batch: copies of spells that aren't
//! cast and tokens that aren't "created" (CR 707.10, 608.3f), copies cast from exile (CR
//! 707.12), replacement effects that turn created tokens into copies before other effects
//! modify how they enter (CR 614.12, 616.1), copies entering together (CR 603.6a),
//! triggers of copy effects, and other abilities of the batch's cards.

use crate::r_p026_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts P0's Kaervek trigger (from casting Lightning Bolt at P1) on the stack with Night's
/// Whisper ({1}{B}) in P0's graveyard; P0 casts the copy. Returns the number of P0's lands
/// tapped for the copy.
fn kaervek_copy_lands(interrupter: bool) -> usize {
    supported("Kaervek, the Punisher");
    let mut t = TestGame::new(2);
    if interrupter {
        t.battlefield(P1, "Aven Interrupter");
    }
    t.battlefield(P0, "Kaervek, the Punisher");
    let whisper = t.graveyard(P0, "Night's Whisper");
    t.lands(P0, "Badlands", 6);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[obj(whisper)]);
    t.settle();
    let before = crate::r_s01_common::tapped_lands(&t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Night's Whisper");
    crate::r_s01_common::tapped_lands(&t, P0) - before
}

#[test]
fn kaervek_casts_the_copy_from_exile() {
    cr!("707.12", "601.2f");
    ruling!(
        "Kaervek, the Punisher",
        "The copy is created in and cast from exile, so effects such as that of Aven Interrupter’s last ability will apply."
    );
    // Aven Interrupter: "Spells your opponents cast from graveyards or from exile cost {2}
    // more to cast."
    assert_eq!(kaervek_copy_lands(false), 2);
    assert_eq!(kaervek_copy_lands(true), 4);
}

/// P0's solved Case of the Shifting Visage ("Whenever you cast a nonlegendary creature
/// spell, copy that spell.").
fn solved_visage(t: &mut TestGame) {
    supported("Case of the Shifting Visage");
    let case = t.battlefield(P0, "Case of the Shifting Visage");
    assert!(mtg_engine::cases::solve(&mut t.g, case));
    t.g.recompute();
}

#[test]
fn shifting_visage_copy_isnt_cast() {
    cr!("707.10", "603.2");
    ruling!(
        "Case of the Shifting Visage",
        "The copy is created on the stack, so it’s not “cast.” Creating the copy won’t cause abilities that trigger when a player casts a spell to trigger."
    );
    // Beast Whisperer: "Whenever you cast a creature spell, draw a card."
    let mut t = TestGame::new(2);
    solved_visage(&mut t);
    t.battlefield(P0, "Beast Whisperer");
    let hand = t.hand_size(P0);
    lands_for(&mut t, P0, "{3}{R}");
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    // One card drawn, for the one spell cast (the Giant was put into the hand after
    // `hand` was counted, then cast).
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tokens_from_resolving_spell_copies_arent_created() {
    cr!("608.3f", "707.10f", "111.1");
    ruling!(
        "Case of the Shifting Visage",
        "The token that a resolving copy of a permanent spell becomes isn’t “created” and won’t interact with abilities that care about tokens being created, such as that of Case of the Pilfered Proof."
    );
    ruling!(
        "Double Major",
        "The token that a resolving copy of a spell becomes isn’t said to have been “created.”"
    );
    // Case of the Pilfered Proof, solved: "If one or more tokens would be created under
    // your control, those tokens plus a Clue token are created instead."
    let mut t = TestGame::new(2);
    solved_visage(&mut t);
    let proof = t.battlefield(P0, "Case of the Pilfered Proof");
    assert!(mtg_engine::cases::solve(&mut t.g, proof));
    t.g.recompute();
    lands_for(&mut t, P0, "{3}{R}");
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Clue").is_empty());
    // Double Major ("Copy target creature spell you control, except it isn't legendary if
    // the spell is legendary.") with Parallel Lives ("If an effect would create one or
    // more tokens under your control, it creates twice that many of those tokens
    // instead."): one token.
    supported("Double Major");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    lands_for(&mut t, P0, "{3}{R}");
    let giant = t.hand(P0, "Hill Giant");
    let spell = t.cast(P0, giant).go();
    lands_for(&mut t, P0, "{G}{U}");
    let dm = t.hand(P0, "Double Major");
    t.cast(P0, dm).target(obj(spell)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
}

#[test]
fn the_fire_crystal_reduces_only_generic_mana_of_red_spells() {
    cr!("601.2f", "118.7d");
    ruling!(
        "The Fire Crystal",
        "The cost reduction applies only to generic mana in the total cost of red spells you cast."
    );
    supported("The Fire Crystal");
    // Hill Giant ({3}{R}) costs {2}{R}; Lightning Bolt ({R}) still costs {R}; Grizzly
    // Bears ({1}{G}) isn't red.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fire Crystal");
    t.lands(P0, "Mountain", 3);
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fire Crystal");
    t.lands(P0, "Wastes", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fire Crystal");
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn nascent_metamorph_copied_card_goes_to_the_bottom_too() {
    cr!("707.2", "701.20a");
    ruling!(
        "Nascent Metamorph",
        "The creature card that Nascent Metamorph becomes a copy of is put onto the bottom of the library along with any other cards."
    );
    // P1's library: Shock, then Hill Giant, on top.
    supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Nascent Metamorph");
    let giant = t.library_top(P1, "Hill Giant");
    let shock = t.library_top(P1, "Shock");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(m, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(name_of(&t, m), "Hill Giant");
    let bottom = crate::r_s15_common::library_bottom_n(&t, P1, 2);
    let names: Vec<String> = bottom.iter().map(|id| name_of(&t, *id)).collect();
    assert!(names.contains(&"Hill Giant".to_string()), "{names:?}");
    assert!(names.contains(&"Shock".to_string()), "{names:?}");
    let _ = (giant, shock);
}

/// P0's Sauron attacks and exiles Serra Angel from P0's graveyard; returns (Sauron, the
/// token). Stops at the end of combat.
fn sauron_attacks(t: &mut TestGame, ring_bearer: bool) -> (ObjectId, ObjectId) {
    supported("Sauron, the Necromancer");
    let sauron = t.battlefield(P0, "Sauron, the Necromancer");
    if ring_bearer {
        mtg_engine::kwa::ring::tempt(&mut t.g, P0, None);
        assert!(mtg_engine::kwa::ring::is_ring_bearer(&t.g, P0, sauron));
    }
    let card = t.graveyard(P0, "Serra Angel");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(sauron, Entity::Player(P1))], &[]);
    let tok = one(new_tokens(t, P0, &before));
    (sauron, tok)
}

fn to_end_step(t: &mut TestGame) {
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn sauron_exiles_the_token_unless_its_your_ring_bearer_as_the_ability_resolves() {
    cr!("603.7", "701.54a", "608.2h");
    ruling!(
        "Sauron, the Necromancer",
        "The delayed triggered ability will exile the token at the beginning of your next end step unless Sauron, the Necromancer is your Ring-bearer at the time that the ability resolves. If Sauron, the Necromancer leaves the battlefield before the delayed triggered ability resolves, that ability will exile the token, even if Sauron, the Necromancer later returns to the battlefield or you control another copy of Sauron, the Necromancer as your Ring-bearer."
    );
    // Not the Ring-bearer: exiled.
    let mut t = TestGame::new(2);
    let (_, tok) = sauron_attacks(&mut t, false);
    to_end_step(&mut t);
    assert!(!t.on_battlefield(tok));
    // The Ring-bearer: it stays.
    let mut t = TestGame::new(2);
    let (_, tok) = sauron_attacks(&mut t, true);
    to_end_step(&mut t);
    assert!(t.on_battlefield(tok));
    // Becomes the Ring-bearer after the attack, before the end step: it stays.
    let mut t = TestGame::new(2);
    let (sauron, tok) = sauron_attacks(&mut t, false);
    t.answer_choose(P0, &[obj(sauron)]);
    mtg_engine::kwa::ring::tempt(&mut t.g, P0, None);
    assert!(mtg_engine::kwa::ring::is_ring_bearer(&t.g, P0, sauron));
    to_end_step(&mut t);
    assert!(t.on_battlefield(tok));
    // It leaves and returns (a new object), which becomes the Ring-bearer: exiled anyway.
    let mut t = TestGame::new(2);
    let (sauron, tok) = sauron_attacks(&mut t, true);
    let gone =
        t.g.move_object(
            sauron,
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    let back =
        t.g.move_object(
            gone,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    t.settle();
    t.answer_choose(P0, &[obj(back)]);
    mtg_engine::kwa::ring::tempt(&mut t.g, P0, None);
    assert!(mtg_engine::kwa::ring::is_ring_bearer(&t.g, P0, back));
    to_end_step(&mut t);
    assert!(!t.on_battlefield(tok));
}

#[test]
fn zndrsplts_judgment_friend_or_foe_is_chosen_for_each_spell() {
    cr!("608.2c", "102.3");
    ruling!(
        "Zndrsplt's Judgment",
        "The designation of friend or foe is only relevant to the spell that asks you to choose. A player you call your friend doesn't become your teammate, and the next “friend or foe” spell you cast could name that player your foe."
    );
    // "For each player, choose friend or foe. Each friend creates a token that's a copy of
    // a creature they control. Each foe returns a creature they control to its owner's
    // hand." First P1 is a friend, then a foe.
    supported("Zndrsplt's Judgment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let cast = |t: &mut TestGame, p1_friend: bool| {
        // Options: 0 = friend, 1 = foe; asked for P0, then P1.
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if p1_friend { 0 } else { 1 }),
        );
        lands_for(t, P0, "{4}{U}");
        let j = t.hand(P0, "Zndrsplt's Judgment");
        t.cast(P0, j).go();
        t.resolve_all();
    };
    cast(&mut t, true);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    assert!(t.on_battlefield(giant));
    // P1 didn't become P0's teammate.
    assert!(t.g.are_opponents(P0, P1));
    cast(&mut t, false);
    // As a foe, P1 returns one of its Giants (the token ceases to exist or the card goes
    // to hand).
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

/// P1's Thalia, Heretic Cathar ("Creatures and nonbasic lands your opponents control
/// enter tapped.") and P0's Strike It Rich ("Create a Treasure token."), with P0's
/// replacement `setup`; returns the token P0 got.
fn treasure_replaced_by_copy(setup: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Heretic Cathar");
    let giant = t.battlefield(P0, "Hill Giant");
    setup(&mut t, giant);
    let before = t.g.battlefield.clone();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    lands_for(&mut t, P0, "{R}");
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    (t, tok)
}

#[test]
fn copy_replacements_apply_before_effects_on_how_tokens_enter() {
    cr!("614.12", "616.1", "707.5");
    ruling!(
        "Moonlit Meditation",
        "The effect of Moonlit Meditation’s last ability applies before anything that modifies how those tokens enter the battlefield."
    );
    ruling!(
        "Esix, Fractal Bloom",
        "This effect applies before anything that modifies how those tokens enter the battlefield."
    );
    // The Treasure becomes a Hill Giant: a creature, so it enters tapped because of
    // Thalia.
    supported("Moonlit Meditation");
    let (t, tok) = treasure_replaced_by_copy(|t, giant| {
        attach_new(t, P0, "Moonlit Meditation", giant);
    });
    assert_eq!(name_of(&t, tok), "Hill Giant");
    assert!(t.obj_now(tok).tapped);
    supported("Esix, Fractal Bloom");
    let (t, tok) = treasure_replaced_by_copy(|t, _| {
        t.battlefield(P0, "Esix, Fractal Bloom");
    });
    assert_eq!(name_of(&t, tok), "Hill Giant");
    assert!(t.obj_now(tok).tapped);
}

#[test]
fn progenitor_mimic_tokens_dont_make_tokens() {
    cr!("603.4", "707.9a");
    ruling!(
        "Progenitor Mimic",
        "The original Progenitor Mimic will create tokens, but those token copies will not."
    );
    // "...except it has "At the beginning of your upkeep, if this creature isn't a token,
    // create a token that's a copy of this creature.""
    supported("Progenitor Mimic");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let m = enter_copying(&mut t, P0, "Progenitor Mimic", giant);
    assert_eq!(name_of(&t, m), "Hill Giant");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // Two upkeeps: two tokens, all from the original.
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 2);
}

#[test]
fn silent_hallcreeper_forgets_its_modes_when_it_leaves() {
    cr!("400.7", "700.2", "510.2");
    ruling!(
        "Silent Hallcreeper",
        "The phrase \"that hasn't been chosen\" refers only to that specific Silent Hallcreeper. If Silent Hallcreeper leaves the battlefield and then returns to the battlefield later, it will be a new object with no memory of the modes that were chosen when it was last on the battlefield."
    );
    supported("Silent Hallcreeper");
    let mut t = TestGame::new(2);
    let creeper = t.battlefield(P0, "Silent Hallcreeper");
    // Mode 1: "Draw a card."
    let attack_drawing = |t: &mut TestGame, c: ObjectId| -> bool {
        let hand = t.hand_size(P0);
        let life = t.life(P1);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(t.g.current(c), Entity::Player(P1))], &[]);
        t.resolve_all();
        t.clear_answers();
        assert!(t.life(P1) < life, "no combat damage");
        let drew = t.hand_size(P0) == hand + 1;
        // On to P0's next turn (the creatures untap; the draw step draws).
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        drew
    };
    assert!(attack_drawing(&mut t, creeper));
    // Again with the same object: "Draw a card" can't be chosen again.
    assert!(!attack_drawing(&mut t, creeper));
    // Another Silent Hallcreeper can choose it.
    let other = t.battlefield(P0, "Silent Hallcreeper");
    assert!(attack_drawing(&mut t, other));
    // It leaves and returns: a new object that can choose it again.
    let gone =
        t.g.move_object(
            creeper,
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    let back =
        t.g.move_object(
            gone,
            Zone::Battlefield,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    unsick(&mut t, back);
    assert!(attack_drawing(&mut t, back));
}

#[test]
fn haunting_imitation_revealed_cards_stay_on_top() {
    cr!("701.20a");
    ruling!(
        "Haunting Imitation",
        "The revealed cards stay on the top of their owners' libraries."
    );
    supported("Haunting Imitation");
    let mut t = TestGame::new(2);
    let giant = t.library_top(P0, "Hill Giant");
    let shock = t.library_top(P1, "Shock");
    let toks = cast_for_tokens(&mut t, P0, "Haunting Imitation", "{2}{U}", &[], None, None);
    let tok = one(toks);
    assert_eq!(name_of(&t, tok), "Hill Giant");
    assert_eq!(t.g.player(P0).library.last(), Some(&giant));
    assert_eq!(t.g.player(P1).library.last(), Some(&shock));
}

#[test]
fn the_eleventh_hour_human_reduces_spells_with_the_doctor_type() {
    cr!("205.3m", "601.2f");
    ruling!(
        "The Eleventh Hour",
        "The same is true for reducing the cost of Doctor spells."
    );
    // Chapter II: "Create a Food token and a 1/1 white Human creature token with "Doctor
    // spells you cast cost {1} less to cast."" Cult Healer ({2}{W}) is a Human Doctor;
    // Doctor Doom, Unrivaled ({2}{B}{B}) only has "Doctor" in its name.
    supported("The Eleventh Hour");
    let human = |t: &mut TestGame| {
        let saga = t.enter(P0, "The Eleventh Hour");
        t.settle();
        t.resolve_all();
        t.g.add_counters(obj(t.g.current(saga)), counters::LORE, 1, None);
        t.g.flush_events();
        t.resolve_all();
        assert!(!crate::r_s01_common::with_subtype(t, P0, "Human").is_empty());
    };
    let mut t = TestGame::new(2);
    human(&mut t);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let healer = t.hand(P0, "Cult Healer");
    assert!(t.cast(P0, healer).try_go().is_ok());
    let mut t = TestGame::new(2);
    human(&mut t);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 1);
    let doom = t.hand(P0, "Doctor Doom, Unrivaled");
    assert!(t.cast(P0, doom).try_go().is_err());
}

#[test]
fn myriad_defending_player_is_the_one_the_creature_attacks() {
    cr!("702.116a", "508.5", "508.5a");
    ruling!(
        "Auton Soldier",
        "The term \"defending player\" in the myriad rules (or any other ability of an attacking creature) refers to the player the creature with myriad was attacking at the time it became an attacking creature this combat, or the controller of the planeswalker or protector of the battle the creature was attacking at the time it became an attacking creature this combat."
    );
    // Auton Soldier enters as a copy of Hill Giant with myriad. Attacking P2's
    // planeswalker, P2 is the defending player: tokens attack P1 and P3, not P2.
    supported("Auton Soldier");
    let mut t = TestGame::new(4);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    let a = enter_copying(&mut t, P0, "Auton Soldier", giant);
    unsick(&mut t, a);
    assert_eq!(name_of(&t, a), "Hill Giant");
    let pw = t.battlefield(P2, "Gideon Jura");
    t.set_step(P0, Step::BeginningOfCombat);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let before = t.g.battlefield.clone();
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, obj(pw))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    let attacked: Vec<Entity> =
        t.g.combat
            .as_ref()
            .unwrap()
            .attackers
            .iter()
            .filter(|x| toks.contains(&x.id))
            .filter_map(|x| x.target)
            .collect();
    assert!(!attacked.contains(&Entity::Player(P2)) && !attacked.contains(&obj(pw)));
    assert!(attacked.contains(&Entity::Player(P1)) && attacked.contains(&Entity::Player(P3)));
}

#[test]
fn echo_chamber_token_enters_and_triggers_enters_abilities() {
    cr!("707.5", "603.6a");
    ruling!(
        "Echo Chamber",
        "The token enters as the creature it is copying. This triggers its own and any other appropriate \"enters\" abilities."
    );
    // Elvish Visionary: "When this creature enters, draw a card." Soul Warden: "Whenever
    // another creature enters, you gain 1 life."
    supported("Echo Chamber");
    let mut t = TestGame::new(2);
    let chamber = t.battlefield(P0, "Echo Chamber");
    t.battlefield(P0, "Soul Warden");
    let ev = t.battlefield(P1, "Elvish Visionary");
    lands_for(&mut t, P0, "{4}");
    t.answer_targets(P1, &[obj(ev)]);
    let hand = t.hand_size(P0);
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        chamber,
        "Create a token",
        &[],
    ));
    assert_eq!(name_of(&t, tok), "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn twinflame_tokens_see_each_other_enter() {
    cr!("603.6a", "603.2");
    ruling!(
        "Twinflame",
        "The tokens see each other enter the battlefield. If any of them have a triggered ability that triggers whenever a creature enters the battlefield, they'll trigger for one another."
    );
    // Two Soul Wardens ("Whenever another creature enters, you gain 1 life."): each
    // original sees both tokens (4) and each token sees the other (2).
    supported("Twinflame");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Soul Warden");
    let b = t.battlefield(P0, "Soul Warden");
    lands_for(&mut t, P0, "{1}{R}{2}{R}");
    let tf = t.hand(P0, "Twinflame");
    let before = t.g.battlefield.clone();
    t.cast(P0, tf).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn vesuvan_duplimancy_trigger_needs_a_single_target_and_resolves_first() {
    cr!("603.2", "603.3", "115.1");
    ruling!(
        "Vesuvan Duplimancy",
        "The triggered ability of Vesuvan Duplimancy triggers only if you cast a spell that targets a single artifact or a single creature and nothing else. The ability will resolve before the spell that caused it to trigger, and a token will be created even if the spell you cast was countered or permanent it was targeting is now an illegal target."
    );
    supported("Vesuvan Duplimancy");
    // Giant Growth on the Giant, countered by the opponent after the trigger resolves...
    // the trigger is put on the stack above it and resolves first; then Cancel counters
    // Giant Growth: the token is there anyway.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    let spell = t.cast(P0, gg).target(obj(giant)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    lands_for(&mut t, P1, "{1}{U}{U}");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(spell)).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
    assert_eq!(t.pt(giant), (3, 3));
    // Polymorphous Rush targeting two of P0's creatures: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{U}{1}{U}");
    let pr = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, pr).targets(&[obj(giant), obj(bears)]).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Shock targeting the opponent's creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for(&mut t, P0, "{R}");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(obj(bears)).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn rally_the_galadhrim_conspire_trigger_can_be_countered() {
    cr!("702.78a", "701.6a");
    ruling!(
        "Rally the Galadhrim",
        "The triggered ability that creates the copy can itself be countered by anything that can counter a triggered ability. If it is countered, no copies will be put onto the stack."
    );
    // Conspire paid by tapping two Grizzly Bears; P1 Stifles the conspire trigger.
    supported("Rally the Galadhrim");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for(&mut t, P0, "{2}{G}{U}");
    let rally = t.hand(P0, "Rally the Galadhrim");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.cast(P0, rally).target(obj(giant)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let trigger = *t.g.stack.last().unwrap();
    lands_for(&mut t, P1, "{U}");
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(obj(trigger)).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
}
