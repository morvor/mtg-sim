//! Rulings batch P153 — "scales with mana value": the mana value of a spell with {X} in
//! its mana cost is computed with the chosen X while it's on the stack (CR 202.3e), a
//! "whenever you cast" trigger resolves before the spell and uses the spell's last known
//! mana value when the spell has been countered (CR 603.10, 608.2h), and a card cast
//! without paying its mana cost has X = 0 (CR 107.3b).

use crate::r_p153_common::*;
use crate::r_s01_common::{stack_library, supported, tokens, triggers_on_stack};
use crate::r_s04_common::on_stack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn metallurgic_summonings_uses_the_chosen_x_of_clash_of_wills() {
    cr!("202.3e", "608.2h");
    ruling!(
        "Metallurgic Summonings",
        "If you chose 2 as the value of X, then Clash of Wills has mana value 3"
    );
    supported("Metallurgic Summonings");
    supported("Clash of Wills");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metallurgic Summonings");
    // P1 casts a spell; P0 responds with Clash of Wills, X = 2.
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, shock).target(Entity::Player(P0)).go();
    let clash = t.hand(P0, "Clash of Wills");
    mana(&mut t, P0, U, 3);
    t.g.turn.priority = Some(P0);
    let clash = t.cast(P0, clash).x(2).target(spell).go();
    assert_eq!(mv(&t, clash), 3);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "create an X/X"), 1);
    t.resolve();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (3, 3));
}

#[test]
fn metallurgic_summonings_uses_the_countered_spells_last_known_mana_value() {
    cr!("603.10", "608.2h", "202.3e");
    ruling!(
        "Metallurgic Summonings",
        "use that spell's mana value as it last existed on the stack to determine the value of X"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metallurgic Summonings");
    let blaze = cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "create an X/X"), 1);
    // The spell is countered with the trigger still on the stack: in the graveyard the
    // card's mana value is 1, but the trigger uses the 4 it had on the stack.
    counter(&mut t, P1, blaze);
    assert_eq!(mv(&t, blaze), 1);
    t.resolve();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
    assert_eq!(t.life(P1), 20);
}

/// Rashmi: "Whenever you cast your first spell each turn, reveal the top card of your
/// library. You may cast it without paying its mana cost if it's a spell with lesser
/// mana value. If you don't cast it, put it into your hand."
#[test]
fn rashmi_counts_the_spells_x_and_the_library_cards_x_as_0() {
    cr!("202.3e", "107.3b");
    ruling!(
        "Rashmi, Eternities Crafter",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. For cards in your library with {X} in their mana costs, X is considered to be 0."
    );
    supported("Rashmi, Eternities Crafter");
    // The top card Walking Ballista ({X}{X}) has mana value 0 in the library: lesser than
    // Blaze's 4 with X = 3, so P0 may cast it without paying its mana cost (X = 0: no X is
    // chosen, it enters with no counters and dies).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    let ballista = t.library_top(P0, "Walking Ballista");
    let from = t.asked().len();
    cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(x_asked(&t, P0, from), 1, "only Blaze's X was chosen");
    assert_eq!(t.zone(ballista), Zone::Stack);
    assert_eq!(mv(&t, ballista), 0);
    t.resolve();
    assert_eq!(t.zone(ballista), Zone::Graveyard(P0));
    // Grizzly Bears (mana value 2) is cast; the top card is Hydroid Krasis ({X}{G}{U}):
    // mana value 2 in the library, not lesser, so it isn't offered and goes to the hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    let krasis = t.library_top(P0, "Hydroid Krasis");
    let from = t.asked().len();
    cast_simple(&mut t, P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(krasis), Zone::Hand(P0));
    assert_eq!(
        crate::r_s04_common::asked_of_since(&t, P0, from, |d| matches!(
            d,
            Decision::YesNo { .. }
        )),
        0
    );
}

#[test]
fn rashmi_uses_the_countered_spells_last_known_mana_value() {
    cr!("603.10", "608.2h");
    ruling!(
        "Rashmi, Eternities Crafter",
        "use that spell's mana value as it last existed on the stack to determine whether or not you may cast the top card of your library"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    let bears = t.library_top(P0, "Grizzly Bears");
    let blaze = cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.settle();
    counter(&mut t, P1, blaze);
    // Blaze is a card with mana value 1 in the graveyard now; as it last existed on the
    // stack it had mana value 4, so Grizzly Bears (2) may be cast.
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Stack);
    t.resolve();
    assert!(t.on_battlefield(bears));
}

