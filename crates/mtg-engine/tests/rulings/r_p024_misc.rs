//! Rulings batch P024 — the rest: attack requirements (CR 508.1d, 508.1h), becoming a copy
//! of a revealed card (CR 701.20, 707.2), triggers that are too late (CR 603.2), copies of
//! double-faced permanents (CR 707.8, 712.14a), undefined characteristics (CR 707.2),
//! nothing to copy (CR 608.2), copy effects overwriting each other (CR 613.2, 613.7),
//! attack triggers for each player (CR 506.2, 508.1), demonstrate (CR 702.144a), the
//! legend rule (CR 704.5j), modes in order (CR 700.2, 608.2c), tokens entering one at a
//! time (CR 608.2c, 603.2), replacement effects (CR 614, 616.1) and multiple triggers.

use crate::r_p024_common::*;
use crate::r_s09_common::{legal_attack, to_combat};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The YesNo decisions `p` was asked since `from`.
fn yes_no_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

// --- Nahiri, the Unforgiving ----------------------------------------------------------------

#[test]
fn nahiris_target_doesnt_attack_if_it_cant_or_it_costs_something() {
    cr!("508.1d", "508.1h");
    ruling!("Nahiri, the Unforgiving", "If the creature targeted by the first loyalty ability can't attack for any reason (such as being tapped), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost. If they don't, it doesn't have to attack in that case either.");
    supported("Nahiri, the Unforgiving");
    supported("Ghostly Prison");
    for case in ["free", "tapped", "cost"] {
        let mut t = TestGame::new(2);
        let nahiri = t.battlefield(P0, "Nahiri, the Unforgiving");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, nahiri, "attacks a player each combat").expect("+1");
        t.resolve_all();
        match case {
            "tapped" => {
                t.g.tap(bears);
            }
            "cost" => {
                t.battlefield(P1, "Ghostly Prison");
                t.lands(P0, "Plains", 2);
            }
            _ => {}
        }
        to_combat(&mut t, P0);
        assert_eq!(legal_attack(&mut t, &[]), case != "free", "{case}");
        if case == "free" {
            assert!(legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
        }
    }
}

// --- Shapeshifter's Marrow -------------------------------------------------------------------

#[test]
fn shapeshifters_marrow_becomes_a_revealed_creature_card_permanently() {
    cr!("707.2", "701.20a", "611.2a");
    ruling!("Shapeshifter's Marrow", "If the revealed card is a creature card, Shapeshifter’s Marrow becomes a copy of that card permanently. It’s no longer an enchantment.");
    ruling!("Shapeshifter's Marrow", "If the revealed card isn’t a creature card, it stays on top of that player’s library. Shapeshifter’s Marrow remains an enchantment.");
    supported("Shapeshifter's Marrow");
    for creature in [true, false] {
        let mut t = TestGame::new(2);
        let marrow = t.battlefield(P0, "Shapeshifter's Marrow");
        let top = t.library_top(P1, if creature { "Hill Giant" } else { "Forest" });
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        t.resolve_all();
        if creature {
            assert!(t.in_graveyard(P1, "Hill Giant"));
            assert_eq!(name_of(&t, marrow), "Hill Giant");
            assert!(is_creature(&t, marrow) && !t.obj_now(marrow).is(CardType::Enchantment));
            // Permanently: still a Hill Giant turns later.
            t.advance_to(P0, Step::Upkeep);
            t.advance_to(P1, Step::Upkeep);
            assert_eq!(name_of(&t, marrow), "Hill Giant");
        } else {
            assert_eq!(t.g.players[1].library.last().copied(), Some(top));
            assert_eq!(name_of(&t, marrow), "Shapeshifter's Marrow");
            assert!(t.obj_now(marrow).is(CardType::Enchantment) && !is_creature(&t, marrow));
        }
    }
}

// --- Nascent Metamorph ----------------------------------------------------------------------

