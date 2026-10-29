//! Rulings batch P058 — more flicker rulings: who controls the returned permanent (CR
//! 110.2, 800.4a), what targeted the old object (CR 400.7, 608.2b), enters triggers of
//! the returned permanent (CR 603.6a), Gossip's Talent, Deadeye Navigator's soulbond,
//! Wispweaver Angel loops and Ruin Ghost's lands.

use crate::r_p058_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s19_common::gain_level;
use crate::r_s21_common::legal_blocks;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Action;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn distinguished_conjurer_sees_the_creature_it_returned_enter() {
    cr!("603.6a", "400.7");
    ruling!(
        "Distinguished Conjurer",
        "Distinguished Conjurer's first ability triggers whenever any creature other than itself enters the battlefield under your control, including those returned by its last ability."
    );
    supported("Distinguished Conjurer");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let c = t.battlefield(P0, "Distinguished Conjurer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, c, "Exile another target").unwrap();
    t.resolve_all();
    assert_ne!(t.g.current(bears), bears);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn illusionists_stratagem_targets() {
    cr!("608.2b", "115.1", "601.2c");
    ruling!(
        "Illusionist's Stratagem",
        "If each target creature is an illegal target as Illusionist’s Stratagem resolves, the spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Illusionist's Stratagem",
        "If you choose two target creatures and one is an illegal target as Illusionist’s Stratagem resolves, you’ll exile the other, return it, and draw a card."
    );
    ruling!(
        "Illusionist's Stratagem",
        "You may cast Illusionist’s Stratagem without any targets if you wish to just draw a card."
    );
    supported("Illusionist's Stratagem");
    for illegal in [2usize, 1, 0] {
        let mut t = TestGame::new(2);
        mana(&mut t, P0, 2);
        t.library_top(P0, "Island");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Gray Ogre");
        let spell = t.hand(P0, "Illusionist's Stratagem");
        t.cast(P0, spell)
            .targets(&[Entity::Object(a), Entity::Object(b)])
            .go();
        let hand = t.hand_size(P0);
        // Responses make targets illegal.
        if illegal >= 1 {
            destroy(&mut t, a);
        }
        if illegal >= 2 {
            destroy(&mut t, b);
        }
        t.resolve_all();
        if illegal == 2 {
            assert_eq!(t.hand_size(P0), hand, "no card drawn");
        } else {
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_ne!(t.g.current(b), b, "the legal target was flickered");
            assert!(t.on_battlefield(b));
        }
    }
    // No targets at all: just draw.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    t.library_top(P0, "Island");
    let spell = t.hand(P0, "Illusionist's Stratagem");
    let hand = t.hand_size(P0);
    t.cast(P0, spell).targets(&[]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Illusionist's Stratagem"));
    assert!(t.in_hand(P0, "Island"));
}

/// P0 casts Threaten on P1's Gray Ogre (gaining control of it until end of turn), then
/// `flicker` flickers it during the turn; returns the game (three players) and the Ogre.
fn stolen_and_flickered(flicker: impl FnOnce(&mut TestGame, ObjectId)) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(3);
    mana(&mut t, P0, 3);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let threaten = t.hand(P0, "Threaten");
    t.cast(P0, threaten).target(ogre).go();
    t.resolve_all();
    assert_eq!(t.obj_now(ogre).controller, P0);
    flicker(&mut t, ogre);
    assert_ne!(t.g.current(ogre), ogre);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(ogre).controller, P0, "indefinitely");
    (t, ogre)
}

fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
    t.settle();
}

