//! Rulings batch P141 — the rest: "no abilities" (Ruxa), anthems counting a creature once
//! (Kirri), untargeted graveyard choices made on resolution (Charmbreaker Devils,
//! Deadbridge Chant), activation restrictions checked only on activation (The
//! Biblioplex), priority after an attack trigger (Ethereal Forager), a planeswalker
//! becoming a creature while attacked (Grand Master of Flowers), drawing from an empty
//! library (Jace, Jace's Archivist), and exiled cards staying exiled (Skyship
//! Weatherlight).

use crate::r_p141_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn ruxa_counts_only_creatures_with_no_rules_text() {
    cr!("113.1", "702.29a", "613.4c");
    ruling!(
        "Ruxa, Patient Professor",
        "A creature or creature card with no abilities is one with no rules text at all."
    );
    supported("Ruxa, Patient Professor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruxa, Patient Professor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tok = create_token(&mut t, P0, "Soldier");
    let thopter = t.battlefield(P0, "Ornithopter");
    let hund = t.battlefield(P0, "Hundroog");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.pt(thopter), (0, 2));
    assert_eq!(t.pt(hund), (4, 7), "cycling is an ability");
    // A creature that lost its abilities counts.
    crate::r_s26_common::modify_until_eot(&mut t, thopter, vec![Modification::RemoveAllAbilities]);
    assert_eq!(t.pt(thopter), (1, 3));
    // Its enters trigger can target a vanilla card, not a cycling one.
    let mut t = TestGame::new(2);
    let b = t.graveyard(P0, "Grizzly Bears");
    let h = t.graveyard(P0, "Hundroog");
    let from = n_asked(&t);
    enter(&mut t, P0, "Ruxa, Patient Professor");
    assert!(offered_target(&t, P0, from, b));
    assert!(!offered_target(&t, P0, from, h));
}

#[test]
fn kirri_gives_a_plant_treefolk_plus_two_once() {
    cr!("613.4c", "205.3");
    ruling!(
        "Kirri, Talented Sprout",
        "A creature that’s both a Plant and a Treefolk gets only +2/+0 from Kirri’s first ability."
    );
    supported("Kirri, Talented Sprout");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kirri, Talented Sprout");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(b), (2, 2));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        b,
        vec![Modification::AddSubtypes(vec!["Plant".into(), "Treefolk".into()])],
    );
    assert_eq!(t.pt(b), (4, 2));
}

#[test]
fn untargeted_graveyard_choices_see_cards_put_there_in_response() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Charmbreaker Devils",
        "Because the first ability doesn't target the instant or sorcery card, any instants or sorceries put into your graveyard in response to that ability may be returned to your hand."
    );
    ruling!(
        "Deadbridge Chant",
        "Because the last ability doesn't target any card in your graveyard, any card put into your graveyard in response to the ability may be chosen."
    );
    supported("Charmbreaker Devils");
    supported("Deadbridge Chant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charmbreaker Devils");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
    let opt = t.hand(P0, "Opt");
    discard(&mut t, P0, opt);
    t.resolve_all();
    assert!(t.in_hand(P0, "Opt"));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deadbridge Chant");
    into_upkeep(&mut t, P0);
    assert_eq!(stacked_triggers(&t), 1);
    let b = t.hand(P0, "Grizzly Bears");
    discard(&mut t, P0, b);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn deadbridge_chant_chooses_uniformly_at_random() {
    cr!("608.2d");
    ruling!(
        "Deadbridge Chant",
        "Each card in your graveyard must have an equal chance of being chosen."
    );
    let (mut creature, mut other) = (0, 0);
    for seed in 0..24u64 {
        let mut t = TestGame::with_config(
            2,
            GameConfig {
                seed,
                ..Default::default()
            },
        );
        t.battlefield(P0, "Deadbridge Chant");
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Opt");
        into_upkeep(&mut t, P0);
        t.resolve_all();
        if t.named_on_battlefield("Grizzly Bears").len() == 1 {
            creature += 1;
        } else {
            assert!(t.in_hand(P0, "Opt"));
            other += 1;
        }
    }
    assert!(creature >= 4 && other >= 4, "{creature} / {other}");
}