#[test]
fn nascent_metamorph_copies_nothing_if_no_creature_card_is_revealed() {
    cr!("701.20a", "608.2c");
    ruling!("Nascent Metamorph", "If the target opponent has no creature cards in their library, Nascent Metamorph doesn't become a copy of anything. The cards are returned to that library in a random order.");
    supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let meta = t.battlefield(P0, "Nascent Metamorph");
    t.library_top(P1, "Forest");
    let size = t.library_size(P1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s01_common::attack_with(&mut t, &[(meta, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(name_of(&t, meta), "Nascent Metamorph");
    assert_eq!(t.pt(meta), (1, 1));
    assert_eq!(t.library_size(P1), size);
    assert!(t.g.players[1].library.iter().any(|c| t.obj(*c).chars.name == "Forest"));
}

// --- Identity Thief ---------------------------------------------------------------------------

/// P0's Identity Thief attacks P1 and becomes a copy of `what` (exiling it); combat
/// finishes unblocked.
fn thief_attacks_copying(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Identity Thief");
    let thief = t.battlefield(P0, "Identity Thief");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(what)]);
    t.attack(&[(thief, Entity::Player(P1))], &[]);
    t.resolve_all();
    thief
}

#[test]
fn identity_thief_is_too_late_for_attack_triggers() {
    cr!("603.2", "508.1m", "707.2");
    ruling!("Identity Thief", "If the targeted creature has an ability that triggers when it attacks, it'll be too late for it to trigger once Identity Thief has that ability. The same is true if another object has an ability that triggers when a creature with certain characteristics attacks and Identity Thief doesn't have those characteristics until after it becomes a copy.");
    supported("Infectious Horror");
    supported("Sanctum Seeker");
    // Infectious Horror: "Whenever this creature attacks, each opponent loses 2 life." The
    // Thief, a 2/2 Infectious Horror, deals only its combat damage.
    let mut t = TestGame::new(2);
    let horror = t.battlefield(P1, "Infectious Horror");
    let thief = thief_attacks_copying(&mut t, horror);
    assert_eq!(name_of(&t, thief), "Infectious Horror");
    assert_eq!(t.life(P1), 18);
    // Sanctum Seeker: "Whenever a Vampire you control attacks, each opponent loses 1 life
    // and you gain 1 life." The Thief becomes a Sengir Vampire after attacking.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctum Seeker");
    let vampire = t.battlefield(P1, "Sengir Vampire");
    let thief = thief_attacks_copying(&mut t, vampire);
    assert!(has_subtype(&t, thief, "Vampire"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn identity_thief_copies_the_face_shown_of_a_double_faced_permanent() {
    cr!("707.8", "712.14a", "202.3b", "701.27c");
    ruling!("Identity Thief", "If the targeted creature is a double-faced or melded permanent, Identity Thief copies the face shown. Because Identity Thief is not a double-faced card or meld card, it won't be able to transform or meld. If it copies a melded permanent or the back face of a double-faced card, its mana value will be 0. Exiled double-faced cards and exiled meld cards will return with their front faces up.");
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets // Insectile Aberration");
    crate::r_s17_common::transform(&mut t, delver);
    assert_eq!(name_of(&t, delver), "Insectile Aberration");
    let thief = thief_attacks_copying(&mut t, delver);
    assert_eq!(name_of(&t, thief), "Insectile Aberration");
    assert_eq!(mv_now(&mut t, thief), 0);
    crate::r_s17_common::transform(&mut t, thief);
    assert_eq!(name_of(&t, thief), "Insectile Aberration");
    // At the beginning of the end step the Delver returns, front face up.
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    let back = t.named_on_battlefield("Delver of Secrets");
    assert_eq!(back.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(back[0]).controller, P1);
    // A melded permanent: P1's Brisela, Voice of Nightmares (melded at P1's end step by
    // Gisela's ability). The Thief is a Brisela with mana value 0 and can't meld; both
    // cards return front face up.
    supported("Gisela, the Broken Blade");
    supported("Bruna, the Fading Light");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Gisela, the Broken Blade");
    t.battlefield(P1, "Bruna, the Fading Light");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::End);
    t.settle();
    t.resolve_all();
    let brisela = t.named_on_battlefield("Brisela, Voice of Nightmares");
    assert_eq!(brisela.len(), 1, "{}", t.dump_log());
    assert_eq!(mv_now(&mut t, brisela[0]), 11);
    t.advance_to(P0, Step::Upkeep);
    let thief = thief_attacks_copying(&mut t, brisela[0]);
    assert_eq!(name_of(&t, thief), "Brisela, Voice of Nightmares");
    assert_eq!(mv_now(&mut t, thief), 0);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gisela, the Broken Blade").len(), 1, "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Bruna, the Fading Light").len(), 1);
    // Only the Thief (a copy until end of turn) is a Brisela.
    assert_eq!(t.named_on_battlefield("Brisela, Voice of Nightmares"), vec![thief]);
}

#[test]
fn identity_thief_copying_a_hamlet_captain_that_became_a_permeating_mass() {
    cr!("707.3", "510.2");
    ruling!("Identity Thief", "For example, if you target a Hamlet Captain that has become a copy of Permeating Mass, Identity Thief will become a Permeating Mass.");
    supported("Permeating Mass");
    supported("Hamlet Captain");
    // The ruling's example: P1's Permeating Mass ("Whenever this creature deals combat
    // damage to a creature, that creature becomes a copy of Permeating Mass.") is blocked
    // by P0's Hamlet Captain, which becomes a Permeating Mass; the Thief copies it.
    let mut t = TestGame::new(2);
    let mass = t.battlefield(P1, "Permeating Mass");
    let captain = t.battlefield(P0, "Hamlet Captain");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(mass, Entity::Player(P0))], &[(captain, mass)]);
    t.resolve_all();
    assert_eq!(name_of(&t, captain), "Permeating Mass");
    t.advance_to(P0, Step::Upkeep);
    let thief = thief_attacks_copying(&mut t, captain);
    assert_eq!(name_of(&t, thief), "Permeating Mass");
    assert_eq!(t.pt(thief), (1, 3));
}

// --- Undefined characteristics, nothing to copy ---------------------------------------------

#[test]
fn volraths_shapeshifter_keeps_its_own_undefined_characteristics() {
    cr!("707.2", "202.1a");
    ruling!("Volrath's Shapeshifter", "If the top card in the graveyard has any other undefined characteristics, then those characteristics are not acquired and the shapeshifter uses the characteristic from its own card.");
    supported("Volrath's Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Volrath's Shapeshifter");
    // Dryad Arbor has no mana cost: the Shapeshifter keeps {1}{U}{U}.
    t.graveyard(P0, "Dryad Arbor");
    t.g.recompute();
    assert_eq!(name_of(&t, shifter), "Dryad Arbor");
    assert_eq!(t.pt(shifter), (1, 1));
    assert_eq!(mv_now(&mut t, shifter), 3);
    assert_eq!(abilities_with(&t, shifter, "Discard a card"), 1);
    // Blue from its own mana cost, green from Dryad Arbor's color indicator.
    interp!("full-text-undefined-mana-cost");
    let c = t.obj_now(shifter).chars.colors;
    assert!(c.contains(Color::Blue) && c.contains(Color::Green));
    // A creature card with a mana cost: the Shapeshifter takes it.
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(mv_now(&mut t, shifter), 4);
    assert_eq!(t.obj_now(shifter).chars.colors, ColorSet::single(Color::Red));
}

#[test]
fn assimilation_aegis_with_nothing_exiled_doesnt_change_the_creature() {
    cr!("608.2c", "702.6a");
    ruling!("Assimilation Aegis", "If there are no creature cards exiled with Assimilation Aegis when its second ability resolves, the creature Assimilation Aegis is attached to won't become a copy of anything else. It will remain just as it was.");
    supported("Assimilation Aegis");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[]);
    let aegis = t.enter(P0, "Assimilation Aegis");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, aegis, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, aegis), Some(obj(bears)));
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn valki_with_no_exiled_card_of_that_mana_value_does_nothing() {
    cr!("608.2c", "107.3a");
    ruling!("Valki, God of Lies // Tibalt, Cosmic Impostor", "If there are no creature cards exiled with Valki with mana value equal to the value of X as Valki's activated ability resolves, nothing happens. God of Lies, indeed.");
    supported("Valki, God of Lies // Tibalt, Cosmic Impostor");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    let valki = t.enter(P0, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    for (x, name) in [(3, "Valki, God of Lies"), (2, "Grizzly Bears")] {
        t.lands(P0, "Wastes", x as usize);
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        activate_containing(&mut t, P0, valki, "Choose a creature card exiled").expect("X");
        t.resolve_all();
        assert_eq!(name_of(&t, valki), name, "X = {x}");
    }
}

#[test]
fn sakashima_the_impostor_that_copies_nothing_is_a_3_1_human_rogue() {
    cr!("707.5", "614.12");
    ruling!("Sakashima the Impostor", "If there is no creature on the battlefield or if you don't choose a creature as Sakashima enters, Sakashima remains a 3/1 Human Rogue.");
    supported("Sakashima the Impostor");
    for creature in [false, true] {
        let mut t = TestGame::new(2);
        if creature {
            t.battlefield(P1, "Hill Giant");
        }
        t.answer_choose(P0, &[]);
        let s = t.enter(P0, "Sakashima the Impostor");
        t.resolve_all();
        assert_eq!(name_of(&t, s), "Sakashima the Impostor");
        assert!(has_subtype(&t, s, "Human") && has_subtype(&t, s, "Rogue"));
        assert_eq!(t.pt(s), (3, 1));
    }
}

#[test]
fn auton_soldier_that_copies_nothing_has_no_myriad_and_dies() {
    cr!("707.5", "614.12", "704.5f");
    ruling!("Auton Soldier", "If you choose not to have Auton Soldier enter the battlefield as a copy of another creature, it won't have myriad. It will also probably die due to having 0 toughness.");
    supported("Auton Soldier");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[]);
    let s = t.enter(P0, "Auton Soldier");
    assert!(!t.obj(s).has_keyword(KeywordKind::Myriad));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Auton Soldier"));
    // Positive control: as a copy, it has myriad.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let s = enter_copying(&mut t, P0, "Auton Soldier", giant);
    assert!(has_kw(&t, s, KeywordKind::Myriad));
}

