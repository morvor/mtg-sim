//! CR 702.166 Bargain.

use crate::common_k702_153_167::*;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn bargain_sacrifices_an_artifact_enchantment_or_token_as_an_optional_cost() {
    cr!("702.166", "702.166a");
    ruling!(
        "Troublemaker Ouphe",
        "You may sacrifice only one artifact, enchantment, or token to pay a spell’s bargain cost."
    );
    ruling!(
        "Candy Grapple",
        "Bargain means “As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.”"
    );
    assert_supported("Candy Grapple");
    // Candy Grapple: -3/-3, or -5/-5 if bargained.
    for what in ["Ornithopter", "Glorious Anthem", "Treasure"] {
        let mut t = TestGame::new(2);
        let fodder = if what == "Treasure" {
            make_token(&mut t, P0, "Treasure")
        } else {
            t.battlefield(P0, what)
        };
        let spare = t.battlefield(P0, "Ornithopter");
        let giant = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Swamp", 2);
        let spell = t.hand(P0, "Candy Grapple");
        pay_with(&mut t, P0, &[fodder]);
        t.cast(P0, spell).target(giant).go();
        assert!(!t.on_battlefield(fodder), "{what} was sacrificed");
        assert!(t.on_battlefield(spare), "only one permanent is sacrificed");
        t.resolve_all();
        // -5/-5 on a 6/4.
        assert!(!t.on_battlefield(giant));
    }
    // Only creatures that aren't artifacts or tokens: no bargain cost can be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    let spell = t.hand(P0, "Candy Grapple");
    t.cast(P0, spell).target(giant).go();
    assert!(optional_costs_offered(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 1));
}

#[test]
fn a_spell_whose_bargain_cost_was_paid_is_bargained() {
    cr!("702.166b");
    ruling!(
        "Troublemaker Ouphe",
        "Bargain represents an optional additional cost. A spell cast with that additional cost paid is “bargained.”"
    );
    assert_supported("Kellan's Lightblades");
    assert_supported("Hamlet Glutton");
    // Kellan's Lightblades: 3 damage to an attacking or blocking creature, or destroy it
    // if bargained.
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P1, mtg_engine::turn::Step::DeclareBlockers);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.g.combat.as_mut().unwrap().attackers.push(
            mtg_engine::combat::AttackerInfo {
                id: wurm,
                target: Some(Entity::Player(P0)),
                original_target: Some(Entity::Player(P0)),
                declared: true,
                blocked: false,
                blockers: vec![],
                band: None,
                defending_player: Some(P0),
            },
        );
        let thopter = t.battlefield(P0, "Ornithopter");
        t.lands(P0, "Plains", 2);
        let spell = t.hand(P0, "Kellan's Lightblades");
        if bargain {
            pay_with(&mut t, P0, &[thopter]);
        } else {
            pay_optional(&mut t, P0, false);
        }
        t.cast(P0, spell).target(wurm).go();
        t.resolve_all();
        assert_eq!(t.on_battlefield(wurm), !bargain);
    }
    // Hamlet Glutton costs {2} less to cast if it's bargained: five lands are enough then.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Forest", 5);
    let glutton = t.hand(P0, "Hamlet Glutton");
    pay_with(&mut t, P0, &[thopter]);
    t.cast(P0, glutton).go();
    t.resolve_all();
    assert_eq!(named(&t, P0, "Hamlet Glutton").len(), 1);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn abilities_linked_to_bargain_see_only_whether_that_object_was_bargained() {
    cr!("702.166c");
    ruling!(
        "Troublemaker Ouphe",
        "If a card or token enters the battlefield as a copy of a permanent that’s already on the battlefield, the new permanent isn’t bargained, even if the original was."
    );
    assert_supported("Troublemaker Ouphe");
    // Troublemaker Ouphe: "When this creature enters, if it was bargained, exile target
    // artifact or enchantment an opponent controls."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let other = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Forest", 2);
    let ouphe = t.hand(P0, "Troublemaker Ouphe");
    pay_with(&mut t, P0, &[thopter]);
    t.cast(P0, ouphe).go();
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    t.resolve_all();
    assert!(!t.on_battlefield(anthem));
    // A Clone copying the bargained Ouphe isn't bargained: its ability doesn't trigger.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    let ouphe_now = named(&t, P0, "Troublemaker Ouphe")[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ouphe_now)]);
    t.resolve_all();
    assert_eq!(named(&t, P0, "Troublemaker Ouphe").len(), 2);
    assert!(t.on_battlefield(other));
    // Nor is an Ouphe that wasn't bargained.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Forest", 2);
    let ouphe = t.hand(P0, "Troublemaker Ouphe");
    pay_optional(&mut t, P0, false);
    t.cast(P0, ouphe).go();
    t.resolve_all();
    assert!(t.on_battlefield(anthem));
}

