//! Rulings batch P085 — graveyard hate: optional ("up to") graveyard targets, targets that
//! become illegal, who chooses, and casting creature cards exiled from graveyards.

use crate::r_p085_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts `spell` (paid from `types`) with the queued targets, then `respond` runs with
/// the spell on the stack, and everything resolves.
fn cast_then(
    t: &mut TestGame,
    spell: &str,
    types: &[(ManaType, u32)],
    targets: &[&[Entity]],
    respond: impl FnOnce(&mut TestGame),
) -> ObjectId {
    supported(spell);
    let id = t.hand(P0, spell);
    pool(t, P0, types);
    for ts in targets {
        t.answer_targets(P0, ts);
    }
    t.cast(P0, id).go();
    respond(t);
    t.resolve_all();
    id
}

/// Moves a graveyard card to exile (a response that makes a graveyard target illegal).
fn exile_now(t: &mut TestGame, card: ObjectId) {
    let c = t.g.current(card);
    t.g.move_object(c, Zone::Exile, events::MoveCause::Effect, None);
}

#[test]
fn rotten_reunion_zero_or_illegal_target() {
    cr!("115.1", "608.2b");
    ruling!(
        "Rotten Reunion",
        "You don't have to choose any targets for Rotten Reunion. However, if you do, and that target is illegal"
    );
    // No target: the Zombie is still created.
    let mut t = TestGame::new(2);
    cast_then(
        &mut t,
        "Rotten Reunion",
        &[(ManaType::B, 1)],
        &[&[]],
        |_| {},
    );
    assert_eq!(tokens_of(&t, P0, "Zombie"), 1);
    // A target that left the graveyard: the spell doesn't resolve, no Zombie.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_then(
        &mut t,
        "Rotten Reunion",
        &[(ManaType::B, 1)],
        &[&[Entity::Object(bears)]],
        |t| exile_now(t, bears),
    );
    assert_eq!(tokens_of(&t, P0, "Zombie"), 0);
}

#[test]
fn heritage_reclamation_third_mode_zero_or_illegal_target() {
    cr!("115.1", "608.2b", "700.2");
    ruling!(
        "Heritage Reclamation",
        "You don’t have to choose a target for Heritage Reclamation if you choose its third mode."
    );
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P1, "Grizzly Bears");
        let spell = t.hand(P0, "Heritage Reclamation");
        pool(&mut t, P0, &[(ManaType::G, 2)]);
        let hand = t.hand_size(P0);
        let targets: Vec<Entity> = if illegal {
            vec![Entity::Object(bears)]
        } else {
            vec![]
        };
        t.cast(P0, spell).modes(&[2]).targets(&targets).go();
        if illegal {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        // The spell left the hand; a card is drawn only if it resolved.
        let drew = t.hand_size(P0) + 1 - hand;
        assert_eq!(drew, if illegal { 0 } else { 1 }, "illegal={illegal}");
    }
}

#[test]
fn turn_the_earth_no_targets_just_gains_life() {
    cr!("115.1", "601.2c");
    ruling!(
        "Turn the Earth",
        "You may choose no targets for Turn the Earth. In that case, no libraries will be shuffled"
    );
    let mut t = TestGame::new(2);
    cast_then(
        &mut t,
        "Turn the Earth",
        &[(ManaType::G, 1)],
        &[&[]],
        |_| {},
    );
    assert_eq!(t.life(P0), 22);
    assert!(!shuffled(&t, P0) && !shuffled(&t, P1));
}

#[test]
fn serene_remembrance_zero_targets_still_shuffles_itself() {
    cr!("115.1", "601.2c");
    ruling!(
        "Serene Remembrance",
        "You may choose zero targets when you cast Serene Remembrance."
    );
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    let id = cast_then(
        &mut t,
        "Serene Remembrance",
        &[(ManaType::G, 1)],
        &[&[]],
        |_| {},
    );
    assert_eq!(t.zone(id), Zone::Library(P0));
    assert_eq!(t.library_size(P0), lib + 1);
}