// --- Copy effects overwriting each other ---------------------------------------------------

#[test]
fn mirage_mirrors_last_resolved_copy_wins_and_all_wear_off_together() {
    cr!("613.2a", "613.7a", "514.2", "405.5");
    ruling!("Mirage Mirror", "If you activate Mirage Mirror's ability multiple times in a turn in response to itself, then each time one of those abilities resolves, it will overwrite whatever Mirage Mirror is copying. Mirage Mirror will wind up as a copy of the permanent targeted by the last ability to resolve. When the turn ends, all instances of the ability will wear off at the same time.");
    supported("Mirage Mirror");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Mirage Mirror");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.activate(P0, mirror, 0, &[obj(giant)]).expect("first");
    t.activate(P0, mirror, 0, &[obj(bears)]).expect("second");
    t.resolve();
    assert_eq!(name_of(&t, mirror), "Grizzly Bears");
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Hill Giant");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, mirror), "Mirage Mirror");
}

#[test]
fn mirror_of_the_forebears_last_resolved_copy_wins_and_all_wear_off_together() {
    cr!("613.2a", "613.7a", "514.2", "607.2a");
    ruling!("Mirror of the Forebears", "If you activate the second ability of Mirror of the Forebears multiple times in a turn in response to itself, then each time one of those abilities resolves, it will overwrite whatever Mirror of the Forebears is copying. Mirror of the Forebears will wind up as a copy of the permanent targeted by the last ability to resolve. When the turn ends, all instances of the ability will wear off at the same time.");
    supported("Mirror of the Forebears");
    let mut t = TestGame::new(2);
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Elf");
    let mirror = t.enter(P0, "Mirror of the Forebears");
    t.resolve_all();
    let elves = t.battlefield(P0, "Llanowar Elves");
    let mystic = t.battlefield(P0, "Elvish Mystic");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, mirror, 0, &[obj(elves)]).expect("first");
    t.activate(P0, mirror, 0, &[obj(mystic)]).expect("second");
    t.resolve();
    assert_eq!(name_of(&t, mirror), "Elvish Mystic");
    t.resolve_all();
    assert_eq!(name_of(&t, mirror), "Llanowar Elves", "{}", t.dump_log());
    assert!(is_artifact(&t, mirror) && is_creature(&t, mirror));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_of(&t, mirror), "Mirror of the Forebears");
    assert!(!is_creature(&t, mirror));
}

