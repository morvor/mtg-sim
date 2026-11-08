//! Rulings batch P070 — permissions to play exiled cards (CR 601.3, 305.1, 611.2): who has
//! them (the player who controlled the ability, even after its source leaves the
//! battlefield or changes control, CR 611.2a), what they allow (casting isn't playing a
//! land, CR 305.1; the card isn't in a hand, CR 400.1), when they can be used (the card's
//! normal timing and land plays, CR 307.1, 305.2), and which cards they cover.

use crate::r_p070_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::facedown::can_look_at;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Gives control of `id` to `p`.
fn give_control(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    let o = t.g.obj_mut(id);
    o.controller = p;
    o.base_controller = p;
    t.g.dirty = true;
    t.g.recompute();
}

/// Whether `p` may cast `card` in its own main phase with an empty stack and plenty of
/// mana.
fn castable_in_main(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.stack.clear();
    t.set_step(p, Step::PostcombatMain);
    lots_of_mana(t, p);
    castable(t, p, card)
}

/// `card` (Grizzly Bears, owned by P1) was exiled by `source`'s ability, controlled by
/// P0. P0 may look at it (when it's face down) and cast it; after P1 gains control of
/// `source`, P1 may do neither and P0 still may; after `source` leaves the battlefield,
/// P0 still may.
fn outlasts_its_source(t: &mut TestGame, source: ObjectId, card: ObjectId, face_down: bool) {
    let c = t.g.current(card);
    assert!(exiled(t, c), "{}", t.dump_log());
    assert_eq!(t.obj_now(c).face_down, face_down);
    let p0_may = |t: &mut TestGame, when: &str| {
        if face_down {
            assert!(can_look_at(&t.g, P0, c), "{when}: P0 looks");
        }
        assert!(
            castable_in_main(t, P0, c),
            "{when}: P0 casts: {}",
            t.dump_log()
        );
    };
    p0_may(t, "at first");
    give_control(t, source, P1);
    if face_down {
        assert!(!can_look_at(&t.g, P1, c), "P1 can't look");
    }
    assert!(!castable_in_main(t, P1, c), "P1 can't cast");
    p0_may(t, "P1 controls the source");
    let owner = t.obj_now(source).owner;
    move_to(t, source, Zone::Graveyard(owner));
    p0_may(t, "the source left the battlefield");
}

/// `attacker` (on P0's battlefield) attacks P1 unblocked and the combat damage step's
/// triggers resolve.
fn connect(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
}

#[test]
fn thief_of_sanity_card_stays_castable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Thief of Sanity",
        "You can still cast the exiled card if Thief of Sanity leaves the battlefield or leaves your control. If another player gains control of Thief of Sanity, that player can't cast the exiled card."
    );
    supported("Thief of Sanity");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Thief of Sanity");
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Forest", "Island"]);
    t.answer_choose(P0, &[o(lib[0])]);
    connect(&mut t, thief);
    outlasts_its_source(&mut t, thief, lib[0], true);
}

#[test]
fn rev_cards_stay_visible_and_castable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Rev, Tithe Extractor",
        "You may look at and cast cards exiled face down with Rev's last ability even if Rev leaves the battlefield. If another player gains control of Rev, that player can't look at or cast the cards you exiled with its last ability, and you still can."
    );
    supported("Rev, Tithe Extractor");
    let mut t = TestGame::new(2);
    let rev = t.battlefield(P0, "Rev, Tithe Extractor");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(rev)]);
    connect(&mut t, rev);
    outlasts_its_source(&mut t, rev, bears, true);
}

#[test]
fn gonti_lord_of_luxury_card_stays_visible_and_castable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Gonti, Lord of Luxury",
        "You may look at and cast that card (and spend mana as though it were mana of any type to do so) even if Gonti leaves the battlefield. If another player gains control of Gonti, that player can't look at or cast the card, and you still can."
    );
    supported("Gonti, Lord of Luxury");
    let mut t = TestGame::new(2);
    let lib = stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Hill Giant", "Forest", "Island"],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[o(lib[0])]);
    let gonti = t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    outlasts_its_source(&mut t, gonti, lib[0], true);
    // Mana of any type: two black mana cast the Bears ({1}{G}).
    let mut t = TestGame::new(2);
    let lib = stack_library(
        &mut t,
        P1,
        &["Grizzly Bears", "Hill Giant", "Forest", "Island"],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[o(lib[0])]);
    t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    mana(&mut t, P0, B, 2);
    t.cast(P0, t.g.current(lib[0])).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(lib[0])), "{}", t.dump_log());
}