#[test]
fn a_copy_of_a_bargained_spell_is_bargained() {
    cr!("702.166b");
    ruling!(
        "Troublemaker Ouphe",
        "If you copy a bargained spell, the copy is also bargained."
    );
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let w1 = t.battlefield(P1, "Craw Wurm");
    let w2 = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Island", 2);
    let grapple = t.hand(P0, "Candy Grapple");
    pay_with(&mut t, P0, &[thopter]);
    let spell = t.cast(P0, grapple).target(w1).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(spell).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(w2)]);
    t.resolve_all();
    // Both got -5/-5 (a 6/4 survives -3/-3).
    assert!(!t.on_battlefield(w1));
    assert!(!t.on_battlefield(w2));
}

#[test]
fn targets_needed_only_if_bargained_are_chosen_only_if_it_was() {
    cr!("702.166d");
    ruling!(
        "Troublemaker Ouphe",
        "You ignore those targeting requirements if those spells aren’t bargained, and you can’t bargain those spells unless you can choose the appropriate targets."
    );
    assert_supported("Brave the Wilds");
    // Brave the Wilds: "If this spell was bargained, target land you control becomes a
    // 3/3 Elemental creature with haste that's still a land. Search your library for a
    // basic land card, reveal it, put it into your hand, then shuffle."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let forest = t.battlefield(P0, "Forest");
    let spell = t.hand(P0, "Brave the Wilds");
    t.library_top(P0, "Plains");
    pay_optional(&mut t, P0, false);
    t.cast(P0, spell).go();
    // Not bargained: no target was chosen.
    let targets_asked = asked_matching(&t, P0, |d| matches!(d, Decision::ChooseTargets { .. }));
    assert_eq!(targets_asked, 0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Plains"));
    assert!(!t.obj_now(forest).is_creature());
    assert!(t.on_battlefield(thopter));
    // Bargained: the land is targeted and animated.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let forest = t.battlefield(P0, "Forest");
    let other = t.battlefield(P0, "Forest");
    let spell = t.hand(P0, "Brave the Wilds");
    pay_with(&mut t, P0, &[thopter]);
    t.cast(P0, spell).target(other).go();
    t.resolve_all();
    assert!(t.obj_now(other).is_creature());
    assert_eq!(t.pt(other), (3, 3));
    assert!(!t.obj_now(forest).is_creature());
    // Without a land to target, it can't be bargained (but can be cast otherwise).
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(ManaType::G, 1);
    let spell = t.hand(P0, "Brave the Wilds");
    pay_with(&mut t, P0, &[thopter]);
    assert!(t.cast(P0, spell).try_go().is_err());
    assert!(t.on_battlefield(thopter));
    t.clear_answers();
    pay_optional(&mut t, P0, false);
    assert!(t.cast(P0, spell).try_go().is_ok());
}

#[test]
fn a_bargained_spell_has_its_additional_effect() {
    cr!("702.166b", "702.166c");
    assert_supported("Archon's Glory");
    // Archon's Glory: "Target creature gets +2/+2 until end of turn. If this spell was
    // bargained, that creature also gains flying and lifelink until end of turn."
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 1);
        let spell = t.hand(P0, "Archon's Glory");
        if bargain {
            let treasure = make_token(&mut t, P0, "Treasure");
            pay_with(&mut t, P0, &[treasure]);
        } else {
            pay_optional(&mut t, P0, false);
        }
        t.cast(P0, spell).target(bears).go();
        t.resolve_all();
        assert_eq!(t.pt(bears), (4, 4));
        assert_eq!(
            has_kw(&t, bears, mtg_engine::keywords::KeywordKind::Flying),
            bargain
        );
        assert_eq!(
            has_kw(&t, bears, mtg_engine::keywords::KeywordKind::Lifelink),
            bargain
        );
    }
}
