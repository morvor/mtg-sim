//! Rulings batch P129: the other abilities of cards with reflexive triggered abilities, and
//! what their reflexive abilities create or change (types, copies, control, phasing).

use crate::r_p129_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn terror_of_towashi_makes_a_lasting_phyrexian_only_while_a_creature() {
    cr!("603.12", "205.3d", "611.2a");
    ruling!(
        "Terror of Towashi",
        "The effect making the creature a Phyrexian doesn’t have a duration."
    );
    ruling!(
        "Terror of Towashi",
        "If the creature stops being a creature after the ability has resolved, it will also stop being a Phyrexian"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let terror = t.battlefield(P0, "Terror of Towashi");
    let god = t.graveyard(P0, "Heliod, God of the Sun");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(god)]);
    attack_with(&mut t, &[(terror, Entity::Player(P1))]);
    t.resolve_all();
    let god = t.g.current(god);
    assert!(t.on_battlefield(god));
    // Devotion to white below five: Heliod isn't a creature, so not a Phyrexian.
    assert!(!is_creature_now(&t, god));
    assert!(!has_subtype(&t, god, "Phyrexian"));
    // With enough devotion it's a creature again, and a Phyrexian again.
    for _ in 0..4 {
        t.battlefield(P0, "Benalish Knight");
    }
    t.g.recompute();
    assert!(is_creature_now(&t, god));
    assert!(has_subtype(&t, god, "Phyrexian"));
    // No duration: it lasts through later turns.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(has_subtype(&t, god, "Phyrexian"));
}

#[test]
fn saheeli_copies_only_the_copiable_values() {
    cr!("603.12", "707.2", "707.9b");
    ruling!(
        "Saheeli, Radiant Creator",
        "It doesn’t copy whether that permanent is tapped or untapped, whether it has any counters on it"
    );
    ruling!(
        "Saheeli, Radiant Creator",
        "If the copied permanent has {X} in its mana cost, X is 0."
    );
    supported("Saheeli, Radiant Creator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Saheeli, Radiant Creator");
    give_energy(&mut t, P0, 3);
    let walker = t.battlefield(P0, "Hangarback Walker");
    put_counters(&mut t, walker, counters::PLUS1, 2);
    t.g.tap(walker);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(walker)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    let copies: Vec<_> = tokens(&t, P0);
    assert_eq!(copies.len(), 1);
    let c = copies[0];
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Hangarback Walker");
    assert!(!t.obj_now(c).tapped);
    assert_eq!(t.counters(c, counters::PLUS1), 0);
    assert_eq!(t.pt(c), (5, 5));
    assert_eq!(t.g.mana_value_of(c), 0);
}

