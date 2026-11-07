//! Rulings batch P124 — permanents that leave and come back: mass exile-and-return
//! (Lae'zel's Acrobatics, Glorious Protector, Hide on the Ceiling), the cards returned as
//! new objects (CR 400.7), triggers that look back in time when their source leaves at
//! the same time (CR 603.10a: Valkyrie's Call, Yarus), and tokens that are copies entering
//! with their "enters with" abilities (CR 614.1c).

use crate::r_p124_common::*;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s11_common::manifest_card;
use crate::r_s17_common::{enter_transformed, face, token_copy};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::PLUS1;
use mtg_engine::*;

#[test]
fn laezels_acrobatics_triggers_wait_and_go_on_the_stack_in_apnap_order() {
    cr!("608.2m", "603.3b", "101.4", "706.2");
    ruling!(
        "Lae'zel's Acrobatics",
        "Any abilities that trigger during the resolution of Lae'zel's Acrobatics will wait to go on the stack until after it's finished resolving."
    );
    supported("Lae'zel's Acrobatics");
    // P0's Elvish Visionary ("When this creature enters, draw a card.") and P1's Soul
    // Warden ("Whenever another creature enters, you gain 1 life."). A roll of 15: the
    // Visionary is exiled, returned, and exiled again.
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P0, "Elvish Visionary");
    let warden = t.battlefield(P1, "Soul Warden");
    crate::r_s35_common::load_dice(&mut t, &[15]);
    cast_new(&mut t, P0, "Lae'zel's Acrobatics", &[]);
    t.resolve();
    // Both triggered while it resolved; they're put on the stack after, P0's first.
    assert_eq!(t.zone(visionary), Zone::Exile);
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).controller, P1, "P1's trigger is on top");
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.life(P1), 21);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    let _ = warden;
    // The cards return at the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Elvish Visionary").len(), 1);
}

