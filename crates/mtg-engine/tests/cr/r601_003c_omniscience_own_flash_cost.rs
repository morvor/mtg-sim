//! A card's own way of casting it as though it had flash for an additional cost ("You may cast
//! this spell as though it had flash if you pay {2} more to cast it.") combined with an
//! alternative cost of nothing from a static ability: Omniscience ("You may cast spells from
//! your hand without paying their mana costs."). The rules:
//!
//! - 601.3c "If an effect allows a player to cast a spell as though it had flash only if an
//!   alternative or additional cost is paid, that player may begin to cast that spell as
//!   though it had flash."
//! - 118.9a "Only one alternative cost can be applied to any one spell as it’s being cast.
//!   The controller of the spell announces their intentions to pay that cost as described in
//!   rule 601.2b."
//! - 118.9d "If an alternative cost is being paid to cast a spell, any additional costs, cost
//!   increases, and cost reductions that affect that spell are applied to that alternative
//!   cost. (See rule 601.2f.)"
//! - Omniscience's ruling (2018-07-13): "If you cast a spell "without paying its mana cost,"
//!   you can't choose to cast it for any alternative costs. You can, however, pay additional
//!   costs, such as kicker costs."
//!
//! So with Omniscience out, such a card in its owner's hand can be cast during an opponent's
//! turn for only the {2} (whether that {2} is read as an additional cost or a cost increase,
//! 118.9d applies it to the alternative cost), and likewise for the other wordings of such a
//! way ("by tapping three untapped creatures you control with flying in addition to paying
//! its other costs", "if you behold a Dragon as an additional cost to cast it"). Before the
//! fix the free way was offered only with normal timing (`Game::permitted_cast_options` gave
//! Omniscience's permission plain `CastMethod::Free` ways of its own; found by the November
//! 2026 mutation run of `permissions.rs`, docs/MUTATION.md). The same fix lets the card's
//! other own ways that aren't alternative costs combine with Omniscience too: fused
//! (CR 702.102a), "as though it had flash if it's cast using teamwork" (Quantum Reduction),
//! offering (CR 702.48a) and prototyped (CR 718.3); never its own alternative costs (evoke,
//! bestow, overload, casting it face down: 118.9a).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The method of a card's "as though it had flash if you pay {2} more" way.
fn flash_way(t: &TestGame, card: ObjectId) -> CastMethod {
    let uid = t
        .obj(card)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("as though it had flash"))
        .expect("flash ability")
        .uid;
    CastMethod::Alternative(uid)
}

/// P0 with Omniscience and `lands` Wastes, `name` in P0's hand, P1's end step with P0 holding
/// priority.
fn setup(name: &str, lands: usize) -> (TestGame, ObjectId, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let wastes = t.lands(P0, "Wastes", lands);
    let card = t.hand(P0, name);
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    (t, card, wastes)
}