#[test]
fn slinza_reduces_generic_mana_and_its_abilities() {
    cr!("601.2f", "202.3", "614.1c", "603.2");
    ruling!(
        "Slinza, the Spiked Stampede",
        "Slinza's first ability can't reduce the amount of colored mana you pay for a spell."
    );
    ruling!(
        "Slinza, the Spiked Stampede",
        "Slinza's first ability doesn't change the mana cost or mana value of any spell."
    );
    ruling!(
        "Slinza, the Spiked Stampede",
        "Slinza's last ability triggers whenever any creature with power 4 or greater enters, not just one you control."
    );
    ruling!(
        "Slinza, the Spiked Stampede",
        "Slinza's second ability applies to all Beasts creatures you control as they enter"
    );
    supported("Slinza, the Spiked Stampede");
    // Ravenous Baloth ({2}{G}{G}) costs {G}{G}: one Forest isn't enough, two are.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slinza, the Spiked Stampede");
    t.lands(P0, "Forest", 1);
    let baloth = t.hand(P0, "Ravenous Baloth");
    assert!(!can_cast(&mut t, P0, baloth, mtg_engine::object::CastMethod::Normal));
    t.lands(P0, "Forest", 1);
    assert!(can_cast(&mut t, P0, baloth, mtg_engine::object::CastMethod::Normal));
    t.answer_yes(P0, false);
    let spell = t.cast(P0, baloth).go();
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    // It enters with a +1/+1 counter.
    let b = t.named_on_battlefield("Ravenous Baloth")[0];
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // An opponent's power-4 creature entering triggers it too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slinza, the Spiked Stampede");
    t.lands(P0, "Forest", 2);
    let giant = t.enter(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // Slinza (5/5) fights the 6/4.
    assert!(!t.on_battlefield(giant));
}

#[test]
fn spectral_adversary_phased_out_permanents_dont_exist() {
    cr!("603.12", "702.26b", "702.26d");
    ruling!(
        "Spectral Adversary",
        "Permanents that are phased out are treated as though they don't exist."
    );
    supported("Spectral Adversary");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(giant), (4, 4));
    for yes in [true, true, true, false] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(anthem), obj(bears)]);
    let adv = t.enter(P0, "Spectral Adversary");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(adv, counters::PLUS1), 2);
    assert!(t.obj_now(bears).phased_out);
    // The Anthem's static ability has no effect; Grizzly Bears can't be targeted.
    t.g.recompute();
    assert_eq!(t.pt(giant), (3, 3));
    assert!(!spell_targets(&mut t, P0, "Lightning Bolt").contains(&obj(bears)));
}

#[test]
fn yes_man_control_and_ownership_when_players_leave() {
    cr!("800.4a", "800.4c", "603.12");
    ruling!(
        "Yes Man, Personal Securitron",
        "When Yes Man's controller leaves the game, if that player wasn't its owner, the effect giving that player control of Yes Man ends."
    );
    ruling!(
        "Yes Man, Personal Securitron",
        "When Yes Man's owner leaves the game, Yes Man leaves the game along with that player."
    );
    supported("Yes Man, Personal Securitron");
    let mut t = TestGame::new(3);
    let ym = t.battlefield(P0, "Yes Man, Personal Securitron");
    let hand = t.hand_size(P0);
    t.activate(P0, ym, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ym).controller, P1);
    assert_eq!(t.hand_size(P0), hand + 2);
    t.g.lose_game(P1);
    t.settle();
    assert_eq!(t.obj_now(ym).controller, P0);
    // Its owner leaves: so does it.
    let mut t = TestGame::new(3);
    let ym = t.battlefield(P0, "Yes Man, Personal Securitron");
    t.activate(P0, ym, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Yes Man, Personal Securitron").is_empty());
}

