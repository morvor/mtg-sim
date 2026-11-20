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
//! 118.9d applies it to the alternative cost). Before the fix the free way was offered only
//! with normal timing (`Game::permitted_cast_options` gave Omniscience's permission plain
//! `CastMethod::Free` ways of its own; found by the November 2026 mutation run of
//! `permissions.rs`, docs/MUTATION.md). The same fix lets the card's other own ways that
//! aren't alternative costs combine with Omniscience too: fused (CR 702.102a), "as though it
//! had flash if it's cast using teamwork" (Quantum Reduction) and prototyped (CR 718.3).

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
    cr!("601.3c", "118.9a", "118.9d", "601.2f");
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

/// Ghitu Fire ({X}{R}) cast with Omniscience as though it had flash: X is 0 and only the {2}
/// is paid, which isn't part of its mana cost: on the stack its mana value is 1.
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
    let spell = t.cast(P0, fire).method(flash).go();
    assert!(wastes.iter().all(|w| t.obj_now(*w).tapped));
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