// --- Echoing Assault --------------------------------------------------------------------------

#[test]
fn echoing_assault_triggers_once_for_each_player_attacked() {
    cr!("508.1", "603.2c", "802.2");
    ruling!("Echoing Assault", "If you attack multiple players in the same declare attackers step, Echoing Assault's last ability triggers once for each player you attacked.");
    supported("Echoing Assault");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Echoing Assault");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s01_common::attack_with(&mut t, &[(giant, Entity::Player(P1)), (bears, Entity::Player(P2))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    let mut names: Vec<String> = toks.iter().map(|c| name_of(&t, *c)).collect();
    names.sort();
    assert_eq!(names, vec!["Grizzly Bears", "Hill Giant"]);
}

// --- Hofri Ghostforge --------------------------------------------------------------------------

#[test]
fn hofri_makes_no_token_if_the_card_cant_be_exiled() {
    cr!("608.2c", "400.7");
    ruling!("Hofri Ghostforge", "If you can't exile the card, perhaps because another effect moved the card before the triggered ability started resolving, you won't create a token.");
    supported("Hofri Ghostforge");
    for moved in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Hofri Ghostforge");
        let giant = t.battlefield(P0, "Hill Giant");
        let before = t.g.battlefield.clone();
        kill(&mut t, giant);
        assert_eq!(t.stack_len(), 1);
        if moved {
            let card = t.g.current(giant);
            t.g.move_object(card, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
            t.settle();
        }
        t.resolve_all();
        let toks = new_tokens(&t, P0, &before);
        assert_eq!(toks.len(), usize::from(!moved), "moved: {moved}");
        if moved {
            assert!(t.in_hand(P0, "Hill Giant"));
        } else {
            assert!(t.in_exile("Hill Giant"));
            assert!(has_subtype(&t, toks[0], "Spirit"));
        }
    }
}

