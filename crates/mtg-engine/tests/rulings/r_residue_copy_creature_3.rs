//! Rulings residue of batches P020, P021 and P023 — what copies copy, for the copiers
//! that need their own setups: copies of cards in graveyards and exile copy only the
//! printed card, not the object it was (CR 707.2, 400.7); copies of an entering creature,
//! of an attacking modified creature, of an artifact creature, and of the creature a copy
//! is copying (CR 707.2, 707.3); last known information (CR 608.2h); {X} in a copied mana
//! cost is 0 (CR 107.3m, 202.3e, 707.2).

use crate::r_p026_common::{disguise_as_bears, kill};
use crate::r_residue_copy_creature::*;
use crate::r_s01_common::supported;
use crate::r_s18_common::lands_for;
use crate::r_s26_common::new_tokens;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s dressed-up Hill Giant (see `dressed_giant`), a copy of Grizzly Bears until end of
/// turn, dies: returns its card in `p`'s graveyard. A copy of the card is a printed Hill
/// Giant, not a Grizzly Bears or anything else the permanent was.
fn dead_dressed_giant(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let giant = crate::r_p021_common::dressed_giant(t, p);
    disguise_as_bears(t, giant);
    kill(t, giant);
    let card = t.g.current(giant);
    assert_eq!(t.zone(card), Zone::Graveyard(p));
    card
}

/// Asserts the token copies the printed Hill Giant card (with power and toughness `pt`,
/// colors `colors`) and nothing about the object the card was.
fn printed_card_copy(t: &TestGame, card: &str, tok: ObjectId, pt: (i32, i32), colors: ColorSet) {
    let o = t.obj_now(tok);
    assert!(o.is_token(), "{card}");
    assert_eq!(o.chars.name, "Hill Giant", "{card}");
    assert_eq!(o.chars.colors, colors, "{card}: colors");
    assert_eq!(t.pt(tok), pt, "{card}: power and toughness");
    assert!(o.counters.values().all(|n| *n == 0), "{card}: counters");
    assert!(t.g.attachments_of(obj(o.id)).is_empty(), "{card}");
}

fn red() -> ColorSet {
    ColorSet::single(Color::Red)
}

#[test]
fn token_copies_of_cards_copy_only_the_printed_card() {
    cr!("707.2", "400.7", "707.9b");
    ruling!(
        "Altaïr Ibn-La'Ahad",
        "Each of the tokens copies exactly what was printed on the card in exile and nothing else. They don't copy any information about the object the card was before it was exiled. "
    );
    ruling!(
        "Abyssal Harvester",
        "Except for the listed exception, the token created by Abyssal Harvester's ability copies exactly what was printed on the original card and nothing else. It doesn't copy information about the object that card was before it was put into that graveyard."
    );
    ruling!(
        "Ardyn, the Usurper",
        "Except for the listed exceptions, the token copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card was before it was put into a graveyard."
    );
    ruling!(
        "Grist, Voracious Larva // Grist, the Plague Swarm",
        "Except for the listed exceptions, the tokens created by Grist, the Plague Swarm's last ability copy exactly what was printed on the original cards and nothing else. They don't copy any information about the objects those cards were before they were put into your graveyard."
    );
    ruling!(
        "Gyrus, Waker of Corpses",
        "Gyrus's token copies exactly what was printed on the original creature card and nothing else. It doesn't copy any effects that modified it before it was put into your graveyard."
    );
    // Altaïr: "Whenever Altaïr attacks, exile up to one target Assassin creature card from
    // your graveyard with a memory counter on it. Then for each creature card you own in
    // exile with a memory counter on it, create a tapped and attacking token that's a copy
    // of it." The card went from the graveyard to exile with a memory counter.
    supported("Altaïr Ibn-La'Ahad");
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t, P0);
    t.g.move_object(
        card,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    let exiled = t.g.current(card);
    t.g.add_counters(obj(exiled), "memory", 1, None);
    let altair = t.battlefield(P0, "Altaïr Ibn-La'Ahad");
    t.answer_targets(P0, &[]);
    let before = t.g.battlefield.clone();
    attack_p1(&mut t, &[altair]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    printed_card_copy(&t, "Altaïr", toks[0], (3, 3), red());
    // Abyssal Harvester: "{T}: Exile target creature card from a graveyard that was put
    // there this turn. Create a token that's a copy of it, except it's a Nightmare in
    // addition to its other types. ..."
    supported("Abyssal Harvester");
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t, P1);
    let h = t.battlefield(P0, "Abyssal Harvester");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(card)]);
    crate::r_s06_common::activate_containing(&mut t, P0, h, "Create a token").unwrap();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    printed_card_copy(&t, "Abyssal Harvester", toks[0], (3, 3), red());
    assert!(has_subtype(&t, toks[0], "Nightmare") && has_subtype(&t, toks[0], "Giant"));
    // Ardyn's Starscourge: "At the beginning of combat on your turn, exile up to one
    // target creature card from a graveyard. If you exiled a card this way, create a
    // token that's a copy of that card, except it's a 5/5 black Demon."
    supported("Ardyn, the Usurper");
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t, P1);
    t.battlefield(P0, "Ardyn, the Usurper");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    printed_card_copy(&t, "Ardyn", toks[0], (5, 5), ColorSet::single(Color::Black));
    assert!(has_subtype(&t, toks[0], "Demon"));
    // Grist, the Plague Swarm's −6: "For each creature card in your graveyard, create a
    // token that's a copy of it, except it's a 1/1 black and green Insect."
    supported("Grist, Voracious Larva // Grist, the Plague Swarm");
    let mut t = TestGame::new(2);
    dead_dressed_giant(&mut t, P0);
    let g = t.battlefield(P0, "Grist, Voracious Larva // Grist, the Plague Swarm");
    mtg_engine::dfc::transform(&mut t.g, g);
    t.g.recompute();
    let g = t.g.current(g);
    t.g.add_counters(obj(g), counters::LOYALTY, 7, None);
    t.g.recompute();
    let before = t.g.battlefield.clone();
    crate::r_s06_common::activate_containing(&mut t, P0, g, "For each creature card").expect("−6");
    t.resolve_all();
    let toks: Vec<ObjectId> = new_tokens(&t, P0, &before)
        .into_iter()
        .filter(|id| name_of(&t, *id) == "Hill Giant")
        .collect();
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    printed_card_copy(
        &t,
        "Grist",
        toks[0],
        (1, 1),
        ColorSet::single(Color::Black).union(ColorSet::single(Color::Green)),
    );
    // Gyrus: "Whenever Gyrus attacks, you may exile target creature card with lesser
    // power from your graveyard. If you do, create a token that's a copy of that card and
    // that's tapped and attacking."
    supported("Gyrus, Waker of Corpses");
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t, P0);
    let gyrus = t.battlefield(P0, "Gyrus, Waker of Corpses");
    t.g.add_counters(obj(gyrus), counters::PLUS1, 5, None);
    t.g.recompute();
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    attack_p1(&mut t, &[gyrus]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    printed_card_copy(&t, "Gyrus", toks[0], (3, 3), red());
}

