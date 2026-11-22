//! Which permission a player uses to play a card, and the terms it comes with (CR 601.3,
//! 305.1; `permissions.rs`). When two or more permissions allow playing a card the same
//! way, the card's own among them (its own ability, such as mayhem or aftermath), the
//! player announces which one they're using (Muldrotha, the Gravetide ruling), by default
//! the card's own, which uses nothing up; an effect's permission chosen is recorded on the
//! card and used up (`permissions::attach`, `permissions::choose_land_permission`,
//! `permissions::choose_cast_permission`; register `permission-choice-always-asked`). In
//! the player's own hand, where the rules let them cast the card, an effect's permission is
//! offered with the card's own way only if it changes how it's cast. A cost a permission
//! sets relative to the spell ("waterbending {X} ..., where X is its mana value") is the
//! spell's: for a split card, that of the half cast, not the card's combined mana value
//! where it is (`permissions::spell_relative_cost`, CR 709.3a). Written for survivors of the
//! November 2026 mutation run of `permissions.rs` (docs/MUTATION.md).

use mtg_engine::decision::{Action, Answer};
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

/// Adds `n` mana of type `ty` to `p`'s pool.
fn mana(t: &mut TestGame, p: PlayerId, ty: mtg_engine::mana::ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
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
/// permission you're using" between that way and its own. (The `attach` fix of commit
/// `c83fe93`, which skips a card's own way with an additional cost of its own outside the
/// hand, leaves cards in the hand as they are: this test still kills both mutants after it.)
#[test]
fn a_card_cast_from_hand_for_its_mana_cost_with_omniscience_out_asks_no_permission() {
    cr!("601.3", "118.9b");
    interp!("permission-choice-always-asked");
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

/// Oscorp Industries, discarded this turn, may be played from P0's graveyard by its own
/// mayhem ability, and Muldrotha's permission to play a land from the graveyard during each
/// of P0's turns allows it too. P0 is asked which permission they're using; by default the
/// land's own, which uses nothing up: Muldrotha's land is still available, and the Forest in
/// P0's graveyard is played with it and Exploration's additional land play. (Written to kill
/// `permissions.rs:620:42` of the November 2026 mutation run, docs/MUTATION.md, before the
/// choice was asked: that mutant, `||` as `&&` in `choose_land_permission`'s first test,
/// took Muldrotha's permission for the land.)
#[test]
fn a_land_played_with_its_own_mayhem_ability_leaves_muldrothas_land_unused() {
    cr!("702.187c", "305.2");
    interp!("permission-choice-always-asked");
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
    let from = t.asked().len();
    t.play_land(P0, oscorp).unwrap();
    assert!(asked_for_a_permission(&t, from), "both permissions offered");
    assert!(t.g.history.once_permissions_used.is_empty());
    assert_eq!(t.obj_now(oscorp).played_with, None);
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

/// The same, with P0 announcing Muldrotha's permission: it's recorded on the land and used
/// up, so the Forest in P0's graveyard can't be played even with Exploration's additional
/// land play (the user, Nov 20: "We should always ask. There could be a permanent on the
/// field that punishes flashback costs specifically or something of the sort, making the
/// cards own permission a worse option.").
#[test]
fn a_mayhem_land_played_with_muldrothas_permission_uses_it_up() {
    cr!("702.187c", "305.2", "601.3");
    interp!("permission-choice-always-asked");
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
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.play_land(P0, oscorp).unwrap();
    assert!(asked_for_a_permission(&t, from));
    assert_eq!(
        t.g.history.once_permissions_used,
        vec![(muldrotha, "land".into())]
    );
    assert_eq!(
        t.obj_now(oscorp).played_with.and_then(|u| u.source),
        Some(muldrotha)
    );
    t.resolve_all();
    assert!(t.play_land(P0, forest).is_err(), "Muldrotha's land is used up");
    assert!(!t.on_battlefield(forest));
}

/// Ribbons, the aftermath half of Cut // Ribbons, may be cast from P0's graveyard by its own
/// aftermath ability and with Kess, Dissident Mage's permission ("Once during each of your
/// turns, you may cast an instant or sorcery spell from your graveyard."). P0 is asked which they're
/// using: by default aftermath's, which leaves Kess's for another card (Divination);
/// announcing Kess's records it on the spell and uses it up.
#[test]
fn an_aftermath_half_with_kess_out_asks_which_permission_and_uses_kess_up_if_chosen() {
    cr!("601.3", "702.127a");
    interp!("permission-choice-always-asked");
    ruling!(
        "Muldrotha, the Gravetide",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    ruling!(
        "Kess, Dissident Mage",
        "If you cast a card from your graveyard using another permission (such as that of flashback), Kess's effect doesn't apply. You can cast another instant or sorcery card from your graveyard."
    );
    for use_kess in [false, true] {
        let mut t = TestGame::new(2);
        let kess = t.battlefield(P0, "Kess, Dissident Mage");
        let card = t.graveyard(P0, "Cut // Ribbons");
        let div = t.graveyard(P0, "Divination");
        let aftermath = CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Aftermath);
        t.g.recompute();
        let ways: Vec<_> = t
            .g
            .cast_options(P0, card)
            .into_iter()
            .filter(|o| o.method == aftermath)
            .collect();
        assert_eq!(ways.len(), 2, "its own and Kess's: {ways:?}");
        assert!(ways.iter().any(|o| o.permission.is_none()));
        assert!(ways
            .iter()
            .any(|o| o.permission.as_ref().and_then(|c| c.source) == Some(kess)));
        mana(&mut t, P0, mtg_engine::mana::ManaType::B, 2);
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        if use_kess {
            t.answer(P0, DecisionKind::Option, Answer::Index(1));
        }
        let from = t.asked().len();
        let spell = t.cast(P0, card).method(aftermath).go();
        assert!(asked_for_a_permission(&t, from), "use Kess {use_kess}");
        assert_eq!(t.g.obj(spell).chars.name, "Ribbons");
        assert_eq!(
            t.g.obj(spell).played_with.and_then(|u| u.source),
            use_kess.then_some(kess),
            "use Kess {use_kess}"
        );
        assert_eq!(
            !t.g.history.once_permissions_used.is_empty(),
            use_kess,
            "{:?}",
            t.g.history.once_permissions_used
        );
        t.resolve_all();
        assert!(t.in_exile("Cut // Ribbons"));
        mana(&mut t, P0, mtg_engine::mana::ManaType::U, 3);
        let can_cast_divination = t
            .g
            .legal_actions(P0)
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == div));
        assert_eq!(can_cast_divination, !use_kess, "use Kess {use_kess}");
    }
}

/// Woe Strider in P0's graveyard can be cast by its own escape ability (an alternative cost
/// that comes with escape's permission) or for its mana cost with Muldrotha's permission to
/// cast a creature spell from the graveyard: both ways are offered, and the one P0 chooses
/// is the permission recorded and used up.
#[test]
fn an_escape_creature_with_muldrotha_offers_both_and_the_chosen_permission_is_used() {
    cr!("601.3", "702.138a");
    interp!("permission-choice-always-asked");
    ruling!(
        "Muldrotha, the Gravetide",
        "If multiple effects allow you to play a card from your graveyard, you must announce which permission you're using as you begin to play the card."
    );
    for escape in [false, true] {
        let mut t = TestGame::new(2);
        let muldrotha = t.battlefield(P0, "Muldrotha, the Gravetide");
        let strider = t.graveyard(P0, "Woe Strider");
        for _ in 0..4 {
            t.graveyard(P0, "Grizzly Bears");
        }
        t.g.recompute();
        let ways = t.g.cast_options(P0, strider);
        let esc = CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Escape);
        assert!(
            ways.iter().any(|o| o.method == esc && o.permission.is_none()),
            "{ways:?}"
        );
        assert!(
            ways.iter().any(|o| o.method == CastMethod::Normal
                && o.permission.as_ref().and_then(|c| c.source) == Some(muldrotha)),
            "{ways:?}"
        );
        mana(&mut t, P0, mtg_engine::mana::ManaType::B, 5);
        let method = if escape { esc } else { CastMethod::Normal };
        let spell = t.cast(P0, strider).method(method).go();
        let used = t.g.obj(spell).played_with.and_then(|u| u.source);
        assert_eq!(used, (!escape).then_some(muldrotha), "escape {escape}");
        assert_eq!(
            t.g.history.once_permissions_used,
            if escape {
                vec![]
            } else {
                vec![(muldrotha, "creature spell".into())]
            }
        );
    }
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

/// Kills `permissions.rs:347:13` (in `permissions::free_option`, `if land || last.free ||
/// last.terms.alt_cost.is_some() { return; }` with `land && last.free || ...`: a land the
/// permission allows also gets the "without paying their mana costs" way, a copy of the same
/// permission). Ignite the Future cast with flashback exiles the top three cards of P0's
/// library, which P0 may play this turn, and "may play cards this way without paying their
/// mana costs". A land is played, not cast: it's put onto the battlefield from the zone it
/// was in as a special action and is never a spell (CR 116.2a, 305.1), so there's no mana
/// cost to pay or not: the exiled Island is played with the one permission and nobody is
/// asked which. The mutant offers the same permission twice for the Island, and playing it
/// asks "Choose the permission you're using".
#[test]
fn a_land_ignite_the_future_exiled_after_flashback_is_played_without_a_permission_choice() {
    cr!("305.1", "116.2a");
    interp!("questions-only-for-a-choice");
    let mut t = TestGame::new(2);
    let ignite = t.graveyard(P0, "Ignite the Future");
    t.lands(P0, "Mountain", 8);
    t.library_top(P0, "Grizzly Bears");
    let island = t.library_top(P0, "Island");
    t.library_top(P0, "Forest");
    t.cast(P0, ignite)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.resolve_all();
    let island = t.g.current(island);
    assert_eq!(t.zone(island), Zone::Exile);
    let from = t.asked().len();
    t.play_land(P0, island).expect("played from exile");
    assert!(!asked_for_a_permission(&t, from));
    assert!(t.on_battlefield(t.g.current(island)));
}