// --- Silverquill Lecturer (demonstrate) --------------------------------------------------------

#[test]
fn demonstrate_copies_resolve_opponents_first_then_yours_then_the_original() {
    cr!("702.144a", "405.5", "707.10");
    ruling!("Silverquill Lecturer", "If you cast a spell with demonstrate and both you and an opponent copy it, the opponent's copy will resolve first, then your copy will resolve, and finally the original spell will resolve.");
    supported("Silverquill Lecturer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill Lecturer");
    lands_for(&mut t, P0, "{1}{G}");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, bears).go();
    t.settle();
    t.resolve();
    // The original, P0's copy, then P1's copy on top.
    let ctl: Vec<PlayerId> = t.g.stack.iter().map(|s| t.g.obj(*s).controller).collect();
    assert_eq!(ctl, vec![P0, P0, P1], "{}", t.dump_log());
    assert_eq!(t.g.stack[0], spell);
    let bears_of = |t: &TestGame, p: PlayerId| {
        t.named_on_battlefield("Grizzly Bears")
            .into_iter()
            .filter(|b| t.obj(*b).controller == p)
            .count()
    };
    t.resolve();
    assert_eq!((bears_of(&t, P0), bears_of(&t, P1)), (0, 1));
    t.resolve();
    assert_eq!((bears_of(&t, P0), bears_of(&t, P1)), (1, 1));
    assert!(t.g.stack.contains(&spell));
    t.resolve();
    assert_eq!((bears_of(&t, P0), bears_of(&t, P1)), (2, 1));
}

// --- The legend rule ---------------------------------------------------------------------------

