//! Rulings batch P101 — permanents and cards whose abilities compare mana values: last
//! known information of a sacrificed or destroyed permanent (CR 608.2h, 712.8e, 202.3e),
//! split cards' combined mana value (202.3d), tokens and copies (202.3a, 707.2), exactly one
//! sacrifice as a cost (601.2b), abilities that may be activated and do nothing (602.2,
//! 608.2c), effects locked in as they resolve (611.2c), study counters on cards exiled
//! by anything (Kianne), targets counting themselves (Squirming Emergence), fights
//! (701.14), triggered rather than activated abilities (Bartered Cow), additional phases
//! (500.8), and a "lose control" trigger (603.6, Gustha's Scepter).

use crate::r_p101_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn pyre_of_heroes_uses_the_sacrificed_back_faces_types_and_its_mana_value() {
    cr!("608.2h", "712.8e", "712.8f", "202.3b", "205.3m");
    ruling!(
        "Pyre of Heroes",
        "Use the creature types and mana value of the creature you sacrificed as it existed on the battlefield to determine what creature card you can find. Notably, if you sacrifice a creature that's a double-faced permanent with its back face up, you'll use the characteristics of the back face, not the front face."
    );
    supported("Pyre of Heroes");
    // The ruling (2021-02-05, Kaldheim, whose double-faced cards are modal) agrees with the
    // CR: a permanent with its back face up has only its back face's characteristics (CR
    // 712.8e, 712.8f), and its mana value "as it existed on the battlefield" is a modal
    // back face's own, while a transforming back face's is calculated from its front face's
    // mana cost (CR 712.8e, 202.3b; Liliana, Heretical Healer's ruling of 2016-04-08 says
    // the same).
    //
    // Alrund, God of the Cosmos ({3}{U}{U}, God) with its back face, Hakka, Whispering
    // Raven ({1}{U}, Bird), up: a Bird with mana value 3 (Eagle of the Watch) can be found;
    // not a God with mana value 3 or 6, nor a Bird with mana value 6.
    let mut t = TestGame::new(2);
    let pyre = t.battlefield(P0, "Pyre of Heroes");
    let hakka = crate::r_p212_common::back_face(
        &mut t,
        P0,
        "Alrund, God of the Cosmos // Hakka, Whispering Raven",
    );
    assert_eq!(t.obj_now(hakka).chars.name, "Hakka, Whispering Raven");
    assert_eq!(mv(&t, hakka), 2);
    let lib = set_library(
        &mut t,
        P0,
        &[
            "Kefnet the Mindful",
            "The Locust God",
            "Duskrider Peregrine",
            "Eagle of the Watch",
        ],
    );
    t.answer_choose(P0, &[obj(hakka)]);
    t.answer_choose(P0, &[obj(lib[3])]);
    let from = t.asked().len();
    mana(&mut t, P0, "2");
    act(&mut t, P0, pyre, "Search your library", &[]).expect("Pyre");
    t.resolve_all();
    assert!(
        was_offered(&t, P0, from, lib[3]),
        "{:?}",
        asked_of(&t, P0, from)
    );
    for c in &lib[..3] {
        assert!(!was_offered(&t, P0, from, *c), "{}", t.obj(*c).chars.name);
    }
    assert_eq!(count_named(&t, "Eagle of the Watch"), 1);
    // Delver of Secrets ({U}, Human Wizard) transformed into Insectile Aberration (Human
    // Insect; mana value 1 from the front face's cost, CR 712.8e): an Insect with mana
    // value 2 (Kraul Harpooner) can be found; a Faerie Wizard (Spellstutter Sprite) can't.
    let mut t = TestGame::new(2);
    let pyre = t.battlefield(P0, "Pyre of Heroes");
    let delver = t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(mv(&t, delver), 1);
    let lib = set_library(
        &mut t,
        P0,
        &["Spellstutter Sprite", "Kraul Harpooner", "Hill Giant"],
    );
    t.answer_choose(P0, &[obj(delver)]);
    t.answer_choose(P0, &[obj(lib[1])]);
    let from = t.asked().len();
    mana(&mut t, P0, "2");
    act(&mut t, P0, pyre, "Search your library", &[]).expect("Pyre");
    t.resolve_all();
    assert!(
        was_offered(&t, P0, from, lib[1]),
        "{:?}",
        asked_of(&t, P0, from)
    );
    assert!(!was_offered(&t, P0, from, lib[0]));
    assert_eq!(count_named(&t, "Kraul Harpooner"), 1);
}