#[test]
fn ixhel_cards_stay_visible_and_castable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Ixhel, Scion of Atraxa",
        "You may look at and cast the exiled cards (and spend mana as though it were mana of any color to do so) even if Ixhel leaves the battlefield. If another player gains control of Ixhel, that player can't look at or cast the cards, and you still can."
    );
    supported("Ixhel, Scion of Atraxa");
    let mut t = TestGame::new(2);
    let ixhel = t.battlefield(P0, "Ixhel, Scion of Atraxa");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.g.players[1].counters.insert("poison".into(), 3);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    outlasts_its_source(&mut t, ixhel, bears, true);
}

#[test]
fn blightwing_bandit_cards_stay_visible_and_castable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Blightwing Bandit",
        "You may look at and cast the exiled cards (and spend mana of any type to do so) even if Blightwing Bandit leaves the battlefield. If another player gains control of Blightwing Bandit, that player can't look at or cast the cards, and you still can."
    );
    supported("Blightwing Bandit");
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Blightwing Bandit");
    let bears = t.library_top(P1, "Grizzly Bears");
    // P0's first spell during P1's turn.
    t.set_step(P1, Step::PrecombatMain);
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, U, 1);
    t.g.turn.priority = Some(P0);
    t.cast(P0, opt).go();
    t.resolve_all();
    outlasts_its_source(&mut t, bb, bears, true);
}

#[test]
fn gonti_canny_acquisitor_cards_stay_visible_and_playable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Gonti, Canny Acquisitor",
        "You may look at and play cards exiled face down with Gonti’s last ability (and spend mana of any type to do so) even if Gonti leaves the battlefield. If another player gains control of Gonti, that player can’t look at or play cards already exiled with Gonti, and you still can."
    );
    supported("Gonti, Canny Acquisitor");
    let mut t = TestGame::new(2);
    let gonti = t.battlefield(P0, "Gonti, Canny Acquisitor");
    let bears = t.library_top(P1, "Grizzly Bears");
    connect(&mut t, gonti);
    outlasts_its_source(&mut t, gonti, bears, true);
}

/// Mindleecher mutates onto P0's Grizzly Bears, exiling the top card of P1's library face
/// down. Returns Mindleecher (the merged permanent) and that card.
fn mindleecher(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId) {
    let card = t.library_top(P1, top);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = t.hand(P0, "Mindleecher");
    mana(t, P0, B, 5);
    t.cast(P0, m)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Mutate,
        ))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert!(exiled(t, card), "{}", t.dump_log());
    (t.g.current(bears), card)
}

#[test]
fn mindleecher_cards_stay_visible_and_playable_by_you_only() {
    cr!("611.2a", "406.3");
    ruling!(
        "Mindleecher",
        "You may look at and play those cards even if Mindleecher leaves the battlefield. If another player gains control of Mindleecher, that player can’t look at or play the cards, and you still can."
    );
    supported("Mindleecher");
    let mut t = TestGame::new(2);
    let (m, bears) = mindleecher(&mut t, "Grizzly Bears");
    outlasts_its_source(&mut t, m, bears, true);
}

#[test]
fn mindleecher_lands_need_a_land_play() {
    cr!("305.2", "305.1");
    ruling!(
        "Mindleecher",
        "You can play lands this way only if you have remaining land plays available for the turn."
    );
    supported("Mindleecher");
    let mut t = TestGame::new(2);
    let (_, forest) = mindleecher(&mut t, "Forest");
    assert!(playable_land(&mut t, P0, forest));
    let mtn = t.hand(P0, "Mountain");
    t.play_land(P0, mtn).unwrap();
    assert!(!playable_land(&mut t, P0, forest));
}

#[test]
fn dragon_kami_reborn_cards_stay_visible_after_it_leaves() {
    cr!("406.3", "714.2b");
    ruling!(
        "The Dragon-Kami Reborn // Dragon-Kami's Egg",
        "You may look at the cards you exile with The Dragon-Kami Reborn's first two chapter abilities, and you may continue to do so after it has left the battlefield."
    );
    supported("The Dragon-Kami Reborn // Dragon-Kami's Egg");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Island"]);
    t.answer_choose(P0, &[o(lib[0])]);
    let saga = t.enter(P0, "The Dragon-Kami Reborn // Dragon-Kami's Egg");
    t.resolve_all();
    let c = t.g.current(lib[0]);
    assert!(exiled(&t, c) && t.obj_now(c).face_down, "{}", t.dump_log());
    assert_eq!(t.counters(c, "hatching"), 1);
    assert!(can_look_at(&t.g, P0, c));
    assert!(!can_look_at(&t.g, P1, c));
    move_to(&mut t, saga, Zone::Graveyard(P0));
    assert!(can_look_at(&t.g, P0, c), "still, after it left");
    assert!(!can_look_at(&t.g, P1, c));
}