#[test]
fn esix_copies_of_a_legendary_creature_are_legendary() {
    cr!("704.5j", "614.1a", "707.2");
    ruling!("Esix, Fractal Bloom", "If you choose a legendary creature, the copies will also be legendary. If this results in you controlling more than one legendary permanent with the same name, you'll put all but one of them into their owner's graveyard.");
    supported("Esix, Fractal Bloom");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Esix, Fractal Bloom");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(isamaru)]);
    // Raise the Alarm: two Isamaru tokens instead of two Soldiers; P0 keeps the original.
    t.answer_choose(P0, &[obj(isamaru)]);
    let toks = cast_spell(&mut t, P0, "Raise the Alarm", "{1}{W}", &[], None, None);
    assert!(t.dump_log().matches("Isamaru, Hound of Konda").count() >= 3);
    assert!(toks.is_empty(), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda"), vec![isamaru]);
    assert!(t.named_on_battlefield("Soldier Token").is_empty());
}

#[test]
fn sakashima_leaving_makes_the_legend_rule_apply_again() {
    cr!("704.5j", "704.3");
    ruling!("Sakashima of a Thousand Faces", "If you control more than one legendary permanent with the same name and the \"legend rule\" begins applying again (perhaps because Sakashima of a Thousand Faces leaves the battlefield), you'll immediately have to comply with the rule and put all but one of those permanents into the graveyard.");
    supported("Sakashima of a Thousand Faces");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    let s = enter_copying(&mut t, P0, "Sakashima of a Thousand Faces", a);
    let b = t.enter(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 3);
    // Sakashima leaves: P0 keeps one of the two Isamarus.
    t.answer_choose(P0, &[obj(b)]);
    let s = t.g.current(s);
    t.g.move_object(s, Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda"), vec![t.g.current(b)]);
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
}

// --- Modes in order -----------------------------------------------------------------------------

#[test]
fn sublime_epiphany_performs_its_modes_in_order() {
    cr!("700.2", "608.2c");
    ruling!("Sublime Epiphany", "If you choose more than one mode, you perform the modes in the order specified. For example, if you choose to counter target spell, return target nonland permanent to its owner's hand, and have target player draw a card, that permanent will be on the battlefield while that spell is countered (and may modify, affect, or trigger on that event) but will not be on the battlefield while that player draws a card.");
    supported("Sublime Epiphany");
    supported("Destiny Spinner");
    supported("Psychosis Crawler");
    // Returning P1's Destiny Spinner ("Creature and enchantment spells you control can't
    // be countered."): it's still there as P1's Grizzly Bears spell would be countered.
    // Returning P1's Psychosis Crawler ("Whenever you draw a card, each opponent loses 1
    // life."): it's gone as P1 draws.
    for bounce in ["Destiny Spinner", "Psychosis Crawler"] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let spinner = t.battlefield(P1, "Destiny Spinner");
        let crawler = t.battlefield(P1, "Psychosis Crawler");
        // Cards in P1's hand: the Crawler's toughness.
        t.hand(P1, "Forest");
        t.hand(P1, "Forest");
        let back = if bounce == "Destiny Spinner" { spinner } else { crawler };
        lands_for(&mut t, P1, "{1}{G}");
        let bears = t.hand(P1, "Grizzly Bears");
        let spell = t.cast(P1, bears).go();
        lands_for(&mut t, P0, "{4}{U}{U}");
        let ep = t.hand(P0, "Sublime Epiphany");
        t.g.turn.priority = Some(P0);
        let hand = t.hand_size(P1);
        t.cast(P0, ep)
            .modes(&[0, 2, 4])
            .target(obj(spell))
            .target(obj(back))
            .target(Entity::Player(P1))
            .go();
        t.resolve_all();
        assert!(t.in_hand(P1, bounce));
        // P1 drew one card and got the permanent back.
        assert_eq!(t.hand_size(P1), hand + 2, "{bounce}");
        if bounce == "Destiny Spinner" {
            // It was there as the Bears would be countered.
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
            // The Crawler saw the draw.
            assert_eq!(t.life(P0), 19);
        } else {
            // The Crawler was gone by the draw (and the Spinner stayed: not countered).
            assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
            assert_eq!(t.life(P0), 20, "{}", t.dump_log());
        }
    }
}

#[test]
fn sublime_epiphany_returns_a_creature_and_copies_it_as_it_last_existed() {
    cr!("608.2b", "608.2h", "700.2");
    ruling!("Sublime Epiphany", "If you choose the third and fourth modes and target the same creature you control both times, you'll return it and then create a token that's a copy of it, using its copiable values as it last appeared on the battlefield. This is because the legality of targets is checked only as the spell begins to resolve, before any actions are performed.");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let toks = cast_spell(&mut t, P0, "Sublime Epiphany", "{4}{U}{U}", &[obj(giant), obj(giant)], Some(&[2, 3]), None);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
}

#[test]
fn cabaretti_confluence_tokens_get_the_bonus_of_a_later_mode() {
    cr!("700.2", "608.2c", "611.2c");
    ruling!("Cabaretti Confluence", "If you choose the first and third mode, the creature tokens will enter the battlefield before the creatures target player controls get +1/+1 and gain first strike. If you target yourself with the third mode, the tokens will get that bonus.");
    supported("Cabaretti Confluence");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let toks = cast_spell(
        &mut t,
        P0,
        "Cabaretti Confluence",
        "{3}{R}{G}{W}",
        &[obj(giant), obj(giant), Entity::Player(P0)],
        Some(&[0, 0, 2]),
        None,
    );
    assert_eq!(toks.len(), 2);
    for c in toks.iter().chain([&giant]) {
        assert_eq!(t.pt(*c), (4, 4));
        assert!(has_kw(&t, *c, KeywordKind::FirstStrike));
    }
}

// --- Saheeli's Artistry ----------------------------------------------------------------------

#[test]
fn saheelis_artistry_tokens_enter_one_at_a_time() {
    cr!("608.2c", "603.2", "603.6a");
    ruling!("Saheeli's Artistry", "If you choose both modes, the tokens enter one at a time. Thus, if the token created by the second mode has an ability that triggers “whenever an artifact enters the battlefield,” the token created by the first mode will already be on the battlefield and won't cause that ability to trigger.");
    supported("Saheeli's Artistry");
    supported("Reckless Fireweaver");
    // Reckless Fireweaver: "Whenever an artifact you control enters, this creature deals 1
    // damage to each opponent." The original sees both tokens; the artifact copy sees only
    // itself.
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P0, "Ornithopter");
    let weaver = t.battlefield(P0, "Reckless Fireweaver");
    let toks = cast_spell(&mut t, P0, "Saheeli's Artistry", "{4}{U}{U}", &[obj(orni), obj(weaver)], Some(&[0, 1]), None);
    assert_eq!(toks.len(), 2);
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

#[test]
fn saheelis_artistry_may_target_one_artifact_creature_twice() {
    cr!("115.3", "700.2");
    ruling!("Saheeli's Artistry", "If you choose both modes, you may target a single artifact creature twice and get two copies of that artifact creature.");
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P0, "Ornithopter");
    let toks = cast_spell(&mut t, P0, "Saheeli's Artistry", "{4}{U}{U}", &[obj(orni), obj(orni)], Some(&[0, 1]), None);
    assert_eq!(toks.len(), 2);
    for c in toks {
        assert_eq!(name_of(&t, c), "Ornithopter");
    }
}

