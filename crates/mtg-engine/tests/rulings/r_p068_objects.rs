//! Rulings batch P068 — cards cast from exile as new objects (CR 400.7): a card that comes
//! back to exile isn't the one a permission or link was about (CR 607.2a); where the spell
//! goes after it resolves (CR 608.2n, 702.27a); Adventures (CR 715.4), split cards (CR
//! 709.3) and modal double-faced cards (CR 712.11) cast this way; illegal targets (CR
//! 608.2b); copies (CR 707.10, 707.12); cost order (CR 601.2f); Knowledge Pool's "if the
//! player does" (CR 608.2c).

use crate::r_p068_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The target candidates offered in every target choice since `from`.
fn targets_offered(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

// ---------------------------------------------------------------------------
// New objects
// ---------------------------------------------------------------------------

#[test]
fn smirking_spelljacker_card_back_in_exile_isnt_exiled_with_it() {
    cr!("400.7", "607.2a", "603.4");
    ruling!(
        "Smirking Spelljacker",
        "If you cast a card exiled with Smirking Spelljacker, it will go to the stack. From there, it will likely go to the battlefield or back to its owner’s graveyard. Even if it returns to exile, it will no longer be exiled with Smirking Spelljacker, so you can’t cast it again."
    );
    supported("Smirking Spelljacker");
    let mut t = TestGame::new(2);
    // P1 casts Grizzly Bears; Smirking Spelljacker enters and exiles it.
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    mana(&mut t, P1, G, 2);
    let spell = t.cast(P1, bears).go();
    t.answer_targets(P0, &[o(spell)]);
    let sj = t.enter(P0, "Smirking Spelljacker");
    t.settle();
    t.resolve();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    // P0's turn: it attacks and P0 casts the Bears.
    t.set_step(P0, Step::PrecombatMain);
    t.g.obj_mut(sj).summoning_sick = false;
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(sj, Entity::Player(P1))]);
    t.resolve_all();
    let on_bf = t.g.current(bears);
    assert!(t.on_battlefield(on_bf), "{}", t.dump_log());
    assert_eq!(t.obj_now(on_bf).controller, P0);
    block_and_finish(&mut t, P1, &[]);
    // The Bears is exiled again: a new object, not exiled with the Spelljacker.
    let back = move_to(&mut t, on_bf, Zone::Exile);
    t.g.obj_mut(sj).tapped = false;
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(sj, Entity::Player(P1))]);
    assert!(t.g.stack.is_empty(), "no trigger: {}", t.dump_log());
    t.resolve_all();
    assert!(exiled(&t, back));
}

#[test]
fn mnemonic_betrayal_card_exiled_again_is_a_new_object_and_stays() {
    cr!("400.7", "603.7");
    ruling!(
        "Mnemonic Betrayal",
        "If you cast a spell this way and that card is exiled, it’s considered a new object. Mnemonic Betrayal no longer allows you to cast it. It remains in exile when Mnemonic Betrayal’s delayed triggered ability resolves."
    );
    supported("Mnemonic Betrayal");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let div = t.graveyard(P1, "Divination");
    let mb = t.hand(P0, "Mnemonic Betrayal");
    pool(&mut t, P0, &[U, B, C]);
    t.cast(P0, mb).go();
    t.resolve_all();
    assert!(exiled(&t, bears) && exiled(&t, div));
    mana(&mut t, P0, B, 2);
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    let on_bf = t.g.current(bears);
    assert!(t.on_battlefield(on_bf), "{}", t.dump_log());
    let back = move_to(&mut t, on_bf, Zone::Exile);
    mana(&mut t, P0, B, 2);
    assert!(!castable(&mut t, P0, back));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1), "{}", t.dump_log());
    assert!(exiled(&t, back));
}

