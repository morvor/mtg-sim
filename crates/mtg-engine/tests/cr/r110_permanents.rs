//! CR 110: permanents — what a permanent is, its owner and controller, permanent types,
//! and status.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_murder(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    let m = t.hand(p, "Murder");
    t.lands(p, "Swamp", 3);
    t.cast(p, m).target(target).go();
    t.resolve();
}

#[test]
fn a_permanent_is_a_card_or_token_on_the_battlefield() {
    cr!("110.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    let alarm = t.hand(P0, "Raise the Alarm");
    t.lands(P0, "Plains", 2);
    t.cast(P0, alarm).go();
    t.resolve();
    let soldiers = tokens_of(&t, P0);
    assert_eq!(soldiers.len(), 2);
    for id in [bears, soldiers[0], soldiers[1]] {
        assert!(matches(&t, id, &Filter::Permanent, P0));
    }
    // A card in another zone isn't a permanent.
    assert!(!matches(&t, in_hand, &Filter::Permanent, P0));
    // It stops being a permanent as it's moved to another zone.
    cast_murder(&mut t, P1, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!matches(&t, bears, &Filter::Permanent, P0));
}

#[test]
fn a_permanents_owner_is_its_cards_owner_and_its_controller_who_it_entered_under() {
    cr!("110.2");
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(bears).owner, P0);
    assert_eq!(t.obj(bears).controller, P0);
    // Another player gains control of it: it has a new controller, the same owner.
    let treason = t.hand(P1, "Act of Treason");
    t.lands(P1, "Mountain", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, treason).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P1);
    assert_eq!(t.obj_now(bears).owner, P0);
    // When it dies it goes to its owner's graveyard.
    cast_murder(&mut t, P1, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_player_instructed_to_put_an_object_onto_the_battlefield_controls_it() {
    cr!("110.2a");
    let mut t = TestGame::new(2);
    // Exhume: each player puts a creature card from their graveyard onto the battlefield.
    let mine = t.graveyard(P0, "Grizzly Bears");
    let theirs = t.graveyard(P1, "Hill Giant");
    let exhume = t.hand(P0, "Exhume");
    t.lands(P0, "Swamp", 2);
    t.cast(P0, exhume).go();
    t.resolve();
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    assert_eq!(t.obj_now(mine).controller, P0);
    // P1 was instructed to put their card onto the battlefield: P1 controls it, although
    // P0 controls Exhume.
    assert_eq!(t.obj_now(theirs).controller, P1);
    // An effect that instructs its controller to put another player's card onto the
    // battlefield (and doesn't say otherwise) puts it under the controller's control.
    let giant = t.graveyard(P1, "Hill Giant");
    let reanimate = card_with(
        "Borrowed Rebirth",
        "{B}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::and(vec![
                    Filter::Type(CardType::Creature),
                    Filter::Card,
                    Filter::InZone(ZoneKind::Graveyard),
                ]),
                "target creature card in a graveyard",
            )],
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::battlefield(),
            },
        )],
    );
    let r = put_in_hand(&mut t, P0, reanimate);
    t.lands(P0, "Swamp", 1);
    t.cast(P0, r).target(giant).go();
    t.resolve();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(giant).owner, P1);
}

/// A spell that gains control of target spell.
fn spell_thief() -> mtg_engine::card::CardDef {
    card_from_text(
        "Spell Thief",
        "{U}",
        "Instant",
        None,
        "Gain control of target spell.",
    )
}

#[test]
fn gaining_control_of_a_permanent_spell_gives_control_of_the_permanent() {
    cr!("110.2b");
    let mut t = TestGame::new(3);
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, bears).go();
    let thief = put_in_hand(&mut t, P0, spell_thief());
    t.lands(P0, "Island", 1);
    t.cast(P0, thief).target(spell).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).controller, P0);
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(bears).controller, P0);
    assert_eq!(t.obj(bears).owner, P1);
    // By default the permanent is controlled by the player who put the spell onto the
    // stack: when P0 leaves the game, the effect giving P0 control ends and P1 controls
    // the Bears again, so it isn't exiled (CR 800.4a).
    t.g.player_loses(P0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn a_nontoken_permanent_has_its_cards_characteristics_as_modified() {
    cr!("110.3");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let o = t.obj(bears);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{1}{G}");
    assert!(o.chars.has_subtype("Bear"));
    assert_eq!(t.pt(bears), (2, 2));
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn instant_and_sorcery_cards_cant_enter_the_battlefield() {
    cr!("110.4");
    let mut t = TestGame::new(2);
    // "Return target card from your graveyard to the battlefield."
    let raise = || {
        card_with(
            "Odd Return",
            "{B}",
            "Sorcery",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(
                    Filter::and(vec![
                        Filter::Card,
                        Filter::InZone(ZoneKind::Graveyard),
                        Filter::OwnedBy(PlayerRel::You),
                    ]),
                    "target card from your graveyard",
                )],
                Effect::Move {
                    what: Sel::Target(0),
                    to: Destination::battlefield(),
                },
            )],
        )
    };
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // Crib Swap is a kindred instant: it can't enter either.
    let swap = t.graveyard(P0, "Crib Swap");
    // Bitterblossom is a kindred enchantment: it can.
    let blossom = t.graveyard(P0, "Bitterblossom");
    t.lands(P0, "Swamp", 3);
    for card in [bolt, swap, blossom] {
        let r = put_in_hand(&mut t, P0, raise());
        t.cast(P0, r).target(card).go();
        t.resolve();
    }
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.zone(swap), Zone::Graveyard(P0));
    assert!(t.on_battlefield(blossom));
    assert!(t.named_on_battlefield("Lightning Bolt").is_empty());
}