/// Each of the nine cards with "You may cast this spell as though it had flash if you pay {2}
/// more to cast it." (with the reminder text "(You may cast it any time you could cast an
/// instant.)" or without it, on a single-faced card or the front face of a double-faced one):
/// with Omniscience out, during P1's end step, P0 casts it for {2} (two Wastes, whatever its
/// mana cost), and not with only one Wastes.
#[test]
fn omniscience_and_each_flash_for_2_more_card_cast_on_an_opponents_turn_for_2() {
    cr!("601.3c", "118.9d", "601.2f");
    ruling!(
        "Omniscience",
        "You can, however, pay additional costs, such as kicker costs."
    );
    for name in [
        "Rout",
        "Harbinger of the Tides",
        "Ghitu Fire",
        "Twilight's Call",
        "Breaking Wave",
        "Saproling Symbiosis",
        "Asinine Antics",
        "Mystical Tether",
        "Oakshade Stalker // Moonlit Ambusher",
    ] {
        let (mut t, card, wastes) = setup(name, 2);
        let flash = flash_way(&t, card);
        let ways = t.g.cast_options(P0, card);
        assert!(
            ways.iter().any(|o| o.method == flash
                && o.flash
                && o.alt_cost.as_ref().is_some_and(|c| c.is_free())
                && o.extra_cost.is_some()),
            "{name}: {ways:?}"
        );
        // Ghitu Fire's X is 0 (Omniscience's ruling), its target P1.
        if name == "Ghitu Fire" {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        let spell = t
            .cast(P0, card)
            .method(flash)
            .try_go()
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(t.obj(spell).zone, Zone::Stack, "{name}");
        assert!(wastes.iter().all(|w| t.obj_now(*w).tapped), "{name}");

        // One Wastes doesn't pay the {2}: cast as though it had flash, it can't be cast.
        let (mut t, card, _) = setup(name, 1);
        let flash = flash_way(&t, card);
        if name == "Ghitu Fire" {
            t.answer_targets(P0, &[Entity::Player(P1)]);
        }
        assert!(t.cast(P0, card).method(flash).try_go().is_err(), "{name}");
        assert_eq!(t.obj(card).zone, Zone::Hand(P0), "{name}");
    }
}

/// The other wordings of such a way: Tegwyll's Scouring ({4}{B}{B} sorcery, "You may cast this
/// spell as though it had flash by tapping three untapped creatures you control with flying in
/// addition to paying its other costs.") and Molten Exhale ({1}{R} sorcery, "You may cast this
/// spell as though it had flash if you behold a Dragon as an additional cost to cast it."): with
/// Omniscience, during P1's upkeep, P0 (no lands) casts each for that additional cost alone;
/// cast without it (Omniscience's own way, with sorcery timing) or without what it needs (two
/// flyers, no Dragon), it can't be.
#[test]
fn omniscience_and_the_other_wordings_of_flash_for_an_additional_cost() {
    cr!("601.3c", "118.9d", "701.4a");
    let opponents_upkeep = |t: &mut TestGame| {
        t.set_step(P1, Step::Upkeep);
        t.g.turn.priority = Some(P0);
    };
    // Tegwyll's Scouring: two flyers aren't enough; three are tapped to cast it.
    for flyers in [2, 3] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Omniscience");
        let birds: Vec<ObjectId> = (0..flyers)
            .map(|_| t.battlefield(P0, "Birds of Paradise"))
            .collect();
        let bears = t.battlefield(P1, "Grizzly Bears");
        let scouring = t.hand(P0, "Tegwyll's Scouring");
        opponents_upkeep(&mut t);
        assert!(t.cast(P0, scouring).method(CastMethod::Free).try_go().is_err());
        let flash = flash_way(&t, scouring);
        let cast = t.cast(P0, scouring).method(flash).try_go();
        if flyers == 2 {
            assert!(cast.is_err());
            assert_eq!(t.obj(scouring).zone, Zone::Hand(P0));
            continue;
        }
        assert!(cast.is_ok(), "{cast:?}");
        assert!(birds.iter().all(|b| t.obj_now(*b).tapped));
        t.resolve_all();
        assert!(!t.on_battlefield(bears));
        assert!(birds.iter().all(|b| !t.on_battlefield(*b)));
    }
    // Molten Exhale: with a Dragon to behold, it deals 4 damage to P1's Grizzly Bears.
    for dragon in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Omniscience");
        if dragon {
            t.battlefield(P0, "Shivan Dragon");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        let exhale = t.hand(P0, "Molten Exhale");
        opponents_upkeep(&mut t);
        assert!(t
            .cast(P0, exhale)
            .method(CastMethod::Free)
            .target(bears)
            .try_go()
            .is_err());
        t.clear_answers();
        let flash = flash_way(&t, exhale);
        let cast = t.cast(P0, exhale).method(flash).target(bears).try_go();
        assert_eq!(cast.is_ok(), dragon, "{cast:?}");
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), !dragon);
    }
}

/// Rout cast with Omniscience during P1's end step for its {2} destroys P1's Grizzly Bears;
/// cast normally then (Omniscience's own way, with sorcery timing) it can't be; during P0's
/// main phase it's cast for nothing at all.
#[test]
fn rout_with_omniscience_cast_at_instant_speed_for_2_or_at_sorcery_speed_for_nothing() {
    cr!("601.3c", "118.9d", "307.1");
    ruling!(
        "Omniscience",
        "You must follow the normal timing permissions and restrictions of each spell you cast."
    );
    let (mut t, rout, wastes) = setup("Rout", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(t.cast(P0, rout).method(CastMethod::Free).try_go().is_err());
    let flash = flash_way(&t, rout);
    t.cast(P0, rout).method(flash).go();
    assert!(wastes.iter().all(|w| t.obj_now(*w).tapped));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));

    // P0's main phase: no mana needed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rout = t.hand(P0, "Rout");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, rout).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

