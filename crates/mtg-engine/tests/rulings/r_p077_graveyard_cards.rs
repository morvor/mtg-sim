//! Rulings batch P077 — using creature cards in a graveyard: last known information of the
//! exiled card (CR 608.2h), copies of graveyard cards that aren't cast, scavenge for each
//! creature card, choices made on resolution, and graveyard-exile triggers.

use crate::r_s01_common::{supported, tokens, with_subtype};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::activate_named;
use crate::r_s06_common::activate_containing;
use crate::r_s21_common::castable;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Tarmogoyf in P0's graveyard with a Lightning Bolt: a 2/3 there (creature and instant
/// cards in graveyards), a 1/2 once it's exiled.
fn goyf_in_graveyard(t: &mut TestGame) -> ObjectId {
    t.graveyard(P0, "Lightning Bolt");
    let goyf = t.graveyard(P0, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (2, 3));
    goyf
}

#[test]
fn exiled_cards_power_and_toughness_as_they_last_existed_in_the_graveyard() {
    cr!("608.2h", "113.7a");
    interp!("exiled-card-power-as-it-last-existed-in-the-graveyard");
    ruling!(
        "Dread Defiler",
        "Use the card’s power as it last existed in your graveyard to determine how much life is lost."
    );
    ruling!(
        "Ritual of the Returned",
        "Use the creature card's power and toughness as it last existed in the graveyard to determine the power and toughness of the Zombie token."
    );
    ruling!(
        "Scour the Desert",
        "Scour the Desert checks the exiled card's toughness in the graveyard, not what it is in exile."
    );
    ruling!(
        "Young Deathclaws",
        "The number of counters that a card’s scavenge ability puts on a creature is based on the card’s power as it last existed in the graveyard."
    );
    // Dread Defiler: "{3}{C}, Exile a creature card from your graveyard: Target opponent
    // loses life equal to the exiled card's power."
    let mut t = TestGame::new(2);
    let defiler = t.battlefield(P0, "Dread Defiler");
    t.lands(P0, "Wastes", 4);
    goyf_in_graveyard(&mut t);
    activate_containing(&mut t, P0, defiler, "loses life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Ritual of the Returned: a 2/3 Zombie.
    let mut t = TestGame::new(2);
    let goyf = goyf_in_graveyard(&mut t);
    crate::r_s25_common::cast_new(
        &mut t,
        P0,
        "Ritual of the Returned",
        &[Entity::Object(goyf)],
    );
    t.resolve_all();
    let z = with_subtype(&t, P0, "Zombie");
    assert_eq!(t.pt(z[0]), (2, 3));
    // Scour the Desert: three Birds.
    let mut t = TestGame::new(2);
    let goyf = goyf_in_graveyard(&mut t);
    crate::r_s25_common::cast_new(&mut t, P0, "Scour the Desert", &[Entity::Object(goyf)]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 3);
    // Young Deathclaws' scavenge: two counters.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Deathclaws");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goyf = goyf_in_graveyard(&mut t);
    t.lands(P0, "Forest", 2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, goyf, "Scavenge", 0).expect("scavenge");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
}

#[test]
fn a_creature_token_isnt_a_creature_card_in_the_graveyard() {
    cr!("111.7", "704.5d");
    ruling!(
        "Dread Defiler",
        "A creature token is not a creature card. Although a token does go to the graveyard briefly if it dies, it ceases to exist before you have the opportunity to do anything."
    );
    supported("Dread Defiler");
    let mut t = TestGame::new(2);
    let defiler = t.battlefield(P0, "Dread Defiler");
    t.lands(P0, "Wastes", 4);
    let token = crate::r_s02_common::create_token(&mut t, P0, "Zombie");
    destroy(&mut t, token);
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(!can_activate(&mut t, P0, defiler));
}

#[test]
fn back_from_the_brink_isnt_casting() {
    cr!("601.2f", "118.9", "707.2");
    ruling!(
        "Back from the Brink",
        "Although you're paying the card's mana cost, you aren't casting that card. Abilities that reduce the cost to cast a creature spell won't apply"
    );
    supported("Back from the Brink");
    // Heartless Summoning: "Creature spells you cast cost {2} less to cast."
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    t.battlefield(P0, "Heartless Summoning");
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    assert!(
        t.activate(P0, brink, 0, &[]).is_err(),
        "{{3}}{{R}} isn't reduced"
    );
    t.lands(P0, "Mountain", 2);
    t.activate(P0, brink, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.g.history.spells_cast.is_empty(), "no spell was cast");
}

#[test]
fn back_from_the_brink_token_enters_with_its_abilities() {
    cr!("603.6a", "614.1c", "707.2");
    ruling!(
        "Back from the Brink",
        "Any \"enters\" abilities of the creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the creature will also work."
    );
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    t.graveyard(P0, "Elvish Visionary");
    t.lands(P0, "Forest", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, brink, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Kalonian Hydra: "Kalonian Hydra enters with four +1/+1 counters on it."
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    t.graveyard(P0, "Kalonian Hydra");
    t.lands(P0, "Forest", 5);
    t.activate(P0, brink, 0, &[]).unwrap();
    t.resolve_all();
    let hydra = t.named_on_battlefield("Kalonian Hydra")[0];
    assert!(t.obj(hydra).is_token());
    assert_eq!(t.counters(hydra, "+1/+1"), 4);
}

#[test]
fn back_from_the_brink_copies_a_double_faced_cards_front_face() {
    cr!("707.8a", "712.8a", "701.27a");
    ruling!(
        "Back from the Brink",
        "If you exile a double-faced creature card this way, you'll pay the mana cost of the front face. The token will be a copy of the front face"
    );
    // The rest of the ruling ("and it won't be able to transform", 2011) is outdated: since
    // the CR effective 2023-04-14, such a token is a double-faced token that can
    // transform (CR 707.8a; docs/RULING_DISCREPANCIES.md).
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    t.graveyard(P0, "Delver of Secrets // Insectile Aberration");
    let island = t.battlefield(P0, "Island");
    t.activate(P0, brink, 0, &[]).unwrap();
    assert!(t.obj(island).tapped, "{{U}} paid");
    t.resolve_all();
    let delver = t.named_on_battlefield("Delver of Secrets")[0];
    assert!(t.obj(delver).is_token());
    assert_eq!(t.pt(delver), (1, 1));
    crate::r_s17_common::transform(&mut t, delver);
    assert_eq!(
        crate::r_s25_common::name_now(&t, delver),
        "Insectile Aberration"
    );
    assert_eq!(t.pt(delver), (3, 2));
}

#[test]
fn young_deathclaws_scavenge_costs_and_two_instances() {
    cr!("702.97a", "113.2c");
    ruling!(
        "Young Deathclaws",
        "While Young Deathclaws is on the battlefield, the scavenge cost of a creature card in your graveyard is equal to its mana cost, including any colored mana requirements."
    );
    ruling!(
        "Young Deathclaws",
        "If a creature card has multiple instances of scavenge, you can activate either ability (but not both, as the card will be exiled when you activate one of them). Young Deathclaws’s ability doesn’t affect the scavenge cost of a creature’s other scavenge ability."
    );
    supported("Young Deathclaws");
    // Hill Giant ({3}{R}): four Swamps can't pay it; a Mountain and three Swamps can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Deathclaws");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(activate_named(&mut t, P0, giant, "Scavenge", 0).is_err());
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Deathclaws");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_named(&mut t, P0, giant, "Scavenge", 0).expect("scavenge for {3}{R}");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    // Dreg Mangler ({1}{B}{G}; scavenge {3}{B}{G}): two scavenge abilities. Three lands pay
    // the granted one, not its own.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Deathclaws");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mangler = t.graveyard(P0, "Dreg Mangler");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    t.g.recompute();
    // (uid, mana cost) of each scavenge ability.
    let scavenges: Vec<(u64, String)> = t
        .obj(mangler)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            mtg_engine::ability::AbilityKind::Activated(act) if a.text.starts_with("Scavenge") => {
                Some((
                    a.uid,
                    format!("{}", act.cost.mana.clone().unwrap_or_default()),
                ))
            }
            _ => None,
        })
        .collect();
    assert_eq!(scavenges.len(), 2, "{scavenges:?}");
    let activate = |t: &mut TestGame, uid: u64| {
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.g.turn.priority = Some(P0);
        let r = t.g.activate_ability(P0, t.g.current(mangler), uid);
        t.g.flush_events();
        r
    };
    let own = scavenges
        .iter()
        .find(|s| s.1 == "{3}{B}{G}")
        .expect("its own scavenge");
    // The granted one: "the scavenge cost is equal to its mana cost" ({1}{B}{G}).
    let granted = scavenges
        .iter()
        .find(|s| s.0 != own.0)
        .expect("the granted one");
    // Its own scavenge ({3}{B}{G}) can't be paid with three lands.
    assert!(activate(&mut t, own.0).is_err());
    t.clear_answers();
    activate(&mut t, granted.0).expect("the granted scavenge");
    // The card is exiled: neither can be activated again.
    assert!(t.in_exile("Dreg Mangler"));
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

#[test]
fn ritual_of_the_returned_with_its_target_gone_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Ritual of the Returned",
        "If the creature card leaves the graveyard in response to Ritual of the Returned, Ritual of the Returned won't resolve and none of its effects will happen. You won't get a Zombie token."
    );
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    crate::r_s25_common::cast_new(
        &mut t,
        P0,
        "Ritual of the Returned",
        &[Entity::Object(giant)],
    );
    crate::r_s05_common::move_to(&mut t, giant, Zone::Hand(P0));
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
}