#[test]
fn idol_of_endurance_card_exiled_again_isnt_returned() {
    cr!("400.7", "610.3", "607.2a");
    ruling!(
        "Idol of Endurance",
        "If you cast a spell with Idol of Endurance's last ability, it becomes a new object. If it returns to exile, it's no longer exiled with Idol of Endurance, and won't be returned to your graveyard once Idol of Endurance leaves the battlefield."
    );
    supported("Idol of Endurance");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let idol = t.enter(P0, "Idol of Endurance");
    t.resolve_all();
    assert!(exiled(&t, bears) && exiled(&t, elves));
    pool(&mut t, P0, &[W, C]);
    t.activate(P0, idol, 0, &[]).unwrap();
    t.resolve_all();
    cast_free(&mut t, P0, bears);
    t.resolve_all();
    let on_bf = t.g.current(bears);
    assert!(t.on_battlefield(on_bf), "{}", t.dump_log());
    let back = move_to(&mut t, on_bf, Zone::Exile);
    move_to(&mut t, idol, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(elves), Zone::Graveyard(P0), "{}", t.dump_log());
    assert!(exiled(&t, back));
}

#[test]
fn spelljack_instant_or_sorcery_goes_to_its_owners_graveyard_or_hand_with_buyback() {
    cr!("608.2n", "702.27a", "118.9");
    ruling!(
        "Spelljack",
        "If you cast an instant or sorcery, it goes to its owner’s graveyard when it resolves."
    );
    ruling!(
        "Spelljack",
        "If you use this on an opponent’s spell which has Buyback, you may pay the Buyback cost as you cast it. If you do so, the card will go to its owner’s hand upon resolution. It will not go to your hand, nor to any player’s graveyard."
    );
    supported("Spelljack");
    // P1's Divination, countered and exiled; P0 casts it in its main phase.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = t.hand(P1, "Divination");
    mana(&mut t, P1, U, 3);
    let spell = t.cast(P1, div).go();
    let sj = t.hand(P0, "Spelljack");
    mana(&mut t, P0, U, 6);
    t.g.turn.priority = Some(P0);
    t.cast(P0, sj).target(spell).go();
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    t.set_step(P0, Step::PrecombatMain);
    cast_free(&mut t, P0, div);
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P0), 2);
    // P1's Capsize (buyback {3}); P0 casts it paying buyback: it returns to P1's hand.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let cap = t.hand(P1, "Capsize");
    mana(&mut t, P1, U, 6);
    t.g.turn.priority = Some(P1);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P1, cap).target(mine).go();
    let sj = t.hand(P0, "Spelljack");
    mana(&mut t, P0, U, 6);
    t.g.turn.priority = Some(P0);
    t.cast(P0, sj).target(spell).go();
    t.resolve_all();
    assert!(exiled(&t, cap), "{}", t.dump_log());
    assert!(t.on_battlefield(mine));
    t.lands(P0, "Island", 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[o(theirs)]);
    cast_free(&mut t, P0, cap);
    t.resolve_all();
    assert_eq!(t.zone(theirs), Zone::Hand(P1), "{}", t.dump_log());
    assert_eq!(t.zone(cap), Zone::Hand(P1), "{}", t.dump_log());
    assert_eq!(tapped_lands(&t, P0), 3);
}

// ---------------------------------------------------------------------------
// Adventures, split cards, modal double-faced cards
// ---------------------------------------------------------------------------

/// Casts Stomp ("Damage can't be prevented this turn. Stomp deals 2 damage to any
/// target.") from the exiled Bonecrusher Giant `giant` for P0, at P1, paying with Swamps.
fn stomp(t: &mut TestGame, giant: ObjectId) {
    t.lands(P0, "Swamp", 2);
    let life = t.life(P1);
    let methods = methods(t, P0, giant);
    let g = t.g.current(giant);
    match methods.iter().find(|m| matches!(m, CastMethod::Half(_))) {
        Some(adv) => {
            let adv = adv.clone();
            t.cast(P0, g).method(adv).target(P1).go();
        }
        None => {
            // Without paying its mana cost: the face is chosen (creature, then Stomp).
            assert!(methods.contains(&CastMethod::Free), "{methods:?}");
            t.answer(P0, DecisionKind::Option, Answer::Index(1));
            t.cast(P0, g).method(CastMethod::Free).target(P1).go();
        }
    }
    t.resolve_all();
    assert_eq!(t.life(P1), life - 2, "{}", t.dump_log());
}

