//! Rulings batch P107 — mana rocks and mana creatures with a second ability: what their
//! mana can pay for, paying a cost with the source's own mana ability, sacrifice and tap
//! costs (CR 106, 118.3, 602.2, 605, 302.6).

use crate::r_p107_common::*;
use crate::r_s17_common::token_copy;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn pyromancers_goggles_mana_can_pay_for_anything() {
    cr!("106.6");
    ruling!(
        "Pyromancer's Goggles",
        "The mana produced by Pyromancer's Goggles can be spent on anything, not just a red instant or sorcery spell."
    );
    // {R} from the Goggles and {G} pay for Grizzly Bears ({1}{G}); nothing is copied.
    let mut t = TestGame::new(2);
    let goggles = t.battlefield(P0, "Pyromancer's Goggles");
    assert!(tap_for_mana(&mut t, P0, goggles, "Add {R}"));
    mana(&mut t, P0, ManaType::G, 1);
    let bears = cast_from_hand(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    let _ = bears;
}

#[test]
fn the_ramos_artifacts_can_be_sacrificed_while_tapped() {
    cr!("602.2", "605.1a");
    ruling!(
        "Eye of Ramos",
        "You can activate the sacrifice ability while this card is tapped."
    );
    ruling!(
        "Heart of Ramos",
        "You can activate the sacrifice ability while this card is tapped."
    );
    ruling!(
        "Horn of Ramos",
        "You can activate the sacrifice ability while this card is tapped."
    );
    ruling!(
        "Skull of Ramos",
        "You can activate the sacrifice ability while this card is tapped."
    );
    ruling!(
        "Tooth of Ramos",
        "You can activate the sacrifice ability while this card is tapped."
    );
    for (name, ty) in [
        ("Eye of Ramos", ManaType::U),
        ("Heart of Ramos", ManaType::R),
        ("Horn of Ramos", ManaType::G),
        ("Skull of Ramos", ManaType::B),
        ("Tooth of Ramos", ManaType::W),
    ] {
        let mut t = TestGame::new(2);
        let ramos = t.battlefield(P0, name);
        assert!(tap_for_mana(&mut t, P0, ramos, "{T}"), "{name}");
        assert!(t.obj_now(ramos).tapped);
        assert!(tap_for_mana(&mut t, P0, ramos, "Sacrifice"), "{name}");
        assert!(t.in_graveyard(P0, name));
        assert_eq!(t.g.player(P0).mana_pool.count(ty), 2, "{name}");
    }
}

#[test]
fn midnight_clocks_mana_pays_for_its_own_hour_counter() {
    cr!("605.3a", "601.2g");
    ruling!(
        "Midnight Clock",
        "You can activate Midnight Clock's mana ability to pay the cost of its second ability."
    );
    // {2} in the pool and no other blue source: the Clock taps for the {U}.
    let mut t = TestGame::new(2);
    let clock = t.battlefield(P0, "Midnight Clock");
    mana(&mut t, P0, ManaType::C, 2);
    act(&mut t, P0, clock, "hour counter", &[]).unwrap();
    assert!(t.obj_now(clock).tapped);
    t.resolve_all();
    assert_eq!(t.counters(clock, "hour"), 1);
}

/// An Eldrazi Scion created by Adverse Conditions cast with no targets.
fn scion_from_adverse_conditions(t: &mut TestGame) -> ObjectId {
    mana(t, P0, ManaType::U, 1);
    mana(t, P0, ManaType::C, 3);
    let spell = cast_from_hand(t, P0, "Adverse Conditions", &[]);
    t.resolve_all();
    assert!(resolved(t, spell));
    let scions = with_subtype(t, P0, "Scion");
    assert_eq!(scions.len(), 1);
    scions[0]
}

#[test]
fn drowner_of_hope_sacrifices_any_scion_and_gets_no_mana_for_it() {
    cr!("118.3", "602.2b");
    ruling!(
        "Drowner of Hope",
        "You can sacrifice any Eldrazi Scion to activate Drowner of Hope's last ability, not just one created by Drowner of Hope."
    );
    ruling!(
        "Drowner of Hope",
        "You can't sacrifice the same permanent to pay two different costs."
    );
    let mut t = TestGame::new(2);
    let drowner = t.battlefield(P0, "Drowner of Hope");
    let scion = scion_from_adverse_conditions(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, drowner, "Tap target creature", &[obj(giant)]).unwrap();
    assert!(!t.g.is_live(scion));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(pool_total(&t, P0), 0);
    assert!(with_subtype(&t, P0, "Scion").is_empty());
}

#[test]
fn relic_of_legends_can_tap_a_summoning_sick_legendary_creature() {
    cr!("302.6", "118.3");
    ruling!(
        "Relic of Legends",
        "You can tap any untapped legendary creature you control, including one you haven't controlled continuously"
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Relic of Legends");
    let isamaru = t.battlefield_sick(P0, "Isamaru, Hound of Konda");
    t.battlefield_sick(P0, "Grizzly Bears");
    assert!(tap_for_mana(&mut t, P0, relic, "legendary"));
    assert!(t.obj_now(isamaru).tapped);
    assert!(!t.obj_now(relic).tapped);
    assert_eq!(pool_total(&t, P0), 1);
    // No other untapped legendary creature: it can't be activated again.
    assert!(!tap_for_mana(&mut t, P0, relic, "legendary"));
}

#[test]
fn victory_chimes_also_untaps_in_its_controllers_untap_step() {
    cr!("502.3");
    ruling!(
        "Victory Chimes",
        "Victory Chimes also untaps as normal during your untap step."
    );
    let mut t = TestGame::new(2);
    let chimes = t.battlefield(P0, "Victory Chimes");
    t.advance_to(P1, Step::Upkeep);
    t.g.tap(chimes);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(chimes).tapped);
    // And during the other player's untap step.
    t.g.tap(chimes);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(chimes).tapped);
}