#[test]
fn rootha_counts_the_x_of_a_spell_cast_this_turn() {
    cr!("202.3e");
    ruling!(
        "Rootha, Mastering the Moment",
        "For spells you cast with {X} in their mana costs, use the value chosen for X to determine the spell's mana value."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootha, Mastering the Moment");
    cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
}

#[test]
fn alchemists_talent_deals_the_spells_mana_value_with_its_x() {
    cr!("202.3e");
    ruling!(
        "Alchemist's Talent",
        "If a spell has {X} in its mana cost, use the value chosen for X to determine that spell's mana value."
    );
    supported("Alchemist's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Alchemist's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 3);
    t.g.recompute();
    // Blaze with X = 2 ({X}{R} = 3 mana), paid with two Mountains and a Treasure.
    t.lands(P0, "Mountain", 2);
    crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals damage equal to that spell's mana value"), 1);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.resolve();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn hammerhead_tyrant_measures_the_spells_mana_value_with_its_x() {
    cr!("202.3e", "115.1a");
    ruling!(
        "Hammerhead Tyrant",
        "If a spell has {X} in its mana cost, use the value of X chosen as it was cast to determine that spell’s mana value."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hammerhead Tyrant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Blaze with X = 3 has mana value 4: Hill Giant (4) may be targeted.
    cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Hand(P1));
    assert!(t.on_battlefield(bears));
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(giant)) && c.contains(&obj(bears))));
    // With X = 0 it has mana value 1: neither creature is a legal target, and the
    // ability (up to one target) targets nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hammerhead Tyrant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_blaze(&mut t, P0, 0, Entity::Player(P1));
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&obj(bears))), "{cands:?}");
}

#[test]
fn teval_loses_life_equal_to_the_spells_mana_value_with_x() {
    cr!("202.3e");
    ruling!(
        "Teval, Arbiter of Virtue",
        "If a spell on the stack has an {X} in its mana cost, use the value of X chosen as it was cast to determine that spell’s mana value."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teval, Arbiter of Virtue");
    cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn ovika_creates_tokens_for_the_spells_mana_value_with_x() {
    cr!("202.3e");
    ruling!(
        "Ovika, Enigma Goliath",
        "If a spell on the stack has {X} in its cost, use the value chosen for X to determine its mana value."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ovika, Enigma Goliath");
    cast_blaze(&mut t, P0, 3, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 4);
}

#[test]
fn skittering_cicada_grows_by_the_colorless_spells_mana_value_with_x() {
    cr!("202.3e");
    ruling!(
        "Skittering Cicada",
        "If the colorless spell has {X} in its mana cost, use the value chosen for X when calculating that spell's mana value."
    );
    supported("Skittering Cicada");
    let mut t = TestGame::new(2);
    let cicada = t.battlefield(P0, "Skittering Cicada");
    let ballista = t.hand(P0, "Walking Ballista");
    mana(&mut t, P0, C, 4);
    t.cast(P0, ballista).x(2).go();
    t.resolve_all();
    assert_eq!(t.pt(cicada), (6, 6));
}

#[test]
fn dancing_from_dark_to_dawn_counts_the_creature_spells_x() {
    cr!("202.3e");
    ruling!(
        "Dancing from Dark to Dawn",
        "If the creature spell has {X} in its mana cost, use the value chosen for X when calculating that spell's mana value."
    );
    supported("Dancing from Dark to Dawn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dancing from Dark to Dawn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ballista = t.hand(P0, "Walking Ballista");
    mana(&mut t, P0, C, 4);
    t.cast(P0, ballista).x(2).go();
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 4);
}

#[test]
fn reinterpret_counts_the_countered_spells_x() {
    cr!("202.3e", "608.2h");
    ruling!(
        "Reinterpret",
        "If a spell with {X} in its mana cost is countered this way, {X} is the value chosen as the spell was cast."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let blaze = cast_blaze(&mut t, P1, 3, Entity::Player(P0));
    let giant = t.hand(P0, "Hill Giant");
    let colossus = t.hand(P0, "Craw Wurm");
    let re = t.hand(P0, "Reinterpret");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, C, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, re).target(blaze).go();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P1));
    // Blaze had mana value 4: Hill Giant (4) may be cast, Craw Wurm (6) may not.
    let cands = entity_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(giant))), "{cands:?}");
    assert!(cands.iter().all(|c| !c.contains(&obj(colossus))), "{cands:?}");
    assert_eq!(t.zone(giant), Zone::Stack);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.life(P0), 20);
}

