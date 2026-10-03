//! Rulings about how costs are paid (gap-mana-payment, see `payment_rules.rs`): "Spend only
//! black mana on X" (Consume Spirit, Drain Life, Crypt Rats, the familiars and Helm of
//! Awakening), "For each {B} in a cost, you may pay 2 life rather than pay that mana"
//! (K'rrik, Son of Yawgmoth), "You can't spend mana to cast this spell" (Hogaak, Arisen
//! Necropolis), "This mana can't be spent to pay generic mana costs" (Jegantha, the
//! Wellspring), and mana that triggers when it's spent to activate an ability (Sunken
//! Palace).

use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// Whether `p` may cast `card` now (normally).
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Cast { card: c, method: CastMethod::Normal } if *c == card)
    })
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

// ---------------------------------------------------------------------------------------
// "Spend only black mana on X."
// ---------------------------------------------------------------------------------------

#[test]
fn consume_spirit_spends_only_black_mana_on_x_and_gains_x_life() {
    cr!("107.3a", "601.2h");
    ruling!(
        "Consume Spirit",
        "The amount of life you gain is equal to the number chosen for X, not the amount of damage Consume Spirit deals"
    );
    supported("Consume Spirit");
    // Consume Spirit ({X}{1}{B}) with X = 1 from a Swamp and two Mountains: three mana for
    // a total cost of three, but the Mountains can't pay X.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 2);
    let spirit = t.hand(P0, "Consume Spirit");
    assert!(t.cast(P0, spirit).target(P1).x(1).try_go().is_err());
    assert!(t.in_hand(P0, "Consume Spirit"));
    // With a second Swamp, black mana pays X and {B}, a Mountain pays {1}.
    t.lands(P0, "Swamp", 1);
    // Healing Salve has prevented the next 3 damage to P1: Consume Spirit deals none, and
    // P0 still gains X life.
    let salve = t.hand(P1, "Healing Salve");
    t.lands(P1, "Plains", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, salve).modes(&[1]).target(P1).go();
    t.resolve();
    t.g.turn.priority = Some(P0);
    t.cast(P0, spirit).target(P1).x(1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn consume_spirit_does_nothing_if_its_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Consume Spirit",
        "If the targeted permanent or player is an illegal target by the time Consume Spirit would resolve, the entire spell doesn’t resolve. You won’t gain any life."
    );
    supported("Consume Spirit");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spirit = t.hand(P0, "Consume Spirit");
    t.cast(P0, spirit).target(bears).x(2).go();
    // P1 destroys the Bears in response.
    let blade = t.hand(P1, "Doom Blade");
    t.lands(P1, "Swamp", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, blade).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.resolve();
    assert!(t.in_graveyard(P0, "Consume Spirit"));
    assert_eq!(t.life(P0), 20);
}

/// P0 casts Consume Spirit ({X}{1}{B}) at P1 with X = 2 from two Swamps and two
/// Mountains, with a permanent making black spells cost {1} less: the reduction reduces the
/// X part, so one black mana pays X, the Mountains the {1} and a Swamp the {B}.
fn consume_spirit_reduced_by(reducer: &str) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    let spirit = t.hand(P0, "Consume Spirit");
    // Without the reduction X = 2 needs three black mana (four lands for a total cost of
    // four, but only two Swamps).
    assert!(t.cast(P0, spirit).target(P1).x(2).try_go().is_err());
    t.battlefield(P0, reducer);
    t.cast(P0, spirit).target(P1).x(2).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn helm_of_awakening_reduces_the_generic_x_that_only_black_mana_can_pay() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Helm of Awakening",
        "The generic X cost is still considered generic even if there is a requirement that a specific color be used for it. For example, “only black mana can be spent this way”. This distinction is important for effects which reduce the generic portion of a spell's cost."
    );
    supported("Helm of Awakening");
    consume_spirit_reduced_by("Helm of Awakening");
}

#[test]
fn thunderscape_familiar_reduces_the_generic_x_that_only_black_mana_can_pay() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Thunderscape Familiar",
        "The generic X cost is still considered generic even if there is a requirement that a specific color be used for it. For example, “only black mana can be spent this way”. This distinction is important for effects which reduce the generic portion of a spell’s cost."
    );
    supported("Thunderscape Familiar");
    consume_spirit_reduced_by("Thunderscape Familiar");
}