// ---------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------

/// Asserts that `p` can cast `card` in its main phase with an empty stack only (not in
/// combat, not with a spell on the stack, not in another player's turn).
fn sorcery_timing_only(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let other = PlayerId(1 - p.0);
    t.set_step(p, Step::PostcombatMain);
    lots_of_mana(t, p);
    assert!(
        castable(t, p, card),
        "main phase, empty stack: {}",
        t.dump_log()
    );
    let bolt = t.hand(other, "Lightning Bolt");
    mana(t, other, R, 1);
    t.g.turn.priority = Some(other);
    t.cast(other, bolt).target(other).go();
    assert!(!castable(t, p, card), "stack not empty");
    t.resolve_all();
    t.set_step(p, Step::BeginningOfCombat);
    assert!(!castable(t, p, card), "combat");
    t.set_step(other, Step::PrecombatMain);
    assert!(!castable(t, p, card), "another player's turn");
}

#[test]
fn thief_of_sanity_doesnt_change_when_you_can_cast_the_card() {
    cr!("601.3", "307.1");
    ruling!(
        "Thief of Sanity",
        "Thief of Sanity's effect doesn't change when you can cast the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty."
    );
    supported("Thief of Sanity");
    let mut t = TestGame::new(2);
    let thief = t.battlefield(P0, "Thief of Sanity");
    let lib = stack_library(&mut t, P1, &["Divination", "Forest", "Island"]);
    t.answer_choose(P0, &[o(lib[0])]);
    connect(&mut t, thief);
    assert!(exiled(&t, lib[0]), "{}", t.dump_log());
    sorcery_timing_only(&mut t, P0, lib[0]);
}

/// Nassari, Dean of Expression (the back face of Uvilda) on P0's battlefield.
fn nassari(t: &mut TestGame) -> ObjectId {
    let n = t.custom(
        P0,
        (*card("Uvilda, Dean of Perfection // Nassari, Dean of Expression")).clone(),
        Zone::Hand(P0),
    );
    mana(t, P0, R, 5);
    t.cast(P0, n).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let n = t.g.current(n);
    assert!(t.on_battlefield(n), "{}", t.dump_log());
    assert_eq!(t.obj_now(n).chars.name, "Nassari, Dean of Expression");
    n
}

#[test]
fn nassari_exiles_face_up_and_its_spells_keep_normal_timing() {
    cr!("406.3", "601.3", "307.1");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "You exile cards with Nassari, Dean of Expression face up. Casting those spells is subject to the normal timing restrictions for spells."
    );
    supported("Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    let mut t = TestGame::new(2);
    nassari(&mut t);
    let div = t.library_top(P1, "Divination");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(exiled(&t, div), "{}", t.dump_log());
    assert!(!t.obj_now(t.g.current(div)).face_down, "face up");
    assert!(can_look_at(&t.g, P1, t.g.current(div)));
    // In the upkeep: not yet (a sorcery).
    lots_of_mana(&mut t, P0);
    assert!(!castable(&mut t, P0, div));
    sorcery_timing_only(&mut t, P0, div);
}

#[test]
fn valgavoth_spells_and_lands_follow_the_normal_timing_rules() {
    cr!("601.3", "305.2", "307.1");
    ruling!(
        "Valgavoth, Terror Eater",
        "You follow all timing rules for spells cast and lands played using the permission granted by Valgavoth's last ability."
    );
    supported("Valgavoth, Terror Eater");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valgavoth, Terror Eater");
    // P1's cards are put into P1's graveyard: exiled instead.
    let div = t.hand(P1, "Divination");
    let div = move_to(&mut t, div, Zone::Graveyard(P1));
    let forest = t.hand(P1, "Forest");
    let forest = move_to(&mut t, forest, Zone::Graveyard(P1));
    assert!(exiled(&t, div) && exiled(&t, forest), "{}", t.dump_log());
    sorcery_timing_only(&mut t, P0, div);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!playable_land(&mut t, P0, forest), "combat");
    t.set_step(P1, Step::PrecombatMain);
    assert!(!playable_land(&mut t, P0, forest), "P1's turn");
    t.set_step(P0, Step::PrecombatMain);
    assert!(playable_land(&mut t, P0, forest));
    let mtn = t.hand(P0, "Mountain");
    t.play_land(P0, mtn).unwrap();
    assert!(!playable_land(&mut t, P0, forest), "no land play left");
}