#[test]
fn returned_under_your_control_you_keep_it() {
    cr!("110.2", "800.4a", "613.1b");
    ruling!(
        "Conjurer's Closet",
        "If you gain control of a creature “until end of turn,” you control it during that turn's end step."
    );
    ruling!(
        "Conjurer's Closet",
        "When an effect returns the exiled card “under your control,” you control it indefinitely after that."
    );
    ruling!(
        "Restoration Angel",
        "When an effect returns the exiled card \"under your control,\" you control it indefinitely after that. If you had temporarily gained control of a creature, it won't return to its previous controller."
    );
    ruling!(
        "Cloudshift",
        "When an effect returns the exiled card \"under your control,\" you control it indefinitely after that. In a multiplayer game, if a player leaves the game, all cards that player owns leave as well. If you leave the game, any creatures you control from Cloudshift's effect are exiled."
    );
    supported("Conjurer's Closet");
    supported("Restoration Angel");
    supported("Cloudshift");
    // Conjurer's Closet: "At the beginning of your end step, you may exile target
    // creature you control, then return that card to the battlefield under your control."
    let closet = |t: &mut TestGame, ogre: ObjectId| {
        t.battlefield(P0, "Conjurer's Closet");
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.resolve_all();
    };
    let angel = |t: &mut TestGame, ogre: ObjectId| {
        t.answer_targets(P0, &[Entity::Object(ogre)]);
        t.answer_yes(P0, true);
        t.enter(P0, "Restoration Angel");
        t.resolve_all();
    };
    let cloudshift = |t: &mut TestGame, ogre: ObjectId| {
        let spell = t.hand(P0, "Cloudshift");
        t.cast(P0, spell).target(ogre).go();
        t.resolve_all();
    };
    // If you leave the game, the creature is exiled; if its owner leaves, it leaves too.
    let (mut t, ogre) = stolen_and_flickered(closet);
    concede(&mut t, P0);
    assert_eq!(t.zone(ogre), Zone::Exile);
    let (mut t, ogre) = stolen_and_flickered(angel);
    concede(&mut t, P0);
    assert_eq!(t.zone(ogre), Zone::Exile);
    let (mut t, ogre) = stolen_and_flickered(cloudshift);
    concede(&mut t, P1);
    assert!(!t.on_battlefield(ogre));
    assert!(t.named_on_battlefield("Gray Ogre").is_empty());
    ruling!(
        "Conjurer's Closet",
        "If you leave the game, any creatures you control from Conjurer's Closet effect are exiled."
    );
    ruling!(
        "Restoration Angel",
        "If you leave the game, a creature you took with Restoration Angel's effect is exiled."
    );
}

#[test]
fn flicker_targets_cards_you_own_but_dont_control() {
    cr!("110.2", "108.3", "400.7");
    ruling!(
        "Slip On the Ring",
        "You can target a creature you own but don't control with Slip On the Ring."
    );
    supported("Slip On the Ring");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    let spell = t.hand(P0, "Slip On the Ring");
    t.cast(P0, spell).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);

    ruling!(
        "Meneldor, Swift Savior",
        "Meneldor's second ability can target any permanent you own, including those another player controls."
    );
    supported("Meneldor, Swift Savior");
    let mut t = TestGame::new(2);
    let meneldor = t.battlefield(P0, "Meneldor, Swift Savior");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(meneldor, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(bears));
    assert_ne!(t.g.current(bears), bears);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn flickering_spirit_returns_to_its_owner() {
    cr!("110.2", "614.12");
    ruling!(
        "Flickering Spirit",
        "If Flickering Spirit's ability is activated, any \"as this enters\" choices for it are made by its owner, not its previous controller."
    );
    supported("Flickering Spirit");
    let mut t = TestGame::new(2);
    mana(&mut t, P1, 2);
    let spirit = t.battlefield(P0, "Flickering Spirit");
    give_control(&mut t, spirit, P1);
    activate_containing(&mut t, P1, spirit, "then return it").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(spirit));
    assert_eq!(t.obj_now(spirit).controller, P0, "its owner controls it");
}

#[test]
fn flickered_objects_and_spells_that_dont_target() {
    cr!("400.7", "608.2b", "608.2h");
    ruling!(
        "Cloudshift",
        "The returned card won't be the target of any spells or abilities that targeted it before. Any spells that don't target, such as Akroma's Vengeance, will still affect it."
    );
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 6);
    mana(&mut t, P1, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(bears).go();
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0, "Shock lost its target");
    assert!(t.in_graveyard(P1, "Shock"));
    // Wrath of God doesn't target: the new Bears is destroyed anyway.
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn ghostly_flicker_different_card_types() {
    cr!("115.1", "400.7");
    ruling!(
        "Ghostly Flicker",
        "The two targets can have different card types. For example, you can target one artifact and one creature with Ghostly Flicker."
    );
    supported("Ghostly Flicker");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Ghostly Flicker");
    t.cast(P0, spell)
        .targets(&[Entity::Object(ring), Entity::Object(bears)])
        .go();
    t.resolve_all();
    for id in [ring, bears] {
        assert!(t.on_battlefield(id));
        assert_ne!(t.g.current(id), id);
    }
}