#[test]
fn drain_life_cost_reducers_can_reduce_the_x_part() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Drain Life",
        "Cost reducers can be used to reduce the X part of the mana cost."
    );
    supported("Drain Life");
    // Drain Life ({X}{1}{B}, X = 1) with Helm of Awakening ("Spells cost {1} less"): the
    // total cost {1}{B} has no X part left, so a Swamp and a Mountain pay it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.battlefield(P0, "Helm of Awakening");
    let drain = t.hand(P0, "Drain Life");
    t.cast(P0, drain).target(P1).x(1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn drain_life_gains_no_more_than_the_creature_s_toughness() {
    cr!("120.3");
    ruling!(
        "Drain Life",
        "You may gain up to the total toughness of the creature even if it was already damaged."
    );
    supported("Drain Life");
    // Drain Life with X = 3 at a damaged Grizzly Bears (2/2 with 1 damage): it deals 3
    // damage, and P0 gains 2 life (its toughness), not 1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].damage = 1;
    let drain = t.hand(P0, "Drain Life");
    t.cast(P0, drain).target(bears).x(3).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn soul_burn_gains_life_for_each_black_mana_spent_on_x() {
    cr!("107.3a", "601.2h");
    ruling!(
        "Soul Burn",
        "Will give 1 life for each black mana used, but not more life than the amount of unprevented damage that is dealt."
    );
    supported("Soul Burn");
    // Soul Burn ({X}{2}{B}) with X = 3, paid with one Swamp and two Mountains for X: it
    // deals 3 damage to P1, and P0 gains 1 life (one {B} was spent on X).
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 2);
    let burn = t.hand(P0, "Soul Burn");
    t.cast(P0, burn).target(P1).x(3).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 21);
    // Black mana spent on X beyond the damage dealt doesn't count: with Healing Salve
    // preventing 3 of 4 damage, P0 gains 1 life for four {B}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.lands(P0, "Wastes", 2);
    let salve = t.hand(P1, "Healing Salve");
    t.lands(P1, "Plains", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, salve).modes(&[1]).target(P1).go();
    t.resolve();
    let burn = t.hand(P0, "Soul Burn");
    t.g.turn.priority = Some(P0);
    t.cast(P0, burn).target(P1).x(4).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

// ---------------------------------------------------------------------------------------
// K'rrik, Son of Yawgmoth
// ---------------------------------------------------------------------------------------

#[test]
fn krrik_pays_2_life_for_each_black_symbol_but_never_for_generic_mana() {
    cr!("118.3", "119.4");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "K'rrik's ability doesn't modify or reduce costs you pay. It changes only how you may pay those costs."
    );
    supported("K'rrik, Son of Yawgmoth");
    let mut t = TestGame::new(2);
    let duress = t.hand(P0, "Duress");
    let stone = t.hand(P0, "Mind Stone");
    assert!(!castable(&mut t, P0, duress));
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    // Duress ({B}) with no mana: 2 life pays its cost, which doesn't change: its mana value
    // is 1, and no mana was spent to cast it.
    assert!(castable(&mut t, P0, duress));
    // Mind Stone ({2}) has no {B}.
    assert!(!castable(&mut t, P0, stone));
    t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P0, duress).target(P1).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 1);
    assert!(t
        .obj_now(spell)
        .stack
        .as_ref()
        .is_some_and(|si| si.cast.mana_spent.is_empty()));
    // (K'rrik's trigger resolves first.)
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Duress"));
}