#[test]
fn rise_of_sozin_ignores_permanents_with_the_name() {
    cr!("714.2b", "701.23a");
    ruling!(
        "The Rise of Sozin // Fire Lord Sozin",
        "This ability has no effect on permanents with the chosen name that are on the battlefield."
    );
    supported("The Rise of Sozin // Fire Lord Sozin");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Rise of Sozin // Fire Lord Sozin");
    put_counters(&mut t, saga, counters::LORE, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let saga = t.g.current(saga);
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.in_hand(P1, "Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn motherlode_counts_nonbasic_lands_as_its_ability_resolves() {
    cr!("608.2h", "107.14");
    ruling!(
        "The Motherlode, Excavator",
        "Use the number of nonbasic lands the target opponent controls when The Motherlode's first ability resolves"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Mutavault");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "The Motherlode, Excavator");
    t.settle();
    // Another nonbasic land arrives before the ability resolves.
    t.battlefield(P1, "Mutavault");
    t.lands(P1, "Forest", 2);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    // Later changes don't matter.
    t.battlefield(P1, "Mutavault");
    assert_eq!(energy(&t, P0), 2);
}

#[test]
fn fugitive_doctor_flashback_x_is_zero() {
    cr!("702.34a", "107.3b", "603.12");
    ruling!(
        "The Fugitive Doctor",
        "If you cast an instant or sorcery with {X} in its mana cost this way, you must choose 0 as the value of X."
    );
    supported("The Fugitive Doctor");
    let mut t = TestGame::new(2);
    let doc = t.battlefield(P0, "The Fugitive Doctor");
    create_token(&mut t, P0, "Clue");
    let blaze = t.graveyard(P0, "Blaze");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[obj(blaze)]);
    attack_with(&mut t, &[(doc, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    let life = t.life(P1);
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 1);
    let blaze = t.g.current(blaze);
    let spell = t
        .cast(P0, blaze)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .x(3)
        .target(P1)
        .go();
    assert_eq!(x_of(&t, spell), Some(0));
    t.resolve_all();
    assert_eq!(t.life(P1), life);
    assert!(t.in_exile("Blaze"));
}

#[test]
fn halo_forager_casts_during_resolution_ignoring_timing() {
    cr!("603.12", "608.2g", "601.2");
    ruling!(
        "Halo Forager",
        "Timing restrictions based on the card's type are ignored."
    );
    supported("Halo Forager");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let div = t.graveyard(P1, "Divination");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(div)]);
    let hand = t.hand_size(P0);
    // During the opponent's turn.
    t.advance_to(P1, Step::Upkeep);
    t.enter(P0, "Halo Forager");
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_exile("Divination"));
}

#[test]
fn rubble_rouser_mana_ability_and_its_reflexive_trigger() {
    cr!("605.1a", "605.3a", "603.12");
    ruling!(
        "Rubble Rouser",
        "Rubble Rouser's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Rubble Rouser",
        "When you activate Rubble Rouser's mana ability, it won't deal damage to each opponent right away."
    );
    supported("Rubble Rouser");
    let mut t = TestGame::new(2);
    let rr = t.battlefield(P0, "Rubble Rouser");
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Island");
    // Activated directly: the mana is added at once; the reflexive ability then triggers.
    t.activate(P0, rr, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0)[3], 1);
    assert_eq!(t.life(P1), 20);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn borborygmos_and_fblthp_goes_into_the_library() {
    cr!("401.7", "602.2", "113.6");
    ruling!(
        "Borborygmos and Fblthp",
        "If there are one or fewer cards in your library, Borborygmos and Fblthp is put on the bottom of your library."
    );
    ruling!(
        "Borborygmos and Fblthp",
        "Only the controller of Borborygmos and Fblthp can activate its last ability, and only if it’s on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    t.library_top(P0, "Island");
    t.lands(P0, "Island", 2);
    let bf = t.battlefield(P0, "Borborygmos and Fblthp");
    // The opponent can't activate it.
    t.lands(P1, "Island", 2);
    assert!(t.activate(P1, bf, 0, &[]).is_err());
    t.activate(P0, bf, 0, &[]).unwrap();
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    assert_eq!(lib.len(), 2);
    let bottom = *lib.first().unwrap();
    let top_ = *lib.last().unwrap();
    let names: Vec<String> = [bottom, top_]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert!(names.contains(&"Borborygmos and Fblthp".to_string()));
    assert!(names.contains(&"Island".to_string()));
    // Not from the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let gy = t.graveyard(P0, "Borborygmos and Fblthp");
    assert!(t.activate(P0, gy, 0, &[]).is_err());
}

#[test]
fn pyroclastic_hellion_in_two_headed_giant() {
    cr!("810.9a", "603.12");
    ruling!(
        "Pyroclastic Hellion",
        "In a Two-Headed Giant game, Pyroclastic Hellion’s ability causes the opposing team to lose 4 life."
    );
    supported("Pyroclastic Hellion");
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    let land = t.lands(P0, "Mountain", 1)[0];
    let before = t.life(P2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(land)]);
    t.enter(P0, "Pyroclastic Hellion");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P2), before - 4);
    assert_eq!(t.life(P3), before - 4);
    let _ = ManaType::R;
}