#[test]
fn tainted_treats_uses_the_mana_value_the_permanent_last_had() {
    cr!("608.2h", "202.3e", "707.2");
    ruling!(
        "Tainted Treats",
        "Use the mana cost of the artifact or creature as it last existed on the battlefield to determine whether its mana value is 4 or less. If it had {X} in its cost, X is 0 for the purpose of determining its mana value."
    );
    // Walking Ballista with three counters: 0, a Food. A Clone (a {3}{U} card) copying
    // Craw Wurm: 6, no Food.
    let mut t = TestGame::new(2);
    let b = ballista(&mut t, P1, 3);
    cast_resolve(&mut t, P0, "Tainted Treats", "1BG", &[obj(b)]);
    assert!(!t.on_battlefield(b));
    assert_eq!(tokens(&t, P0).len(), 1);
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(wurm)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(mv(&t, clone), 6);
    cast_resolve(&mut t, P0, "Tainted Treats", "1BG", &[obj(clone)]);
    assert!(t.in_graveyard(P1, "Clone"));
    assert!(tokens(&t, P0).is_empty(), "6, not the Clone card's 4");
}

#[test]
fn enigmatic_incarnation_sacrifices_just_one_enchantment() {
    cr!("608.2d", "701.21a");
    ruling!(
        "Enigmatic Incarnation",
        "While resolving Enigmatic Incarnation’s ability, you can’t sacrifice multiple enchantments to get multiple creatures."
    );
    supported("Enigmatic Incarnation");
    // Two Pacifisms (2): one is sacrificed, one Gray Ogre (3) is found.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enigmatic Incarnation");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let a = crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", obj(goblin));
    let b = crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", obj(goblin));
    set_library(&mut t, P0, &["Gray Ogre", "Gray Ogre", "Island"]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    let from = t.asked().len();
    end_step(&mut t, P0);
    t.resolve_all();
    let max = asked_of(&t, P0, from).iter().find_map(|d| match d {
        Decision::ChooseEntities {
            candidates, max, ..
        } if candidates.contains(&obj(a)) => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some(1));
    assert_eq!(
        t.on_battlefield(a) as usize + t.on_battlefield(b) as usize,
        1
    );
    assert_eq!(count_named(&t, "Gray Ogre"), 1);
}

#[test]
fn abilities_that_may_be_activated_and_do_nothing() {
    cr!("602.2", "608.2c", "606.3");
    ruling!(
        "Vivien on the Hunt",
        "You may activate Vivien on the Hunt's first ability without sacrificing a creature. If you do, the ability does nothing."
    );
    ruling!(
        "Steel Hellkite",
        "You may activate the last ability even if Steel Hellkite hasn't dealt combat damage to any players that turn. If you do, the ability won't do anything."
    );
    supported("Vivien on the Hunt");
    // Vivien +2, declining to sacrifice: the Bears stays and nothing is searched for, not a
    // Gray Ogre (3, for the Bears) nor a Raging Goblin (1, as for a mana value of 0).
    let mut t = TestGame::new(2);
    let vivien = t.battlefield(P0, "Vivien on the Hunt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lib = set_library(&mut t, P0, &["Raging Goblin", "Gray Ogre", "Island"]);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.answer_choose(P0, &[obj(lib[0])]);
    let loyalty = t.counters(vivien, "loyalty");
    t.activate(P0, vivien, 0, &[]).expect("Vivien +2");
    t.resolve_all();
    assert_eq!(t.counters(vivien, "loyalty"), loyalty + 2);
    assert!(t.on_battlefield(bears));
    assert_eq!(count_named(&t, "Gray Ogre"), 0);
    assert_eq!(count_named(&t, "Raging Goblin"), 0);
    assert_eq!(t.library_size(P0), 3);
    // Steel Hellkite with X = 0, no combat damage dealt: nothing is destroyed.
    let mut t = TestGame::new(2);
    let hellkite = t.battlefield(P0, "Steel Hellkite");
    let thopter = t.battlefield(P1, "Ornithopter");
    answer_x(&mut t, P0, 0);
    act(&mut t, P0, hellkite, "Destroy each nonland permanent", &[]).expect("Hellkite");
    t.resolve_all();
    assert!(t.on_battlefield(thopter));
    assert!(t.on_battlefield(hellkite));
}

#[test]
fn anchor_to_reality_and_neoform_sacrifice_exactly_one() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Anchor to Reality",
        "You must sacrifice exactly one artifact or creature to cast this spell; you can't cast it without sacrificing an artifact or creature, and you can't sacrifice additional artifacts or creatures."
    );
    ruling!(
        "Neoform",
        "You must sacrifice exactly one creature to cast this spell; you can't cast it without sacrificing one, and you can't sacrifice additional creatures."
    );
    for (name, cost) in [("Anchor to Reality", "2UU"), ("Neoform", "GU")] {
        supported(name);
        // Nothing to sacrifice: it can't be cast.
        let mut t = TestGame::new(2);
        let c = t.hand(P0, name);
        mana(&mut t, P0, cost);
        assert!(!crate::r_s21_common::castable(&mut t, P0, c), "{name}");
        // Two creatures: exactly one is sacrificed.
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let from = t.asked().len();
        t.g.turn.priority = Some(P0);
        t.cast(P0, c).go();
        let sac = asked_of(&t, P0, from).iter().find_map(|d| match d {
            Decision::ChooseEntities {
                min,
                max,
                candidates,
                ..
            } if candidates.contains(&obj(a)) => Some((*min, *max)),
            _ => None,
        });
        assert_eq!(sac, Some((1, 1)), "{name}");
        assert_eq!(count_named(&t, "Grizzly Bears"), 1, "{name}");
    }
}

#[test]
fn dominating_vampires_control_lasts_the_turn_whatever_the_vampire_count() {
    cr!("611.2c", "608.2h", "514.2");
    ruling!(
        "Dominating Vampire",
        "You will keep control of the creature until end of turn even if the number of Vampires you control changes after the ability has resolved."
    );
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let dv = t.enter(P0, "Dominating Vampire");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    destroy(&mut t, hawk);
    destroy(&mut t, dv);
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn mutinous_massacre_untaps_and_hastes_every_creature() {
    cr!("611.2c", "702.10b", "701.26b");
    ruling!(
        "Mutinous Massacre",
        "You’ll untap all creatures, including those you already control. They’ll also gain haste even if they were already untapped."
    );
    supported("Mutinous Massacre");
    // P0's tapped Gray Ogre (3) came under P0's control this turn; P1's Air Elemental (5)
    // is untapped. Neither has haste of its own.
    let mut t = TestGame::new(2);
    let ogre = t.battlefield_sick(P0, "Gray Ogre");
    t.g.tap(ogre);
    let elemental = t.battlefield_sick(P1, "Air Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for c in [ogre, elemental] {
        assert!(!has_kw(&t, c, mtg_engine::keywords::KeywordKind::Haste));
    }
    // Even: the Bears (2) is destroyed; the Ogre (3) and the Elemental (5) aren't.
    crate::r_s03_common::respond(&mut t, P0, even);
    cast_resolve(&mut t, P0, "Mutinous Massacre", "3BBRR", &[]);
    assert!(!t.on_battlefield(bears));
    for c in [ogre, elemental] {
        assert_eq!(t.obj_now(c).controller, P0);
        assert!(!t.obj_now(c).tapped);
        assert!(has_kw(&t, c, mtg_engine::keywords::KeywordKind::Haste));
    }
    // Both can attack this turn.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(ogre, pl(P1)), (elemental, pl(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}

/// Answers an odd/even choice with "even".
fn even(_: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.to_lowercase().contains("even"))
            .map(Answer::Index),
        _ => None,
    }
}

#[test]
fn a_split_card_has_one_combined_mana_value_for_atemsis_and_azors_gateway() {
    cr!("202.3d", "709.4");
    ruling!(
        "Atemsis, All-Seeing",
        "The mana value of a split card is determined by combining its two halves. For example, the mana value of Fire//Ice in your hand is 4."
    );
    ruling!(
        "Azor's Gateway // Sanctum of the Sun",
        "The mana value of a split card, such as a card with aftermath from the Amonkhet block, is based on the combined mana cost of its two halves. A split card doesn't have two mana values."
    );
    supported("Atemsis, All-Seeing");
    // Atemsis hits P1 with a hand of Forest (0), Opt (1), Grizzly Bears (2), Divination
    // (3), Fire // Ice (4) and Air Elemental (5): six different mana values; P1 loses.
    let mut t = TestGame::new(2);
    let atemsis = t.battlefield(P0, "Atemsis, All-Seeing");
    for c in [
        "Forest",
        "Opt",
        "Grizzly Bears",
        "Divination",
        "Fire // Ice",
        "Air Elemental",
    ] {
        t.hand(P0, c);
    }
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(atemsis, pl(P1))]);
    // The game ends as the trigger resolves (in the combat damage step).
    t.g.run_until(10_000, |g| {
        g.player(P1).has_lost || g.turn.step == Step::EndOfCombat
    });
    assert!(t.has_lost(P1), "{}", t.dump_log());
    // Azor's Gateway: Forest, Opt, Bears, Divination and Fire // Ice — five values.
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Azor's Gateway // Sanctum of the Sun");
    let cards: Vec<ObjectId> = [
        "Forest",
        "Opt",
        "Grizzly Bears",
        "Divination",
        "Fire // Ice",
    ]
    .iter()
    .map(|c| t.hand(P0, c))
    .collect();
    for c in &cards {
        t.g.objects[gate.0 as usize].tapped = false;
        mana(&mut t, P0, "1");
        t.answer_choose(P0, &[obj(*c)]);
        t.activate(P0, gate, 0, &[]).expect("Gateway");
        t.resolve_all();
    }
    assert_eq!(t.obj_now(gate).face, FaceState::Back);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn pest_control_destroys_tokens_but_not_a_token_copy_of_a_4() {
    cr!("202.3a", "707.2", "111.1");
    ruling!(
        "Pest Control",
        "The mana value of a token that isn’t a copy of another object is 0. A token that is a copy of another object has the same mana cost as that object."
    );
    let mut t = TestGame::new(2);
    let thopter = create_token(&mut t, P1, "Thopter");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Cackling Counterpart", "1UU", &[obj(giant)]);
    let copy = tokens(&t, P0)[0];
    assert_eq!(mv(&t, copy), 4);
    cast_resolve(&mut t, P0, "Pest Control", "WB", &[]);
    assert!(!t.on_battlefield(thopter));
    assert!(t.on_battlefield(copy));
    assert!(t.on_battlefield(giant));
}

