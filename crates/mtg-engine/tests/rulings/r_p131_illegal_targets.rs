//! Rulings batch P131 — "regrowth" spells and abilities whose targets become illegal before
//! they resolve (CR 608.2b): with every target illegal they don't resolve and none of their
//! other effects happen; with one target still legal they resolve and do what they can.

use crate::r_p131_common::*;
use crate::r_s06_common::has_kw;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Exiles a card from wherever it is (it's no longer in the zone it was targeted in).
fn exile_it(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.move_object(id, Zone::Exile, MoveCause::Effect, None);
    t.settle();
}

/// A spell that returns a target card from P0's graveyard and does something else: cast at
/// a Grizzly Bears card in P0's graveyard which is exiled in response.
fn spell_fizzles(card: &str, extra_targets: &[Entity]) {
    supported(card);
    let mut t = TestGame::new(2);
    let target = t.graveyard(P0, "Grizzly Bears");
    let mut targets = vec![obj(target)];
    targets.extend_from_slice(extra_targets);
    let s = cast(&mut t, P0, card, &targets);
    fizzles(&mut t, s, |t| exile_it(t, target));
    assert!(t.in_exile("Grizzly Bears"), "{card}");
    assert!(tokens(&t, P0).is_empty(), "{card}");
}

#[test]
fn regrowth_spells_with_an_illegal_target_do_nothing_else() {
    cr!("608.2b");
    ruling!(
        "Emergency Weld",
        "If the card in your graveyard is an illegal target as Emergency Weld tries to resolve, Emergency Weld is removed from the stack and you won't create a token."
    );
    ruling!(
        "Pulse of Murasa",
        "If the target card is an illegal target by the time Pulse of Murasa tries to resolve, the spell doesn’t resolve. You won’t gain 6 life."
    );
    ruling!(
        "Urborg Uprising",
        "If you chose at least one target and all targets are illegal as Urborg Uprising tries to resolve, the spell won’t resolve and you won’t draw a card."
    );
    spell_fizzles("Emergency Weld", &[]);
    spell_fizzles("Pulse of Murasa", &[]);
    spell_fizzles("Urborg Uprising", &[]);
}

#[test]
fn verdant_rebirth_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Verdant Rebirth",
        "If the target creature is an illegal target by the time Verdant Rebirth resolves, the entire spell doesn’t resolve. You won’t draw a card."
    );
    supported("Verdant Rebirth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = cast(&mut t, P0, "Verdant Rebirth", &[obj(bears)]);
    fizzles(&mut t, s, |t| exile_it(t, bears));
}