#[test]
fn wispweaver_angels_can_flicker_each_other() {
    cr!("603.6a", "400.7", "603.3d");
    ruling!(
        "Wispweaver Angel",
        "Wispweaver Angel's triggered ability can target another Wispweaver Angel. If so, the two Angels can loop in and out of exile as many times as you'd like"
    );
    supported("Wispweaver Angel");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Wispweaver Angel");
    // B enters: its trigger targets A.
    t.answer_targets(P0, &[Entity::Object(a)]);
    let b = t.enter(P0, "Wispweaver Angel");
    t.settle();
    // A returns: its trigger targets B; then B returns: its trigger targets A; then stop.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.resolve();
    assert_ne!(t.g.current(a), a);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(t.g.current(a))]);
    t.resolve();
    assert_ne!(t.g.current(b), b);
    let a2 = t.g.current(a);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.g.current(a), a2, "declined");
    assert_eq!(t.named_on_battlefield("Wispweaver Angel").len(), 2);
}

#[test]
fn deadeye_navigator_paired_abilities() {
    cr!("702.95c", "113.7a", "400.7");
    ruling!(
        "Deadeye Navigator",
        "Once Deadeye Navigator or the creature it’s paired with is exiled, the other creature will no longer have the activated ability. However, you can activate the ability of one creature in response to activating the ability of the other creature."
    );
    supported("Deadeye Navigator");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let nav = t.enter(P0, "Deadeye Navigator");
    t.resolve_all();
    let has_flicker = |t: &TestGame, id: ObjectId| {
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .any(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("Exile"))
    };
    assert!(has_flicker(&t, bears) && has_flicker(&t, nav));
    // The Bears' ability, and the Navigator's in response: both resolve.
    activate_containing(&mut t, P0, bears, "Exile").unwrap();
    activate_containing(&mut t, P0, nav, "Exile").unwrap();
    assert_eq!(t.stack_len(), 2);
    // The Navigator returns unpaired (it doesn't pair again).
    t.answer_choose(P0, &[]);
    t.resolve();
    assert_ne!(t.g.current(nav), nav);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_ne!(t.g.current(bears), bears, "the Bears' ability still resolved");
    assert!(!has_flicker(&t, bears), "no longer paired");
}

#[test]
fn pegasus_guardian_counts_permanents_you_controlled() {
    cr!("603.4", "603.10a");
    ruling!(
        "Pegasus Guardian // Rescue the Foal",
        "Pegasus Guardian's triggered ability will trigger even if Pegasus Guardian wasn't under your control at the time the permanent you controlled left the battlefield."
    );
    supported("Pegasus Guardian // Rescue the Foal");
    let mut t = TestGame::new(2);
    let guardian = t.battlefield(P1, "Pegasus Guardian // Rescue the Foal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    give_control(&mut t, guardian, P0);
    through_end_step(&mut t, P0);
    let pegasi: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.controller == P0 && o.chars.has_subtype("Pegasus"))
        .collect();
    assert_eq!(pegasi.len(), 1, "{}", t.dump_log());
}