#[test]
fn stream_of_consciousness_targets() {
    cr!("115.1", "601.2c");
    ruling!(
        "Stream of Consciousness",
        "You choose the target cards; the target player doesn’t."
    );
    ruling!(
        "Stream of Consciousness",
        "You may target zero cards in any graveyard and have a target player simply shuffle their library."
    );
    // P0 picks two of P1's three graveyard cards: exactly those are shuffled in.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let c = t.graveyard(P1, "Llanowar Elves");
    cast_then(
        &mut t,
        "Stream of Consciousness",
        &[(ManaType::U, 2)],
        &[
            &[Entity::Player(P1)],
            &[Entity::Object(a), Entity::Object(c)],
        ],
        |_| {},
    );
    assert_eq!(t.zone(a), Zone::Library(P1));
    assert_eq!(t.zone(c), Zone::Library(P1));
    assert_eq!(t.zone(b), Zone::Graveyard(P1));
    assert!(!chose_anything(&t, P1), "the target player made a choice");
    // Zero cards: P1 just shuffles.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    cast_then(
        &mut t,
        "Stream of Consciousness",
        &[(ManaType::U, 2)],
        &[&[Entity::Player(P1)], &[]],
        |_| {},
    );
    assert!(shuffled(&t, P1));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn memorys_journey_needs_a_player_but_no_cards() {
    cr!("115.1", "601.2c");
    ruling!(
        "Memory's Journey",
        "You don't have to target any cards when you cast Memory's Journey, but you must target a player."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast_then(
        &mut t,
        "Memory's Journey",
        &[(ManaType::U, 2)],
        &[&[Entity::Player(P1)], &[]],
        |_| {},
    );
    assert!(shuffled(&t, P1));
    // The player target is required; the card targets are "up to three".
    let mins: Vec<u32> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { min, .. } => Some(*min),
            _ => None,
        })
        .collect();
    assert_eq!(mins, vec![1, 0]);
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

#[test]
fn not_forgotten_choice_and_token() {
    cr!("608.2c", "111.2");
    ruling!(
        "Not Forgotten",
        "You choose whether to put the target card on the top or bottom of its owner’s library as Not Forgotten resolves."
    );
    ruling!(
        "Not Forgotten",
        "You get the Spirit token, not the owner of the target card."
    );
    for bottom in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if bottom { 1 } else { 0 }),
        );
        cast_then(
            &mut t,
            "Not Forgotten",
            &[(ManaType::W, 2)],
            &[&[Entity::Object(bears)]],
            |_| {},
        );
        let now = t.g.current(bears);
        let lib = &t.g.player(P1).library;
        if bottom {
            assert_eq!(lib.first(), Some(&now));
        } else {
            assert_eq!(lib.last(), Some(&now));
        }
        assert_eq!(tokens_of(&t, P0, "Spirit"), 1);
        assert_eq!(tokens_of(&t, P1, "Spirit"), 0);
        // The choice was made by P0 as the spell resolved.
        assert!(!chose_anything(&t, P1));
    }
}

