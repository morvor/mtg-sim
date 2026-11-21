//! Rulings batch P026 — tokens that are copies of cards in graveyards or exile copy what's
//! printed on the card (CR 707.2) and nothing about the permanent it was (CR 400.7); a
//! copy of a creature that died uses its last known information (CR 608.2h); values an
//! effect sets from a card are locked in as the token is created (CR 608.2h); and the
//! delayed triggers that exile or sacrifice such tokens track the token (CR 603.7).

use crate::r_p026_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's dressed-up Hill Giant, a copy of Grizzly Bears until end of turn, dies: returns
/// its card in P0's graveyard. A copy of the card is a printed Hill Giant, not a Grizzly
/// Bears or anything else the permanent was.
fn dead_dressed_giant(t: &mut TestGame) -> ObjectId {
    let giant = dressed_giant(t, P0);
    disguise_as_bears(t, giant);
    kill(t, giant);
    let card = t.g.current(giant);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    card
}

#[test]
fn tokens_copied_from_graveyard_cards_copy_only_the_printed_card() {
    cr!("707.2", "400.7", "707.9b");
    ruling!(
        "Feldon of the Third Path",
        "The token copies exactly what was printed on the original creature card (except that the copy is also an artifact) and nothing else."
    );
    ruling!(
        "Mordor on the March",
        "The token copies exactly what was printed on the original creature card and nothing else."
    );
    ruling!(
        "Offspring's Revenge",
        "The token copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies. It doesn’t copy any information about the object the card was before it was put into your graveyard."
    );
    // Feldon: "{2}{R}, {T}: Create a token that's a copy of target creature card in your
    // graveyard, except it's an artifact in addition to its other types."
    supported("Feldon of the Third Path");
    let mut t = TestGame::new(2);
    let feldon = t.battlefield(P0, "Feldon of the Third Path");
    let card = dead_dressed_giant(&mut t);
    lands_for(&mut t, P0, "{2}{R}");
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        feldon,
        "Create a token",
        &[obj(card)],
    ));
    assert_printed_giant(&t, tok, Some((3, 3)), true);
    assert!(t.obj_now(tok).is(CardType::Artifact));
    // Mordor on the March: "Exile a creature card from your graveyard. Create a token
    // that's a copy of it."
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t);
    t.answer_choose(P0, &[obj(card)]);
    let tok = one(cast_for_tokens(
        &mut t,
        P0,
        "Mordor on the March",
        "{3}{B}{R}",
        &[],
        None,
        None,
    ));
    assert_printed_giant(&t, tok, Some((3, 3)), true);
    // Offspring's Revenge: "...exile target red, white, or black creature card from your
    // graveyard. Create a token that's a copy of that card, except it's 1/1."
    supported("Offspring's Revenge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Offspring's Revenge");
    let card = dead_dressed_giant(&mut t);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_printed_giant(&t, tok, Some((1, 1)), true);
}

#[test]
fn dino_dna_token_copies_only_the_printed_card_with_its_modifications() {
    cr!("707.2", "707.9b", "607.2a");
    ruling!(
        "Dino DNA",
        "The token copies exactly what was printed on the original card and nothing else, except the characteristics it specifically modifies."
    );
    // "Imprint — {1}, {T}: Exile target creature card from a graveyard. / {6}: Create a
    // token that's a copy of target creature card exiled with this artifact, except it's a
    // 6/6 green Dinosaur creature with trample."
    supported("Dino DNA");
    let mut t = TestGame::new(2);
    let dna = t.battlefield(P0, "Dino DNA");
    let card = dead_dressed_giant(&mut t);
    lands_for(&mut t, P0, "{1}");
    t.activate(P0, dna, 0, &[obj(card)]).expect("imprint");
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.g.untap(dna);
    lands_for(&mut t, P0, "{6}");
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        dna,
        "Create a token",
        &[obj(exiled)],
    ));
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Hill Giant");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(has_subtype(&t, tok, "Dinosaur") && !has_subtype(&t, tok, "Giant"));
    assert!(has_kw(&t, tok, KeywordKind::Trample));
    assert_eq!(t.pt(tok), (6, 6));
    assert_eq!(crate::r_s26_common::mv(&mut t, tok), 4);
}