#[test]
fn saheelis_artistry_artifact_exception_is_copiable() {
    cr!("707.9b", "707.3");
    ruling!("Saheeli's Artistry", "If you chose the second mode, the token is an artifact in addition to its other types. This is a copiable value of the token that other effects may copy.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let toks = cast_spell(&mut t, P0, "Saheeli's Artistry", "{4}{U}{U}", &[obj(bears)], Some(&[1]), None);
    assert_eq!(toks.len(), 1);
    assert!(is_artifact(&t, toks[0]));
    let clone = clone_of(&mut t, P0, toks[0]);
    assert!(is_artifact(&t, clone) && is_creature(&t, clone));
    assert!(has_subtype(&t, clone, "Bear"));
}

// --- Replacement effects ------------------------------------------------------------------------

#[test]
fn moonlit_meditation_declined_isnt_offered_again_that_turn() {
    cr!("614.1a", "614.5");
    ruling!("Moonlit Meditation", "If you choose not to apply the replacement effect, you will not get the choice to apply it again until the next turn.");
    supported("Moonlit Meditation");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    crate::r_s06_common::attach_new(&mut t, P0, "Moonlit Meditation", obj(giant));
    let from = t.asked().len();
    t.answer_yes(P0, false);
    cast_spell(&mut t, P0, "Raise the Alarm", "{1}{W}", &[], None, None);
    assert_eq!(yes_no_asked(&t, P0, from), 1);
    let from = t.asked().len();
    cast_spell(&mut t, P0, "Raise the Alarm", "{1}{W}", &[], None, None);
    assert_eq!(yes_no_asked(&t, P0, from), 0);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Next turn it's offered again.
    t.advance_to(P1, Step::Upkeep);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    lands_for(&mut t, P0, "{1}{W}");
    let r = t.hand(P0, "Raise the Alarm");
    t.g.turn.priority = Some(P0);
    t.cast(P0, r).go();
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, from), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
}

#[test]
fn infinite_reflections_apply_in_the_order_you_choose() {
    cr!("616.1", "614.1c", "707.5");
    ruling!("Infinite Reflection", "If you control more than one Infinite Reflection, you can apply those copy effects in any order. Creatures you control will enter as a copy of the one whose copy effect you apply last.");
    supported("Infinite Reflection");
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let angel = t.battlefield(P0, "Serra Angel");
        crate::r_s06_common::attach_new(&mut t, P0, "Infinite Reflection", obj(giant));
        crate::r_s06_common::attach_new(&mut t, P0, "Infinite Reflection", obj(angel));
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        let bears = t.enter(P0, "Grizzly Bears");
        t.resolve_all();
        let options = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseReplacement { options } => Some(options.clone()),
                _ => None,
            })
            .expect("the order was asked");
        assert_eq!(options.len(), 2);
        let name = name_of(&t, bears);
        assert!(name == "Hill Giant" || name == "Serra Angel");
        // The other order gives the other creature.
        if first == 0 {
            let other = name;
            let mut t = TestGame::new(2);
            let giant = t.battlefield(P0, "Hill Giant");
            let angel = t.battlefield(P0, "Serra Angel");
            crate::r_s06_common::attach_new(&mut t, P0, "Infinite Reflection", obj(giant));
            crate::r_s06_common::attach_new(&mut t, P0, "Infinite Reflection", obj(angel));
            t.answer(P0, DecisionKind::Replacement, Answer::Index(1));
            let bears = t.enter(P0, "Grizzly Bears");
            t.resolve_all();
            assert_ne!(name_of(&t, bears), other);
        }
    }
}

// --- Multiple triggers, copiable triggered abilities -----------------------------------------