/// Press the Enemy: "Return target spell or nonland permanent an opponent controls to its
/// owner's hand. You may cast an instant or sorcery spell with equal or lesser mana value
/// from your hand without paying its mana cost."
#[test]
fn press_the_enemy_measures_a_spells_x_and_a_permanents_x_as_0() {
    cr!("202.3e", "608.2h");
    ruling!(
        "Press the Enemy",
        "If a spell with {X} in its mana cost is returned to a player's hand this way, {X} is the value chosen as the spell was cast for the purpose of evaluating the spell's mana value."
    );
    ruling!(
        "Press the Enemy",
        "If a permanent with {X} in its mana cost is returned to a player's hand this way, {X} is 0 for the purpose of evaluating its mana value."
    );
    supported("Press the Enemy");
    // A spell: P1's Blaze with X = 3 (mana value 4) is returned; Lightning Bolt (1) and
    // a mana value 4 instant may be cast, a mana value 5 one may not.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let blaze = cast_blaze(&mut t, P1, 3, Entity::Player(P0));
    let bolt = t.hand(P0, "Lightning Bolt");
    let four = t.hand(P0, "Fact or Fiction");
    let five = t.hand(P0, "Opportunity");
    let press = t.hand(P0, "Press the Enemy");
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, C, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, press).target(blaze).go();
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.zone(blaze), Zone::Hand(P1));
    let cands = entity_candidates(&t, P0, from);
    assert!(
        cands
            .iter()
            .any(|c| c.contains(&obj(bolt)) && c.contains(&obj(four)) && !c.contains(&obj(five))),
        "{cands:?}"
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    // A permanent: P1's Walking Ballista (two counters) has mana value 0 on the
    // battlefield, so only a mana value 0 spell could be cast: Lightning Bolt is not offered.
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P1, "Walking Ballista");
    t.g.objects[ballista.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.g.recompute();
    let bolt = t.hand(P0, "Lightning Bolt");
    let press = t.hand(P0, "Press the Enemy");
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, C, 2);
    let from = t.asked().len();
    t.cast(P0, press).target(ballista).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(ballista), Zone::Hand(P1));
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
    let cands = entity_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&obj(bolt))), "{cands:?}");
}

#[test]
fn plasm_capture_includes_the_countered_spells_x() {
    cr!("202.3e", "608.2h");
    ruling!(
        "Plasm Capture",
        "if you counter a spell that costs {X}{R} with X equal to 5, you'll produce six mana"
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let blaze = cast_blaze(&mut t, P1, 5, Entity::Player(P0));
    let pc = t.hand(P0, "Plasm Capture");
    mana(&mut t, P0, G, 2);
    mana(&mut t, P0, U, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, pc).target(blaze).go();
    t.resolve();
    assert_eq!(t.zone(blaze), Zone::Graveyard(P1));
    t.advance_to(P0, Step::PrecombatMain);
    for _ in 0..6 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
    }
    t.resolve_all();
    assert_eq!(t.g.players[0].mana_pool.total(), 6);
}

#[test]
fn dovescape_counts_the_mana_value_of_the_half_that_was_cast() {
    cr!("202.3d", "709.3");
    ruling!(
        "Dovescape",
        "If a split card is cast, Dovescape makes tokens equal to the mana value of the half that was cast."
    );
    supported("Dovescape");
    supported("Fire // Ice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dovescape");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Fire ({1}{R}, mana value 2) is cast: two Birds, not the four of the whole card.
    let fire_ice = t.hand(P1, "Fire // Ice");
    mana(&mut t, P1, R, 2);
    t.g.turn.priority = Some(P1);
    let spell = t
        .cast(P1, fire_ice)
        .method(CastMethod::Half(0))
        .target(bears)
        .go();
    assert_eq!(mv(&t, spell), 2);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Graveyard(P1));
    assert!(t.on_battlefield(bears));
    let birds = tokens(&t, P1);
    assert_eq!(birds.len(), 2);
    assert!(t.obj_now(birds[0]).chars.has_subtype("Bird"));
}

#[test]
fn erratic_cyclops_trigger_resolves_before_the_spell_even_if_it_is_countered() {
    cr!("603.3", "608.2h", "603.10");
    ruling!(
        "Erratic Cyclops",
        "Erratic Cyclops’s triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Erratic Cyclops");
    let blaze = cast_blaze(&mut t, P0, 2, Entity::Player(P1));
    t.settle();
    // The trigger is above the spell: it resolves first.
    assert_eq!(on_stack(&t, "gets +X/+0"), 1);
    assert_eq!(t.pt(cyclops), (0, 8));
    t.resolve();
    assert_eq!(t.pt(cyclops), (3, 8));
    assert_eq!(t.zone(blaze), Zone::Stack);
    assert_eq!(t.life(P1), 20);
    // Countered before the trigger resolves: still +3/+0.
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Erratic Cyclops");
    let blaze = cast_blaze(&mut t, P0, 2, Entity::Player(P1));
    t.settle();
    counter(&mut t, P1, blaze);
    t.resolve();
    assert_eq!(t.pt(cyclops), (3, 8));
}