// --- Copiers with their own setups -------------------------------------------------------

#[test]
fn mirror_march_copies_the_printed_creature_or_what_it_copies() {
    cr!("707.2", "707.3", "608.2h", "603.2");
    ruling!(
        "Mirror March",
        "Each token copies exactly what is printed on the creature and nothing else (unless that creature is copying something else; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Mirror March",
        "If the copied creature is copying something else, each token enters the battlefield as whatever that creature is copying."
    );
    // "Whenever a nontoken creature you control enters, flip a coin until you lose a
    // flip. For each flip you won, create a token that's a copy of that creature. Those
    // tokens gain haste." One flip won. The Hill Giant is dressed up while the trigger
    // waits.
    supported("Mirror March");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror March");
    t.g.dice.loaded_coins.extend([true, false]);
    let hill = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_p021_common::dress(&mut t, P0, hill);
    assert_eq!(t.pt(hill), (10, 8));
    let before = t.g.battlefield.clone();
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_printed_copy(&t, "Mirror March", toks[0], &giant(), (3, 3), false);
    assert!(!t.obj_now(toks[0]).tapped && has_kw(&t, toks[0], KeywordKind::Haste));
    // A Clone enters as a copy of Serra Angel: the token is a Serra Angel.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror March");
    let angel = t.battlefield(P0, "Serra Angel");
    t.g.dice.loaded_coins.extend([true, false]);
    let before = t.g.battlefield.clone();
    crate::r_p023_common::clone_of(&mut t, P0, angel);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    crate::r_p023_common::assert_angel(&t, toks[0], true);
}