#[test]
fn ugin_the_spirit_dragon_isnt_a_dragon() {
    cr!("205.3j", "306.3");
    ruling!(
        "Ugin, the Spirit Dragon",
        "Ugin, the Spirit Dragon is not a Dragon card; that is, he doesn't have the creature type Dragon. Spells and abilities that refer to Dragon cards or Dragons don't apply to Ugin."
    );
    // Dragonlord's Servant ("Dragon spells you cast cost {1} less to cast.") doesn't reduce
    // Ugin's cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord's Servant");
    let ugin = t.hand(P0, "Ugin, the Spirit Dragon");
    assert!(!t.obj_now(ugin).chars.has_subtype("Dragon"));
    assert_eq!(cost_to_cast(&mut t, P0, ugin, &[]), Some(8));
    let on = t.battlefield(P0, "Ugin, the Spirit Dragon");
    assert!(!t.obj_now(on).chars.has_subtype("Dragon"));
}

#[test]
fn kianne_counts_studied_cards_exiled_by_anything_and_imbraham_may_have_x_0() {
    // Not linked abilities (CR 607): they count any study counters on cards P0 owns.
    cr!("122.1", "107.3");
    ruling!(
        "Kianne, Dean of Substance // Imbraham, Dean of Theory",
        "The second abilities of both Kianne and Imbraham look at all cards you own in exile with study counters, even if they were exiled by a different permanent, such as another Kianne or Imbraham."
    );
    ruling!(
        "Kianne, Dean of Substance // Imbraham, Dean of Theory",
        "You may choose 0 as the value of X as you activate Imbraham’s last ability. If you do, you will not exile any cards from your library. However, you may still put a card previously exiled with a study counter on it into your hand."
    );
    let name = "Kianne, Dean of Substance // Imbraham, Dean of Theory";
    // Cards exiled by something else, with study counters: Hill Giant (4), two Grizzly
    // Bears (2), a Forest (a land), and P1's Air Elemental (not P0's): two values.
    let studied = |t: &mut TestGame, p: PlayerId, c: &str| -> ObjectId {
        let id = t.exile(p, c);
        put(t, id, "study", 1);
        id
    };
    let mut t = TestGame::new(2);
    let kianne = t.battlefield(P0, name);
    for c in ["Hill Giant", "Grizzly Bears", "Grizzly Bears", "Forest"] {
        studied(&mut t, P0, c);
    }
    studied(&mut t, P1, "Air Elemental");
    t.exile(P0, "Craw Wurm");
    mana(&mut t, P0, "4G");
    act(&mut t, P0, kianne, "Create a 0/0", &[]).expect("Kianne");
    t.resolve_all();
    let fractal = tokens(&t, P0)[0];
    assert_eq!(t.counters(fractal, counters::PLUS1), 2);
    // Imbraham with X = 0: nothing is exiled from the library; the studied Hill Giant may
    // still be put into P0's hand.
    let mut t = TestGame::new(2);
    let imbraham = crate::r_p212_common::back_face(&mut t, P0, name);
    assert_eq!(t.obj_now(imbraham).chars.name, "Imbraham, Dean of Theory");
    let giant = studied(&mut t, P0, "Hill Giant");
    let lib = t.library_size(P0);
    mana(&mut t, P0, "UU");
    answer_x(&mut t, P0, 0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    act(&mut t, P0, imbraham, "Exile the top X cards", &[]).expect("Imbraham");
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib);
    assert!(t.in_hand(P0, "Hill Giant"), "{}", t.dump_log());
}

