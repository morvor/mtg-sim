//! Item `rulings-residue-type-change` — interaction rulings about zone changes and new
//! objects (CR 400.7), copiable values (CR 707), face-down permanents (CR 708), last
//! known information against a card's own characteristics, and players leaving a
//! multiplayer game (CR 800.4a).

use crate::r_residue_type_change_common::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn phyrexian_ingester_uses_the_exiled_cards_own_pt() {
    cr!("400.7", "608.2h", "613.4c");
    ruling!(
        "Phyrexian Ingester",
        "Phyrexian Ingester will get bonuses based on the card's power and toughness in exile. Counters, Auras, and Equipment it had on it before it was exiled won't add to those numbers."
    );
    supported("Phyrexian Ingester");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, bears, 2);
    cast_resolve(&mut t, P0, "Holy Strength", &[obj(bears)]);
    let sword = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(sword, obj(bears)));
    assert_eq!(pt_now(&mut t, bears), (7, 6));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let ingester = t.enter(P0, "Phyrexian Ingester");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(pt_now(&mut t, ingester), (5, 5));
}

#[test]
fn cloning_of_shredder_copies_only_the_printed_card() {
    cr!("707.2", "400.7", "707.9b");
    ruling!(
        "The Cloning of Shredder",
        "Except for the listed exceptions, the token created by one of The Cloning of Shredder's chapter abilities copies exactly what was printed on the original card and nothing else."
    );
    supported("The Cloning of Shredder");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A Bears with counters, a Giant Growth and a Phantasmal-Form-like change dies.
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 2);
    giant_growth(&mut t, bears);
    diminish(&mut t, bears);
    destroy(&mut t, bears);
    let card = t.g.current(bears);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.answer_targets(P0, &[obj(card)]);
    t.enter(P0, "The Cloning of Shredder");
    t.resolve_all();
    let toks: Vec<_> = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert!(t.obj_now(tok).is_token());
    assert_eq!(pt_now(&mut t, tok), (2, 2));
    assert_eq!(plus1(&t, tok), 0);
    assert_eq!(creature_types(&mut t, tok), vec!["Bear", "Mutant"]);
}

#[test]
fn cloning_of_shredder_exceptions_are_copiable_values() {
    cr!("707.9b", "707.2", "205.4");
    ruling!(
        "The Cloning of Shredder",
        "The token isn't legendary and is a Mutant in addition to its other creature types. This is a copiable value of the token that other effects may copy."
    );
    supported("The Cloning of Shredder");
    supported("Clone");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dog = t.graveyard(P0, "Isamaru, Hound of Konda");
    t.answer_targets(P0, &[obj(dog)]);
    t.enter(P0, "The Cloning of Shredder");
    t.resolve_all();
    let tok = t.named_on_battlefield("Isamaru, Hound of Konda")[0];
    assert!(!legendary(&t, tok));
    assert_eq!(creature_types(&mut t, tok), vec!["Dog", "Mutant"]);
    // A Clone copying the token copies those exceptions.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(tok)]);
    let clone = cast_new(&mut t, P0, "Clone", &[]);
    t.resolve_all();
    let clone = t.g.current(clone);
    assert_eq!(t.zone(clone), Zone::Battlefield);
    assert_eq!(t.obj_now(clone).chars.name, "Isamaru, Hound of Konda");
    assert!(!legendary(&t, clone));
    assert_eq!(creature_types(&mut t, clone), vec!["Dog", "Mutant"]);
    // Both stay: neither is legendary.
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 2);
}

/// P0's Exalted Angel dies with Yedora on the battlefield and returns face down as a
/// Forest land. Returns the face-down permanent.
fn yedora_angel(t: &mut TestGame) -> ObjectId {
    supported("Yedora, Grave Gardener");
    t.battlefield(P0, "Yedora, Grave Gardener");
    let angel = t.battlefield(P0, "Exalted Angel");
    t.answer_yes(P0, true);
    destroy(t, angel);
    t.resolve_all();
    let now = t.g.current(angel);
    assert!(t.on_battlefield(now) && t.obj_now(now).face_down);
    assert!(is_type(t, now, CardType::Land) && subtype(t, now, "Forest"));
    now
}

