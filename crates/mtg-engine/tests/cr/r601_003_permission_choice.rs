//! Which permission a player uses to play a card, and the terms it comes with (CR 601.3,
//! 305.1; `permissions.rs`). A card that its own permission lets the player play (a rule,
//! such as casting it from the hand, or its own ability, such as mayhem) is played with that
//! permission: an effect's permission that would let them play it the same way isn't
//! attached to that way, isn't asked about and isn't used up (`permissions::attach`,
//! `permissions::choose_land_permission`). The player announces which permission they're
//! using (Muldrotha, the Gravetide ruling); the engine announces the card's own one for them,
//! which leaves them every option the other would (it uses nothing up). A cost a permission
//! sets relative to the spell ("waterbending {X} ..., where X is its mana value") is the
//! spell's: for a split card, that of the half cast, not the card's combined mana value
//! where it is (`permissions::spell_relative_cost`, CR 709.3a). Written for survivors of the
//! November 2026 mutation run of `permissions.rs` (docs/MUTATION.md).

use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether a "Choose the permission you're using" question was asked since `from`.
fn asked_for_a_permission(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseOption { prompt, .. } | Decision::ChooseEntities { prompt, .. } => {
            prompt.starts_with("Choose the permission")
        }
        _ => false,
    })
}

/// `p` discards `card` from their hand; returns the card in the graveyard.
fn discard(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    let new = t.g.discard(p, card, None).expect("discarded");
    t.g.flush_events();
    new
}

/// Kills `permissions.rs:510:46` with `==` and with `>=` (in `permissions::attach`,
/// `perm.terms.cost_reduction > 0` as `== 0` or `>= 0`: true for every permission that
/// reduces nothing, so each of them "changes the way" a card's own permission allows). The
/// Grizzly Bears in P0's hand can be cast by the rules (its own permission) for its mana cost,
/// and without paying it by Omniscience (an alternative cost, CR 118.9, offered as a way of
/// its own). Omniscience's permission, for cards in P0's hand, also allows the Bears, but
/// cast for its mana cost the Bears use none of it. The mutants add a second way to cast it
/// for its mana cost, "with Omniscience's permission", and casting it then asks "Choose the
/// permission you're using" between that way and its own. (The pending `attach` fix, which
/// skips a card's own way with an additional cost of its own outside the hand, leaves cards
/// in the hand as they are: this test still kills both mutants after it.)
#[test]
fn a_card_cast_from_hand_for_its_mana_cost_with_omniscience_out_asks_no_permission() {
    cr!("601.3", "118.9b");
    interp!("own-permission-first");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    let forests = t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.g.recompute();
    // For its mana cost and without paying it: once each, with no effect's permission.
    let ways = t.g.cast_options(P0, bears);
    assert_eq!(ways.len(), 2, "{ways:?}");
    assert!(ways.iter().all(|o| o.permission.is_none()), "{ways:?}");
    assert_eq!(
        ways.iter()
            .filter(|o| o.method == CastMethod::Normal)
            .count(),
        1
    );
    assert_eq!(
        ways.iter().filter(|o| o.method == CastMethod::Free).count(),
        1
    );
    // Cast for its mana cost: no permission to announce, and the Forests pay {1}{G}.
    let from = t.asked().len();
    t.cast(P0, bears).go();
    assert!(!asked_for_a_permission(&t, from));
    assert!(forests.iter().all(|f| t.obj_now(*f).tapped));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

/// Kills `permissions.rs:620:42` (in `permissions::choose_land_permission`, `g.obj(card).zone
/// == Zone::Hand(p) || crate::kw::playable_lands(g, p).contains(&card)` with `&&`: the land's
/// own permission is never taken, as `kw::playable_lands` lists no card in a hand, and a land
/// that an effect's permission also allows is played with that one). Oscorp Industries,
/// discarded this turn, may be played from P0's graveyard by its own mayhem ability, and
/// Muldrotha's permission to play a land from the graveyard during each of P0's turns allows
/// it too. The engine plays it with its own permission, which uses nothing up: Muldrotha's
/// land is still available, and the Forest in P0's graveyard is played with it and
/// Exploration's additional land play. The mutant plays Oscorp Industries with Muldrotha's
/// permission (the only effect's permission there, so nobody is asked), whose land is then
/// used up: the Forest can't be played.
#[test]
fn a_land_played_with_its_own_mayhem_ability_leaves_muldrothas_land_unused() {
    cr!("702.187c", "305.2");
    interp!("own-permission-first");
    ruling!(
        "Muldrotha, the Gravetide",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    let mut t = TestGame::new(2);
    let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
    t.battlefield(P0, "Exploration");
    let oscorp = t.hand(P0, "Oscorp Industries");
    let oscorp = discard(&mut t, P0, oscorp);
    let forest = t.graveyard(P0, "Forest");
    t.play_land(P0, oscorp).unwrap();
    assert!(t.g.history.once_permissions_used.is_empty());
    // "When this land enters from a graveyard, you lose 2 life."
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // Muldrotha's permission plays the Forest, with the additional land play.
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "land".into())]
    );
}

/// Kills `permissions.rs:598:21` (in `permissions::spell_relative_cost`, the arm for an
/// `Effect::KeywordAction` cost deleted) and `583:38` with `false` (the guard `matches!(**s,
/// Sel::This)` of "its mana value": no amount is given for the spell). Hama, the Bloodbender's
/// permission lets P0 cast the card it exiled "by waterbending {X} rather than paying its mana
/// cost, where X is its mana value". It exiles Fire // Ice, whose mana value in exile is that
/// of its two halves combined, 4 (CR 709.4b); cast as Fire it's a spell with mana value 2
/// (CR 709.3a), and that's the X. With either mutant the X stays "its mana value", read as the
/// spell is paid for (on the stack: 2, so the payment itself doesn't change), but read for the
/// card in exile (4) when the engine judges whether P0 can begin to cast it: with an Island
/// and Hama to tap, P0 can pay {2} but not {4}, so the cast isn't among P0's legal actions.
#[test]
fn hama_waterbends_the_mana_value_of_the_half_cast_from_a_split_card() {
    cr!("709.4b", "709.3a", "701.67a", "118.9");
    ruling!(
        "Fire // Ice",
        "A split card's characteristics are a combination of its two halves while it is not on the stack"
    );
    ruling!(
        "Hama, the Bloodbender",
        "You can tap any untapped creature or artifact you control to pay a waterbend cost, even one you haven't controlled continuously since the beginning of your most recent turn."
    );
    let mut t = TestGame::new(2);
    let fire_ice = t.graveyard(P1, "Fire // Ice");
    // Hama enters: P1 mills three cards, and P0 exiles Fire // Ice from P1's graveyard.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(fire_ice)]);
    let hama = t.enter(P0, "Hama, the Bloodbender");
    t.resolve_all();
    let exiled = t.g.current(fire_ice);
    assert_eq!(t.zone(exiled), Zone::Exile, "{}", t.dump_log());
    assert_eq!(t.g.mana_value_of(exiled), 4);
    let island = t.battlefield(P0, "Island");
    // Fire (or Ice) for waterbend {2}: the Island and Hama pay it.
    let pay = CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST);
    t.g.turn.priority = Some(P0);
    let actions = t.g.legal_actions(P0);
    assert!(
        actions.contains(&Action::Cast {
            card: exiled,
            method: pay.clone()
        }),
        "{actions:?}"
    );
    let fire = t.cast(P0, exiled).method(pay).target(P1).go();
    assert_eq!(t.obj(fire).chars.name, "Fire");
    assert!(t.obj_now(island).tapped);
    assert!(t.obj_now(hama).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