/// With mana for both, P0 chooses between casting Rout as though it had flash paying its mana
/// cost and the {2} (seven Plains) or paying only the {2} (Omniscience's alternative cost): two
/// ways of casting it with the same flash ability, the second named as cast without paying its
/// mana cost. Chosen, it taps two Plains.
#[test]
fn with_mana_for_both_the_player_chooses_whether_to_pay_routs_mana_cost() {
    cr!("118.9", "601.2b", "601.3c");
    let (mut t, rout, plains) = setup("Rout", 0);
    let plains = [plains, t.lands(P0, "Plains", 7)].concat();
    let flash = flash_way(&t, rout);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, rout).method(flash).go();
    let offered: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1, "{offered:?}");
    assert_eq!(offered[0].len(), 2, "{offered:?}");
    assert_eq!(offered[0][1], "without paying its mana cost");
    assert_eq!(plains.iter().filter(|l| t.obj_now(**l).tapped).count(), 2);
}

/// Ghitu Fire ({X}{R}) cast with Omniscience as though it had flash: X is 0 (an X of 3 the
/// player would announce isn't a legal choice) and only the {2} is paid, which isn't part of
/// its mana cost: on the stack its mana value is 1, and it deals no damage.
#[test]
fn ghitu_fire_cast_free_with_flash_x_is_0_and_the_2_isnt_its_mana_cost() {
    cr!("601.3c", "118.9d", "107.3b", "202.3e");
    ruling!(
        "Ghitu Fire",
        "The extra 2 mana to cast it as an instant does not count as part of the mana cost."
    );
    ruling!(
        "Omniscience",
        "If a spell has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let (mut t, fire, wastes) = setup("Ghitu Fire", 2);
    let flash = flash_way(&t, fire);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, fire).method(flash).x(3).go();
    assert!(wastes.iter().all(|w| t.obj_now(*w).tapped));
    assert_eq!(t.obj(spell).stack.as_ref().and_then(|s| s.x), Some(0));
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

/// Wear // Tear with Omniscience: cast from P0's hand, both halves, without paying their mana
/// costs (P0 has no lands): P1's Ornithopter and Glorious Anthem are destroyed.
#[test]
fn omniscience_casts_a_fuse_card_fused_without_paying_its_mana_cost() {
    cr!("702.102a", "118.9");
    ruling!(
        "Wear // Tear",
        "If you cast a split card with fuse from your hand without paying its mana cost, you can choose to use its fuse ability and cast both halves without paying their mana costs."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let thopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let wt = t.hand(P0, "Wear // Tear");
    t.set_step(P0, Step::PrecombatMain);
    let fuse = CastMethod::Keyword(KeywordKind::Fuse);
    let ways = t.g.cast_options(P0, wt);
    assert!(
        ways.iter()
            .any(|o| o.method == fuse && o.alt_cost.as_ref().is_some_and(|c| c.is_free())),
        "{ways:?}"
    );
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    t.cast(P0, wt).method(fuse).go();
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
    assert!(!t.on_battlefield(anthem));
}

/// Quantum Reduction ({1}{U} Aura, "Teamwork 2 ... You may cast this spell as though it had
/// flash if it's cast using teamwork.") with Omniscience: during P1's turn P0, with no lands,
/// casts it using teamwork (Grizzly Bears tapped) without paying its mana cost.
#[test]
fn omniscience_and_flash_if_cast_using_teamwork() {
    cr!("601.3c", "118.9d", "702.194a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Quantum Reduction");
    t.set_step(P1, Step::BeginningOfCombat);
    let teamwork = CastMethod::Keyword(KeywordKind::Teamwork);
    assert!(t.cast(P0, c).method(CastMethod::Free).target(wurm).try_go().is_err());
    t.clear_answers();
    t.cast(P0, c).method(teamwork).target(wurm).go();
    assert!(t.obj(bears).tapped);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (1, 4));
}

/// Patron of the Kitsune ({4}{W}{W}, "Fox offering (You may cast this spell any time you could
/// cast an instant by sacrificing a Fox and paying the difference in mana costs between this
/// and the sacrificed Fox. Mana cost includes color.)") with Omniscience: during P1's upkeep
/// P0, with no lands, casts it with offering (Kitsune Blademaster, a Fox, sacrificed) without
/// paying its mana cost; cast without offering it can't be then.
#[test]
fn omniscience_and_offering() {
    cr!("702.48a", "601.3c", "118.9d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let fox = t.battlefield(P0, "Kitsune Blademaster");
    let patron = t.hand(P0, "Patron of the Kitsune");
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, patron).method(CastMethod::Free).try_go().is_err());
    t.cast(P0, patron)
        .method(CastMethod::Keyword(KeywordKind::Offering))
        .go();
    assert!(!t.on_battlefield(fox));
    assert!(t.in_graveyard(P0, "Kitsune Blademaster"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Patron of the Kitsune").len(), 1);
}

/// Blitz Automaton ({7}, prototype {2}{R} — 3/2) with Omniscience and no lands: cast
/// prototyped without paying its mana cost, it's a 3/2.
#[test]
fn omniscience_casts_a_prototype_card_prototyped_without_paying_its_mana_cost() {
    cr!("718.3", "702.160a", "118.9");
    ruling!(
        "Blitz Automaton",
        "if an effect allows you to cast an artifact card without paying its mana cost, you could either cast Blitz Automaton normally, or as a prototyped spell."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let c = t.hand(P0, "Blitz Automaton");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, c)
        .method(CastMethod::Keyword(KeywordKind::Prototype))
        .go();
    t.resolve_all();
    let automaton = t.named_on_battlefield("Blitz Automaton")[0];
    assert_eq!(t.pt(automaton), (3, 2));
}

/// Only one alternative cost: Omniscience's doesn't combine with a card's own alternative
/// costs. With Omniscience (P0 has no lands, P0's main phase) Walker of the Grove (evoke {4}{G})
/// can't be evoked, Kestia, the Cultivator (bestow {3}{G}{W}{U}) can't be cast bestowed,
/// Cyclonic Rift (overload {6}{U}) can't be overloaded and Akroma, Angel of Fury (morph) can't be
/// cast face down (for {3}, CR 702.37c) without paying those costs; each is cast normally
/// without paying its mana cost: Walker of the Grove stays (not evoked), Kestia is a creature,
/// Cyclonic Rift returns only its target.
#[test]
fn omniscience_doesnt_combine_with_the_cards_own_alternative_costs() {
    cr!("118.9a", "601.2b", "702.74a", "702.103a", "702.96a", "702.37c");
    ruling!(
        "Walker of the Grove",
        "If you're casting a spell \"without paying its mana cost,\" you can't use its evoke ability."
    );
    ruling!(
        "Kestia, the Cultivator",
        "Bestow is an alternative cost to cast the spell with bestow. It can't be combined with other alternative costs, such as casting a spell \"without paying its mana cost.\""
    );
    let main_phase = || {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Omniscience");
        t.set_step(P0, Step::PrecombatMain);
        t.g.turn.priority = Some(P0);
        t
    };
    let free_with = |t: &TestGame, card: ObjectId, m: CastMethod| {
        t.g.cast_options(P0, card)
            .iter()
            .any(|o| o.method == m && o.alt_cost.as_ref().is_some_and(|c| c.is_free()))
    };

    let mut t = main_phase();
    let walker = t.hand(P0, "Walker of the Grove");
    let evoke = CastMethod::Keyword(KeywordKind::Evoke);
    assert!(!free_with(&t, walker, evoke.clone()));
    assert!(t.cast(P0, walker).method(evoke).try_go().is_err());
    t.cast(P0, walker).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Walker of the Grove").len(), 1);

    let mut t = main_phase();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let kestia = t.hand(P0, "Kestia, the Cultivator");
    let bestow = CastMethod::Keyword(KeywordKind::Bestow);
    assert!(!free_with(&t, kestia, bestow.clone()));
    assert!(t.cast(P0, kestia).method(bestow).target(bears).try_go().is_err());
    t.clear_answers();
    t.cast(P0, kestia).method(CastMethod::Free).go();
    t.resolve_all();
    let k = t.named_on_battlefield("Kestia, the Cultivator")[0];
    assert!(t.obj(k).is_creature());
    assert_eq!(t.obj(k).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));

    let mut t = main_phase();
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Ornithopter");
    let rift = t.hand(P0, "Cyclonic Rift");
    let overload = CastMethod::Keyword(KeywordKind::Overload);
    assert!(!free_with(&t, rift, overload.clone()));
    assert!(t.cast(P0, rift).method(overload).try_go().is_err());
    t.cast(P0, rift).method(CastMethod::Free).target(theirs).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(other));

    let mut t = main_phase();
    let akroma = t.hand(P0, "Akroma, Angel of Fury");
    let face_down = CastMethod::FaceDown(KeywordKind::Morph);
    assert!(!free_with(&t, akroma, face_down.clone()));
    assert!(t.cast(P0, akroma).method(face_down).try_go().is_err());
    assert_eq!(t.obj(akroma).zone, Zone::Hand(P0));
}
