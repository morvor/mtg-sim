//! roundtrip-tail-4 (docs/agents/items/roundtrip-tail-4.md): parser bugs the round trip
//! found in its clusters, and cards whose rendering differs in wording, checked in play.

use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::types::CardType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn discerning_taste_counts_the_cards_put_into_the_graveyard_not_the_one_taken() {
    cr!("608.2c", "400.7");
    // "Put one of them into your hand and the rest into your graveyard. You gain life
    // equal to the greatest power among creature cards put into your graveyard this way."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Grizzly Bears");
    let wurm = t.library_top(P0, "Craw Wurm");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Discerning Taste");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The Bears' power (2), not the Wurm's (6).
    assert_eq!(t.life(P0), 22, "{}", t.dump_log());
}

#[test]
fn milled_cards_exiled_instead_were_not_put_into_the_graveyard_this_way() {
    cr!("608.2c", "614.1a", "701.17c");
    // Liliana's Indignation: "Mill X cards. Target player loses 2 life for each creature
    // card put into your graveyard this way." With Rest in Peace, the milled cards are
    // exiled instead: none was put into the graveyard.
    let mut t = TestGame::new(2);
    t.enter(P0, "Rest in Peace");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, spell).x(2).target(P1).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
}

#[test]
fn milled_cards_put_into_the_graveyard_still_count() {
    cr!("608.2c");
    // (The same without Rest in Peace: two creature cards, 4 life.)
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Liliana's Indignation");
    t.cast(P0, spell).x(2).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
}

#[test]
fn coerced_confession_draws_nothing_for_cards_exiled_instead() {
    cr!("608.2c", "614.1a");
    // "Target player mills four cards. You draw a card for each creature card put into
    // their graveyard this way." Leyline of the Void exiles the opponent's milled cards.
    let mut t = TestGame::new(2);
    t.enter(P0, "Leyline of the Void");
    for _ in 0..4 {
        t.library_top(P1, "Grizzly Bears");
    }
    for _ in 0..4 {
        t.library_top(P0, "Forest");
    }
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Coerced Confession");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 0, "{}", t.dump_log());
}

#[test]
fn king_narfis_betrayal_exiles_a_card_from_each_graveyard() {
    cr!("714.2b", "608.2c");
    // "I — Each player mills four cards. Then you may exile a creature or planeswalker card
    // from each graveyard."
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        for _ in 0..4 {
            t.library_top(p, "Forest");
        }
    }
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.enter(P0, "King Narfi's Betrayal");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
    // II: the exiled cards can be cast (they were exiled with the Saga).
    let saga = t.named_on_battlefield("King Narfi's Betrayal")[0];
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    t.lands(P0, "Mountain", 4);
    let giant_now = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Exile && o.chars.name == "Hill Giant")
        .map(|o| o.id)
        .expect("exiled Hill Giant");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P0, giant_now).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1, "{}", t.dump_log());
}

#[test]
fn song_of_blood_counts_creature_cards_milled_into_the_graveyard() {
    cr!("603.7a", "608.2c");
    // "Mill four cards. Whenever a creature attacks this turn, it gets +1/+0 until end of
    // turn for each creature card put into your graveyard this way."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let spell = t.hand(P0, "Song of Blood");
    t.cast(P0, spell).go();
    t.resolve_all();
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2), "{}", t.dump_log());
}

#[test]
fn verdant_command_counters_only_a_loyalty_ability_of_a_planeswalker() {
    cr!("606.2", "115.1");
    // "Counter target loyalty ability of a planeswalker.": other permanents can have
    // loyalty abilities (CR 606.2); the source must be a planeswalker.
    for still_planeswalker in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
        let w = t.battlefield(P0, "Ajani Goldmane");
        let vc = t.hand(P1, "Verdant Command");
        t.lands(P1, "Forest", 2);
        t.activate(P0, w, 0, &[]).expect("+1");
        let ability = *t.g.stack.last().expect("on the stack");
        if !still_planeswalker {
            let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
            ctx.targets = vec![vec![Entity::Object(w)]];
            let e = Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(vec![CardType::Planeswalker])],
                duration: Duration::EndOfTurn,
            };
            t.g.exec(&e, &mut ctx);
            t.g.recompute();
        }
        let _ = t
            .cast(P1, vc)
            .modes(&[1, 3])
            .target(Entity::Object(ability))
            .target(P1)
            .try_go();
        t.resolve_all();
        // Ajani's +1: "You gain 2 life" (countered only while Ajani is a planeswalker).
        let expected = if still_planeswalker { 20 } else { 22 };
        assert_eq!(t.life(P0), expected, "{still_planeswalker}\n{}", t.dump_log());
    }
}