#[test]
fn squirming_emergence_counts_its_own_target() {
    cr!("115.1", "601.2c", "608.2b");
    ruling!(
        "Squirming Emergence",
        "The target nonland permanent card still counts toward the number of permanent cards in your graveyard for the purposes of Squirming Emergence."
    );
    // Hill Giant (4) with three other permanent cards: four, so it's a legal target.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    for c in ["Forest", "Grizzly Bears", "Ornithopter"] {
        t.graveyard(P0, c);
    }
    cast_resolve(&mut t, P0, "Squirming Emergence", "1BG", &[obj(giant)]);
    assert!(t.on_battlefield(giant), "{}", t.dump_log());
}

#[test]
fn tenth_district_heros_second_ability_checks_for_a_detective_on_resolution() {
    cr!("602.2", "608.2c", "205.3m");
    ruling!(
        "Tenth District Hero",
        "You can activate Tenth District Hero's second ability regardless of what creature types it is. The ability checks Tenth District Hero's creature types when it resolves. If Tenth District Hero isn't a Detective at that time, the ability does nothing."
    );
    let evidence = |t: &mut TestGame| -> Vec<Entity> {
        (0..2)
            .map(|_| obj(t.graveyard(P0, "Grizzly Bears")))
            .collect()
    };
    // A Human, not a Detective: the ability resolves and does nothing.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Tenth District Hero");
    let ev = evidence(&mut t);
    t.answer_choose(P0, &ev);
    mana(&mut t, P0, "2W");
    act(&mut t, P0, hero, "Collect evidence 4", &[]).expect("Hero");
    t.resolve_all();
    assert_eq!(t.obj_now(hero).chars.name, "Tenth District Hero");
    assert_eq!(t.pt(hero), (2, 3));
    // Made a Detective first: it becomes Mileva, the Stalwart.
    let ev = evidence(&mut t);
    t.answer_choose(P0, &ev);
    mana(&mut t, P0, "1W");
    act(&mut t, P0, hero, "Collect evidence 2", &[]).expect("Hero");
    t.resolve_all();
    let ev = evidence(&mut t);
    t.answer_choose(P0, &ev);
    mana(&mut t, P0, "2W");
    act(&mut t, P0, hero, "Collect evidence 4", &[]).expect("Hero");
    t.resolve_all();
    assert_eq!(t.obj_now(hero).chars.name, "Mileva, the Stalwart");
    assert_eq!(t.pt(hero), (5, 5));
}