#[test]
fn mardu_siegebreaker_tokens_copy_only_the_printed_exiled_card() {
    cr!("707.2", "400.7", "508.4");
    ruling!(
        "Mardu Siegebreaker",
        "The token copies what’s printed on the exiled card and nothing else. It doesn’t copy any effects that changed its characteristics before it was exiled."
    );
    // "When this creature enters, exile up to one other target creature you control until
    // this creature leaves the battlefield. / Whenever this creature attacks, for each
    // opponent, create a tapped token that's a copy of the exiled card attacking that
    // opponent."
    supported("Mardu Siegebreaker");
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P0);
    // It's a copy of Grizzly Bears as it's exiled: the token is still a Hill Giant.
    disguise_as_bears(&mut t, giant);
    t.answer_targets(P0, &[obj(giant)]);
    let sb = t.enter(P0, "Mardu Siegebreaker");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    let sb = t.g.current(sb);
    t.set_step(P0, Step::BeginningOfCombat);
    let before = t.g.battlefield.clone();
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(sb, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_printed_giant(&t, tok, Some((3, 3)), false);
    assert!(t.obj_now(tok).tapped && t.g.is_attacking(tok));
}

#[test]
fn foggy_swamp_visions_tokens_copy_only_the_printed_cards() {
    cr!("707.2", "400.7");
    ruling!(
        "Foggy Swamp Visions",
        "The tokens copy exactly what was printed on the original cards and nothing else."
    );
    // "As an additional cost to cast this spell, waterbend {X}. Exile X target creature
    // cards from graveyards. For each creature card exiled this way, create a token that's
    // a copy of it." X = 1.
    let mut t = TestGame::new(2);
    let card = dead_dressed_giant(&mut t);
    let tok = one(cast_for_tokens(
        &mut t,
        P0,
        "Foggy Swamp Visions",
        "{2}{B}{B}",
        &[obj(card)],
        None,
        Some(1),
    ));
    assert_printed_giant(&t, tok, Some((3, 3)), true);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn hofri_copies_the_creature_as_it_last_existed_on_the_battlefield() {
    cr!("608.2h", "707.2", "707.3");
    ruling!(
        "Hofri Ghostforge",
        "The copy uses the copiable values of the creature as it last existed on the battlefield before it died, not as it existed in the graveyard before it was exiled."
    );
    // P0's Clone copying Serra Angel dies: the token is a Serra Angel (and a Spirit), not a
    // Clone.
    supported("Hofri Ghostforge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hofri Ghostforge");
    let clone = cloned_angel(&mut t, P0);
    let before = t.g.battlefield.clone();
    kill(&mut t, clone);
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Serra Angel");
    assert!(has_subtype(&t, tok, "Angel") && has_subtype(&t, tok, "Spirit"));
    assert!(has_kw(&t, tok, KeywordKind::Flying));
    // Hofri gives Spirits +1/+1.
    assert_eq!(t.pt(tok), (5, 5));
}

/// P0's Mimic Vat with a Hill Giant card exiled with it.
fn vat_with_giant(t: &mut TestGame) -> ObjectId {
    supported("Mimic Vat");
    let vat = t.battlefield(P0, "Mimic Vat");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    kill(t, giant);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    vat
}

#[test]
fn mimic_vat_copies_the_card_exiled_with_it_as_the_ability_resolves() {
    cr!("608.2h", "607.2a", "707.2");
    ruling!(
        "Mimic Vat",
        "The token created by the second ability will be a copy of whatever card is exiled with Mimic Vat at the time the ability resolves. This might not be the same card that was exiled with Mimic Vat at the time the ability was activated."
    );
    let mut t = TestGame::new(2);
    let vat = vat_with_giant(&mut t);
    lands_for(&mut t, P0, "{3}");
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, vat, "Create a token").expect("activate");
    // In response, Serra Angel dies and is exiled with the Vat instead.
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_yes(P0, true);
    kill(&mut t, angel);
    t.resolve();
    assert!(t.in_exile("Serra Angel") && t.in_graveyard(P1, "Hill Giant"));
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Serra Angel");
}