/// The "you may exile a creature card" decisions (yes/no or choosing the card) asked of
/// P0 since `from`.
fn resolution_choices(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| {
            *p == P0 && matches!(d, Decision::ChooseEntities { .. } | Decision::YesNo { .. })
        })
        .count()
}

#[test]
fn master_skald_and_masked_vandal_target_on_stack_exile_on_resolution() {
    cr!("603.3d", "608.2c", "608.2d");
    ruling!(
        "Master Skald",
        "You choose the target artifact or enchantment card in your graveyard as you put the triggered ability on the stack. Once the ability starts resolving, you choose which creature card to exile from your graveyard, if any. At that point, no one can respond to your choice or take any actions until the ability is finished resolving."
    );
    ruling!(
        "Masked Vandal",
        "You choose the target artifact or enchantment an opponent controls as you put the triggered ability on the stack. Once the ability starts resolving, you choose which creature card to exile from your graveyard, if any. At that point, no one can respond to your choice or take any actions until the ability is finished resolving."
    );
    // (card, target in P0's graveyard or P1's battlefield)
    for (name, yard_target) in [("Master Skald", true), ("Masked Vandal", false)] {
        supported(name);
        let mut t = TestGame::new(2);
        let target = if yard_target {
            t.graveyard(P0, "Sol Ring")
        } else {
            t.battlefield(P1, "Sol Ring")
        };
        t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.answer_yes(P0, true);
        t.enter(P0, name);
        let from = t.asked().len();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let top = crate::r_s04_common::top_of_stack(&t);
        assert_eq!(
            crate::r_s25_common::targets_of(&t, top),
            vec![Entity::Object(target)],
            "{name}"
        );
        assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
        // The creature card isn't chosen yet: that happens as it resolves, which then
        // finishes without anyone getting priority.
        assert_eq!(resolution_choices(&t, from), 0, "{name}");
        t.resolve();
        assert!(resolution_choices(&t, from) > 0, "{name}");
        assert!(t.in_exile("Grizzly Bears"), "{name}");
        if yard_target {
            assert!(t.in_hand(P0, "Sol Ring"));
        } else {
            assert!(t.in_exile("Sol Ring"));
        }
    }
}