#[test]
fn glorious_protector_exiles_on_resolution_and_new_objects_return() {
    cr!("603.6a", "610.3c", "400.7", "704.5m", "704.5n");
    ruling!(
        "Glorious Protector",
        "Glorious Protector's enters-the-battlefield ability doesn't target any creatures. You choose which non-Angel creatures to exile, if any, as that ability resolves."
    );
    ruling!(
        "Glorious Protector",
        "If Glorious Protector leaves the battlefield before its triggered ability resolves, no creatures will be exiled."
    );
    ruling!(
        "Glorious Protector",
        "Auras attached to any exiled permanents will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanents will cease to exist."
    );
    supported("Glorious Protector");
    // The Bears with Holy Strength, Bonesplitter and a +1/+1 counter, and a Hill Giant
    // that enters after the trigger is put on the stack (it can still be chosen).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    give_plus1(&mut t, bears, 1);
    let protector = t.enter(P0, "Glorious Protector");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears), obj(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_ne!(t.zone(aura), Zone::Battlefield);
    assert!(t.on_battlefield(splitter));
    assert_eq!(attached_to(&t, splitter), None);
    // Protector leaves: they return as new objects without counters.
    exile_now(&mut t, protector);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert_ne!(back, bears);
    assert_eq!(t.counters(back, PLUS1), 0);
    assert!(t.on_battlefield(t.g.current(giant)));
    // Protector leaving before its trigger resolves: nothing is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let protector = t.enter(P0, "Glorious Protector");
    t.settle();
    exile_now(&mut t, protector);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn hide_on_the_ceiling_returns_a_double_faced_card_front_face_up() {
    cr!("712.14", "400.7", "603.7");
    ruling!(
        "Hide on the Ceiling",
        "If a double-faced card is exiled this way, it will return to the battlefield with its front face up, regardless of which face was up when it was exiled."
    );
    supported("Hide on the Ceiling");
    let mut t = TestGame::new(2);
    let aberration = enter_transformed(&mut t, P0, "Delver of Secrets // Insectile Aberration");
    assert_eq!(face(&t, aberration), FaceState::Back);
    t.lands(P0, "Island", 2);
    let hide = t.hand(P0, "Hide on the Ceiling");
    t.cast(P0, hide).x(1).target(aberration).go();
    t.resolve_all();
    assert_eq!(t.zone(aberration), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.current(aberration);
    assert!(t.on_battlefield(back));
    assert_eq!(face(&t, back), FaceState::Front);
    assert_eq!(t.obj(back).chars.name.as_str(), "Delver of Secrets");
}

#[test]
fn for_the_common_good_copies_enter_with_their_counters() {
    cr!("614.1c", "707.2", "111.1");
    ruling!(
        "For the Common Good",
        "Any “as [this permanent] enters” or “[this permanent] enters with” abilities of the new tokens will work."
    );
    supported("For the Common Good");
    // A token copy of Triskelion ("This creature enters with three +1/+1 counters on
    // it."); For the Common Good with X=1 makes another.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Triskelion");
    let first = token_copy(&mut t, P0, card)[0];
    assert_eq!(t.counters(first, PLUS1), 3);
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "For the Common Good");
    t.cast(P0, spell).x(1).target(first).go();
    t.resolve_all();
    let all = t.named_on_battlefield("Triskelion");
    assert_eq!(all.len(), 2);
    for id in all {
        assert_eq!(t.counters(id, PLUS1), 3);
    }
}

#[test]
fn valkyries_call_dying_with_the_creatures_triggers_for_each() {
    cr!("603.10a", "603.6c", "400.7");
    ruling!(
        "Valkyrie's Call",
        "If Valkyrie's Call goes to the graveyard at the same time as one or more nontoken, non-Angel creatures you control die, its ability triggers once for each of those creatures."
    );
    supported("Valkyrie's Call");
    // Planar Cleansing: "Destroy all nonland permanents."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valkyrie's Call");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Planar Cleansing", &[]);
    for name in ["Grizzly Bears", "Hill Giant"] {
        let id = t.named_on_battlefield(name);
        assert_eq!(id.len(), 1, "{name}");
        assert_eq!(t.counters(id[0], PLUS1), 1);
        assert!(t.obj(id[0]).chars.has_subtype("Angel"));
        assert!(has_kw(&mut t, id[0], KeywordKind::Flying));
    }
}

#[test]
fn yarus_returns_face_down_creatures() {
    cr!("603.10a", "708.2", "400.7", "701.40a");
    ruling!(
        "Yarus, Roar of the Old Gods",
        "If Yarus, Roar of the Old Gods dies at the same time as one or more face-down creatures you control, its last ability triggers for each of those creatures."
    );
    ruling!(
        "Yarus, Roar of the Old Gods",
        "If a face-down creature dies but the permanent card it becomes in the graveyard leaves the graveyard before Yarus, Roar of the Old Gods's last ability resolves, it will not return to the battlefield."
    );
    supported("Yarus, Roar of the Old Gods");
    // Two manifested cards and Yarus die to Wrath of God: both come back face up.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yarus, Roar of the Old Gods");
    manifest_card(&mut t, P0, "Grizzly Bears");
    manifest_card(&mut t, P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    for name in ["Grizzly Bears", "Hill Giant"] {
        let id = t.named_on_battlefield(name);
        assert_eq!(id.len(), 1, "{name}");
        assert!(!t.obj(id[0]).face_down);
    }
    // The card leaves the graveyard first: it stays where it went.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yarus, Roar of the Old Gods");
    let fd = manifest_card(&mut t, P0, "Grizzly Bears");
    destroy(&mut t, fd);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.current(fd);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    exile_now(&mut t, card);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn yarus_returned_card_that_cant_be_turned_face_up_stays_face_down() {
    cr!("708.2a", "708.2");
    ruling!(
        "Yarus, Roar of the Old Gods",
        "If a permanent card that would be returned to the battlefield face down by Yarus, Roar of the Old Gods's last ability can't be turned face up for some reason, it'll still be returned to the battlefield, but it will stay face down."
    );
    supported("Karlov Watchdog");
    // P1's Karlov Watchdog: "Permanents your opponents control can't be turned face up
    // during your turn." On P1's turn, P0's face-down Bears dies.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yarus, Roar of the Old Gods");
    let fd = manifest_card(&mut t, P0, "Grizzly Bears");
    t.battlefield(P1, "Karlov Watchdog");
    t.set_step(P1, Step::PrecombatMain);
    destroy(&mut t, fd);
    t.resolve_all();
    let back = t.g.current(fd);
    assert!(t.on_battlefield(back));
    assert!(t.obj(back).face_down);
    // A 2/2 with no name, mana cost or creature types (and only the haste Yarus gives).
    t.g.recompute();
    let o = t.obj(back);
    assert!(o.chars.name.is_empty() && o.chars.subtypes.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(t.pt(back), (2, 2));
    assert!(!has_kw(&mut t, back, KeywordKind::Ward));
}

#[test]
fn yarus_draws_once_per_player_dealt_damage_each_step() {
    cr!("603.2c", "510.2", "702.7b");
    ruling!(
        "Yarus, Roar of the Old Gods",
        "Yarus, Roar of the Old Gods's last ability triggers once for each player that face-down creatures you control dealt combat damage to"
    );
    // Three players: two face-down creatures attack P1, one attacks P2: two draws. With
    // first strike on one of P1's attackers, P1 is dealt damage in both steps: three.
    for first_strike in [false, true] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Yarus, Roar of the Old Gods");
        let a = manifest_card(&mut t, P0, "Grizzly Bears");
        let b = manifest_card(&mut t, P0, "Grizzly Bears");
        let c = manifest_card(&mut t, P0, "Grizzly Bears");
        if first_strike {
            modify_until_eot(
                &mut t,
                a,
                vec![mtg_engine::ability::Modification::AddKeyword(
                    mtg_engine::keywords::Keyword::new(KeywordKind::FirstStrike),
                )],
            );
        }
        let hand = t.hand_size(P0);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(
            &[
                (a, Entity::Player(P1)),
                (b, Entity::Player(P1)),
                (c, Entity::Player(P2)),
            ],
            &[],
        );
        t.resolve_all();
        let want = if first_strike { 3 } else { 2 };
        assert_eq!(t.hand_size(P0), hand + want, "first strike: {first_strike}");
    }
}