#[test]
fn covetous_urge_and_robber_of_the_rich_adventure_you_cast_the_creature_later() {
    cr!("715.4", "715.3d", "609.4b");
    ruling!(
        "Covetous Urge",
        "If you cast an Adventure with Covetous Urge, it's you, not the card's owner, who may cast the creature later."
    );
    ruling!(
        "Robber of the Rich",
        "If you cast an Adventure with Robber of the Rich, it's you, not the card's owner, who may cast the creature later."
    );
    supported("Covetous Urge");
    supported("Robber of the Rich");
    for which in ["Covetous Urge", "Robber of the Rich"] {
        let mut t = TestGame::new(2);
        let giant = if which == "Covetous Urge" {
            let giant = t.hand(P1, "Bonecrusher Giant // Stomp");
            let cu = t.hand(P0, "Covetous Urge");
            pool(&mut t, P0, &[U, U, B, B]);
            t.answer_choose_moved(P0, &[o(giant)]);
            t.cast(P0, cu).target(P1).go();
            t.resolve_all();
            giant
        } else {
            let robber = t.battlefield(P0, "Robber of the Rich");
            let giant = t.library_top(P1, "Bonecrusher Giant // Stomp");
            t.hand(P1, "Island");
            attack_with(&mut t, &[(robber, Entity::Player(P1))]);
            block_and_finish(&mut t, P1, &[]);
            t.resolve_all();
            t.set_step(P0, Step::PostcombatMain);
            giant
        };
        assert!(exiled(&t, giant), "{which}: {}", t.dump_log());
        stomp(&mut t, giant);
        // On an adventure in exile: P0 may cast the creature; P1 may not.
        assert!(exiled(&t, giant), "{which}");
        // The permission's "any color" doesn't apply to the new object: red mana.
        t.lands(P0, "Mountain", 3);
        assert!(castable(&mut t, P0, giant), "{which}: {}", t.dump_log());
        t.set_step(P1, Step::PrecombatMain);
        mana(&mut t, P1, R, 3);
        assert!(!castable(&mut t, P1, giant), "{which}");
    }
}

#[test]
fn gales_redirection_can_cast_an_adventurer_cards_adventure() {
    cr!("715.3", "715.3a");
    ruling!(
        "Gale's Redirection",
        "If the exiled card is an adventurer card, you may cast the Adventure spell this way."
    );
    // P1 casts Bonecrusher Giant (the creature); P0 exiles it with Gale's Redirection.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.hand(P1, "Bonecrusher Giant // Stomp");
    mana(&mut t, P1, R, 3);
    let spell = t.cast(P1, giant).go();
    let gr = t.hand(P0, "Gale's Redirection");
    pool(&mut t, P0, &[U, U, C, C, C]);
    t.g.turn.priority = Some(P0);
    t.cast(P0, gr).target(spell).go();
    t.resolve_all();
    assert!(exiled(&t, giant), "{}", t.dump_log());
    // Whatever the roll, Stomp can be cast from exile.
    t.set_step(P0, Step::PrecombatMain);
    stomp(&mut t, giant);
}