#[test]
fn gollum_optional_target() {
    cr!("115.1", "608.2b");
    ruling!(
        "Gollum the Abandoned",
        "You don't have to choose a target for Gollum's second ability."
    );
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        supported("Gollum the Abandoned");
        let bears = t.graveyard(P1, "Grizzly Bears");
        if illegal {
            t.answer_targets(P0, &[Entity::Object(bears)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        t.enter(P0, "Gollum the Abandoned");
        t.settle();
        if illegal {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.life(P1),
            if illegal { 20 } else { 18 },
            "illegal={illegal}"
        );
    }
}

#[test]
fn rooftop_percher_optional_targets() {
    cr!("115.1", "608.2b");
    ruling!(
        "Rooftop Percher",
        "You don't have to choose any targets for Rooftop Percher's triggered ability."
    );
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        supported("Rooftop Percher");
        let a = t.graveyard(P1, "Grizzly Bears");
        let b = t.graveyard(P1, "Hill Giant");
        if illegal {
            t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
        } else {
            t.answer_targets(P0, &[]);
        }
        t.enter(P0, "Rooftop Percher");
        t.settle();
        if illegal {
            exile_now(&mut t, a);
            exile_now(&mut t, b);
        }
        t.resolve_all();
        assert_eq!(
            t.life(P0),
            if illegal { 20 } else { 23 },
            "illegal={illegal}"
        );
    }
    // Only one of two targets illegal: the ability resolves (CR 608.2b).
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Rooftop Percher");
    t.settle();
    exile_now(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.zone(b), Zone::Exile);
}

#[test]
fn immersturm_predator_illegal_target_no_counter() {
    cr!("115.1", "608.2b");
    ruling!(
        "Immersturm Predator",
        "you won't put a +1/+1 counter on Immersturm Predator"
    );
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        supported("Immersturm Predator");
        let pred = t.battlefield(P0, "Immersturm Predator");
        let bears = t.graveyard(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.g.tap(pred);
        t.g.flush_events();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if illegal {
            exile_now(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(
            t.counters(pred, "+1/+1"),
            if illegal { 0 } else { 1 },
            "illegal={illegal}"
        );
    }
    // No target at all: the counter is still put.
    let mut t = TestGame::new(2);
    let pred = t.battlefield(P0, "Immersturm Predator");
    t.answer_targets(P0, &[]);
    t.g.tap(pred);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(pred, "+1/+1"), 1);
}

#[test]
fn stonecloaker_may_return_itself() {
    cr!("608.2c");
    ruling!(
        "Stonecloaker",
        "You may return Stonecloaker itself to its owner's hand as its first triggered ability resolves."
    );
    supported("Stonecloaker");
    // Alone: it must return itself.
    let mut t = TestGame::new(2);
    let s = t.enter(P0, "Stonecloaker");
    t.resolve_all();
    assert_eq!(t.zone(s), Zone::Hand(P0));
    // With another creature, it may choose itself.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = t.enter(P0, "Stonecloaker");
    t.settle();
    t.answer_choose(P0, &[Entity::Object(s)]);
    t.resolve_all();
    assert_eq!(t.zone(s), Zone::Hand(P0));
    assert!(t.on_battlefield(bears));
}

#[test]
fn cremate_and_crypt_creeper_need_a_graveyard_target() {
    cr!("601.2c", "602.2b", "115.1");
    ruling!(
        "Cremate",
        "You must be able to target a card in a graveyard to cast Cremate."
    );
    ruling!(
        "Crypt Creeper",
        "You must be able to target a card in a graveyard to activate Crypt Creeper’s ability. It can’t target itself."
    );
    supported("Cremate");
    supported("Crypt Creeper");
    let mut t = TestGame::new(2);
    let cremate = t.hand(P0, "Cremate");
    pool(&mut t, P0, &[(ManaType::B, 1)]);
    assert!(t.cast(P0, cremate).try_go().is_err());
    assert!(t.in_hand(P0, "Cremate"));
    let creeper = t.battlefield(P0, "Crypt Creeper");
    assert!(t.activate(P0, creeper, 0, &[]).is_err());
    assert!(
        t.on_battlefield(creeper),
        "the cost was paid without a target"
    );
    // With a card in a graveyard, both work.
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.activate(P0, creeper, 0, &[Entity::Object(bears)])
        .unwrap();
    // Only the Bears were a possible target: not the Creeper itself.
    let candidates: Vec<Vec<Entity>> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(candidates, vec![vec![Entity::Object(bears)]]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn dreams_of_steel_and_oil_must_choose() {
    cr!("608.2c");
    ruling!(
        "Dreams of Steel and Oil",
        "You must choose an artifact or creature card from their hand if they reveal one."
    );
    supported("Dreams of Steel and Oil");
    let mut t = TestGame::new(2);
    let h = t.hand(P1, "Grizzly Bears");
    let g = t.graveyard(P1, "Ornithopter");
    // P0 tries to choose nothing: the engine still chooses one of each.
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    cast_then(
        &mut t,
        "Dreams of Steel and Oil",
        &[(ManaType::B, 1)],
        &[&[Entity::Player(P1)]],
        |_| {},
    );
    assert_eq!(t.zone(h), Zone::Exile);
    assert_eq!(t.zone(g), Zone::Exile);
}

#[test]
fn carrion_beetles_zero_to_three_targets() {
    cr!("115.1", "602.2b");
    ruling!(
        "Carrion Beetles",
        "You pick the 0, 1, 2, or 3 target cards on announcement."
    );
    supported("Carrion Beetles");
    let mut t = TestGame::new(2);
    let beetles = t.battlefield(P0, "Carrion Beetles");
    pool(&mut t, P0, &[(ManaType::B, 3)]);
    assert!(t.activate(P0, beetles, 0, &[]).unwrap().is_some());
    t.resolve_all();
    let cards: Vec<_> = ["Grizzly Bears", "Hill Giant", "Llanowar Elves"]
        .iter()
        .map(|n| t.graveyard(P1, n))
        .collect();
    t.g.untap(beetles);
    pool(&mut t, P0, &[(ManaType::B, 3)]);
    t.answer_targets(
        P0,
        &cards.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>(),
    );
    t.activate(P0, beetles, 0, &[]).unwrap();
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile));
}

#[test]
fn rapid_decay_partial_targets() {
    cr!("608.2b");
    ruling!(
        "Rapid Decay",
        "You pick the cards during announcement. If any are not there on resolution, any others that are there are still affected."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let c = t.graveyard(P1, "Llanowar Elves");
    cast_then(
        &mut t,
        "Rapid Decay",
        &[(ManaType::B, 2)],
        &[&[Entity::Object(a), Entity::Object(b), Entity::Object(c)]],
        |t| {
            let bn = t.g.current(b);
            t.g.move_object(bn, Zone::Hand(P1), events::MoveCause::Effect, None);
        },
    );
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(c), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Hand(P1));
}

#[test]
fn dawnhand_dissident_cast_from_exile_follows_timing() {
    cr!("601.3", "307.1");
    ruling!(
        "Dawnhand Dissident",
        "You must still pay all costs and follow any timing restrictions and permissions for creature spells you cast from exile this way."
    );
    supported("Dawnhand Dissident");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dawnhand Dissident");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    let own = t.graveyard(P0, "Grizzly Bears");
    // Blight 2 twice (two -1/-1 counters on each Hill Giant): four counters to remove.
    for card in [theirs, own] {
        let giant = t.battlefield(P0, "Hill Giant");
        t.g.untap(d);
        t.answer_choose(P0, &[Entity::Object(giant)]);
        t.activate(P0, d, 1, &[Entity::Object(card)]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(card), Zone::Exile);
    }
    let theirs = t.g.current(theirs);
    let own = t.g.current(own);
    // A card P0 doesn't own can't be cast this way.
    pool(&mut t, P0, &[(ManaType::G, 2)]);
    assert!(t.cast(P0, theirs).try_go().is_err());
    // P1's turn: not castable.
    t.set_step(P1, Step::PrecombatMain);
    pool(&mut t, P0, &[(ManaType::G, 2)]);
    assert!(t.cast(P0, own).try_go().is_err());
    // P0's main phase with a nonempty stack: not castable (creature timing).
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    pool(&mut t, P1, &[(ManaType::R, 1)]);
    t.cast(P1, bolt).target(P1).go();
    pool(&mut t, P0, &[(ManaType::G, 2)]);
    assert!(t.cast(P0, own).try_go().is_err());
    t.resolve_all();
    // Without the mana: no.
    t.g.players[P0.idx()].mana_pool = Default::default();
    assert!(t.cast(P0, own).try_go().is_err());
    // Main phase, empty stack, costs paid: yes.
    pool(&mut t, P0, &[(ManaType::G, 2)]);
    t.cast(P0, own).go();
    t.resolve_all();
    assert_eq!(t.zone(own), Zone::Battlefield);
    // Three of the four -1/-1 counters were removed as the additional cost.
    let left: u32 = t
        .named_on_battlefield("Hill Giant")
        .iter()
        .map(|g| t.counters(*g, "-1/-1"))
        .sum();
    assert_eq!(left, 1);
}