#[test]
fn mirror_style_master_tokens_copy_only_the_printed_creature() {
    cr!("707.2", "508.4", "700.9");
    ruling!(
        "Mirror-Style Master",
        "Each of the tokens copy exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on. (Basically, it won’t copy anything that made it modified in the first place. C’est la vie.)"
    );
    // "Whenever this creature attacks, for each attacking modified creature you control,
    // create a tapped and attacking token that's a copy of that creature." The dressed-up
    // Hill Giant (equipped, with counters) is modified; the tokens aren't.
    supported("Mirror-Style Master");
    let mut t = TestGame::new(2);
    let msm = t.battlefield(P0, "Mirror-Style Master");
    let hill = crate::r_p021_common::dressed_giant(&mut t, P0);
    crate::r_p021_common::untap(&mut t, hill);
    let before = t.g.battlefield.clone();
    attack_p1(&mut t, &[msm, hill]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_printed_copy(&t, "Mirror-Style Master", toks[0], &giant(), (3, 3), false);
    assert!(t.obj_now(toks[0]).tapped && t.g.is_attacking(toks[0]));
}

/// P0's Myr Enforcer (a colorless 4/4 Myr artifact creature), dressed up (see
/// `dressed_giant`).
fn dressed_enforcer(t: &mut TestGame) -> ObjectId {
    let e = t.battlefield(P0, "Myr Enforcer");
    crate::r_p021_common::dress(t, P0, e);
    assert_eq!(t.pt(e), (11, 9));
    e
}

#[test]
fn march_of_progress_copies_only_the_printed_artifact_creature() {
    cr!("707.2");
    ruling!(
        "March of Progress",
        "Each token copies exactly what was printed on the original artifact creature and nothing more (unless that creature is copying something else or is a token; see below. It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    supported("March of Progress");
    let mut t = TestGame::new(2);
    let e = dressed_enforcer(&mut t);
    let toks = cast_for_tokens(&mut t, P0, "March of Progress", "{2}{U}", &[obj(e)]);
    assert_eq!(toks.len(), 1);
    assert_printed_copy(&t, "March of Progress", toks[0], &enforcer(), (4, 4), false);
    assert!(!t.obj_now(toks[0]).tapped);
}

#[test]
fn copied_x_is_0() {
    cr!("107.3m", "202.3e", "707.2", "707.9b");
    ruling!(
        "March of Progress",
        "If the copied artifact creature has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Gyrus, Waker of Corpses",
        "If the copied creature card has {X} in its mana cost, X is considered to be 0."
    );
    // March of Progress copies a Hangarback Walker ({X}{X}, "This creature enters with X
    // +1/+1 counters on it.") cast with X = 2: the token's mana value is 0 and it enters
    // with no counters (Glorious Anthem keeps it alive as a 1/1).
    supported("Hangarback Walker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.lands(P0, "Wastes", 4);
    let h = t.hand(P0, "Hangarback Walker");
    t.cast(P0, h).x(2).go();
    t.resolve_all();
    let walker = t.named_on_battlefield("Hangarback Walker")[0];
    assert_eq!(t.counters(walker, counters::PLUS1), 2);
    let toks = cast_for_tokens(&mut t, P0, "March of Progress", "{2}{U}", &[obj(walker)]);
    assert_eq!(toks.len(), 1);
    assert_eq!(crate::r_s26_common::mv(&mut t, toks[0]), 0);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
    assert_eq!(t.pt(toks[0]), (1, 1));
    // Gyrus and Soul Separator copy a Benevolent Hydra card ({X}{G}{G}, "This creature
    // enters with X +1/+1 counters on it."): mana value 2, no counters.
    let hydra_token = |gyrus: bool| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Glorious Anthem");
        let card = t.graveyard(P0, "Benevolent Hydra");
        let before = t.g.battlefield.clone();
        if gyrus {
            let g = t.battlefield(P0, "Gyrus, Waker of Corpses");
            t.g.add_counters(obj(g), counters::PLUS1, 3, None);
            t.g.recompute();
            t.answer_targets(P0, &[obj(card)]);
            t.answer_yes(P0, true);
            attack_p1(&mut t, &[g]);
        } else {
            let sep = t.battlefield(P0, "Soul Separator");
            lands_for(&mut t, P0, "{5}");
            t.answer_targets(P0, &[obj(card)]);
            crate::r_s06_common::activate_containing(&mut t, P0, sep, "Exile target").unwrap();
            t.resolve_all();
        }
        let tok = new_tokens(&t, P0, &before)
            .into_iter()
            .find(|id| name_of(&t, *id) == "Benevolent Hydra")
            .expect("a token copy of the Hydra");
        assert_eq!(crate::r_s26_common::mv(&mut t, tok), 2);
        assert_eq!(t.counters(tok, counters::PLUS1), 0);
        tok
    };
    ruling!(
        "Soul Separator",
        "If the copied creature card has {X} in its mana cost, X is considered to be 0."
    );
    supported("Gyrus, Waker of Corpses");
    supported("Soul Separator");
    hydra_token(true);
    hydra_token(false);
}

#[test]
fn auton_soldier_myriad_tokens_copy_what_it_copies_not_its_state() {
    cr!("702.116a", "707.2", "707.9b", "707.3");
    ruling!(
        "Auton Soldier",
        "Each token created by the myriad ability will be a copy of whatever Auton Soldier is copying, along with the listed exceptions. However, it doesn't copy whether Auton Soldier is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    // Auton Soldier enters as a copy of a Hill Giant (except it isn't legendary, is an
    // artifact, and has myriad), is dressed up, and attacks P1 of three players: the
    // myriad token attacking P2 is an artifact Hill Giant with myriad, as printed.
    supported("Auton Soldier");
    let mut t = TestGame::new(3);
    let hill = t.battlefield(P1, "Hill Giant");
    let a = copy_with(&mut t, "Auton Soldier", hill)[0];
    crate::r_p023_common::unsick(&mut t, a);
    crate::r_p021_common::dress(&mut t, P0, a);
    crate::r_p021_common::untap(&mut t, a);
    assert_eq!(t.pt(a), (10, 8));
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    attack_p1(&mut t, &[a]);
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_printed_copy(&t, "Auton Soldier", toks[0], &giant(), (3, 3), false);
    assert!(t.obj_now(toks[0]).is(CardType::Artifact));
    assert!(has_kw(&t, toks[0], KeywordKind::Myriad));
}

/// P0's Curie, Emergent Intelligence ("{1}{U}, Exile another nontoken artifact creature
/// you control: Curie becomes a copy of the exiled creature, except it has "Whenever this
/// creature deals combat damage to a player, draw cards equal to its base power."")
/// exiles `what`.
fn curie_exiles(t: &mut TestGame, what: ObjectId) -> ObjectId {
    supported("Curie, Emergent Intelligence");
    let curie = t.battlefield(P0, "Curie, Emergent Intelligence");
    lands_for(t, P0, "{1}{U}");
    t.answer_choose(P0, &[obj(what)]);
    crate::r_s06_common::activate_containing(t, P0, curie, "Exile another")
        .expect("Curie's ability");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(what)), Zone::Exile);
    curie
}

#[test]
fn curie_copies_the_exiled_creature_not_its_state() {
    cr!("707.2", "707.9a", "608.2h");
    ruling!(
        "Curie, Emergent Intelligence",
        "Curie's last ability won't copy whether the exiled creature was tapped or untapped, whether it had any counters on it or Auras attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    let mut t = TestGame::new(2);
    let e = dressed_enforcer(&mut t);
    let curie = curie_exiles(&mut t, e);
    assert_printed_copy(&t, "Curie", curie, &enforcer(), (4, 4), false);
    assert!(!t.obj_now(curie).tapped);
    assert_eq!(
        abilities_with(&t, curie, "draw cards equal to its base power"),
        1
    );
}

#[test]
fn curie_copies_what_the_exiled_creature_was_copying() {
    cr!("707.2", "707.3", "608.2h", "707.9a");
    ruling!(
        "Curie, Emergent Intelligence",
        "Curie's last ability refers to the exiled creature, not the exiled card. In most cases, this means Curie copies exactly what was printed on the original creature and nothing else, plus the additional ability. However, if the exiled creature was copying something else when it was last on the battlefield, Curie copies whatever that creature was copying, plus the additional ability."
    );
    // Phyrexian Metamorph ("You may have this creature enter as a copy of an artifact or
    // creature on the battlefield, except it's an artifact in addition to its other
    // types.") entered as a copy of Serra Angel: Curie exiles it and becomes an artifact
    // Serra Angel, not a Phyrexian Metamorph.
    supported("Phyrexian Metamorph");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    let m = crate::r_p023_common::enter_copying(&mut t, P0, "Phyrexian Metamorph", angel);
    assert_eq!(name_of(&t, m), "Serra Angel");
    let curie = curie_exiles(&mut t, m);
    crate::r_p023_common::assert_angel(&t, curie, true);
    assert!(t.obj_now(curie).is(CardType::Artifact));
    assert_eq!(
        abilities_with(&t, curie, "draw cards equal to its base power"),
        1
    );
    // The card in exile is a Phyrexian Metamorph.
    assert_eq!(name_of(&t, t.g.current(m)), "Phyrexian Metamorph");
}

#[test]
fn curie_gone_draws_its_last_base_power() {
    cr!("608.2h", "113.7a", "208.4");
    ruling!(
        "Curie, Emergent Intelligence",
        "If Curie is no longer on the battlefield when its first ability resolves, use its base power as it last existed on the battlefield to determine how many cards to draw."
    );
    // Curie, a copy of Myr Enforcer (base power 4) with "Whenever this creature deals
    // combat damage to a player, draw cards equal to its base power.", gets +3/+3 and
    // deals combat damage; it's destroyed with the trigger waiting: P0 draws 4 (not 1,
    // Curie's own printed power, nor 7).
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Myr Enforcer");
    let curie = curie_exiles(&mut t, e);
    crate::r_p023_common::unsick(&mut t, curie);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        curie,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(3),
            mtg_engine::ability::Value::c(3),
        )],
    );
    assert_eq!(t.pt(curie), (7, 7));
    for _ in 0..10 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    crate::r_s01_common::attack_with(&mut t, &[(curie, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, curie);
    assert!(!t.on_battlefield(curie));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}