#[test]
fn krrik_life_cant_pay_the_generic_x_only_black_mana_can_pay() {
    cr!("107.3a", "118.3");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "You can't pay 2 life to pay for generic mana in costs you pay, even if an effect says that you must spend black mana to pay that generic mana."
    );
    supported("K'rrik, Son of Yawgmoth");
    // Consume Spirit ({X}{1}{B}) with K'rrik and two Mountains: life pays {B}, a Mountain
    // pays {1}, but X needs black mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    t.lands(P0, "Mountain", 2);
    let spirit = t.hand(P0, "Consume Spirit");
    assert!(t.cast(P0, spirit).target(P1).x(1).try_go().is_err());
    assert_eq!(t.life(P0), 20);
    t.cast(P0, spirit).target(P1).x(0).go();
    assert_eq!(t.life(P0), 18);
    // With a Swamp, black mana pays X = 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Swamp", 1);
    let spirit = t.hand(P0, "Consume Spirit");
    t.cast(P0, spirit).target(P1).x(1).go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn krrik_hybrid_symbols_are_paid_with_life_only_by_choosing_black() {
    cr!("118.13a", "601.2b");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "If a cost contains a mana symbol that may be paid in multiple ways, such as {B/R}, {B/P}, or {2/B}, you choose how you'll pay it before you do so. If you choose to pay {B} this way, K'rrik's ability allows you to pay life rather than pay that mana."
    );
    supported("Rakdos Shred-Freak");
    // Rakdos Shred-Freak ({B/R}{B/R}) with no mana: 2 life pays each symbol.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    let freak = t.hand(P0, "Rakdos Shred-Freak");
    t.cast(P0, freak).go();
    assert_eq!(t.life(P0), 16);
    // With two Mountains, P0 chooses to pay one symbol with {R} and the other with life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    t.lands(P0, "Mountain", 2);
    let freak = t.hand(P0, "Rakdos Shred-Freak");
    // "How will you pay {B/R}?": either way, {B}, {R}, 2 life.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.cast(P0, freak).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|l| t.obj_now(**l).chars.name == "Mountain" && t.obj_now(**l).tapped)
            .count(),
        1
    );
    // Choosing {R} for both with no red mana can't be paid with life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    let freak = t.hand(P0, "Rakdos Shred-Freak");
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    assert!(t.cast(P0, freak).try_go().is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn krrik_pays_life_for_activation_costs_and_special_actions() {
    cr!("116.2b", "118.3", "602.2b");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "K'rrik's ability lets you pay 2 life for {B} in any cost you pay, including the mana costs of spells, activation costs, and even costs for special actions (such as morph). Any time you pay mana, that's a cost."
    );
    supported("Drudge Skeletons");
    supported("Grinning Demon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    // Drudge Skeletons' "{B}: Regenerate" with no mana: 2 life.
    let skeletons = t.battlefield(P0, "Drudge Skeletons");
    t.activate(P0, skeletons, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    // Grinning Demon cast face down for {3}, turned face up for its morph cost {2}{B}{B}:
    // two Wastes and 4 life.
    let wastes = t.lands(P0, "Wastes", 5);
    let demon = t.hand(P0, "Grinning Demon");
    t.cast(P0, demon)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let demon = t.g.current(demon);
    assert!(t.obj_now(demon).face_down);
    assert_eq!(wastes.iter().filter(|w| !t.obj_now(**w).tapped).count(), 2);
    t.g.turn.priority = Some(P0);
    let up = Action::Special(SpecialAction::TurnFaceUp { obj: demon });
    assert!(t.g.legal_actions(P0).contains(&up));
    t.g.perform_action(P0, up).unwrap();
    assert!(!t.obj_now(demon).face_down);
    assert_eq!(t.life(P0), 14);
    assert!(wastes.iter().all(|w| t.obj_now(*w).tapped));
}

#[test]
fn krrik_mana_value_is_7_even_if_life_paid_for_its_phyrexian_mana() {
    cr!("107.4f", "202.3");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, K'rrik's mana value is always 7."
    );
    supported("K'rrik, Son of Yawgmoth");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let krrik = t.hand(P0, "K'rrik, Son of Yawgmoth");
    let spell = t.cast(P0, krrik).go();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 7);
    t.resolve();
    assert_eq!(t.obj_now(krrik).chars.mana_value(), 7);
}

