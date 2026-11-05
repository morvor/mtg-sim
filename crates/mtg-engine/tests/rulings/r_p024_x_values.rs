//! Rulings batch P024 — {X} in the mana cost of a copied permanent or card is 0 (CR
//! 107.3, 202.3e, 707.2), while a copy of a spell copies the value of X and the mode
//! (CR 707.10).

use crate::r_p023_common::hydra_cast_with_x3;
use crate::r_p024_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const HYDRA: &str = "Benevolent Hydra";

/// Asserts `id` is a Benevolent Hydra ({X}{G}{G}, "enters with X +1/+1 counters") with
/// X = 0: mana value 2 and no counters.
fn hydra_with_x0(t: &mut TestGame, id: ObjectId) {
    assert_eq!(name_of(t, id), HYDRA);
    assert_eq!(mv_now(t, id), 2);
    assert_eq!(t.counters(id, counters::PLUS1), 0);
}

#[test]
fn stolen_identity_copy_of_an_x_permanent_has_x_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!(
        "Stolen Identity",
        "If the copied permanent has {X} in its mana cost, X is considered to be zero."
    );
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let toks = cast_spell(
        &mut t,
        P0,
        "Stolen Identity",
        "{4}{U}{U}",
        &[obj(hydra)],
        None,
        None,
    );
    assert_eq!(toks.len(), 1);
    hydra_with_x0(&mut t, toks[0]);
}

#[test]
fn riku_copies_a_creature_with_x_0_but_a_spell_with_its_x() {
    cr!("107.3", "202.3e", "707.2", "707.10");
    ruling!("Riku of Two Reflections", "If the creature had {X} in its mana cost, X must be 0. Note this is different from an {X} found in the mana cost of a copied spell.");
    ruling!(
        "Riku of Two Reflections",
        "If the copied spell has {X} in its mana cost, the value of X is copied."
    );
    supported("Riku of Two Reflections");
    // A creature: the token's X is 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    t.lands(P0, "Forest", 5);
    let hydra = t.hand(P0, HYDRA);
    t.cast(P0, hydra).x(3).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1, "Riku's trigger");
    let before = t.g.battlefield.clone();
    lands_for(&mut t, P0, "{G}{U}");
    t.answer_yes(P0, true);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    hydra_with_x0(&mut t, toks[0]);
    // A spell: Blaze ({X}{R}: X damage to any target) with X = 3 and its copy deal 6.
    supported("Blaze");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    lands_for(&mut t, P0, "{3}{R}");
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(3).target(Entity::Player(P1)).go();
    lands_for(&mut t, P0, "{U}{R}");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
}

#[test]
fn riku_copies_a_modal_spell_with_its_mode() {
    cr!("707.10", "700.2");
    ruling!("Riku of Two Reflections", "If the copied spell is modal (that is, it says “Choose one —” or the like), the copy has the same mode as the original spell. You can't choose a different one.");
    supported("Riku of Two Reflections");
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Riku of Two Reflections");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{R}{W}");
    let charm = t.hand(P0, "Boros Charm");
    // "Boros Charm deals 4 damage to target player or planeswalker."
    t.cast(P0, charm)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    lands_for(&mut t, P0, "{U}{R}");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "{}", t.dump_log());
    // No mode was chosen for the copy, and the Bears didn't get double strike.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert!(!has_kw(
        &t,
        bears,
        mtg_engine::keywords::KeywordKind::DoubleStrike
    ));
}

#[test]
fn mimic_vat_copy_of_an_x_card_has_x_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Mimic Vat", "If the exiled card has {X} in its mana cost (such as Protean Hydra), X is considered to be 0.");
    supported("Mimic Vat");
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let hydra = hydra_cast_with_x3(&mut t, P0);
    t.answer_yes(P0, true);
    kill(&mut t, hydra);
    t.resolve_all();
    assert!(t.in_exile(HYDRA));
    t.lands(P0, "Wastes", 3);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    hydra_with_x0(&mut t, toks[0]);
    // Protean Hydra ({X}{G}, a 0/0 that enters with X +1/+1 counters): its copy enters
    // with none (P0's Soul Warden sees it enter) and dies.
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let protean = t.battlefield(P1, "Protean Hydra");
    t.answer_yes(P0, true);
    kill(&mut t, protean);
    t.resolve_all();
    assert!(t.in_exile("Protean Hydra"));
    supported("Soul Warden");
    t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activation");
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "{}", t.dump_log());
    assert!(t.named_on_battlefield("Protean Hydra").is_empty());
    assert!(t.in_exile("Protean Hydra"));
}

#[test]
fn caller_of_the_untamed_copy_of_an_x_card_has_x_0() {
    cr!("107.3", "202.3e", "707.2", "607.2n");
    ruling!("Caller of the Untamed", "If there is an {X} in the mana cost of the exiled creature card, that X is 0. Don’t confuse it with the {X} in the activation cost of Caller of the Untamed’s ability.");
    supported("Caller of the Untamed");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Caller of the Untamed");
    let side = t.g.add_to_sideboard(P0, vec![card(HYDRA)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(side[0])]);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    assert!(t.in_exile(HYDRA));
    // X = 2, the card's mana value (its X is 0): the token's X is 0 too, not 2.
    let caller = t.battlefield(P0, "Caller of the Untamed");
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    let before = t.g.battlefield.clone();
    t.activate(P0, caller, 0, &[]).expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    hydra_with_x0(&mut t, toks[0]);
}