#[test]
fn biblioplex_checks_hand_size_only_on_activation() {
    cr!("602.5b", "608.2");
    ruling!(
        "The Biblioplex",
        "Although you must have exactly zero or seven cards in your hand to activate the last ability, it doesn’t matter how many cards you have in your hand as that ability resolves."
    );
    supported("The Biblioplex");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "The Biblioplex");
    t.lands(P0, "Wastes", 2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Opt"]);
    assert_eq!(t.hand_size(P0), 0);
    yes(&mut t, P0);
    t.activate(P0, b, 1, &[]).unwrap();
    // In response, P0 draws a card.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Opt"));
    assert_eq!(t.hand_size(P0), 2);
    // With two cards in hand it can't be activated.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "The Biblioplex");
    t.lands(P0, "Wastes", 2);
    fill_hand(&mut t, P0, 2);
    assert!(t.activate(P0, b, 1, &[]).is_err());
}

#[test]
fn ethereal_forager_returned_card_can_be_cast_before_blocks() {
    cr!("508.2", "117.3b", "702.66a");
    ruling!(
        "Ethereal Forager",
        "After Ethereal Forager’s triggered ability resolves, players will have priority to cast spells (such as the card you returned to your hand, if it’s legal to do so) before blockers are declared."
    );
    supported("Ethereal Forager");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Opt");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Island", 2);
    let f = t.hand(P0, "Ethereal Forager");
    t.cast(P0, f).go();
    t.resolve_all();
    assert!(t.in_exile("Opt"));
    let f = t.named_on_battlefield("Ethereal Forager")[0];
    t.g.objects[f.0 as usize].summoning_sick = false;
    yes(&mut t, P0);
    attack_with(&mut t, &[(f, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Opt"));
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.lands(P0, "Island", 1);
    let opt = t.g.player(P0).hand.iter().copied().find(|id| t.g.obj(*id).chars.name == "Opt").unwrap();
    assert!(crate::r_s02_common::can_cast(&mut t, P0, opt, CastMethod::Normal));
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Opt"));
}

#[test]
fn fell_shepherd_ignores_discarded_creature_cards() {
    cr!("400.7", "603.10");
    ruling!(
        "Fell Shepherd",
        "Creature cards that were put into your graveyard from any zone other than the battlefield, such as a card that was discarded, won’t return to your hand."
    );
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fell Shepherd");
    let died = t.battlefield(P0, "Hill Giant");
    destroy(&mut t, died);
    let h = t.hand(P0, "Grizzly Bears");
    discard(&mut t, P0, h);
    yes(&mut t, P0);
    t.attack(&[(fs, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn skull_of_orm_returns_any_enchantment_card() {
    cr!("205.2a", "303.4");
    ruling!("Skull of Orm", "Can bring back any enchantment.");
    supported("Skull of Orm");
    for name in ["Holy Strength", "Dawn Evangel", "The Tale of Tamiyo", "Necropotence"] {
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, "Skull of Orm");
        t.lands(P0, "Wastes", 5);
        let c = t.graveyard(P0, name);
        t.activate(P0, s, 0, &[obj(c)]).unwrap();
        t.resolve_all();
        assert!(t.in_hand(P0, name), "{name}");
    }
}

#[test]
fn paleoloth_sees_a_big_creature_entering_with_it() {
    cr!("603.6a", "603.10");
    ruling!(
        "Paleoloth",
        "If Paleoloth and another creature with power 5 or greater enter under your control at the same time, Paleoloth's triggered ability will trigger."
    );
    supported("Paleoloth");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    yes(&mut t, P0);
    enter_together(&mut t, P0, &["Paleoloth", "Craw Wurm"]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn grand_master_of_flowers_becoming_a_creature_leaves_combat() {
    cr!("506.4", "506.4c", "306.4");
    ruling!(
        "Grand Master of Flowers",
        "If Grand Master of Flowers somehow gains enough loyalty to become a creature while it is being attacked, it is removed from combat. If blockers haven't been declared yet, it may then block."
    );
    supported("Grand Master of Flowers");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    t.set_step(P1, Step::BeginningOfCombat);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(giant, obj(gm))]);
    put_counters(&mut t, gm, "loyalty", 4);
    assert!(t.obj_now(gm).is(types::CardType::Creature));
    // Unblocked, the Giant deals no combat damage.
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.counters(gm, "loyalty"), 7);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(gm).damage, 0);
    // It may block the creature that was attacking it.
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    t.set_step(P1, Step::BeginningOfCombat);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(giant, obj(gm))]);
    put_counters(&mut t, gm, "loyalty", 4);
    block_and_finish(&mut t, P0, &[(gm, giant)]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn jace_ultimate_on_an_empty_hand_leaves_an_empty_library() {
    cr!("704.5b", "121.4");
    ruling!(
        "Jace, the Mind Sculptor",
        "If the target player for Jace's last ability has no cards in hand, that player shuffles nothing into their library, and that player's library will remain empty. That player won't lose the game until they try to draw from the empty library."
    );
    supported("Jace, the Mind Sculptor");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, the Mind Sculptor");
    put_counters(&mut t, j, "loyalty", 9);
    assert_eq!(t.hand_size(P1), 0);
    t.activate(P0, j, 3, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert!(!t.has_lost(P1));
    t.g.draw_cards(P1, 1);
    t.settle();
    assert!(t.has_lost(P1));
}

#[test]
fn jaces_archivist_can_deck_everyone_for_a_draw() {
    cr!("704.5b", "104.4a");
    ruling!(
        "Jace's Archivist",
        "If this ability causes all players to do this, the game is a draw."
    );
    supported("Jace's Archivist");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Jace's Archivist");
    t.lands(P0, "Island", 1);
    for p in [P0, P1] {
        t.g.players[p.idx()].library.truncate(1);
        fill_hand(&mut t, p, 2);
    }
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(matches!(t.g.result, Some(GameResult::Draw)), "{:?}", t.g.result);
    // Positive control: only the player with the short library loses.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Jace's Archivist");
    t.lands(P0, "Island", 1);
    t.g.players[1].library.truncate(1);
    fill_hand(&mut t, P1, 2);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.has_lost(P1) && !t.has_lost(P0));
}

#[test]
fn skyship_weatherlight_leaving_strands_its_exiled_cards() {
    cr!("607.2a", "400.7");
    ruling!(
        "Skyship Weatherlight",
        "If this card leaves the battlefield, the remaining cards that were exiled don’t come back."
    );
    supported("Skyship Weatherlight");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Ornithopter", "Grizzly Bears"]);
    let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose_moved(P0, &es);
    let s = enter(&mut t, P0, "Skyship Weatherlight");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter") && t.in_exile("Grizzly Bears"));
    // Positive control: its ability returns one at random.
    t.lands(P0, "Wastes", 4);
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    destroy(&mut t, s);
    t.advance_to(P1, Step::End);
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(t.hand_size(P0), 1);
    let _ = Zone::Exile;
}

/// Puts Gathering Stone onto the battlefield for P0, choosing the creature type `ty`.
fn gathering_stone(t: &mut TestGame, ty: &str) {
    // The options offered (every creature type), from a throwaway game.
    let mut probe = TestGame::new(2);
    enter(&mut probe, P0, "Gathering Stone");
    let options = probe
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            decision::Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .expect("a creature type is chosen");
    let i = options.iter().position(|o| o == ty).expect("the type");
    t.answer(P0, DecisionKind::Option, decision::Answer::Index(i));
    enter(t, P0, "Gathering Stone");
    t.resolve_all();
}

#[test]
fn gathering_stone_reduces_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Gathering Stone",
        "Gathering Stone's second ability applies only to generic mana in the total cost of spells you cast of the chosen type."
    );
    supported("Gathering Stone");
    // Elvish Visionary ({1}{G}) costs {G}; Llanowar Elves ({G}) still costs {G}.
    let mut t = TestGame::new(2);
    gathering_stone(&mut t, "Elf");
    t.lands(P0, "Forest", 1);
    let v = t.hand(P0, "Elvish Visionary");
    assert!(t.cast(P0, v).try_go().is_ok());
    let mut t = TestGame::new(2);
    gathering_stone(&mut t, "Elf");
    t.lands(P0, "Wastes", 1);
    let e = t.hand(P0, "Llanowar Elves");
    assert!(t.cast(P0, e).try_go().is_err());
}