#[test]
fn master_skald_exiling_its_own_target_returns_nothing() {
    cr!("608.2b", "400.7");
    ruling!(
        "Master Skald",
        "If you choose an artifact creature card or an enchantment creature card in your graveyard as the target, you could exile that card as the triggered ability is resolving, but if you do, you won’t return it to your hand."
    );
    let mut t = TestGame::new(2);
    let thopter = t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.enter(P0, "Master Skald");
    t.resolve_all();
    assert!(t.in_exile("Ornithopter"));
    assert!(!t.in_hand(P0, "Ornithopter"));
}

#[test]
fn rank_officer_in_two_headed_giant_and_one_discard() {
    cr!("810.8a", "608.2d");
    ruling!(
        "Rank Officer",
        "In a Two-Headed Giant game, Rank Officer’s last ability causes the opposing team to lose 4 life"
    );
    ruling!(
        "Rank Officer",
        "While resolving Rank Officer’s triggered ability, you can’t discard multiple cards to create multiple Zombie tokens."
    );
    supported("Rank Officer");
    let mut t = crate::r_s33_common::two_headed_giant();
    let officer = t.battlefield(P0, "Rank Officer");
    t.lands(P0, "Swamp", 2);
    t.graveyard(P0, "Grizzly Bears");
    let foe = t.g.opponents(P0)[0];
    let before = t.life(foe);
    activate_containing(&mut t, P0, officer, "loses 2 life").unwrap();
    t.resolve_all();
    assert_eq!(t.life(foe), before - 4);

    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Forest");
    let b = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Rank Officer");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(
        with_subtype(&t, P0, "Zombie").len(),
        2,
        "Rank Officer and one token"
    );
}