#[test]
fn elite_arcanist_split_card_x_is_the_total_and_each_copy_picks_a_half() {
    cr!("709.4b", "709.3", "707.12");
    ruling!(
        "Elite Arcanist",
        "If the exiled card is a split card, then X is determined by the combined mana cost of both halves. However, when you cast the copy, you choose only one half to cast. You may cast a different half each time you create and cast a copy."
    );
    supported("Elite Arcanist");
    let mut t = TestGame::new(2);
    let fi = t.hand(P0, "Fire // Ice");
    t.answer_choose(P0, &[o(fi)]);
    let ea = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    assert!(exiled(&t, fi), "{}", t.dump_log());
    t.g.obj_mut(ea).summoning_sick = false;
    let victim = t.battlefield(P1, "Grizzly Bears");
    // X = 4 ({1}{R} + {1}{U}): three mana isn't enough.
    mana(&mut t, P0, C, 3);
    assert!(t.activate(P0, ea, 0, &[]).is_err(), "{}", t.dump_log());
    // With four: cast Fire (2 damage to P1).
    mana(&mut t, P0, C, 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, ea, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    // Again: this time Ice (tap the Bears, draw a card).
    t.g.obj_mut(ea).tapped = false;
    mana(&mut t, P0, C, 4);
    let hand = t.hand_size(P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(victim)]);
    t.activate(P0, ea, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(victim).tapped, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(exiled(&t, fi));
}

#[test]
fn jodah_can_cast_the_nonlegendary_back_face_of_a_legendary_mdfc() {
    cr!("712.11b", "712.11c", "712.8a");
    ruling!(
        "Jodah, the Unifier",
        "If the legendary nonland card exiled this way is a modal double-faced card, and the back face is also a nonland card face, you may cast either face, even if the back face isn’t legendary."
    );
    supported("Jodah, the Unifier");
    // Extus, Oriq Overlord (legendary creature, mana value 4) // Awaken the Blood Avatar
    // (a sorcery: "Each opponent sacrifices a creature. Create a 3/6 black and red Avatar
    // creature token with haste ...").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jodah, the Unifier");
    let extus = t.library_top(P0, "Extus, Oriq Overlord // Awaken the Blood Avatar");
    let jodah2 = t.hand(P0, "Jodah, the Unifier");
    t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, &[W, U, B, R, G]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let from = t.asked().len();
    t.cast(P0, jodah2).go();
    t.settle();
    t.resolve();
    let opts: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Avatar Token").len(),
        1,
        "{opts:?}: {}",
        t.dump_log()
    );
    assert_eq!(t.zone(extus), Zone::Graveyard(P0));
}

#[test]
fn tibalts_trickery_mdfc_back_face_name() {
    cr!("712.8a", "712.11b", "712.11c");
    ruling!(
        "Tibalt's Trickery",
        "If the spell that's countered was the back face of a modal double-faced card, no card in the player's library will have the same name as that spell because those cards all use only their front-face characteristics while in the library; however, if the first card the player exiles that has a different name than the spell is a modal double-faced card, they may cast either face, even if the back face has the same name as the countered spell."
    );
    supported("Tibalt's Trickery");
    // P1 casts Tibalt, Cosmic Impostor (the back face of Valki, God of Lies); its library
    // has another Valki // Tibalt under three Forests.
    let mut t = TestGame::new(2);
    t.g.players[1].library.clear();
    let lib = crate::r_s01_common::stack_library(
        &mut t,
        P1,
        &[
            "Forest",
            "Forest",
            "Forest",
            "Valki, God of Lies // Tibalt, Cosmic Impostor",
            "Hill Giant",
        ],
    );
    t.set_step(P1, Step::PrecombatMain);
    let vt = t.hand(P1, "Valki, God of Lies // Tibalt, Cosmic Impostor");
    pool(&mut t, P1, &[B, R, C, C, C, C, C]);
    let spell = t.cast(P1, vt).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj_now(spell).chars.name, "Tibalt, Cosmic Impostor");
    let tt = t.hand(P0, "Tibalt's Trickery");
    mana(&mut t, P0, R, 2);
    t.g.turn.priority = Some(P0);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, tt).target(spell).go();
    t.resolve_all();
    let found = t.g.current(lib[3]);
    assert!(t.on_battlefield(found), "{}", t.dump_log());
    assert_eq!(t.obj_now(found).chars.name, "Tibalt, Cosmic Impostor");
    assert_eq!(t.obj_now(found).controller, P1);
}

