//! cards5-referents-statics, family `copies`: copies of cards (CR 707.12).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn god_eternal_kefnet_copies_the_revealed_sorcery_for_two_less() {
    cr!("707.12", "601.2f");
    supported("God-Eternal Kefnet");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    t.battlefield(P0, "God-Eternal Kefnet");
    let island = t.lands(P0, "Island", 1)[0];
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.library_top(P0, "Divination");
    // Reveal it; cast the copy (for {U}: one Island is enough).
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    // The Divination drawn, and the copy's two cards.
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(t.in_hand(P0, "Divination"));
    assert!(t.obj_now(island).tapped);
}

#[test]
fn uvilda_exiled_card_is_cast_for_four_less_when_the_last_refine_counter_leaves() {
    cr!("601.2f", "608.2g");
    supported("Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let uvilda = t.battlefield(P0, "Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    t.g.objects[uvilda.0 as usize].summoning_sick = false;
    let div = t.hand(P0, "Divination");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(div)]);
    t.activate(P0, uvilda, 0, &[]).expect("activates");
    t.resolve_all();
    let exiled = t.g.current(div);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    assert_eq!(t.counters(exiled, "refine"), 3);
    // The last refine counter goes: cast it for {U} ({2}{U} less {4} generic).
    let island = t.lands(P0, "Island", 1)[0];
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    t.g.remove_counters(Entity::Object(exiled), "refine", 3);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.obj_now(island).tapped);
}

#[test]
fn caller_of_the_untamed_x_is_the_mana_value_of_the_copied_card() {
    cr!("107.3c", "707.12");
    interp!("x-defined-by-target");
    supported("Caller of the Untamed");
    use mtg_engine::decision::Answer;
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let caller = t.battlefield(P0, "Caller of the Untamed");
    t.g.objects[caller.0 as usize].summoning_sick = false;
    // Exiled with a card named Caller of the Untamed before the game: Hill Giant (4) and
    // Grizzly Bears (2).
    for name in ["Hill Giant", "Grizzly Bears"] {
        let c = t.exile(P0, name);
        t.g.named_exiles
            .push((P0, "Caller of the Untamed".into(), c));
    }
    t.lands(P0, "Forest", 4);
    // X = 3: no such card, the ability can't be activated for it.
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let r = t.activate(P0, caller, 0, &[]);
    let x3 = r.ok().flatten().and_then(|a| t.g.obj(a).stack.as_ref().and_then(|s| s.x));
    assert!(x3.is_none() || x3 == Some(2) || x3 == Some(4), "{x3:?}");
    t.resolve_all();
    // X = 2: a copy of the Bears.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let caller = t.battlefield(P0, "Caller of the Untamed");
    t.g.objects[caller.0 as usize].summoning_sick = false;
    for name in ["Hill Giant", "Grizzly Bears"] {
        let c = t.exile(P0, name);
        t.g.named_exiles
            .push((P0, "Caller of the Untamed".into(), c));
    }
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, caller, 0, &[]).expect("activates");
    t.resolve_all();
    let bears: Vec<ObjectId> = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t.obj_now(bears[0]).is_token());
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn spellweaver_helix_copies_the_other_imprinted_card() {
    cr!("707.12", "607.2a");
    supported("Spellweaver Helix");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    let spike = t.graveyard(P1, "Lava Spike");
    t.answer_targets(P0, &[Entity::Object(div), Entity::Object(spike)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    assert!(t.in_exile("Divination") && t.in_exile("Lava Spike"));
    // Casting a Divination: copy the Lava Spike and cast it at P1.
    t.lands(P0, "Island", 3);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let mine = t.hand(P0, "Divination");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, mine).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Lava Spike"));
}

#[test]
fn hofri_ghostforge_token_returns_the_exiled_card_when_it_leaves() {
    cr!("707.9", "607.2a", "400.7");
    supported("Hofri Ghostforge");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Hofri Ghostforge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let token: Vec<ObjectId> = t
        .named_on_battlefield("Grizzly Bears")
        .into_iter()
        .filter(|o| t.obj_now(*o).is_token())
        .collect();
    assert_eq!(token.len(), 1);
    assert!(t.obj_now(token[0]).chars.has_subtype("Spirit"));
    // Hofri's Spirits get +1/+1.
    assert_eq!(t.pt(token[0]), (3, 3));
    t.g.destroy(token[0], None);
    t.settle();
    t.resolve_all();
    assert!(!t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn ludevic_transforms_into_a_copy_with_a_counter_per_exiled_creature_card() {
    cr!("712.20", "707.9b", "122.6");
    supported("Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    use mtg_engine::decision::Answer;
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let ludevic = t.battlefield(P0, "Ludevic, Necrogenius // Olag, Ludevic's Hubris");
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 4);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(giant), Entity::Object(bears)]);
    t.answer_choose_moved(P0, &[Entity::Object(giant)]);
    t.activate(P0, ludevic, 0, &[]).expect("activates");
    t.resolve_all();
    let o = t.obj_now(ludevic);
    assert_eq!(o.chars.name.as_str(), "Olag, Ludevic's Hubris");
    assert!(o.chars.has_subtype("Zombie") && o.chars.has_subtype("Giant"));
    assert_eq!(t.counters(ludevic, "+1/+1"), 2);
    assert_eq!(t.pt(ludevic), (6, 6));
}

#[test]
fn spellweaver_helix_makes_one_copy_when_both_cards_have_the_name() {
    cr!("707.12", "607.2a");
    ruling!(
        "Spellweaver Helix",
        "If the two imprinted sorcery cards have the same name and a card with that name is cast, only one copy is created"
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let a = t.graveyard(P1, "Lava Spike");
    let b = t.graveyard(P1, "Lava Spike");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    let mine = t.hand(P0, "Lava Spike");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.cast(P0, mine).target(Entity::Player(P1)).go();
    t.resolve_all();
    // The spell and one copy.
    assert_eq!(t.life(P1), 14);
}

#[test]
fn spellweaver_helix_does_nothing_with_one_imprinted_card_left() {
    cr!("607.2a");
    ruling!(
        "Spellweaver Helix",
        "If there’s only one imprinted sorcery card, nothing happens"
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let div = t.graveyard(P1, "Divination");
    let spike = t.graveyard(P1, "Lava Spike");
    t.answer_targets(P0, &[Entity::Object(div), Entity::Object(spike)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    // The Lava Spike leaves exile: only the Divination is still exiled with it.
    let spike = t.g.current(spike);
    t.g.move_object(spike, mtg_engine::object::Zone::Graveyard(P1), events::MoveCause::Effect, None);
    t.settle();
    t.lands(P0, "Island", 3);
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let mine = t.hand(P0, "Divination");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    t.cast(P0, mine).go();
    t.resolve_all();
    // Divination resolved (two cards, one of them replacing the Divination itself); no
    // copy of anything, and nothing of the Divination in exile was copied.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn god_eternal_kefnet_reduces_the_copy_by_exactly_two() {
    cr!("707.12", "601.2f");
    supported("God-Eternal Kefnet");
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    t.battlefield(P0, "God-Eternal Kefnet");
    // Tidings ({3}{U}{U}) costs {1}{U}{U}: all three Islands.
    let islands = t.lands(P0, "Island", 3);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    t.library_top(P0, "Tidings");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 5);
    assert!(islands.iter().all(|i| t.obj_now(*i).tapped));
}