#[test]
fn graha_tia_trigger_resolves_before_the_spell_even_if_it_leaves_the_stack() {
    cr!("603.3", "608.2h", "603.10");
    ruling!(
        "G'raha Tia, Scion Reborn",
        "G'raha Tia's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("G'raha Tia, Scion Reborn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "G'raha Tia, Scion Reborn");
    let blaze = cast_blaze(&mut t, P0, 2, Entity::Player(P1));
    t.settle();
    assert_eq!(on_stack(&t, "pay X life"), 1);
    counter(&mut t, P1, blaze);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (4, 4));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sunbirds_invocation_doesnt_trigger_on_itself_and_a_free_x_spell_has_x_0() {
    cr!("603.6d", "107.3b");
    ruling!(
        "Sunbird's Invocation",
        "Casting Sunbird's Invocation won't cause its own ability to trigger."
    );
    ruling!(
        "Sunbird's Invocation",
        "If a revealed card in your library has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Sunbird's Invocation");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    let inv = t.hand(P0, "Sunbird's Invocation");
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, C, 5);
    t.cast(P0, inv).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "reveal the top X cards"), 0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 32);
    // Hill Giant (mana value 4) is cast from the hand: the top four cards are revealed and
    // Blaze may be cast without paying its mana cost, with X = 0: no X is chosen and P1
    // takes no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunbird's Invocation");
    let blaze = stack_library(&mut t, P0, &["Forest", "Blaze", "Forest", "Forest"])[1];
    let from = t.asked().len();
    cast_simple(&mut t, P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[obj(blaze)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.zone(blaze), Zone::Stack);
    assert_eq!(x_asked(&t, P0, from), 0);
    assert_eq!(mv(&t, blaze), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn baral_and_kari_zev_count_spells_cast_before_they_arrived() {
    cr!("603.2", "603.6d");
    ruling!(
        "Baral and Kari Zev",
        "This means if you cast an instant or sorcery spell, then Baral and Kari Zev comes under your control, its triggered ability won’t trigger that turn."
    );
    supported("Baral and Kari Zev");
    let mut t = TestGame::new(2);
    cast_blaze(&mut t, P0, 0, Entity::Player(P1));
    t.resolve_all();
    t.battlefield(P0, "Baral and Kari Zev");
    cast_blaze(&mut t, P0, 0, Entity::Player(P1));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "first instant or sorcery"), 0);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // The next turn: the first instant that turn triggers it.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    cast_blaze(&mut t, P0, 0, Entity::Player(P1));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "first instant or sorcery"), 1);
}

#[test]
fn a_face_down_creature_spell_has_mana_value_0() {
    cr!("708.2", "202.3a");
    ruling!(
        "Endrek Sahr, Master Breeder",
        "Face-down spells are creature spells, but they have mana value 0."
    );
    supported("Endrek Sahr, Master Breeder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Endrek Sahr, Master Breeder");
    let morph = t.hand(P0, "Willbender");
    mana(&mut t, P0, C, 3);
    let spell = t
        .cast(P0, morph)
        .method(CastMethod::FaceDown(mtg_engine::keywords::KeywordKind::Morph))
        .go();
    assert!(t.g.obj(spell).is(CardType::Creature));
    assert_eq!(mv(&t, spell), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Thrull"), 1);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // A face-up creature spell with mana value 2 makes two.
    cast_simple(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn excaliburs_mana_value_is_12_whatever_the_reduction() {
    cr!("202.3", "601.2f");
    ruling!(
        "Excalibur, Sword of Eden",
        "Excalibur’s mana value doesn’t change no matter what the total mana value of historic permanents you control is."
    );
    supported("Excalibur, Sword of Eden");
    let mut t = TestGame::new(2);
    // Historic permanents with total mana value 7: Excalibur costs {5}.
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Bosh, Iron Golem");
    let sword = t.hand(P0, "Excalibur, Sword of Eden");
    mana(&mut t, P0, C, 4);
    let spell = t.cast(P0, sword).go();
    assert_eq!(mv(&t, spell), 12);
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    t.resolve();
    assert!(t.on_battlefield(spell));
    assert_eq!(mv(&t, spell), 12);
}