#[test]
fn krrik_counter_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3");
    ruling!(
        "K'rrik, Son of Yawgmoth",
        "K'rrik's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("K'rrik, Son of Yawgmoth");
    let mut t = TestGame::new(2);
    let krrik = t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    t.lands(P0, "Swamp", 1);
    let duress = t.hand(P0, "Duress");
    let spell = t.cast(P0, duress).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters Duress with the trigger on the stack.
    let cs = t.hand(P1, "Counterspell");
    t.lands(P1, "Island", 2);
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Duress"));
    assert_eq!(t.counters(krrik, counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------------------
// Jegantha, the Wellspring: "This mana can't be spent to pay generic mana costs."
// ---------------------------------------------------------------------------------------

#[test]
fn jegantha_mana_pays_a_two_hybrid_symbol_only_with_its_colored_half() {
    cr!("106.6", "107.4e");
    ruling!(
        "Jegantha, the Wellspring",
        "You can spend mana from Jegantha's mana ability to pay for a hybrid symbol such as {2/W}, but only if you choose to pay the colored mana component, not the generic mana component."
    );
    supported("Jegantha, the Wellspring");
    // Spectral Procession ({2/W}{2/W}{2/W}): Jegantha's {W} pays one symbol, Wastes pay the
    // others with their generic half. With three Wastes, Jegantha's {U}{B}{R}{G} can't pay
    // the missing generic mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jegantha, the Wellspring");
    t.lands(P0, "Wastes", 3);
    let procession = t.hand(P0, "Spectral Procession");
    assert!(t.cast(P0, procession).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    t.cast(P0, procession).go();
    // What's left in the pool is Jegantha's other four mana.
    assert_eq!(
        pool(&t, P0),
        vec![ManaType::U, ManaType::B, ManaType::R, ManaType::G]
    );
    t.resolve();
    let tokens = t.g.battlefield.iter().filter(|o| t.obj_now(**o).is_token()).count();
    assert_eq!(tokens, 3);
}

#[test]
fn jegantha_mana_cant_pay_numbers_or_x() {
    cr!("106.6", "107.3", "107.4b");
    ruling!(
        "Jegantha, the Wellspring",
        "A generic mana cost is usually represented by numeric mana symbols ({1}, {2}, and so on) and also {X}. It is any cost requiring mana where that cost isn't {C}, {W}, {U}, {B}, {R}, or {G}."
    );
    supported("Jegantha, the Wellspring");
    // Grizzly Bears ({1}{G}): Jegantha's {G} pays {G}, but its other mana can't pay {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jegantha, the Wellspring");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, bears));
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    // Consume Spirit ({X}{1}{B}) with X = 1: Jegantha's {B} pays {B}, and black mana from
    // Jegantha can't pay X: a Swamp must.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jegantha, the Wellspring");
    t.lands(P0, "Wastes", 1);
    let spirit = t.hand(P0, "Consume Spirit");
    assert!(t.cast(P0, spirit).target(P1).x(1).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, spirit).target(P1).x(1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------------------
// Hogaak, Arisen Necropolis: "You can't spend mana to cast this spell."
// ---------------------------------------------------------------------------------------

#[test]
fn hogaak_can_be_cast_only_with_convoke_and_delve() {
    cr!("601.2h", "702.51a", "702.66a");
    supported("Hogaak, Arisen Necropolis");
    // Hogaak ({5}{B/G}{B/G}) in P0's graveyard, with seven Swamps: mana can't pay any of it.
    let mut t = TestGame::new(2);
    let hogaak = t.graveyard(P0, "Hogaak, Arisen Necropolis");
    t.lands(P0, "Swamp", 7);
    assert!(!castable(&mut t, P0, hogaak));
    assert!(t.cast(P0, hogaak).try_go().is_err());
    // Two Grizzly Bears convoke the hybrid symbols; four other cards in the graveyard
    // delve {4}, and the last {1} would need mana.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.graveyard(P0, "Lightning Bolt");
    }
    assert!(!castable(&mut t, P0, hogaak));
    // A fifth card pays the rest.
    t.graveyard(P0, "Lightning Bolt");
    assert!(castable(&mut t, P0, hogaak));
    t.cast(P0, hogaak).go();
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t
        .g
        .battlefield
        .iter()
        .filter(|l| t.obj_now(**l).chars.name == "Swamp")
        .all(|l| !t.obj_now(*l).tapped));
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hogaak, Arisen Necropolis").len(), 1);
}

#[test]
fn hogaak_s_black_symbols_may_be_paid_with_life_instead_of_mana() {
    cr!("601.2h", "118.3", "702.51a");
    supported("Hogaak, Arisen Necropolis");
    // With K'rrik ("For each {B} in a cost, you may pay 2 life rather than pay that mana"),
    // paying 2 life for a {B/G} isn't spending mana. K'rrik convokes one {B/G}, life pays
    // the other, five cards delve {5}; the Swamps stay untapped.
    let mut t = TestGame::new(2);
    let krrik = t.battlefield(P0, "K'rrik, Son of Yawgmoth");
    let hogaak = t.graveyard(P0, "Hogaak, Arisen Necropolis");
    t.lands(P0, "Swamp", 3);
    for _ in 0..5 {
        t.graveyard(P0, "Lightning Bolt");
    }
    assert!(castable(&mut t, P0, hogaak));
    t.cast(P0, hogaak).go();
    assert!(t.obj_now(krrik).tapped);
    assert_eq!(t.life(P0), 18);
    assert!(t
        .g
        .battlefield
        .iter()
        .filter(|l| t.obj_now(**l).chars.name == "Swamp")
        .all(|l| !t.obj_now(*l).tapped));
}

// ---------------------------------------------------------------------------------------
// Sunken Palace: "When you spend this mana to cast a spell or activate an ability, copy
// that spell or ability."
// ---------------------------------------------------------------------------------------

/// P0 activates Sunken Palace's second ability (two Islands pay {1}{U}, seven cards are
/// exiled from the graveyard): {U} in the pool that copies what it's spent on.
fn palace_mana(t: &mut TestGame) {
    let palace = t.battlefield(P0, "Sunken Palace");
    t.lands(P0, "Island", 2);
    for _ in 0..7 {
        t.graveyard(P0, "Lightning Bolt");
    }
    t.activate(P0, palace, 1, &[]).unwrap();
    assert_eq!(pool(t, P0), vec![ManaType::U]);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn sunken_palace_copies_the_ability_its_mana_is_spent_to_activate() {
    cr!("106.6", "603.7a", "707.10");
    supported("Sunken Palace");
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    // Mind Stone's "{1}, {T}, Sacrifice ~: Draw a card." paid with the Palace's {U}: the
    // delayed trigger copies it; the copy resolves first, then the ability.
    let stone = t.battlefield(P0, "Mind Stone");
    let hand = t.hand_size(P0);
    t.activate(P0, stone, 1, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The trigger resolved: the copy is on the stack above the ability.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn sunken_palace_copies_the_spell_its_mana_is_spent_to_cast() {
    cr!("106.6", "603.7a", "707.10");
    supported("Sunken Palace");
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    // Opt ({U}: scry 1, then draw a card) and its copy.
    let opt = t.hand(P0, "Opt");
    let hand = t.hand_size(P0);
    t.cast(P0, opt).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

// ---------------------------------------------------------------------------------------
// Thieving Skydiver: "Kicker {X}. X can't be 0."
// ---------------------------------------------------------------------------------------

/// P0 casts Thieving Skydiver ({1}{U}) kicked with X = 1 from three Islands, its enters
/// trigger targeting `artifact`; returns the Skydiver.
fn kicked_skydiver(t: &mut TestGame, artifact: ObjectId) -> ObjectId {
    t.lands(P0, "Island", 3);
    let diver = t.hand(P0, "Thieving Skydiver");
    t.answer_targets(P0, &[Entity::Object(artifact)]);
    t.cast(P0, diver).kicked(true).x(1).go();
    // Kicker {X}: X can't be 0.
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::ChooseX { min: 1, .. })));
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    diver
}

#[test]
fn thieving_skydiver_takes_an_equipment_and_attaches_it() {
    cr!("107.3a", "702.33a", "701.3a");
    ruling!(
        "Thieving Skydiver",
        "Thieving Skydiver's ability can target an artifact you already control. You'll attach it to Thieving Skydiver if it's an Equipment."
    );
    supported("Thieving Skydiver");
    // P0's own Bonesplitter, unattached.
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let diver = kicked_skydiver(&mut t, splitter);
    t.resolve();
    assert_eq!(t.obj_now(splitter).controller, P0);
    assert_eq!(
        t.obj_now(splitter).attached_to,
        Some(Entity::Object(t.g.current(diver)))
    );
    assert_eq!(t.pt(diver), (4, 1));
    // Not kicked: no X is announced and nothing triggers.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let diver = t.hand(P0, "Thieving Skydiver");
    t.cast(P0, diver).kicked(false).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseX { .. })));
}

#[test]
fn thieving_skydiver_gone_leaves_the_equipment_where_it_was_but_stolen() {
    cr!("701.3b", "611.2a");
    ruling!(
        "Thieving Skydiver",
        "If the Equipment can't be attached to Thieving Skydiver, most likely because Thieving Skydiver has left the battlefield before its triggered ability resolves, the Equipment remains attached to whatever it's currently attached to or remains unattached if attached to nothing."
    );
    ruling!(
        "Thieving Skydiver",
        "The control-change effect of Thieving Skydiver lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Thieving Skydiver leaves the battlefield."
    );
    supported("Thieving Skydiver");
    // P1's Bonesplitter is attached to P1's Grizzly Bears. With the trigger on the stack,
    // P1 kills the Skydiver with Lightning Bolt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    let diver = kicked_skydiver(&mut t, splitter);
    let diver = t.g.current(diver);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(diver).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Thieving Skydiver"));
    t.resolve();
    assert_eq!(t.obj_now(splitter).controller, P0);
    assert_eq!(t.obj_now(splitter).attached_to, Some(Entity::Object(bears)));
    // It lasts beyond the turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.obj_now(splitter).controller, P0);
}

#[test]
fn sunken_palace_copies_a_keyword_s_activated_ability() {
    cr!("106.6", "702.29a", "707.10");
    ruling!(
        "Sunken Palace",
        "Activated abilities contain a colon. They're generally written \"[Cost]: [Effect].\" Some keyword abilities are activated abilities and will have a colon in their reminder text."
    );
    supported("Sunken Palace");
    supported("Lonely Sandbar");
    // Lonely Sandbar's cycling ({U}, discard this card: draw a card) is an activated
    // ability: paid with the Palace's {U}, it's copied.
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    let sandbar = t.hand(P0, "Lonely Sandbar");
    let hand = t.hand_size(P0);
    let cycling = t
        .obj_now(sandbar)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count()
        - 1;
    t.activate(P0, sandbar, cycling, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn sunken_palace_copy_may_have_new_targets() {
    cr!("707.10c", "115.7d");
    ruling!(
        "Sunken Palace",
        "The copy will have the same targets as the spell or ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them."
    );
    supported("Sunken Palace");
    // Unsummon ({U}) targeting one of P1's Grizzly Bears, cast with the Palace's {U}: the
    // copy targets the other one.
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(a).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve_all();
    assert_eq!(t.zone(a), mtg_engine::object::Zone::Hand(P1));
    assert_eq!(t.zone(b), mtg_engine::object::Zone::Hand(P1));
    // Keeping the targets: the copy returns the same Bears, and Unsummon then does nothing.
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let unsummon = t.hand(P0, "Unsummon");
    t.cast(P0, unsummon).target(a).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(a), mtg_engine::object::Zone::Hand(P1));
    assert!(t.on_battlefield(b));
}

#[test]
fn sunken_palace_copy_keeps_the_damage_division() {
    cr!("707.10", "707.10c", "601.2d");
    ruling!(
        "Sunken Palace",
        "If the spell or ability has damage divided as it was cast or activated, the division can't be changed (although the targets receiving that damage still can)."
    );
    supported("Sunken Palace");
    supported("Arc Lightning");
    // Arc Lightning ({2}{R}: 3 damage divided among one, two, or three targets), cast with
    // the Palace's {U} and two Mountains: 2 to Grizzly Bears A, 1 to P1. The copy changes
    // its first target to Bears B, which gets that 2.
    let mut t = TestGame::new(2);
    palace_mana(&mut t);
    t.lands(P0, "Mountain", 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let arc = t.hand(P0, "Arc Lightning");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, arc)
        .targets(&[Entity::Object(a), Entity::Player(P1)])
        .go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn thunderscape_familiar_reduces_a_black_and_green_spell_by_1() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Thunderscape Familiar",
        "If a spell is both black and green, you pay {1} less, not {2} less."
    );
    supported("Thunderscape Familiar");
    supported("Moldering Karok");
    // Moldering Karok ({2}{B}{G}) costs {1}{B}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thunderscape Familiar");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let karok = t.hand(P0, "Moldering Karok");
    assert!(!castable(&mut t, P0, karok));
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, karok));
    t.cast(P0, karok).go();
    t.resolve();
    assert!(t.on_battlefield(karok));
}