#[test]
fn urborg_uprising_with_no_targets_still_draws() {
    cr!("608.2b");
    ruling!(
        "Urborg Uprising",
        "If you chose at least one target and all targets are illegal as Urborg Uprising tries to resolve"
    );
    let mut t = TestGame::new(2);
    let s = cast(&mut t, P0, "Urborg Uprising", &[]);
    assert!(targets_of(&t, s).is_empty());
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn morgue_burst_returning_nothing_deals_no_damage() {
    cr!("608.2b");
    ruling!(
        "Morgue Burst",
        "If the creature card is an illegal target when Morgue Burst tried to resolve (most likely because it was exiled in response), it won’t be returned to your hand and no damage will be dealt to the target."
    );
    supported("Morgue Burst");
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    let s = cast(&mut t, P0, "Morgue Burst", &[obj(wurm), Entity::Player(P1)]);
    exile_it(&mut t, wurm);
    t.resolve_all();
    // The player is still a legal target, so the spell resolves, but returns nothing.
    assert!(resolved(&t, s));
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.life(P1), 20);
    // Legal: the card's power in damage.
    let mut t = TestGame::new(2);
    let wurm = t.graveyard(P0, "Craw Wurm");
    cast(&mut t, P0, "Morgue Burst", &[obj(wurm), Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn warren_pilferers_with_an_illegal_target_doesnt_gain_haste() {
    cr!("608.2b");
    ruling!(
        "Warren Pilferers",
        "If the card is an illegla target as the ability resolves, the ability won’t resolve and none of its effects will happen. Warren Pilferers won’t gain haste."
    );
    supported("Warren Pilferers");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let goblin = t.graveyard(P0, "Raging Goblin");
        t.answer_targets(P0, &[obj(goblin)]);
        let wp = t.enter(P0, "Warren Pilferers");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if !legal {
            exile_it(&mut t, goblin);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Raging Goblin"), legal);
        let haste = has_kw(&t, t.g.current(wp), KeywordKind::Haste);
        assert_eq!(haste, legal);
    }
}

#[test]
fn dross_skullbomb_with_an_illegal_target_doesnt_draw() {
    cr!("608.2b");
    ruling!(
        "Dross Skullbomb",
        "If the target of Dross Skullbomb's last ability isn't a legal target as the ability tries to resolve (probably because it has left the graveyard), you won't draw a card."
    );
    supported("Dross Skullbomb");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let bomb = t.battlefield(P0, "Dross Skullbomb");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Wastes", 2);
        act(&mut t, P0, bomb, "Return target", &[obj(bears)]).unwrap();
        if !legal {
            exile_it(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), legal);
        assert_eq!(t.hand_size(P0), if legal { 2 } else { 0 });
    }
}

#[test]
fn modal_regrowth_with_one_illegal_target_returns_the_other() {
    cr!("608.2b", "700.2");
    ruling!(
        "Reviving Melody",
        "If you choose both modes, and only one of the cards is still a legal target when Reviving Melody tries to resolve, only that card is returned to your hand."
    );
    ruling!(
        "Remember the Fallen",
        "If you choose both modes, and only one of the targeted cards is still a legal target when Remember the Fallen tries to resolve, only that card is returned to your hand."
    );
    for (card, other) in [
        ("Reviving Melody", "Pacifism"),
        ("Remember the Fallen", "Sol Ring"),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let o = t.graveyard(P0, other);
        lands_for_cost(&mut t, P0, card);
        let c = t.hand(P0, card);
        let s = t
            .cast(P0, c)
            .modes(&[0, 1])
            .targets(&[obj(bears)])
            .targets(&[obj(o)])
            .go();
        exile_it(&mut t, bears);
        t.resolve_all();
        assert!(resolved(&t, s), "{card}");
        assert!(t.in_hand(P0, other), "{card}");
        assert!(t.in_exile("Grizzly Bears"), "{card}");
    }
}

#[test]
fn greenwarden_with_an_illegal_target_cant_exile_itself() {
    cr!("608.2b");
    ruling!(
        "Greenwarden of Murasa",
        "If the target card becomes illegal before the last ability resolves, it won't resolve. You can't exile Greenwarden of Murasa in that case, even if you want to."
    );
    supported("Greenwarden of Murasa");
    let mut t = TestGame::new(2);
    let gw = t.battlefield(P0, "Greenwarden of Murasa");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, gw);
    assert_eq!(t.stack_len(), 1);
    exile_it(&mut t, bears);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Greenwarden of Murasa"));
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
}

#[test]
fn greenwarden_decides_on_resolution_whether_to_exile_itself() {
    cr!("603.3d", "608.2");
    ruling!(
        "Greenwarden of Murasa",
        "You decide whether to exile Greenwarden of Murasa as the last ability resolves."
    );
    for exile in [true, false] {
        let mut t = TestGame::new(2);
        let gw = t.battlefield(P0, "Greenwarden of Murasa");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let from = t.asked().len();
        destroy(&mut t, gw);
        // On the stack and targeting, but nobody was asked yet.
        assert_eq!(t.stack_len(), 1);
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
        t.answer_yes(P0, exile);
        t.resolve_all();
        assert_eq!(t.in_exile("Greenwarden of Murasa"), exile);
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), exile);
    }
}