#[test]
fn hedonists_trove_lands_need_a_land_play() {
    cr!("305.2", "305.1");
    ruling!(
        "Hedonist's Trove",
        "Under normal circumstances, you can play a land card exiled with Hedonist’s Trove only if you haven’t played a land yet that turn."
    );
    supported("Hedonist's Trove");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P1, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Hedonist's Trove");
    t.resolve_all();
    assert!(exiled(&t, forest), "{}", t.dump_log());
    assert!(playable_land(&mut t, P0, forest));
    let mtn = t.hand(P0, "Mountain");
    t.play_land(P0, mtn).unwrap();
    assert!(!playable_land(&mut t, P0, forest));
}

#[test]
fn ornate_kanzashi_cards_from_previous_turns_cant_be_played() {
    cr!("611.2a");
    ruling!(
        "Ornate Kanzashi",
        "You can’t play cards you exiled with Ornate Kanzashi on previous turns."
    );
    supported("Ornate Kanzashi");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Ornate Kanzashi");
    let bears = t.library_top(P1, "Grizzly Bears");
    mana(&mut t, P0, C, 2);
    t.activate(P0, k, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, bears));
    assert!(castable_in_main(&mut t, P0, bears));
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(exiled(&t, bears));
    assert!(!castable_in_main(&mut t, P0, bears), "{}", t.dump_log());
}

#[test]
fn robber_of_the_rich_card_castable_whatever_became_of_the_rogue() {
    cr!("601.3", "506.4");
    ruling!(
        "Robber of the Rich",
        "You can cast the exiled card if the Rogue that attacked is still in combat, if it's left combat, if it's left the battlefield, or even if combat is over."
    );
    supported("Robber of the Rich");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robber of the Rich");
    t.hand(P1, "Island");
    let bolt = t.library_top(P1, "Lightning Bolt");
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    t.resolve_all();
    assert!(exiled(&t, bolt), "{}", t.dump_log());
    mana(&mut t, P0, B, 1);
    assert!(castable(&mut t, P0, bolt), "still in combat");
    // It leaves combat (and the battlefield).
    move_to(&mut t, r, Zone::Graveyard(P0));
    assert!(castable(&mut t, P0, bolt), "left the battlefield");
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, bolt), "combat over");
    // A turn P0 didn't attack with a Rogue: no.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, R, 1);
    assert!(!castable(&mut t, P0, bolt));
}

// ---------------------------------------------------------------------------
// Which cards; casting isn't playing
// ---------------------------------------------------------------------------

#[test]
fn tinybones_plays_cards_a_previous_tinybones_stashed() {
    cr!("604.1", "122.1");
    ruling!(
        "Tinybones, Bauble Burglar",
        "Tinybones's second ability allows you to play all exiled cards you don't own with stash counters on them  , regardless of whether they were put there by the Tinybones you currently control or a Tinybones that was previously on the battlefield."
    );
    supported("Tinybones, Bauble Burglar");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Tinybones, Bauble Burglar");
    let bears = t.hand(P1, "Grizzly Bears");
    let rot = t.hand(P0, "Mind Rot");
    mana(&mut t, P0, B, 3);
    t.cast(P0, rot).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, bears), "{}", t.dump_log());
    assert_eq!(t.counters(t.g.current(bears), "stash"), 1);
    assert!(castable_in_main(&mut t, P0, bears));
    // The first Tinybones leaves; another one is on the battlefield.
    move_to(&mut t, first, Zone::Graveyard(P0));
    assert!(!castable_in_main(&mut t, P0, bears), "no Tinybones");
    t.battlefield(P0, "Tinybones, Bauble Burglar");
    assert!(castable_in_main(&mut t, P0, bears), "{}", t.dump_log());
}