#[test]
fn goldhound_is_a_treasure_artifact_and_a_dog_creature() {
    cr!("205.3g", "205.3m", "302.6");
    ruling!(
        "Goldhound",
        "Even when it is on a creature, Treasure is an artifact type and not a creature type."
    );
    ruling!(
        "Goldhound",
        "Since Goldhound is a creature, its {T} ability can't be activated the turn that it enters"
    );
    assert_eq!(subtype_kind("Treasure"), Some(SubtypeKind::Artifact));
    assert_eq!(subtype_kind("Dog"), Some(SubtypeKind::Creature));
    let mut t = TestGame::new(2);
    let hound = t.battlefield_sick(P0, "Goldhound");
    let o = t.obj_now(hound);
    assert!(o.chars.has_subtype("Treasure") && o.chars.has_subtype("Dog"));
    // Summoning sick: its {T} mana ability can't be activated; with haste it can.
    assert!(!tap_for_mana(&mut t, P0, hound, "Sacrifice"));
    assert!(t.on_battlefield(hound));
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Goldhound");
    assert!(tap_for_mana(&mut t, P0, hound, "Sacrifice"));
    assert!(!t.on_battlefield(hound));
}

#[test]
fn parcel_myr_entering_isnt_creating_a_clue_but_a_token_copy_is() {
    cr!("111.2", "701.7a", "707.12");
    ruling!(
        "Parcel Myr",
        "Even though it is a Clue, Parcel Myr is not (normally) a token, and therefore isn't “created”"
    );
    // Doubling Season: "If an effect would create one or more tokens under your control,
    // it creates twice that many of those tokens instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let myr = t.enter(P0, "Parcel Myr");
    t.settle();
    assert_eq!(t.named_on_battlefield("Parcel Myr").len(), 1);
    let copies = token_copy(&mut t, P0, myr);
    assert_eq!(copies.len(), 2);
    // A copy of the Parcel Myr spell resolves into a token that isn't created.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::C, 1);
    mana(&mut t, P0, ManaType::G, 1);
    let spell = cast_from_hand(&mut t, P0, "Parcel Myr", &[]);
    cast_from_hand(&mut t, P0, "Double Major", &[obj(spell)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Parcel Myr").len(), 2);
    assert_eq!(
        t.named_on_battlefield("Parcel Myr")
            .iter()
            .filter(|id| t.obj(**id).is_token())
            .count(),
        1
    );
}

#[test]
fn parcel_myr_sacrificed_for_its_own_ability_is_one_clue_sacrificed() {
    cr!("118.3", "701.21a");
    ruling!(
        "Parcel Myr",
        "You can't sacrifice Parcel Myr to activate its own ability and also to activate another ability"
    );
    // Ashnod's Altar ("Sacrifice a creature: Add {C}{C}") has nothing left to sacrifice
    // once Parcel Myr paid its own cost; Thorough Investigation sees one Clue sacrificed.
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Thorough Investigation");
    let myr = t.battlefield(P0, "Parcel Myr");
    mana(&mut t, P0, ManaType::C, 2);
    act(&mut t, P0, myr, "Draw a card", &[]).unwrap();
    assert!(!t.g.is_live(myr));
    assert!(!tap_for_mana(&mut t, P0, altar, "Sacrifice a creature"));
    t.settle();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "venture"), 1);
}