#[test]
fn yedora_forest_turned_face_up_for_its_morph_cost_is_the_printed_card() {
    cr!("708.8", "702.37e", "708.2a", "116.2b");
    ruling!(
        "Yedora, Grave Gardener",
        "If the face-down card has morph, you can pay its morph cost to turn it face up."
    );
    ruling!(
        "Yedora, Grave Gardener",
        "If one of these Forests is turned face up, the effect making it a Forest land will expire. The face-up permanent will again have its printed name, type line, mana cost, abilities, and so on."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let now = yedora_angel(&mut t);
    t.lands(P0, "Plains", 4);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: now }))
        .expect("turned face up");
    t.g.flush_events();
    t.resolve_all();
    let o = t.obj_now(now);
    assert!(!o.face_down);
    assert_eq!(o.chars.name, "Exalted Angel");
    assert!(is_type(&mut t, now, CardType::Creature));
    assert!(!is_type(&mut t, now, CardType::Land));
    assert!(!subtype(&mut t, now, "Forest") && subtype(&mut t, now, "Angel"));
    assert_eq!(pt_now(&mut t, now), (4, 5));
    assert!(has_kw(
        &mut t,
        now,
        mtg_engine::keywords::KeywordKind::Flying
    ));
}

#[test]
fn illithid_harvester_face_down_permanent_is_revealed_as_it_leaves() {
    cr!("708.9", "400.7");
    ruling!(
        "Illithid Harvester // Plant Tadpoles",
        "If a face-down permanent leaves the battlefield, its owner must reveal it to all players."
    );
    supported("Illithid Harvester // Plant Tadpoles");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Illithid Harvester // Plant Tadpoles");
    t.resolve_all();
    assert!(t.obj_now(bears).face_down);
    assert_eq!(pt_now(&mut t, bears), (2, 2));
    let from = t.g.turn_events.len();
    cast_resolve(&mut t, P0, "Unsummon", &[obj(bears)]);
    assert!(t.g.turn_events[from..].iter().any(|e| matches!(e,
        mtg_engine::events::Event::Custom { name, player: Some(P1), .. }
            if name.as_str() == mtg_engine::facedown::REVEALED)));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

/// Activates the permanent's mana ability (the `index`th activated ability) as a player
/// would (it doesn't use the stack, CR 605.3a).
fn tap_for_mana(t: &mut TestGame, land: ObjectId, index: usize) {
    t.activate(P0, land, index, &[]).unwrap();
    t.settle();
}

/// Thran Portal on P0's battlefield, entering as it would with "Forest" chosen.
fn portal(t: &mut TestGame, chosen: &str) -> ObjectId {
    supported("Thran Portal");
    choose_basic(t, chosen);
    let p = t.enter(P0, "Thran Portal");
    t.g.recompute();
    p
}

/// Queues P0's answer to a "Choose a basic land type" decision.
fn choose_basic(t: &mut TestGame, ty: &str) {
    let i = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
        .iter()
        .position(|x| *x == ty)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
}

/// The number of activated mana abilities the permanent has now.
fn mana_abilities(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, mtg_engine::ability::AbilityKind::Activated(x) if x.is_mana_ability))
        .count()
}

#[test]
fn thran_portal_gained_mana_ability_costs_life_too() {
    cr!("305.6", "118.8", "605.1a");
    ruling!(
        "Thran Portal",
        "If Thran Portal somehow gains another mana ability, that ability also costs an additional 1 life to activate."
    );
    supported("Chromatic Lantern");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let p = portal(&mut t, "Forest");
    assert!(subtype(&mut t, p, "Forest") && subtype(&mut t, p, "Gate"));
    assert_eq!(mana_abilities(&mut t, p), 1);
    // Its intrinsic Forest ability costs 1 life.
    tap_for_mana(&mut t, p, 0);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // A mana ability Chromatic Lantern gives it costs 1 life too.
    t.battlefield(P0, "Chromatic Lantern");
    t.g.objects[p.0 as usize].tapped = false;
    assert_eq!(mana_abilities(&mut t, p), 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    tap_for_mana(&mut t, p, 1);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn thran_portal_copy_without_a_chosen_type_has_no_mana_abilities() {
    cr!("707.2", "305.6", "614.12");
    ruling!(
        "Thran Portal",
        "If a Thran Portal somehow enters the battlefield without a basic land type having been chosen for it, it does not have any mana abilities. The same is true for a land that is already on the battlefield that becomes a copy of it."
    );
    supported("Thespian's Stage");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let p = portal(&mut t, "Island");
    let stage = t.battlefield(P0, "Thespian's Stage");
    t.lands(P0, "Wastes", 2);
    // Thespian's Stage's second activated ability: "becomes a copy of target land".
    let n = t
        .obj_now(stage)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    t.activate(P0, stage, n - 1, &[obj(p)]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(stage).chars.name, "Thran Portal");
    assert!(subtype(&mut t, stage, "Gate"));
    for ty in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(!subtype(&mut t, stage, ty), "{ty}");
    }
    assert_eq!(mana_abilities(&mut t, stage), 0);
}

#[test]
fn thran_portal_entering_as_a_copy_chooses_a_type() {
    cr!("707.2", "614.12", "614.1c", "305.6");
    ruling!(
        "Thran Portal",
        "If a land enters the battlefield as a copy of Thran Portal (as opposed to a land that is already on the battlefield becoming a copy of it), its controller gets to choose a basic land type for it and it has that type."
    );
    supported("Vesuva");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let p = portal(&mut t, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(p)]);
    choose_basic(&mut t, "Mountain");
    let v = t.enter(P0, "Vesuva");
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name, "Thran Portal");
    assert!(subtype(&mut t, v, "Mountain") && !subtype(&mut t, v, "Island"));
    assert_eq!(mana_abilities(&mut t, v), 1);
    t.g.objects[v.0 as usize].tapped = false;
    tap_for_mana(&mut t, v, 0);
    assert_eq!(t.life(P0), 19);
    assert_eq!(
        t.g.player(P0)
            .mana_pool
            .count(mtg_engine::mana::ManaType::R),
        1
    );
}