#[test]
fn fiend_of_the_shadows_card_can_only_be_played() {
    cr!("702.29a", "305.1");
    ruling!(
        "Fiend of the Shadows",
        "You may only play the card, meaning play it as a land or cast it as a spell. It's not in your hand. You can't discard it, activate its cycling abilities, and so on."
    );
    supported("Fiend of the Shadows");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fiend of the Shadows");
    let sandbar = t.hand(P1, "Lonely Sandbar");
    t.answer_choose(P1, &[o(sandbar)]);
    connect(&mut t, f);
    assert!(exiled(&t, sandbar), "{}", t.dump_log());
    t.set_step(P0, Step::PostcombatMain);
    lots_of_mana(&mut t, P0);
    let c = t.g.current(sandbar);
    // Not in P0's hand; its cycling ability can't be activated.
    assert!(!t.g.player(P0).hand.contains(&c));
    t.g.turn.priority = Some(P0);
    let acts = t.g.legal_actions(P0);
    assert!(
        !acts
            .iter()
            .any(|a| matches!(a, Action::Activate { source, .. } if *source == c)),
        "{acts:?}"
    );
    // It can be played as a land.
    assert!(playable_land(&mut t, P0, c));
    t.play_land(P0, c).unwrap();
    assert!(t.on_battlefield(t.g.current(c)));
}

#[test]
fn ramirez_cant_play_exiled_lands() {
    cr!("305.1", "601.3");
    ruling!(
        "Ramirez DePietro, Pillager",
        "You may not play lands exiled with the last ability."
    );
    supported("Ramirez DePietro, Pillager");
    for (name, land) in [("Forest", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Ramirez DePietro, Pillager");
        let c = t.library_top(P1, name);
        connect(&mut t, r);
        assert!(exiled(&t, c), "{}", t.dump_log());
        t.set_step(P0, Step::PostcombatMain);
        if land {
            assert!(!playable_land(&mut t, P0, c));
        } else {
            assert!(castable_in_main(&mut t, P0, c), "positive control");
        }
    }
}

#[test]
fn mnemonic_betrayal_cant_play_exiled_lands() {
    cr!("305.1", "601.3");
    ruling!(
        "Mnemonic Betrayal",
        "You can’t play land cards exiled this way."
    );
    supported("Mnemonic Betrayal");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P1, "Forest");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let mb = t.hand(P0, "Mnemonic Betrayal");
    pool(&mut t, P0, &[U, B, C]);
    t.cast(P0, mb).go();
    t.resolve_all();
    assert!(exiled(&t, forest) && exiled(&t, bears), "{}", t.dump_log());
    assert!(!playable_land(&mut t, P0, forest));
    assert!(castable_in_main(&mut t, P0, bears), "positive control");
}

#[test]
fn stolen_goods_land_cards_remain_exiled() {
    cr!("305.1", "601.3");
    ruling!(
        "Stolen Goods",
        "You cast the card from exile. Each land card exiled this way will remain exiled."
    );
    supported("Stolen Goods");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Forest", "Island", "Grizzly Bears"]);
    let sg = t.hand(P0, "Stolen Goods");
    pool(&mut t, P0, &[U, C, C, C]);
    t.cast(P0, sg).target(P1).go();
    t.resolve_all();
    assert!(lib.iter().all(|c| exiled(&t, *c)), "{}", t.dump_log());
    assert!(!playable_land(&mut t, P0, lib[0]));
    assert!(!playable_land(&mut t, P0, lib[1]));
    cast_free(&mut t, P0, lib[2]);
    assert!(is_on_stack(&t, lib[2]), "cast from exile");
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]));
}