#[test]
fn etali_primal_conqueror_order_and_targets_of_the_spells_it_casts() {
    cr!("608.2g", "601.2c", "115.1");
    ruling!(
        "Etali, Primal Conqueror // Etali, Primal Sickness",
        "If you cast more than one of the exiled cards, you choose the order in which to cast them. A spell you cast this way can be the target of a later spell you cast this way. However, permanent spells cast this way won’t resolve until you’re done casting spells, so the permanents they become can’t be the target of spells cast this way. For example, if you exile Twincast and Lightning Strike, you can cast Lightning Strike and then cast Twincast targeting it; but if you exile a creature card and an Aura card, you can’t cast that Aura targeting that creature."
    );
    supported("Etali, Primal Conqueror // Etali, Primal Sickness");
    // Lightning Strike, then Twincast targeting it.
    let mut t = TestGame::new(2);
    let strike = t.library_top(P0, "Lightning Strike");
    let twincast = t.library_top(P1, "Twincast");
    t.answer_choose_moved(P0, &[o(strike), o(twincast)]);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    target_named_when_offered(&mut t, P0, "Lightning Strike");
    t.enter(P0, "Etali, Primal Conqueror // Etali, Primal Sickness");
    t.resolve_all();
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
    // Grizzly Bears and Rancor: the Bears spell isn't a creature on the battlefield, so
    // Rancor can only enchant Etali.
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let rancor = t.library_top(P1, "Rancor");
    t.answer_choose_moved(P0, &[o(bears), o(rancor)]);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    let from = t.asked().len();
    let etali = t.enter(P0, "Etali, Primal Conqueror // Etali, Primal Sickness");
    t.resolve_all();
    let offered = targets_offered(&t, from);
    assert!(offered.contains(&o(etali)), "{offered:?}: {}", t.dump_log());
    let bears_now = t.g.current(bears);
    assert!(t.on_battlefield(bears_now), "{}", t.dump_log());
    assert!(!offered
        .iter()
        .any(|e| matches!(e, Entity::Object(x) if t.g.current(*x) == bears_now)));
    let r = t.g.current(rancor);
    assert_eq!(t.obj_now(r).attached_to, Some(o(etali)), "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Targets, costs, copies
// ---------------------------------------------------------------------------

#[test]
fn quintorius_illegal_target_no_spirit_token() {
    cr!("608.2b");
    ruling!(
        "Quintorius, Loremaster",
        "If the target of Quintorius's triggered ability is an illegal target at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't create a Spirit token."
    );
    supported("Quintorius, Loremaster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quintorius, Loremaster");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[o(bolt)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    move_to(&mut t, bolt, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.named_on_battlefield("Spirit Token").is_empty(), "{}", t.dump_log());
    assert_eq!(t.zone(bolt), Zone::Hand(P0));
}

#[test]
fn ethersworn_sphinx_cost_increases_apply_before_affinity() {
    cr!("601.2f");
    ruling!(
        "Ethersworn Sphinx",
        "If there are any additional costs or cost increases that would apply to Ethersworn Sphinx, apply those before applying any cost reductions."
    );
    // {7}{W}{U}, one more for Sphere of Resistance, eight less for eight artifacts
    // (Sphere and seven Ornithopters): {W}{U}. Reducing first would leave {1}{W}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    for _ in 0..7 {
        t.battlefield(P0, "Ornithopter");
    }
    let es = t.hand(P0, "Ethersworn Sphinx");
    pool(&mut t, P0, &[W, U]);
    t.cast(P0, es).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ethersworn Sphinx").len(), 1);
    // With seven artifacts it costs {1}{W}{U}: W and U alone don't pay for it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    for _ in 0..6 {
        t.battlefield(P0, "Ornithopter");
    }
    t.answer_yes(P0, false);
    let es = t.hand(P0, "Ethersworn Sphinx");
    pool(&mut t, P0, &[W, U]);
    assert!(t.cast(P0, es).try_go().is_err());
}

#[test]
fn volcanic_torrent_counts_cast_copies_not_copied_spells() {
    cr!("707.10", "707.12", "702.85a");
    ruling!(
        "Volcanic Torrent",
        "If you copy a spell, Volcanic Torrent won't count the copy. However, if an effect instructs you to copy a card and then cast the copy, it will be counted by Volcanic Torrent."
    );
    let mut t = TestGame::new(2);
    let ancients = t.battlefield(P1, "Indomitable Ancients");
    // 1: Lightning Bolt. 2: Twincast copying it (the copy isn't cast).
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, R, 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    let tc = t.hand(P0, "Twincast");
    mana(&mut t, P0, U, 2);
    t.answer_yes(P0, false);
    t.cast(P0, tc).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // 3: Elite Arcanist's copy of Shock, cast.
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[o(shock)]);
    let ea = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    t.g.obj_mut(ea).summoning_sick = false;
    mana(&mut t, P0, C, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, ea, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "{}", t.dump_log());
    // 4: Volcanic Torrent (its cascade finds Grizzly Bears, not cast).
    t.library_top(P0, "Grizzly Bears");
    let vt = t.hand(P0, "Volcanic Torrent");
    mana(&mut t, P0, R, 5);
    t.answer_yes(P0, false);
    t.cast(P0, vt).go();
    t.resolve_all();
    assert_eq!(t.obj_now(ancients).damage, 4, "{}", t.dump_log());
}