#[test]
fn mimic_vat_and_flameshadow_tokens_are_exiled_whoever_controls_them() {
    cr!("603.7c", "603.7");
    ruling!(
        "Mimic Vat",
        "The token is exiled at the beginning of the next end step regardless of who controls it at that time, whether the exiled card is still exiled at that time, or whether Mimic Vat is still on the battlefield at that time."
    );
    ruling!(
        "Flameshadow Conjuring",
        "The token is exiled at the beginning of the next end step regardless of who controls it at that time."
    );
    // Mimic Vat: the token changes control, the Vat is destroyed and the card returns to
    // the graveyard (another creature card is exiled with the Vat first).
    let mut t = TestGame::new(2);
    let vat = vat_with_giant(&mut t);
    lands_for(&mut t, P0, "{3}");
    let tok = one(activate_for_tokens(&mut t, P0, vat, "Create a token", &[]));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    kill(&mut t, vat);
    give_control(&mut t, tok, P1);
    assert_eq!(t.obj_now(tok).controller, P1);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
    // Flameshadow Conjuring's token, controlled by the opponent at the end step.
    supported("Flameshadow Conjuring");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Flameshadow Conjuring");
    lands_for(&mut t, P0, "{R}");
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.enter(P0, "Hill Giant");
    t.settle();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    give_control(&mut t, tok, P1);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(tok));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn soul_separator_zombie_size_is_locked_in_from_the_graveyard_card() {
    cr!("608.2h", "604.3");
    ruling!(
        "Soul Separator",
        "The power and toughness of the Zombie token are set as it’s created based on the card as it last existed in your graveyard. They won’t update if the exiled card’s power and toughness change."
    );
    // "{5}, {T}, Sacrifice this artifact: Exile target creature card from your graveyard.
    // Create a token that's a copy of that card, except it's 1/1, it's a Spirit in addition
    // to its other types, and it has flying. Create a black Zombie creature token with
    // power equal to that card's power and toughness equal to that card's toughness."
    // Tarmogoyf in the graveyard, with Soul Separator (an artifact) there too, is a 2/3;
    // once exiled it's a 1/2.
    supported("Soul Separator");
    let mut t = TestGame::new(2);
    let sep = t.battlefield(P0, "Soul Separator");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    lands_for(&mut t, P0, "{5}");
    let toks = activate_for_tokens(&mut t, P0, sep, "Exile target", &[obj(goyf)]);
    assert_eq!(toks.len(), 2);
    let zombie = one(toks
        .iter()
        .copied()
        .filter(|id| has_subtype(&t, *id, "Zombie"))
        .collect());
    let spirit = one(tokens_named(&t, "Tarmogoyf"));
    assert_eq!(t.pt(spirit), (1, 1));
    let exiled = t.g.current(goyf);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.pt(exiled), (1, 2));
    assert_eq!(t.pt(zombie), (2, 3));
    // An instant goes to a graveyard: the exiled Tarmogoyf grows, the Zombie doesn't.
    t.graveyard(P1, "Shock");
    t.g.recompute();
    assert_eq!(t.pt(exiled), (2, 3));
    assert_eq!(t.pt(zombie), (2, 3));
}

#[test]
fn dollhouse_of_horrors_token_counts_itself() {
    cr!("707.9b", "613.4c");
    ruling!(
        "Dollhouse of Horrors",
        "The token creature counts itself when counting Constructs you control."
    );
    // "{1}, {T}, Exile a creature card from your graveyard: Create a token that's a copy
    // of the exiled card, except it's a 0/0 Construct artifact in addition to its other
    // types and it has "This token gets +1/+1 for each Construct you control.""
    supported("Dollhouse of Horrors");
    let mut t = TestGame::new(2);
    let house = t.battlefield(P0, "Dollhouse of Horrors");
    let card = t.graveyard(P0, "Hill Giant");
    lands_for(&mut t, P0, "{1}");
    t.answer_choose(P0, &[obj(card)]);
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        house,
        "Create a token",
        &[],
    ));
    assert!(has_subtype(&t, tok, "Construct"));
    assert_eq!(t.pt(tok), (1, 1));
}