#[test]
fn ashiok_casts_any_face_up_card_an_opponent_owns_in_exile() {
    cr!("608.2g", "406.3");
    ruling!(
        "Ashiok, Nightmare Muse",
        "You may cast any face-up cards your opponents own from exile while resolving Ashiok's last ability, not just those exiled by Ashiok's second ability or by Ashiok's Nightmare tokens."
    );
    supported("Ashiok, Nightmare Muse");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
    t.g.obj_mut(a).counters.insert("loyalty".into(), 7);
    // Exiled by something else entirely (P1's own Lightning Bolt card, put into exile).
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt = move_to(&mut t, bolt, Zone::Exile);
    t.answer_choose(P0, &[o(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, a, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

#[test]
fn knowledge_pool_casts_any_other_card_it_exiled_whoever_owns_it() {
    cr!("607.2a", "608.2g");
    ruling!(
        "Knowledge Pool",
        "You may cast any other card exiled by Knowledge Pool, including one owned by an opponent. Any card exiled by Knowledge Pool's \"enters\" ability or its other triggered ability may be cast."
    );
    supported("Knowledge Pool");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    assert!(exiled(&t, bears));
    // P0 casts Opt from hand: it's exiled, and P0 casts P1's Bears (exiled as it entered).
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, U, 1);
    t.answer_choose_moved(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(exiled(&t, opt), "{}", t.dump_log());
    let b = t.g.current(bears);
    assert!(t.on_battlefield(b), "{}", t.dump_log());
    assert_eq!(t.obj_now(b).controller, P0);
    // P1 casts Shock from hand: P1 casts P0's Opt (exiled by the other trigger).
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.hand(P1, "Shock");
    mana(&mut t, P1, R, 1);
    let from = t.asked().len();
    t.answer_choose_moved(P1, &[o(opt)]);
    t.answer_yes(P1, true);
    t.cast(P1, shock).target(P0).go();
    t.resolve_all();
    assert!(was_offered(&t, from, opt), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Opt"), "{}", t.dump_log());
}

#[test]
fn knowledge_pool_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Knowledge Pool",
        "Timing restrictions based on the card's type are ignored. For example, you can cast an exiled creature card this way. Other restrictions, such as Spinal Embrace's \"Cast Spinal Embrace only during combat\" are not ignored."
    );
    supported("Knowledge Pool");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Panic", "Forest"]);
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    // P0 casts Opt from hand on P1's turn.
    t.set_step(P1, Step::PrecombatMain);
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, U, 1);
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(lib[0])]);
    t.answer_yes(P0, true);
    t.g.turn.priority = Some(P0);
    t.cast(P0, opt).go();
    t.resolve();
    // The creature spell was cast (on P1's turn); Panic wasn't castable.
    assert!(is_on_stack(&t, lib[0]), "{}", t.dump_log());
    assert!(was_offered(&t, from, lib[0]));
    assert!(!was_offered(&t, from, lib[1]), "{}", t.dump_log());
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(lib[0])));
}

#[test]
fn knowledge_pool_card_can_be_kicked() {
    cr!("601.2b", "702.33d", "118.9");
    ruling!(
        "Knowledge Pool",
        "You may pay additional costs, such as kicker costs, of the exiled card."
    );
    supported("Knowledge Pool");
    let mut t = TestGame::new(2);
    let burst = t.library_top(P1, "Burst Lightning");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 4);
    t.answer_choose_moved(P0, &[o(burst)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "kicked: 4 damage: {}", t.dump_log());
}

#[test]
fn discover_the_impossible_casts_an_instant_half_of_a_card_that_isnt() {
    cr!("715.3a", "406.3b", "608.2g");
    ruling!(
        "Discover the Impossible",
        "You may cast the card you exile with Discover the Impossible as long as the resulting spell is an instant spell with mana value 2 or less, even if the card you exiled doesn't match that criteria. This may happen if you exile a split card, a modal double-faced card, an adventurer card, or similar."
    );
    supported("Discover the Impossible");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    // Bonecrusher Giant (a creature card, mana value 3) // Stomp ({1}{R} instant).
    let giant = t.library_top(P0, "Bonecrusher Giant // Stomp");
    let d = t.hand(P0, "Discover the Impossible");
    pool(&mut t, P0, &[U, C, C]);
    t.answer_choose(P0, &[o(giant)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, d).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "Stomp was cast: {}", t.dump_log());
    // An adventurer card cast as an Adventure goes on an adventure.
    assert!(exiled(&t, giant));
}

#[test]
fn codie_lets_you_cast_an_instant_face_of_a_permanent_mdfc() {
    cr!("712.11b", "712.11c", "601.3");
    ruling!(
        "Codie, Vociferous Codex",
        "While you control Codie (as much as anyone can really “control” Codie), you may cast any face of a modal double-faced card that’s an instant or sorcery face, even if the other face is a permanent face."
    );
    supported("Codie, Vociferous Codex");
    supported("Flamescroll Celebrant // Revel in Silence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Codie, Vociferous Codex");
    let c = t.hand(P0, "Flamescroll Celebrant // Revel in Silence");
    mana(&mut t, P0, R, 2);
    mana(&mut t, P0, W, 2);
    // Flamescroll Celebrant, a creature: no.
    assert!(t.cast(P0, c).try_go().is_err());
    // Revel in Silence, an instant: yes.
    let spell = t.cast(P0, c).method(CastMethod::Half(1)).go();
    assert_eq!(t.obj_now(spell).chars.name, "Revel in Silence");
    t.resolve_all();
}