#[test]
fn boxing_rings_treasure_counts_any_fight() {
    cr!("701.14a");
    ruling!(
        "Boxing Ring",
        "You may activate Boxing Ring's last ability if you control a creature that fought due to any effect, not just a creature that fought in the Boxing Ring."
    );
    supported("Boxing Ring");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Boxing Ring");
    let giant = t.battlefield(P0, "Hill Giant");
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(!can_activate(&mut t, P0, ring));
    cast_resolve(&mut t, P0, "Prey Upon", "G", &[obj(giant), obj(goblin)]);
    assert!(!t.on_battlefield(goblin));
    assert!(can_activate(&mut t, P0, ring));
    act(&mut t, P0, ring, "Create a Treasure", &[]).expect("Ring");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

/// P0 activates Arena with `mine` and P1's `theirs` as targets (P1 choosing it); the
/// ability stays on the stack.
fn arena(t: &mut TestGame, mine: ObjectId, theirs: ObjectId) -> ObjectId {
    let arena = t.battlefield(P0, "Arena");
    mana(t, P0, "3");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P1, &[obj(theirs)]);
    act(t, P0, arena, "Tap target creature you control", &[]).expect("Arena");
    t.settle();
    arena
}

#[test]
fn arenas_fight_with_tapped_creatures_illegal_targets_and_current_power() {
    cr!("701.14a", "701.14b", "608.2b", "608.2h");
    ruling!(
        "Arena",
        "Either or both creatures can be tapped prior to using the ability and it still works."
    );
    ruling!(
        "Arena",
        "If either target is illegal at the time the ability resolves (whether because it has left the battlefield, because it has gained Shroud or Protection, or for any other reason), the ability taps the remaining target, but no damage is dealt. If both targets are illegal, the ability doesn't resolve."
    );
    ruling!(
        "Arena",
        "The power of the creatures when the effect resolves is used to determine how much damage is done."
    );
    supported("Arena");
    // Both already tapped: they still fight.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.g.tap(bears);
    t.g.tap(goblin);
    arena(&mut t, bears, goblin);
    t.resolve_all();
    assert!(!t.on_battlefield(goblin));
    assert_eq!(dmg(&t, bears), 1);
    // P1's target leaves: P0's creature is tapped, no damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    arena(&mut t, bears, goblin);
    bounce(&mut t, goblin);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(dmg(&t, bears), 0);
    // P1's target gains shroud (Lightning Greaves): the same. Both gain shroud: the ability
    // doesn't resolve, and neither is tapped.
    for both in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let goblin = t.battlefield(P1, "Raging Goblin");
        arena(&mut t, bears, goblin);
        crate::r_s06_common::attach_new(&mut t, P1, "Lightning Greaves", obj(goblin));
        if both {
            crate::r_s06_common::attach_new(&mut t, P0, "Lightning Greaves", obj(bears));
        }
        t.resolve_all();
        assert_eq!(t.obj_now(bears).tapped, !both, "both = {both}");
        assert!(!t.obj_now(goblin).tapped, "both = {both}");
        assert_eq!((dmg(&t, bears), dmg(&t, goblin)), (0, 0), "both = {both}");
        assert!(t.on_battlefield(goblin));
    }
    // Giant Growth in response: the Bears deals 5.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    arena(&mut t, bears, wurm);
    crate::r_p003_common::giant_growth(&mut t, bears);
    t.resolve_all();
    // Craw Wurm (6/4) is dealt 5: destroyed (the Bears' 2 wouldn't have been lethal).
    assert!(!t.on_battlefield(wurm), "{}", t.dump_log());
}