#[test]
fn nahiri_checks_her_loyalty_as_the_ability_resolves() {
    cr!("608.2b");
    ruling!(
        "Nahiri, the Unforgiving",
        "The last loyalty ability will check Nahiri's loyalty as the ability tries to resolve to see if the target is still legal. If it's not, the ability won't resolve and none of its effects will happen. The card won't be exiled and no token will be created."
    );
    // "0: Exile target creature or Equipment card with mana value less than Nahiri's
    // loyalty from your graveyard. Create a token that's a copy of it." Hill Giant (mana
    // value 4) with Nahiri at 5 loyalty; Shock deals 2 damage to her in response.
    supported("Nahiri, the Unforgiving");
    let mut t = TestGame::new(2);
    let nahiri = t.battlefield(P0, "Nahiri, the Unforgiving");
    assert_eq!(t.counters(nahiri, counters::LOYALTY), 5);
    let card = t.graveyard(P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(card)]);
    activate_containing(&mut t, P0, nahiri, "Exile target").expect("activate");
    lands_for(&mut t, P1, "{R}");
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(obj(nahiri)).go();
    t.resolve();
    assert_eq!(t.counters(nahiri, counters::LOYALTY), 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(new_tokens(&t, P0, &before).is_empty());
}

#[test]
fn the_scarab_god_counts_zombies_as_its_upkeep_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "The Scarab God",
        "The number of Zombies you control is counted as The Scarab God's first ability resolves. Players can try to change that number in response to the ability (perhaps by activating its second ability)."
    );
    // "{2}{U}{B}: Exile target creature card from a graveyard. Create a token that's a copy
    // of it, except it's a 4/4 black Zombie." No Zombies as the upkeep trigger triggers.
    supported("The Scarab God");
    let mut t = TestGame::new(2);
    let god = t.battlefield(P0, "The Scarab God");
    let card = t.graveyard(P1, "Hill Giant");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    lands_for(&mut t, P0, "{2}{U}{B}");
    t.answer_targets(P0, &[obj(card)]);
    activate_containing(&mut t, P0, god, "Exile target").expect("activate");
    t.resolve();
    assert_eq!(tokens_named(&t, "Hill Giant").len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn the_mimeoplasms_cards_may_come_from_the_same_graveyard() {
    cr!("614.1c");
    ruling!(
        "The Mimeoplasm",
        "The two creature cards may come from the same graveyard or different graveyards."
    );
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P1, "Craw Wurm");
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm), obj(giant)]);
    t.answer_choose_moved(P0, &[obj(giant)]);
    let m = t.enter(P0, "The Mimeoplasm");
    t.resolve_all();
    assert!(t.in_exile("Craw Wurm") && t.in_exile("Hill Giant"));
    assert_eq!(name_of(&t, m), "Hill Giant");
    // Craw Wurm's power is 6.
    assert_eq!(t.counters(m, counters::PLUS1), 6);
}

#[test]
fn tawnos_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "111.10a");
    ruling!(
        "Tawnos, Solemn Survivor",
        "The token you create with the first ability copies the original characteristics of the token it’s copying as stated by the effect that created that token. The newly created token doesn’t copy whether the original token is tapped or untapped, whether it has any counters on it, whether it has any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    // "{2}, {T}: Create a token that's a copy of up to one target artifact token you
    // control. Mill two cards." The Treasure is tapped, has a +1/+1 counter, and is an
    // animated red 4/4 artifact creature until end of turn.
    supported("Tawnos, Solemn Survivor");
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Solemn Survivor");
    let treasure = crate::r_p023_common::token_of(
        &mut t,
        P0,
        mtg_engine::tokens_predefined::predefined("Treasure").unwrap(),
    );
    modify(
        &mut t,
        treasure,
        vec![
            mtg_engine::ability::Modification::AddTypes(vec![CardType::Creature]),
            mtg_engine::ability::Modification::SetPT(
                Some(mtg_engine::ability::Value::c(4)),
                Some(mtg_engine::ability::Value::c(4)),
            ),
            mtg_engine::ability::Modification::SetColors(red()),
        ],
    );
    plus1(&mut t, treasure, 1);
    t.g.tap(t.g.current(treasure));
    assert!(is_creature(&t, treasure));
    lands_for(&mut t, P0, "{2}");
    let tok = one(activate_for_tokens(
        &mut t,
        P0,
        tawnos,
        "{2}, {T}",
        &[obj(treasure)],
    ));
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, t.obj_now(treasure).chars.name);
    assert!(has_subtype(&t, tok, "Treasure"));
    assert!(o.is(CardType::Artifact) && !is_creature(&t, tok));
    assert!(o.chars.colors.is_colorless());
    assert!(fresh(&t, tok));
}
