//! roundtrip-tail-4 (docs/agents/items/roundtrip-tail-4.md): parser bugs the round trip
//! found in its clusters, and cards whose rendering differs in wording, checked in play.

use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::types::{CardType, TypeLine};
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

#[test]
fn typhoon_deals_each_opponent_damage_for_their_own_islands() {
    cr!("120.3", "608.2c");
    // "~ deals damage to each opponent equal to the number of Islands that player controls."
    let mut t = TestGame::new(3);
    t.lands(P1, "Island", 2);
    t.lands(P2, "Island", 1);
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Typhoon");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2)),
        (20, 18, 19),
        "{}",
        t.dump_log()
    );
}

#[test]
fn rite_of_flame_counts_the_cards_in_every_graveyard() {
    cr!("106.1", "608.2c");
    // "Add {R}{R}, then add {R} for each card named Rite of Flame in each graveyard."
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Rite of Flame");
    t.graveyard(P1, "Rite of Flame");
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Rite of Flame");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 4, "{}", t.dump_log());
}

#[test]
fn natural_balance_trims_and_fills_each_players_lands() {
    cr!("101.4", "608.2c");
    // "Each player who controls six or more lands chooses five lands they control and
    // sacrifices the rest. Each player who controls four or fewer lands may search their
    // library for up to X basic land cards and put them onto the battlefield, where X is
    // five minus the number of lands they control. Then each player who searched their
    // library this way shuffles."
    let mut t = TestGame::new(2);
    t.g.search_finds_by_default = true;
    t.lands(P0, "Forest", 7);
    t.lands(P1, "Plains", 3);
    for _ in 0..4 {
        t.library_top(P1, "Plains");
    }
    t.answer_yes(P1, true);
    let library = t.library_size(P1);
    let spell = t.hand(P0, "Natural Balance");
    t.cast(P0, spell).go();
    t.resolve_all();
    let lands = |t: &TestGame, p| {
        t.g.permanents_controlled_by(p)
            .iter()
            .filter(|o| t.g.obj(**o).chars.card_types.contains(CardType::Land))
            .count()
    };
    assert_eq!(lands(&t, P0), 5, "{}", t.dump_log());
    assert_eq!(lands(&t, P1), 5, "{}", t.dump_log());
    assert_eq!(t.library_size(P1), library - 2);
}

#[test]
fn treasure_keeper_puts_the_card_not_cast_on_the_bottom_too() {
    cr!("608.2c");
    // "You may cast that card without paying its mana cost. Put all revealed cards not
    // cast this way on the bottom of your library in a random order."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    let keeper = t.battlefield(P0, "Treasure Keeper");
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.answer_yes(P0, false);
    t.cast(P0, murder).target(keeper).go();
    t.resolve_all();
    let lib: Vec<String> = t.g.players[P0.idx()]
        .library
        .iter()
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    // Bottom first: the Bears (not cast) and the Forest.
    let mut bottom = lib[..2].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Grizzly Bears"], "{lib:?}\n{}", t.dump_log());
}

#[test]
fn king_narfis_betrayal_exiles_from_the_graveyards_that_have_such_a_card() {
    cr!("714.2b", "608.2c");
    // "Exile a creature or planeswalker card from each graveyard": a graveyard with no
    // such card gives none, the others still give one each.
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        for _ in 0..4 {
            t.library_top(p, "Forest");
        }
    }
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P2, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.enter(P0, "King Narfi's Betrayal");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.graveyard_size(P1), 4, "{}", t.dump_log());
}

#[test]
fn soul_shatter_each_opponent_sacrifices_among_their_own_permanents() {
    cr!("608.2c", "701.21a");
    // "Each opponent sacrifices a creature or planeswalker with the greatest mana value
    // among creatures and planeswalkers they control."
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    t.battlefield(P2, "Llanowar Elves");
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Soul Shatter");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "{}", t.dump_log());
    assert!(t.in_graveyard(P2, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn cards_a_dig_put_into_the_graveyard_are_not_counted_after_another_instruction() {
    cr!("608.2c");
    // A dig's "the rest" is what "put into a graveyard this way" counts right after it
    // (Discerning Taste), not after another instruction: here only the destroyed
    // artifact.
    use mtg_engine::object::Characteristics;
    use mtg_engine::oracle::{self, CompileContext};
    let text = "Look at the top two cards of your library. Put one of them into your hand and \
                the rest into your graveyard. Then destroy all artifacts. You gain 3 life for \
                each card put into a graveyard this way.";
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: "Test Dig Then Destroy",
        full_name: "Test Dig Then Destroy",
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    let def = mtg_engine::card::CardDef::custom(Characteristics {
        name: "Test Dig Then Destroy".into(),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        ..Default::default()
    });
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sol Ring");
    // Looked at: the Wurm (taken) and the Giant (the rest).
    t.library_top(P0, "Hill Giant");
    let wurm = t.library_top(P0, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Sol Ring"), "{}", t.dump_log());
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
}