#[test]
fn the_master_cant_target_a_card_put_into_a_graveyard_without_milling() {
    cr!("701.17a", "701.17c", "115.1");
    ruling!(
        "The Master, Transcendent",
        "If a player is instructed to put a card into their graveyard from their library without using the word \"mill,\""
    );
    supported("Grisly Salvage");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let m = t.battlefield(P0, "The Master, Transcendent");
    // Grisly Salvage puts the rest of the top five into the graveyard: not milled.
    let top = (0..5)
        .map(|_| t.library_top(P0, "Hill Giant"))
        .collect::<Vec<_>>();
    t.answer_choose(P0, &[]);
    cast_resolve(&mut t, P0, "Grisly Salvage", &[]);
    let in_gy: Vec<ObjectId> = top
        .iter()
        .map(|c| t.g.current(*c))
        .filter(|c| t.zone(*c) == Zone::Graveyard(P0))
        .collect();
    assert!(!in_gy.is_empty());
    // A milled one is a legal target, for comparison.
    let milled_card = milled(&mut t, "Serra Angel");
    let from = t.asked().len();
    t.activate(P0, m, 0, &[obj(milled_card)]).unwrap();
    let cands: Vec<Entity> = asked_since(&t, from)
        .iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .expect("asked for a target");
    assert!(cands.contains(&obj(milled_card)));
    for c in in_gy {
        assert!(!cands.contains(&obj(c)));
    }
}

#[test]
fn witness_protection_name_cant_be_named() {
    cr!("201.3", "201.4");
    ruling!(
        "Witness Protection",
        "If an effect instructs a player to choose a card name, that player may not name \"Legitimate Businessperson,\" even if a creature currently has that name."
    );
    supported("Witness Protection");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Witness Protection", &[obj(bears)]);
    assert_eq!(t.obj_now(bears).chars.name, "Legitimate Businessperson");
    t.answer(
        P1,
        DecisionKind::Name,
        Answer::Text("Legitimate Businessperson".into()),
    );
    let needle = t.enter(P1, "Pithing Needle");
    let named = t.obj_now(needle).choices.card_name.clone();
    assert!(named.as_ref().is_none_or(|n| n.is_empty()), "{named:?}");
    // A real card name is fine.
    t.answer(P1, DecisionKind::Name, Answer::Text("Mind Stone".into()));
    let needle = t.enter(P1, "Pithing Needle");
    assert_eq!(
        t.obj_now(needle).choices.card_name.as_deref(),
        Some("Mind Stone")
    );
}

#[test]
fn cyclopean_mummy_goes_to_the_graveyard_then_is_exiled_by_a_trigger() {
    cr!("603.6c", "603.10a", "700.4");
    ruling!(
        "Cyclopean Mummy",
        "The Mummy does go to the graveyard and trigger abilities which watch for something going to the graveyard. It then gets exiled as a triggered ability."
    );
    supported("Cyclopean Mummy");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    let mummy = t.battlefield(P0, "Cyclopean Mummy");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, mummy);
    t.settle();
    let card = t.g.current(mummy);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Cyclopean Mummy"));
    assert_eq!(t.life(P1), 19);
}

/// A three-player game: P0's Etrata (owned by P0) deals combat damage to P1, who already
/// owns `hits` exiled cards with hit counters. Returns the game after combat.
fn etrata(hits: usize) -> TestGame {
    supported("Etrata, the Silencer");
    let mut t = TestGame::new(3);
    let e = t.battlefield(P0, "Etrata, the Silencer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..hits {
        let c = t.exile(P1, "Hill Giant");
        t.g.objects[c.0 as usize].counters.insert("hit".into(), 1);
    }
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    t
}