/// Araumi's ruling "If the creature card has {X} in its mana cost, you can choose any value
/// for X, but the value you choose won't apply to the tokens that are created. X will be 0
/// for those tokens." is cited only for its second sentence, the tokens' X: its first
/// sentence ("you can choose any value for X, but the value you choose won't apply to the
/// tokens") presupposes an X choice the engine doesn't make, and its "you can choose any
/// value for X" disagrees with CR 107.3h ("If an effect instructs a player to pay an object's mana cost
/// that includes {X}, the value of X is treated as 0 unless the object is a spell on the
/// stack.", since the CR effective 2015-01-23) and with the rulings of Varolz and Young
/// Deathclaws on the same wording ("The scavenge cost is equal to its mana cost": "If the
/// creature card you scavenge has {X} in its mana cost, X is 0."). The engine follows the
/// CR (provisional register entry `x-in-a-cost-equal-to-a-cards-mana-cost-is-0`, for the
/// user to decide; work item ruling-conflicts-3, case 6): X is 0 for the encore cost and
/// for the tokens.
#[test]
fn araumi_encore_cost_and_tokens_have_x_0() {
    cr!("107.3h", "202.3e", "702.141a", "707.2");
    interp!("x-in-a-cost-equal-to-a-cards-mana-cost-is-0");
    ruling!("Araumi of the Dead Tide", "X will be 0 for those tokens.");
    supported("Araumi of the Dead Tide");
    let mut t = TestGame::new(2);
    let araumi = t.battlefield(P0, "Araumi of the Dead Tide");
    let hydra = t.graveyard(P0, HYDRA);
    let fodder = t.graveyard(P0, "Shock");
    t.answer_choose(P0, &[obj(fodder)]);
    t.activate(P0, araumi, 0, &[obj(hydra)])
        .expect("activation");
    t.resolve_all();
    // Encore {X}{G}{G}: X is 0, so {G}{G}, though P0 would announce 3.
    t.lands(P0, "Forest", 5);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(3));
    t.g.recompute();
    let uid = t
        .g
        .obj(hydra)
        .chars
        .abilities
        .iter()
        .find(|a| {
            a.text == "Encore" && matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))
        })
        .map(|a| a.uid)
        .expect("encore");
    let before = t.g.battlefield.clone();
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, hydra, uid).expect("encore");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    hydra_with_x0(&mut t, toks[0]);
}

#[test]
fn the_myriad_pools_copy_of_an_x_spell_has_x_0_on_the_battlefield() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("The Everflowing Well // The Myriad Pools", "If the copied spell has {X} in its mana cost, X is still 0 for the purposes of determining the mana value of the permanent on the battlefield.");
    supported("The Everflowing Well // The Myriad Pools");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let well = t.battlefield(P0, "The Everflowing Well // The Myriad Pools");
    assert!(mtg_engine::dfc::transform(&mut t.g, well));
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Benevolent Hydra with X = 1, the {1} paid with The Myriad Pools' {U}.
    t.lands(P0, "Forest", 2);
    let hydra = t.hand(P0, HYDRA);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, hydra).x(1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve();
    hydra_with_x0(&mut t, bears);
    t.resolve_all();
    let h = t.named_on_battlefield(HYDRA);
    assert_eq!(h.len(), 2);
}

// --- Niko, Absorb Identity: copies of a creature copying something else ------------------

/// P0's Niko (with its two Shards) exiles `what` (P0's), the Shards becoming copies of
/// it; returns the Shards.
fn niko_exiles(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    supported("Niko, Light of Hope");
    let niko = t.enter(P0, "Niko, Light of Hope");
    t.resolve_all();
    unsick(t, niko);
    let shards = crate::r_s01_common::with_subtype(t, P0, "Shard");
    assert_eq!(shards.len(), 2);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(what)]);
    activate_containing(t, P0, niko, "Exile target").expect("activation");
    t.resolve_all();
    shards
}

#[test]
fn niko_shards_copy_what_the_exiled_creature_was_copying() {
    cr!("707.3", "107.3", "202.3e");
    ruling!("Niko, Light of Hope", "If the creature that was exiled was copying something else, the Shards that become copies of that creature will use the copiable values of that creature. In most cases, they will be copies of whatever that creature was copying. If it was copying a permanent or card with {X} in its mana cost, X is 0.");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P0);
    for s in niko_exiles(&mut t, clone) {
        assert_angel(&t, s, true);
    }
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let clone = clone_of(&mut t, P0, hydra);
    for s in niko_exiles(&mut t, clone) {
        hydra_with_x0(&mut t, s);
    }
}

/// P0 casts Absorb Identity returning `what` to its owner's hand and has P0's
/// Shapesharer become a copy of it; returns the Shapesharer.
fn absorb_identity(t: &mut TestGame, what: ObjectId) -> ObjectId {
    let sharer = t.battlefield(P0, "Shapesharer");
    t.answer_yes(P0, true);
    cast_spell(t, P0, "Absorb Identity", "{1}{U}", &[obj(what)], None, None);
    sharer
}

#[test]
fn absorb_identity_shapeshifters_copy_what_the_returned_creature_was_copying() {
    cr!("707.3", "107.3", "202.3e", "608.2h");
    ruling!("Absorb Identity", "If the creature that was returned was copying something else, the Shapeshifters that become copies of that creature will use the copiable values of that creature. In most cases, they will be copies of whatever that creature was copying. If it was copying a permanent or card with {X} in its mana cost, X is 0.");
    let mut t = TestGame::new(2);
    let clone = cloned_angel(&mut t, P1);
    let s = absorb_identity(&mut t, clone);
    assert!(t.in_hand(P1, "Clone"));
    assert_angel(&t, s, true);
    let mut t = TestGame::new(2);
    let hydra = hydra_cast_with_x3(&mut t, P0);
    let clone = clone_of(&mut t, P1, hydra);
    let s = absorb_identity(&mut t, clone);
    hydra_with_x0(&mut t, s);
}