#[test]
fn spectral_searchlight_may_choose_its_controller() {
    cr!("106.4", "605.1a");
    ruling!("Spectral Searchlight", "You may choose yourself.");
    supported("Spectral Searchlight");
    supported("Victory Chimes");
    // The chosen player chooses the color: P0 chooses P0 (then green), or P1 (who
    // chooses red).
    for (who, color) in [(P0, ManaType::G), (P1, ManaType::R)] {
        let mut t = TestGame::new(2);
        let light = t.battlefield(P0, "Spectral Searchlight");
        t.answer_choose(P0, &[Entity::Player(who)]);
        let ci = [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G]
            .iter()
            .position(|c| *c == color)
            .unwrap();
        t.answer(
            who,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(ci),
        );
        activate_containing(&mut t, P0, light, "Choose a player").unwrap();
        t.resolve_all();
        assert_eq!(t.g.player(who).mana_pool.count(color), 1, "{who:?}");
        assert_eq!(pool_total(&t, P0) + pool_total(&t, P1), 1);
        assert!(t.obj_now(light).tapped);
    }
    // Victory Chimes: "{T}: A player of your choice adds {C}."
    let mut t = TestGame::new(2);
    let chimes = t.battlefield(P0, "Victory Chimes");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, chimes, "of your choice").unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::C), 1);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn boneyard_desecrator_checks_the_sacrificed_creatures_last_known_types() {
    cr!("608.2h", "700.12");
    ruling!(
        "Boneyard Desecrator",
        "Use the creature types of the sacrificed creature as it last existed on the battlefield to determine whether or not it was an outlaw."
    );
    supported("Boneyard Desecrator");
    supported("Thallid Omnivore");
    // A Rogue (outlaw) gives a Treasure; a Grizzly Bears doesn't.
    for (fodder, treasure) in [("Grizzly Bears", 0), ("Kitesail Freebooter", 1)] {
        let mut t = TestGame::new(2);
        let desecrator = t.battlefield(P0, "Boneyard Desecrator");
        let f = t.battlefield(P0, fodder);
        mana(&mut t, P0, ManaType::B, 1);
        mana(&mut t, P0, ManaType::C, 1);
        t.answer_choose(P0, &[obj(f)]);
        act(&mut t, P0, desecrator, "outlaw", &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.counters(desecrator, counters::PLUS1), 1, "{fodder}");
        assert_eq!(with_subtype(&t, P0, "Treasure").len(), treasure, "{fodder}");
    }
    // A creature that was a Rogue only on the battlefield (Xenograft: "As this enchantment
    // enters, choose a creature type. Each creature you control is the chosen type in
    // addition to its other types.") counts: its types as it last existed.
    let mut t = TestGame::new(2);
    let desecrator = t.battlefield(P0, "Boneyard Desecrator");
    crate::r_p125_common::choose_creature_type(&mut t, P0, "Rogue");
    t.enter(P0, "Xenograft");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(bear).chars.has_subtype("Rogue"));
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(bear)]);
    act(&mut t, P0, desecrator, "outlaw", &[]).unwrap();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    // Thallid Omnivore: "If a Saproling was sacrificed this way, you gain 2 life."
    let mut t = TestGame::new(2);
    let omni = t.battlefield(P0, "Thallid Omnivore");
    let sap = crate::r_s02_common::create_token(&mut t, P0, "Saproling");
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(sap)]);
    act(&mut t, P0, omni, "Saproling", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(omni), (5, 5));
}