#[test]
fn two_genestealer_patriarchs_make_two_tokens() {
    cr!("603.2", "603.10a", "707.9b");
    ruling!("Genestealer Patriarch", "If you control more than one Genestealer Patriarch as a creature with an infection counter on it dies, each one's ability will trigger, and you will make two tokens that are copies of it.");
    supported("Genestealer Patriarch");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Genestealer Patriarch");
    t.battlefield(P0, "Genestealer Patriarch");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(obj(giant), "infection", 1, None);
    let before = t.g.battlefield.clone();
    kill(&mut t, giant);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2);
    for c in toks {
        assert_eq!(name_of(&t, c), "Hill Giant");
        assert!(has_subtype(&t, c, "Tyranid") && has_subtype(&t, c, "Giant"));
    }
}

#[test]
fn progenitor_mimics_gained_trigger_is_copiable() {
    cr!("707.9a", "707.3");
    ruling!("Progenitor Mimic", "If you choose to have Progenitor Mimic enter the battlefield as a copy of a creature, the triggered ability it gains will become part of its copiable values. For example, suppose Progenitor Mimic enters the battlefield as a copy of Runeclaw Bear, a 2/2 green Bear creature with mana cost {1}{G}. The resulting object is a 2/2 green Bear creature named Runeclaw Bear with mana cost {1}{G} and with “At the beginning of your upkeep, if this creature isn't a token, put a token onto the battlefield that's a copy of this creature.” If another Progenitor Mimic enters the battlefield as a copy of that creature, it will be a Runeclaw Bear with two instances of the triggered ability.");
    supported("Progenitor Mimic");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Runeclaw Bear");
    let a = enter_copying(&mut t, P0, "Progenitor Mimic", bear);
    let b = enter_copying(&mut t, P0, "Progenitor Mimic", a);
    for (m, n) in [(a, 1), (b, 2)] {
        assert_eq!(name_of(&t, m), "Runeclaw Bear");
        assert_eq!(t.pt(m), (2, 2));
        assert_eq!(t.obj_now(m).chars.colors, ColorSet::single(Color::Green));
        assert_eq!(mv_now(&mut t, m), 2);
        assert_eq!(abilities_with(&t, m, "At the beginning of your upkeep"), n);
    }
    // At P0's upkeep: one token from the first, two from the second.
    let before = t.g.battlefield.clone();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(new_tokens(&t, P0, &before).len(), 3);
}

// --- Stolen Identity (cipher) ---------------------------------------------------------------

#[test]
fn stolen_identity_can_be_encoded_on_its_new_token_if_its_a_creature() {
    cr!("702.99a", "608.2c");
    ruling!("Stolen Identity", "If the token you create is a copy of a creature, you can exile Stolen Identity encoded on that token.");
    use mtg_engine::kw::cipher::encoded_on;
    // A copy of P1's Hill Giant: P0's only creature, the token, is offered and chosen.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{4}{U}{U}");
    let card = t.hand(P0, "Stolen Identity");
    let before = t.g.battlefield.clone();
    t.cast(P0, card).target(obj(giant)).go();
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[]);
    t.clear_answers();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![obj(toks[0])]]);
    // P0 chooses it (the token didn't exist when the spell was cast: P0's agent picks the
    // creature offered).
    struct FirstOffered(Box<dyn mtg_engine::decision::Agent>);
    impl mtg_engine::decision::Agent for FirstOffered {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            match d {
                Decision::ChooseEntities { candidates, .. } if !candidates.is_empty() => {
                    Answer::Entities(vec![candidates[0]])
                }
                _ => self.0.decide(g, p, d),
            }
        }
    }
    let mut t = TestGame::new(2);
    {
        let mut agents = t.g.agents.0.lock().unwrap();
        let inner = std::mem::replace(&mut agents[0], Box::new(mtg_engine::decision::PassiveAgent));
        agents[0] = Box::new(FirstOffered(inner));
    }
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for(&mut t, P0, "{4}{U}{U}");
    let card = t.hand(P0, "Stolen Identity");
    let before = t.g.battlefield.clone();
    t.cast(P0, card).target(obj(giant)).go();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(encoded_on(&t.g, t.g.current(card)), Some(toks[0]), "{}", t.dump_log());
    assert!(t.in_exile("Stolen Identity"));
    // A copy of a noncreature artifact: no creature to encode it on.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    lands_for(&mut t, P0, "{4}{U}{U}");
    let card = t.hand(P0, "Stolen Identity");
    t.cast(P0, card).target(obj(stone)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 2);
    assert!(t.in_graveyard(P0, "Stolen Identity"));
}