#[test]
fn permanent_spells_are_artifact_battle_creature_enchantment_or_planeswalker_spells() {
    cr!("110.4b");
    let mut t = TestGame::new(2);
    let f = |t: &mut TestGame, p: PlayerId, card: &str| {
        let c = t.hand(p, card);
        t.cast(p, c).go()
    };
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bears = f(&mut t, P0, "Grizzly Bears");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    // "Counter target permanent spell."
    let denial = card_from_text(
        "Permanent Denial",
        "{U}",
        "Instant",
        None,
        "Counter target permanent spell.",
    );
    let d = put_in_hand(&mut t, P1, denial);
    let cands = spell_target_candidates(&t, P1, d, 0);
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(!cands.contains(&Entity::Object(bolt)));
}

#[test]
fn a_permanent_that_loses_all_permanent_types_is_still_a_permanent() {
    cr!("110.4c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let unmake = card_with(
        "Unmake Type",
        "{U}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::Type(CardType::Creature),
                "target creature",
            )],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let u = put_in_hand(&mut t, P0, unmake);
    t.lands(P0, "Island", 1);
    t.cast(P0, u).target(bears).go();
    t.resolve();
    t.settle();
    assert!(t.on_battlefield(bears));
    assert!(t.obj(bears).chars.card_types.is_empty());
    assert!(matches(&t, bears, &Filter::Permanent, P0));
    assert!(!matches(&t, bears, &Filter::Type(CardType::Creature), P0));
}

#[test]
fn permanents_enter_untapped_face_up_and_phased_in_unless_told_otherwise() {
    cr!("110.5", "110.5b");
    let mut t = TestGame::new(2);
    // A tapped, face-down permanent returned to its owner's hand and cast again is a new
    // permanent: untapped, face up, and phased in.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, bears));
    t.g.objects[bears.0 as usize].tapped = true;
    let o = t.obj(bears);
    assert!(o.tapped && o.face_down && !o.phased_out);
    let unsummon = t.hand(P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.cast(P0, unsummon).target(bears).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let card = t.g.current(bears);
    t.lands(P0, "Forest", 2);
    t.cast(P0, card).go();
    t.resolve();
    let back = t.named_on_battlefield("Grizzly Bears")[0];
    let o = t.obj(back);
    assert!(!o.tapped && !o.face_down && !o.phased_out);
    // "This land enters tapped" says otherwise.
    let cove = t.hand(P0, "Tranquil Cove");
    t.play_land(P0, cove).unwrap();
    let cove = t.named_on_battlefield("Tranquil Cove")[0];
    assert!(t.obj(cove).tapped);
}

#[test]
fn status_is_not_a_characteristic_and_isnt_copied() {
    cr!("110.5a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let clone = t.hand(P0, "Clone");
    t.lands(P0, "Island", 4);
    t.cast(P0, clone).go();
    t.resolve();
    let bears_now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears_now.len(), 2);
    let copy = *bears_now.iter().find(|b| **b != bears).unwrap();
    // The copy has the copied characteristics but not the original's tapped status.
    assert_eq!(t.pt(copy), (2, 2));
    assert!(t.obj(bears).tapped);
    assert!(!t.obj(copy).tapped);
}

#[test]
fn a_face_down_status_affects_characteristics() {
    cr!("110.5a");
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Ainok Tracker");
    assert_eq!(t.pt(tracker), (3, 3));
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, tracker));
    t.g.recompute();
    // Face down: a nameless 2/2 with no abilities (CR 708.2).
    assert_eq!(t.pt(tracker), (2, 2));
    assert!(t.obj(tracker).chars.name.is_empty());
    assert!(t.obj(tracker).chars.abilities.is_empty());
}

#[test]
fn a_permanent_keeps_its_status_until_something_changes_it() {
    cr!("110.5c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    // Its characteristics change (it stops being a creature): it stays tapped.
    let unmake = card_with(
        "Unmake Type",
        "{U}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::Type(CardType::Creature),
                "target creature",
            )],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let u = put_in_hand(&mut t, P0, unmake);
    t.lands(P0, "Island", 1);
    t.cast(P0, u).target(bears).go();
    t.resolve();
    assert!(t.obj(bears).tapped);
    // Turn-based actions change it: it untaps during its controller's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj(bears).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj(bears).tapped);
}

#[test]
fn only_permanents_have_status() {
    cr!("110.5d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    assert!(matches(&t, bears, &Filter::Tapped, P0));
    cast_murder(&mut t, P1, bears);
    // A card in a graveyard is neither tapped nor untapped.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!matches(&t, bears, &Filter::Tapped, P0));
    assert!(!matches(&t, bears, &Filter::Untapped, P0));
    let in_hand = t.hand(P0, "Hill Giant");
    assert!(!matches(&t, in_hand, &Filter::Untapped, P0));
}