#[test]
fn bartered_cows_ability_is_triggered_not_activated() {
    cr!("603.1", "602.1", "701.9a");
    ruling!(
        "Bartered Cow",
        "Bartered Cow's ability is a triggered ability, not an activated ability. It doesn't allow you to discard it or sacrifice it whenever you want; rather, you need some other way of discarding it or causing it to die."
    );
    let mut t = TestGame::new(2);
    let cow = t.battlefield(P0, "Bartered Cow");
    let inhand = t.hand(P0, "Bartered Cow");
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    assert!(!t.g.legal_actions(P0).iter().any(|a| matches!(
        a,
        mtg_engine::decision::Action::Activate { source, .. } if *source == cow || *source == inhand
    )));
    // Discarded to Tormenting Voice's cost: a Food.
    t.answer_choose(P0, &[obj(inhand)]);
    cast_resolve(&mut t, P0, "Tormenting Voice", "1R", &[]);
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn fury_of_the_horde_outside_a_main_phase_only_untaps() {
    cr!("500.8", "702.8a");
    ruling!(
        "Fury of the Horde",
        "If it’s somehow not a main phase when Fury of the Horde resolves, all it does is untap all creatures that attacked that turn. No new phases are created."
    );
    supported("Fury of the Horde");
    // With Leyline of Anticipation, P0 casts it in the end of combat step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Anticipation");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, pl(P1))], &[]);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.g.turn.step, Step::EndOfCombat);
    let before = t.g.turn.schedule.clone();
    cast_resolve(&mut t, P0, "Fury of the Horde", "5RR", &[]);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.g.turn.schedule, before, "no new phases");
}

#[test]
fn gusthas_scepter_triggers_on_losing_control_or_leaving() {
    cr!("603.6c", "603.10a", "406.6");
    ruling!(
        "Gustha's Scepter",
        "The last ability will trigger when a different player gains control of Gustha’s Scepter, as well as when Gustha’s Scepter leaves the battlefield."
    );
    supported("Gustha's Scepter");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let scepter = t.battlefield(P0, "Gustha's Scepter");
        let card = t.hand(P0, "Hill Giant");
        t.answer_choose(P0, &[obj(card)]);
        act(&mut t, P0, scepter, "Exile a card from your hand", &[]).expect("Scepter");
        t.resolve_all();
        assert_eq!(t.zone(card), Zone::Exile);
        if leaves {
            destroy(&mut t, scepter);
        } else {
            give_control(&mut t, scepter, P1);
        }
        t.resolve_all();
        assert!(
            t.in_graveyard(P0, "Hill Giant"),
            "leaves = {leaves}: {}",
            t.dump_log()
        );
    }
}