#[test]
fn alaundo_mana_value_0_card_gets_no_time_counters_and_never_triggers() {
    cr!("202.3a", "122.1");
    ruling!(
        "Alaundo the Seer",
        "If you exile a card with a mana value of 0, such as a land card, no time counters will be placed on it. Unless you find a way to put time counters on it in exile, the triggered ability it gains from Alaundo will never trigger."
    );
    supported("Alaundo the Seer");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alaundo the Seer");
    let forest = t.hand(P0, "Forest");
    t.answer_choose(P0, &[o(forest)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert_eq!(t.counters(forest, "time"), 0);
    // Activating again removes no counter from it; nothing triggers.
    t.g.obj_mut(a).tapped = false;
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[o(bears)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, forest));
    assert_eq!(t.counters(bears, "time"), 2);
    assert!(t.g.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Knowledge Pool
// ---------------------------------------------------------------------------

/// Knowledge Pool enters under P0's control (each player exiles the top three cards of
/// their library: P0's are `mine`, top first); P1 casts Lightning Bolt at P0 from its
/// hand in its main phase. Returns the Bolt spell.
fn knowledge_pool(t: &mut TestGame, mine: &[&str]) -> ObjectId {
    supported("Knowledge Pool");
    crate::r_s01_common::stack_library(t, P0, mine);
    crate::r_s01_common::stack_library(t, P1, &["Forest", "Forest", "Forest"]);
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 6);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(t, P1, R, 1);
    t.cast(P1, bolt).target(P0).go()
}

#[test]
fn knowledge_pool_spell_countered_first_nothing_is_cast() {
    cr!("608.2c", "603.3");
    ruling!(
        "Knowledge Pool",
        "If the original spell isn't exiled (perhaps because it's countered by another spell or ability before Knowledge Pool's second triggered ability resolves), the rest of the ability does nothing. The player doesn't get to cast an exiled card."
    );
    let mut t = TestGame::new(2);
    // Daring Apprentice: "{T}, Sacrifice this creature: Counter target spell." (A spell
    // cast from P0's hand would itself be exiled by Knowledge Pool.)
    let da = t.battlefield(P0, "Daring Apprentice");
    let bolt = knowledge_pool(&mut t, &["Grizzly Bears", "Divination", "Forest"]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.g.turn.priority = Some(P0);
    t.activate(P0, da, 0, &[o(bolt)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "{}", t.dump_log());
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.g.exile.len(), 6);
    assert!(offered_since(&t, from).is_empty());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn knowledge_pool_with_no_nonland_cards_still_exiles_the_spell() {
    cr!("608.2c");
    ruling!(
        "Knowledge Pool",
        "If there are no nonland cards exiled by Knowledge Pool, the original spell is still exiled."
    );
    let mut t = TestGame::new(2);
    knowledge_pool(&mut t, &["Island", "Island", "Island"]);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
}