#[test]
fn a_cost_reducer_sacrificed_to_pay_the_cost_still_reduces_it() {
    cr!("601.2f", "601.2h");
    ruling!(
        "Thunderscape Familiar",
        "If this card is sacrificed to pay part of a spell’s cost, the cost reduction still applies."
    );
    ruling!(
        "Helm of Awakening",
        "If this card is sacrificed to pay part of a spell's cost, the cost reduction still applies."
    );
    supported("Altar's Reap");
    supported("Shrapnel Blast");
    // Altar's Reap ({1}{B}, sacrifice a creature: draw two cards) sacrificing Thunderscape
    // Familiar: the total cost {B} was locked in before the Familiar was sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thunderscape Familiar");
    t.lands(P0, "Swamp", 1);
    let reap = t.hand(P0, "Altar's Reap");
    let hand = t.hand_size(P0);
    t.cast(P0, reap).go();
    assert!(t.in_graveyard(P0, "Thunderscape Familiar"));
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    // Shrapnel Blast ({1}{R}, sacrifice an artifact: 5 damage) sacrificing Helm of
    // Awakening ("Spells cost {1} less to cast"): one Mountain pays {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Mountain", 1);
    let blast = t.hand(P0, "Shrapnel Blast");
    t.cast(P0, blast).target(P1).go();
    assert!(t.in_graveyard(P0, "Helm of Awakening"));
    t.resolve();
    assert_eq!(t.life(P1), 15);
}