#[test]
fn god_pharaohs_gift_chooses_on_resolution_and_makes_a_4_4_black_zombie() {
    cr!("608.2c", "707.9b", "707.2");
    ruling!(
        "God-Pharaoh's Gift",
        "The ability of God-Pharaoh's Gift doesn't target the creature card you'll exile. You choose one as the ability resolves. No player may take actions between the time you choose a creature card to exile and the time you create the token."
    );
    ruling!(
        "God-Pharaoh's Gift",
        "The token is a Zombie instead of its other types (unlike Zombies created by an eternalize ability) and is black instead of its other colors. Its power and toughness are 4/4. These are copiable values of the token that other effects may copy."
    );
    ruling!(
        "God-Pharaoh's Gift",
        "Unlike the tokens created by an eternalize ability, this token has the mana cost and thus mana value of the card it's copying."
    );
    supported("God-Pharaoh's Gift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Pharaoh's Gift");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.settle();
    let top = crate::r_s04_common::top_of_stack(&t);
    assert!(crate::r_s25_common::targets_of(&t, top).is_empty());
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve();
    assert_eq!(crate::r_s08_common::priority_asks_of(&t, P1, from), 0);
    let token = t.named_on_battlefield("Hill Giant")[0];
    let o = t.obj(token);
    assert!(o.is_token());
    assert_eq!(t.pt(token), (4, 4));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Black));
    assert!(o.chars.has_subtype("Zombie"));
    assert!(!o.chars.has_subtype("Giant"));
    assert_eq!(crate::r_s08_common::mana_value(&t, token), 4);
    assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    // A Clone copying it: a 4/4 black Zombie Hill Giant with that mana cost, no haste.
    t.answer_choose(P0, &[Entity::Object(token)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    let c = t.obj_now(clone);
    assert_eq!(c.chars.name, "Hill Giant");
    assert_eq!(t.pt(clone), (4, 4));
    assert_eq!(c.chars.colors, ColorSet::single(Color::Black));
    assert!(c.chars.has_subtype("Zombie") && !c.chars.has_subtype("Giant"));
    assert_eq!(crate::r_s08_common::mana_value(&t, clone), 4);
    assert!(!c.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn drudge_spell_destroys_every_skeleton_token_there_when_it_leaves() {
    cr!("603.6c", "701.19c");
    ruling!(
        "Drudge Spell",
        "The fact that Skeleton tokens are destroyed is an aspect of the enchantment and not of the tokens. Thus, only Skeleton tokens which are on the battlefield when it leaves the battlefield are destroyed. Ones which are going to appear due to use of the ability but which have not yet appeared are safe."
    );
    ruling!(
        "Drudge Spell",
        "This destroys all Skeleton tokens from all sources if it leaves the battlefield. It does not just destroy ones generated by this Drudge Spell or just ones you control."
    );
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Drudge Spell");
    t.lands(P0, "Swamp", 4);
    crate::r_s07_common::graveyard_n(&mut t, P0, "Grizzly Bears", 4);
    activate_containing(&mut t, P0, ds, "Skeleton").unwrap();
    t.resolve_all();
    let first = with_subtype(&t, P0, "Skeleton");
    assert_eq!(first.len(), 1);
    // Another player's Skeleton token from another source.
    let theirs = crate::r_s02_common::create_token(&mut t, P1, "Skeleton");
    // A second activation on the stack as Drudge Spell leaves.
    activate_containing(&mut t, P0, ds, "Skeleton").unwrap();
    destroy(&mut t, ds);
    t.resolve_all();
    assert!(!t.on_battlefield(first[0]));
    assert!(!t.on_battlefield(theirs));
    let left = with_subtype(&t, P0, "Skeleton");
    assert_eq!(left.len(), 1, "the one not yet created is safe");
}

#[test]
fn sanctifier_of_souls_ability_works_only_on_the_battlefield() {
    cr!("113.6", "602.2");
    ruling!(
        "Sanctifier of Souls",
        "The last ability functions only while Sanctifier of Souls is on the battlefield. You can’t activate it while Sanctifier of Souls is in your graveyard."
    );
    supported("Sanctifier of Souls");
    let mut t = TestGame::new(2);
    let s = t.graveyard(P0, "Sanctifier of Souls");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    assert!(!can_activate(&mut t, P0, s));
    let s = t.battlefield(P0, "Sanctifier of Souls");
    assert!(can_activate(&mut t, P0, s));
}

#[test]
fn waste_management_makes_rogues_kicked_or_not() {
    cr!("702.33d", "608.2h");
    ruling!(
        "Waste Management",
        "Waste Management's controller will create a 2/2 black Rogue creature token for each creature card exiled this way whether it was kicked or not."
    );
    supported("Waste Management");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Forest");
    let wm = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Waste Management");
    t.cast(P0, wm)
        .kicked(false)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Rogue").len(), 1);
    // Kicked: target player's graveyard.
    let mut t = TestGame::new(2);
    crate::r_s07_common::graveyard_n(&mut t, P1, "Grizzly Bears", 3);
    t.graveyard(P1, "Forest");
    t.lands(P0, "Swamp", 7);
    let wm = t.hand(P0, "Waste Management");
    t.cast(P0, wm).kicked(true).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(with_subtype(&t, P0, "Rogue").len(), 3);
    let _ = tokens(&t, P0);
}

#[test]
fn scholar_of_combustion_card_follows_timing_and_costs() {
    cr!("307.1", "601.2f");
    ruling!(
        "Scholar of Combustion",
        "You pay all costs and follow all timing rules for the instant or sorcery card cast with the permission granted by Scholar of Combustion's ability. For example, if the exiled card is a sorcery, you may cast it only during your main phase while the stack is empty."
    );
    supported("Scholar of Combustion");
    let mut t = TestGame::new(2);
    let axe = t.graveyard(P0, "Lava Axe");
    t.answer_targets(P0, &[Entity::Object(axe)]);
    t.enter(P0, "Scholar of Combustion");
    t.resolve_all();
    let axe = t.g.current(axe);
    assert_eq!(t.zone(axe), Zone::Exile);
    assert!(!castable(&mut t, P0, axe), "no mana");
    t.lands(P0, "Mountain", 5);
    assert!(castable(&mut t, P0, axe));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, axe));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, axe));
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, axe).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    let _ = Answer::Default;
}