#[test]
fn ruin_ghost_flickers_a_land() {
    cr!("400.7", "110.2", "614.1c", "603.6a");
    ruling!(
        "Ruin Ghost",
        "As Ruin Ghost's ability resolves, the targeted land is exiled, then immediately returned to the battlefield. The land that enters is a different permanent from the one that left."
    );
    ruling!(
        "Ruin Ghost",
        "The land returns under your control, regardless of who owns it."
    );
    ruling!(
        "Ruin Ghost",
        "The returned land behaves like any other land that's put onto the battlefield. If it has an ability that says it enters tapped, it does so. (Otherwise it enters untapped.) If it has an \"enters\" triggered ability, that ability triggers."
    );
    supported("Ruin Ghost");
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Ruin Ghost");
    t.lands(P0, "Plains", 2);
    // A tapped Forest P1 owns, controlled by P0: it returns untapped, under P0's control.
    let forest = t.battlefield(P1, "Forest");
    give_control(&mut t, forest, P0);
    t.g.objects[forest.0 as usize].tapped = true;
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_containing(&mut t, P0, ghost, "Exile target land").unwrap();
    t.resolve_all();
    assert_ne!(t.g.current(forest), forest);
    assert!(t.on_battlefield(forest));
    assert!(!t.obj_now(forest).tapped);
    assert_eq!(t.obj_now(forest).controller, P0);
    assert_eq!(t.obj_now(forest).owner, P1);
    // Scoured Barrens ("This land enters tapped. When this land enters, you gain 1 life.").
    let mut t = TestGame::new(2);
    let ghost = t.battlefield(P0, "Ruin Ghost");
    t.lands(P0, "Plains", 1);
    let barrens = t.battlefield(P0, "Scoured Barrens");
    t.answer_targets(P0, &[Entity::Object(barrens)]);
    activate_containing(&mut t, P0, ghost, "Exile target land").unwrap();
    t.resolve_all();
    assert!(t.obj_now(barrens).tapped);
    assert_eq!(t.life(P0), 21);
}

/// P0's Gossip's Talent at level 3, with plenty of mana.
fn gossips_talent(t: &mut TestGame) -> ObjectId {
    mana(t, P0, 3);
    let g = t.battlefield(P0, "Gossip's Talent");
    gain_level(t, P0, g, 2).unwrap();
    t.resolve_all();
    gain_level(t, P0, g, 3).unwrap();
    t.resolve_all();
    g
}

#[test]
fn gossips_talent_unblockable_and_power_changes() {
    cr!("608.2b", "509.1b", "611.2c");
    ruling!(
        "Gossip's Talent",
        "If the target creature's power is increased to 4 or greater after Gossip's Talent's level 2 class ability triggers but before it resolves, the ability doesn't resolve. However, if instead the creature's power is increased to 4 or greater after the ability resolves, it still can't be blocked that turn."
    );
    supported("Gossip's Talent");
    for before in [true, false] {
        let mut t = TestGame::new(2);
        gossips_talent(&mut t);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Wood");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        if before {
            pump(&mut t, bears, 2);
        }
        t.resolve_all();
        if !before {
            pump(&mut t, bears, 2);
        }
        assert_eq!(t.pt(bears).0, 4);
        assert_eq!(
            legal_blocks(&mut t, P1, &[(wall, bears)]),
            before,
            "before: {before}"
        );
    }
}

fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    use mtg_engine::ability::*;
    crate::r_s05_common::run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![id])),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

#[test]
fn gossips_talent_combat_damage_triggers() {
    cr!("603.3b", "510.2", "506.4", "702.4c");
    ruling!(
        "Gossip's Talent",
        "Gossip's Talent's level 3 class ability triggers once for each creature you control that deals combat damage to a player during that combat damage step."
    );
    supported("Gossip's Talent");
    let mut t = TestGame::new(2);
    gossips_talent(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(bears, Entity::Player(P1)), (ogre, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.stack_len(), 2, "one trigger per creature");
    // Exile the Ogre, not the Bears.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let flickered = [bears, ogre]
        .iter()
        .filter(|id| t.g.current(**id) != **id)
        .count();
    assert_eq!(flickered, 1);

    ruling!(
        "Gossip's Talent",
        "If you choose to exile a creature with double strike when Gossip's Talent's level 3 class ability resolves, it won't be an attacking creature anymore when it returns to the battlefield, and it won't deal combat damage during the regular combat damage step."
    );
    let mut t = TestGame::new(2);
    gossips_talent(&mut t);
    let ace = t.battlefield(P0, "Fencing Ace");
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(ace, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 19);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_ne!(t.g.current(ace), ace);
    assert!(!crate::r_s10_common::attacking(&t, ace));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19, "no regular combat damage");
}