#[test]
fn etrata_is_shuffled_into_its_owners_library_regardless() {
    cr!("608.2c", "104.3e", "800.4a");
    ruling!(
        "Etrata, the Silencer",
        "Etrata's owner shuffles it into their library as the triggered ability resolves, regardless of how many cards are in exile or whether the player loses the game."
    );
    for (hits, loses) in [(0, false), (2, true)] {
        let t = etrata(hits);
        assert_eq!(t.has_lost(P1), loses);
        assert!(t.named_on_battlefield("Etrata, the Silencer").is_empty());
        assert!(t
            .g
            .player(P0)
            .library
            .iter()
            .any(|c| t.g.obj(*c).chars.name == "Etrata, the Silencer"));
    }
}

/// A three-player game: P0 casts Xantcha, which enters under P1's control. Returns it.
fn xantcha(t: &mut TestGame) -> ObjectId {
    supported("Xantcha, Sleeper Agent");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let x = cast_new(t, P0, "Xantcha, Sleeper Agent", &[]);
    t.resolve_all();
    let x = t.g.current(x);
    assert!(t.on_battlefield(x));
    assert_eq!(controller(t, x), P1);
    x
}

#[test]
fn xantcha_leaves_with_its_owner_and_is_exiled_if_its_controller_leaves() {
    cr!("800.4a");
    ruling!(
        "Xantcha, Sleeper Agent",
        "In a multiplayer game, if Xantcha's owner leaves the game, Xantcha leaves the game with them. If Xantcha's controller leaves the game, Xantcha is exiled."
    );
    // The owner leaves: Xantcha leaves the game with them.
    let mut t = TestGame::new(3);
    let x = xantcha(&mut t);
    t.g.lose_game(P0);
    t.settle();
    assert!(!t.on_battlefield(t.g.current(x)));
    assert!(t.named_on_battlefield("Xantcha, Sleeper Agent").is_empty());
    // The controller leaves: Xantcha is exiled.
    let mut t = TestGame::new(3);
    let x = xantcha(&mut t);
    t.g.lose_game(P1);
    t.settle();
    assert_eq!(t.zone(t.g.current(x)), Zone::Exile);
}

/// The player activates Xantcha's last ability ({3}, Wastes added) and it resolves.
fn xantcha_ability(t: &mut TestGame, x: ObjectId, who: PlayerId) {
    t.lands(who, "Wastes", 3);
    t.g.turn.priority = Some(who);
    t.activate(who, x, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn xantcha_activator_draws_controller_loses_life() {
    cr!("602.2", "109.5");
    ruling!(
        "Xantcha, Sleeper Agent",
        "The player who draws a card when Xantcha's last ability resolves is the player who activated the ability."
    );
    ruling!(
        "Xantcha, Sleeper Agent",
        "Xantcha's controller may activate its last ability. In this case, the same player loses 2 life and draws a card."
    );
    let mut t = TestGame::new(3);
    let x = xantcha(&mut t);
    for _ in 0..3 {
        for p in [P0, P1, P2] {
            t.library_top(p, "Island");
        }
    }
    let hands = |t: &TestGame| [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)];
    // P2 (neither owner nor controller) activates: P1 loses 2, P2 draws.
    let before = hands(&t);
    xantcha_ability(&mut t, x, P2);
    let after = hands(&t);
    assert_eq!((t.life(P1), t.life(P2)), (18, 20));
    assert_eq!(
        [
            after[0] - before[0],
            after[1] - before[1],
            after[2] - before[2]
        ],
        [0, 0, 1]
    );
    // P1 (its controller) activates: P1 loses 2 and draws.
    let before = hands(&t);
    xantcha_ability(&mut t, x, P1);
    let after = hands(&t);
    assert_eq!(t.life(P1), 16);
    assert_eq!(
        [
            after[0] - before[0],
            after[1] - before[1],
            after[2] - before[2]
        ],
        [0, 1, 0]
    );
}

#[test]
fn xantcha_enters_under_the_opponents_control_by_replacement() {
    cr!("614.1c", "614.12", "603.6a");
    ruling!(
        "Xantcha, Sleeper Agent",
        "Xantcha's first ability is a replacement effect that modifies how it enters the battlefield, not a triggered ability."
    );
    supported("Impassioned Orator");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Impassioned Orator");
    t.battlefield(P1, "Impassioned Orator");
    let x = xantcha(&mut t);
    // It never was P0's: P0's Orator didn't trigger, P1's did.
    assert_eq!((t.life(P0), t.life(P1)), (20, 21));
    // No control change happened after it entered.
    assert!(!t.g.turn_events.iter().any(|e| matches!(e,
        mtg_engine::events::Event::ControlChanged { obj, .. } if t.g.current(*obj) == x)));
}
