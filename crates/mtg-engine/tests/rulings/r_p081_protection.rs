//! Rulings batch P081 — protection from colors and "can't be the target of" effects on
//! creatures that punish black, red and blue decks: what protection stops (CR 702.16),
//! targeting restrictions that affect the game rules rather than granting abilities
//! (CR 608.2b, 115.5), replacement effects of Sanctifier en-Vec (CR 614.1a, 704.3),
//! counting colored permanents, and black cards changing zones (Compost).

use crate::r_p081_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Whether `p` could cast the spell `name` targeting `target` now.
fn targetable(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) -> bool {
    let target = t.g.current(target);
    casts_targeting(t, p, name, obj(target))
}

#[test]
fn great_sable_stag_cant_be_equipped_by_a_black_equipment() {
    cr!("702.16d", "704.5n", "702.16b");
    ruling!(
        "Great Sable Stag",
        "Although the reminder text doesn’t say so, Great Sable Stag also can’t be equipped by blue or black Equipment."
    );
    supported("Great Sable Stag");
    supported("Mask of Griselbrand");
    let mut t = TestGame::new(2);
    let stag = t.battlefield(P0, "Great Sable Stag");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Mask of Griselbrand ({1}{B}{B}) can't target it with equip, and can't be attached
    // to it by an effect either.
    let mask = t.battlefield(P0, "Mask of Griselbrand");
    t.lands(P0, "Wastes", 3);
    assert!(!ability_targets(&mut t, mask, 0).contains(&obj(stag)));
    assert!(ability_targets(&mut t, mask, 0).contains(&obj(bears)));
    assert!(!t.g.attach(mask, obj(stag)));
    t.settle();
    assert_eq!(t.obj_now(mask).attached_to, None);
    // A colorless Equipment is fine.
    let axe = t.battlefield(P0, "Bonesplitter");
    assert!(ability_targets(&mut t, axe, 0).contains(&obj(stag)));
}

#[test]
fn great_sable_stag_is_targeted_by_counterspells_whose_other_effects_still_happen() {
    cr!("113.6g", "608.2c", "115.1a");
    ruling!(
        "Great Sable Stag",
        "Great Sable Stag can be targeted by spells that try to counter it (such as Cancel). Those spells will resolve, but the part of their effect that would counter Great Sable Stag won’t do anything. Any other effects those spells have will work as normal."
    );
    supported("Dissolve");
    let mut t = TestGame::new(2);
    let stag = cast_new(&mut t, P0, "Great Sable Stag", &[]);
    assert!(targetable(&mut t, P1, "Cancel", stag));
    // Dissolve: "Counter target spell. Scry 1."
    let from = mark(&t);
    cast_new(&mut t, P1, "Dissolve", &[obj(stag)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Dissolve"), "it resolved");
    let scried = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Scry { .. }));
    assert!(scried);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(stag)));
}

#[test]
fn autumns_veil_is_a_rule_about_your_spells_and_creatures_not_an_ability() {
    cr!("611.2c", "608.2b", "115.5", "702.16a");
    ruling!(
        "Autumn's Veil",
        "Autumn’s Veil affects any spells and creatures you happen to control at any point during the rest of the turn, not just spells and creatures you control as it resolves. That’s because it doesn’t grant an ability to those spells or creatures; rather, it affects the game rules and states something that’s now true about those spells and creatures."
    );
    ruling!(
        "Autumn's Veil",
        "Autumn’s Veil has no effect on abilities. After it resolves, spells you control may be countered by abilities from blue or black sources, and creatures you control may be targeted by abilities from blue or black sources."
    );
    ruling!(
        "Autumn's Veil",
        "Autumn’s Veil itself may be countered by blue or black spells. Its effect doesn’t apply until after it resolves."
    );
    ruling!(
        "Display of Dominance",
        "If you choose the second mode, Display of Dominance will affect any permanent you happen to control at any point during the rest of the turn, not just permanents you control as it resolves. That’s because it doesn’t grant an ability to those permanents; rather, it affects the game rules and states something that’s now true about those permanents."
    );
    ruling!(
        "Display of Dominance",
        "Permanents you control may be the targets of abilities from blue or black sources controlled by your opponents."
    );
    supported("Autumn's Veil");
    supported("Display of Dominance");
    supported("Prodigal Sorcerer");
    // Veil, then a creature cast afterwards: Counterspell targets it but doesn't counter
    // it; Unsummon can't target the creature; the blue Prodigal Sorcerer's ability can.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Autumn's Veil", &[]);
    t.resolve_all();
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Counterspell"));
    assert_eq!(t.stack_len(), 1, "not countered");
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert!(!targetable(&mut t, P1, "Unsummon", bears));
    let tim = t.battlefield(P1, "Prodigal Sorcerer");
    assert!(ability_targets(&mut t, tim, 0).contains(&obj(bears)));
    // Display of Dominance's second mode, the same way (permanents).
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Display of Dominance");
    let card = t.hand(P0, "Display of Dominance");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P0, card).go();
    t.resolve_all();
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(!targetable(&mut t, P1, "Unsummon", bears));
    let tim = t.battlefield(P1, "Prodigal Sorcerer");
    assert!(ability_targets(&mut t, tim, 0).contains(&obj(bears)));
    // The Veil itself is countered by Counterspell.
    let mut t = TestGame::new(2);
    let veil = cast_new(&mut t, P0, "Autumn's Veil", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(veil)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Autumn's Veil"));
    // Without the Veil, Counterspell counters the Bears.
    let mut t = TestGame::new(2);
    let s = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(s)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_spell_already_targeting_your_creature_fizzles_when_the_veil_resolves() {
    cr!("608.2b", "115.5");
    ruling!(
        "Autumn's Veil",
        "If a creature you control is being targeted by a spell when Autumn’s Veil resolves, nothing happens right away. When that spell would resolve, its color is checked. If it’s blue or black, that creature will be an illegal target for that spell, and will be unable to be affected by it. If all that spell’s targets have become illegal by the time it would resolve, it won’t resolve."
    );
    ruling!(
        "Display of Dominance",
        "If you choose the second mode, and if a permanent you control is being targeted by a spell when Display of Dominance resolves, nothing happens right away. When that spell would resolve, its color is checked. If it’s blue or black, that permanent will be an illegal target for that spell and won’t be affected by it. If all that spell’s targets have become illegal by the time it would resolve, it’s countered."
    );
    ruling!(
        "Display of Dominance",
        "If you choose the second mode, no new blue or black spell may be cast by an opponent that turn targeting a permanent you control after Display of Dominance resolves."
    );
    for name in ["Autumn's Veil", "Display of Dominance"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let unsummon = cast_new(&mut t, P1, "Unsummon", &[obj(bears)]);
        lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        if name == "Display of Dominance" {
            t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        }
        t.cast(P0, card).go();
        t.resolve();
        assert!(t.on_battlefield(bears), "{name}: nothing happens right away");
        assert_eq!(t.stack_len(), 1);
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}: Unsummon didn't resolve");
        assert_eq!(t.zone(unsummon), Zone::Graveyard(P1), "{name}");
        // No new Unsummon may target it.
        assert!(!targetable(&mut t, P1, "Unsummon", bears), "{name}");
        // A green spell may (Giant Growth from P1).
        assert!(targetable(&mut t, P1, "Giant Growth", bears), "{name}");
    }
}

#[test]
fn an_aura_spell_targets_but_an_attached_aura_doesnt() {
    cr!("303.4a", "303.4b", "115.1a");
    ruling!(
        "Autumn's Veil",
        "Keep in mind that an Aura spell targets the permanent it will enchant (but an Aura on the battlefield doesn’t target the permanent it’s attached to)."
    );
    ruling!(
        "Display of Dominance",
        "Keep in mind that an Aura spell targets the permanent it will enchant (but an Aura on the battlefield doesn’t target the permanent it’s attached to)."
    );
    supported("Claustrophobia");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    // P1's Claustrophobia (blue Aura) already enchants the Bears.
    let aura = crate::r_s06_common::attach_new(&mut t, P1, "Claustrophobia", bears);
    cast_new(&mut t, P0, "Autumn's Veil", &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(aura).attached_to, Some(obj(bears)), "stays attached");
    // A new Claustrophobia can't target the Giant.
    assert!(!targetable(&mut t, P1, "Claustrophobia", other));
}

#[test]
fn devout_lightcaster_must_exile_your_own_black_permanent_if_its_the_only_one() {
    cr!("603.3c", "115.1a");
    ruling!(
        "Devout Lightcaster",
        "Devout Lightcaster's triggered ability isn't optional. When it enters, if you're the only player that controls a black permanent, you'll have to target a black permanent you control."
    );
    supported("Devout Lightcaster");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Vampire Nighthawk");
    let from = mark(&t);
    t.enter(P0, "Devout Lightcaster");
    t.resolve_all();
    assert_eq!(t.zone(knight), Zone::Exile);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands, vec![vec![obj(knight)]]);
}

#[test]
fn kithkin_zealot_and_honorable_scout_count_each_permanent_once() {
    cr!("608.2h", "105.2c");
    ruling!(
        "Kithkin Zealot",
        "Each permanent is counted only once. For example, if the targeted player controls a black creature, a red enchantment, and a black-red creature, you’ll gain 3 life."
    );
    ruling!(
        "Honorable Scout",
        "If a creature is both black and red, you gain 2 life, not 4."
    );
    supported("Kithkin Zealot");
    supported("Honorable Scout");
    supported("Impulsive Maneuvers");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Black Knight");
    t.battlefield(P1, "Impulsive Maneuvers");
    t.battlefield(P1, "Carnage Gladiator");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Kithkin Zealot");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Carnage Gladiator");
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Honorable Scout");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn fiendslayer_paladin_is_hit_by_untargeted_red_spells() {
    cr!("115.1a", "608.2c");
    ruling!(
        "Fiendslayer Paladin",
        "Fiendslayer Paladin can be affected by black or red spells that don't target it. For example, a red spell that \"deals 3 damage to each creature\" would deal damage to Fiendslayer Paladin."
    );
    supported("Fiendslayer Paladin");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    let pal = t.battlefield(P0, "Fiendslayer Paladin");
    assert!(!targetable(&mut t, P1, "Lightning Bolt", pal));
    assert!(targetable(&mut t, P0, "Lightning Bolt", pal), "your own red spells may");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(died(&t, pal));
}

#[test]
fn council_guardian_votes_as_the_ability_resolves_even_once_it_left() {
    cr!("701.38a", "608.2h", "603.3", "702.16a");
    ruling!(
        "Council Guardian",
        "If Council Guardian is not on the battlefield when its ability resolves, the vote will still take place; it simply won't have any effect. However, cards that care about voting (for example, Grudge Keeper) will still see the vote."
    );
    ruling!(
        "Council Guardian",
        "The vote happens as the ability resolves. Until this happens, Council Guardian doesn't have protection from any color. Spells and abilities may target Council Guardian as normal in response to its enters-the-battlefield ability, but not in response to the vote. Once players have started voting, it's too late to respond."
    );
    supported("Council Guardian");
    supported("Grudge Keeper");
    // Grudge Keeper (P1's): "Whenever players finish voting, each opponent who voted for a
    // choice you didn't vote for loses 2 life."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grudge Keeper");
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // blue
    t.answer(P1, DecisionKind::Option, Answer::Index(1)); // black
    let g = t.enter(P0, "Council Guardian");
    t.settle();
    // In response, it can be targeted by a blue spell: no protection yet.
    assert!(targetable(&mut t, P1, "Unsummon", g));
    cast_new(&mut t, P1, "Murder", &[obj(g)]);
    t.resolve();
    assert!(died(&t, g));
    t.resolve_all();
    assert_eq!(t.life(P0), 18, "the vote took place: Grudge Keeper saw it");
    // Left alone, the protection is there once the vote has happened.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // blue
    t.answer(P1, DecisionKind::Option, Answer::Index(0)); // blue
    let g = t.enter(P0, "Council Guardian");
    t.settle();
    assert!(targetable(&mut t, P1, "Unsummon", g));
    t.resolve_all();
    assert!(!targetable(&mut t, P1, "Unsummon", g));
}

#[test]
fn sanctifier_en_vec_exiles_what_dies_with_it_and_spells_that_kill_it_slowly() {
    cr!("614.1a", "704.3", "608.2m", "704.5f");
    ruling!(
        "Sanctifier en-Vec",
        "If a black or red permanent would go to the graveyard at the same time as Sanctifier en-Vec, that permanent will be exiled instead."
    );
    ruling!(
        "Sanctifier en-Vec",
        "If a black or red spell causes Sanctifier en-Vec to have lethal damage marked on it or causes it to have 0 toughness, Sanctifier en-Vec will not be put into a graveyard until the next time state-based actions are checked after the spell has finished resolving. In this case, the spell will be exiled instead of going to its owner's graveyard."
    );
    ruling!(
        "Sanctifier en-Vec",
        "If a black or red spell causes Sanctifier en-Vec to leave the battlefield as part of its resolution, that spell will go to its owner's graveyard and won't be exiled instead."
    );
    supported("Sanctifier en-Vec");
    // Wrath of God: the Black Knight dying alongside it is exiled.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanctifier en-Vec");
    let knight = t.battlefield(P1, "Black Knight");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Wrath of God", &[]);
    t.resolve_all();
    assert!(died(&t, s));
    assert_eq!(t.zone(knight), Zone::Exile);
    // Infest (black, -2/-2 to all creatures): the 2/2 dies to state-based actions after
    // Infest has finished resolving, so Infest is exiled.
    supported("Infest");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanctifier en-Vec");
    t.set_step(P1, Step::PrecombatMain);
    let infest = cast_new(&mut t, P1, "Infest", &[]);
    t.resolve_all();
    assert!(died(&t, s));
    assert_eq!(t.zone(infest), Zone::Exile);
    // Damnation (black, destroy all creatures): it leaves during the resolution, and
    // Damnation goes to the graveyard.
    supported("Damnation");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanctifier en-Vec");
    t.set_step(P1, Step::PrecombatMain);
    let damnation = cast_new(&mut t, P1, "Damnation", &[]);
    t.resolve_all();
    assert!(died(&t, s));
    assert_eq!(t.zone(damnation), Zone::Graveyard(P1));
}

#[test]
fn sanctifier_en_vec_reads_the_color_in_the_zone_the_card_leaves() {
    cr!("614.1a", "709.3", "707.2", "400.7");
    ruling!(
        "Sanctifier en-Vec",
        "Sanctifier En-Vec's last ability cares only what the color of the card would be in the zone it's moving from, not what it would be in the graveyard. For example, if a Clone is a copy of a red creature on the battlefield and then dies, the Clone will be exiled. Conversely, if you cast Order, the white half of the split card Order//Chaos, it will not be exiled as it resolves because it is not red before it goes to the graveyard."
    );
    supported("Clone");
    supported("Order // Chaos");
    // Clone copying Hill Giant (red) dies: exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctifier en-Vec");
    let giant = t.battlefield(P1, "Hill Giant");
    let clone = crate::r_p017_common::enter_copying(&mut t, P1, "Clone", Some(giant));
    assert!(colors(&t, clone).contains(&Color::Red));
    cast_new(&mut t, P0, "Swords to Plowshares", &[obj(giant)]);
    t.resolve_all();
    cast_new(&mut t, P1, "Murder", &[obj(clone)]);
    t.resolve_all();
    assert_eq!(t.zone(clone), Zone::Exile);
    // Order ("Exile target attacking creature", white) cast by P1 while P0 attacks:
    // the spell is white on the stack and goes to the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctifier en-Vec");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.lands(P1, "Plains", 2);
    t.lands(P1, "Wastes", 2);
    let card = t.hand(P1, "Order // Chaos");
    t.answer_targets(P1, &[obj(bears)]);
    let spell = t.cast(P1, card).method(CastMethod::Half(0)).go();
    assert_eq!(colors(&t, spell), vec![Color::White]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(spell), Zone::Graveyard(P1));
}

#[test]
fn mask_of_law_and_grace_changed_to_white_falls_off() {
    cr!("612.1", "704.5m", "702.16d");
    ruling!(
        "Mask of Law and Grace",
        "If a text-changing effect is used to change one of the color words so it grants Protection from white, then it will cause itself to be put into the graveyard."
    );
    supported("Mask of Law and Grace");
    supported("Mind Bend");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mask = crate::r_s06_common::attach_new(&mut t, P0, "Mask of Law and Grace", bears);
    t.g.recompute();
    // Mind Bend: black -> white.
    let idx = |c: Color, without: Option<Color>| {
        Color::ALL
            .iter()
            .filter(|x| Some(**x) != without)
            .position(|x| *x == c)
            .unwrap()
    };
    t.answer(P0, DecisionKind::Option, Answer::Index(idx(Color::Black, None)));
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(idx(Color::White, Some(Color::Black))),
    );
    cast_new(&mut t, P0, "Mind Bend", &[obj(mask)]);
    t.resolve_all();
    assert!(died(&t, mask));
    assert!(t.on_battlefield(bears));
}

#[test]
fn radiant_essence_and_crusading_knight_count_every_opponent() {
    cr!("608.2h", "102.3");
    ruling!(
        "Radiant Essence",
        "In a multiplayer game, it gets the bonus if at least one opponent controls a black permanent."
    );
    ruling!("Crusading Knight", "It counts all opponents' swamps.");
    supported("Radiant Essence");
    supported("Crusading Knight");
    let mut t = TestGame::new(3);
    let essence = t.battlefield(P0, "Radiant Essence");
    let knight = t.battlefield(P0, "Crusading Knight");
    assert_eq!(t.pt(essence), (2, 3));
    assert_eq!(t.pt(knight), (2, 2));
    t.lands(P1, "Swamp", 2);
    t.lands(P2, "Swamp", 1);
    t.battlefield(P2, "Black Knight");
    t.g.recompute();
    assert_eq!(t.pt(essence), (3, 5));
    assert_eq!(t.pt(knight), (5, 5));
    // P0's own Swamp and black permanent don't count.
    t.lands(P0, "Swamp", 1);
    t.g.recompute();
    assert_eq!(t.pt(knight), (5, 5));
}

#[test]
fn sandstalker_moloch_counts_a_countered_or_pending_blue_spell() {
    cr!("603.4", "608.2h");
    ruling!(
        "Sandstalker Moloch",
        "It doesn’t matter if the blue and/or black spell an opponent cast that turn resolved, didn’t resolve, was countered, or is still on the stack."
    );
    supported("Sandstalker Moloch");
    for countered in [true, false] {
        let mut t = TestGame::new(2);
        crate::r_s01_common::stack_library(
            &mut t,
            P0,
            &["Grizzly Bears", "Island", "Island", "Island"],
        );
        t.set_step(P1, Step::PrecombatMain);
        let opt = cast_new(&mut t, P1, "Opt", &[]);
        if countered {
            cast_new(&mut t, P0, "Counterspell", &[obj(opt)]);
            t.resolve();
            assert!(t.in_graveyard(P1, "Opt"));
        }
        // Opt is still on the stack otherwise.
        let hand = t.hand_size(P0);
        t.answer_choose_moved(P0, &[obj(t.g.player(P0).library[t.g.player(P0).library.len() - 1])]);
        let moloch = cast_new(&mut t, P0, "Sandstalker Moloch", &[]);
        t.resolve();
        assert!(t.on_battlefield(t.g.current(moloch)));
        assert_eq!(triggers_from(&t, moloch), 1, "countered: {countered}");
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 1, "countered: {countered}");
        assert!(t.in_hand(P0, "Grizzly Bears"));
    }
}

#[test]
fn lands_are_colorless_for_celestial_purge_and_lightwielder_paladin() {
    cr!("105.2c", "202.2b", "115.1a");
    ruling!(
        "Celestial Purge",
        "Lands are colorless (even if their frames have some colored elements to them). You can’t target a Swamp, a Mountain, or any other land with Celestial Purge (unless some other effect has turned that land black or red)."
    );
    ruling!(
        "Lightwielder Paladin",
        "Lands are colorless (even if their frames have some colored elements to them). You can’t target a Swamp, a Mountain, or any other land with Lightwielder Paladin’s ability (unless some other effect has turned that land black or red)."
    );
    supported("Celestial Purge");
    supported("Lightwielder Paladin");
    let mut t = TestGame::new(2);
    let swamp = t.battlefield(P1, "Swamp");
    let mtn = t.battlefield(P1, "Mountain");
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    let cands = spell_targets(&mut t, P0, "Celestial Purge");
    assert!(!cands.contains(&obj(swamp)) && !cands.contains(&obj(mtn)));
    assert!(cands.contains(&obj(knight)));
    // The Paladin deals combat damage to P1: the trigger's candidates.
    let mut t = TestGame::new(2);
    let pal = t.battlefield(P0, "Lightwielder Paladin");
    let swamp = t.battlefield(P1, "Swamp");
    let knight = t.battlefield(P1, "Vampire Nighthawk");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(pal, pl(P1))]);
    let from = mark(&t);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(knight)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(!cands[0].contains(&obj(swamp)));
    assert!(cands[0].contains(&obj(knight)));
    assert_eq!(t.zone(knight), Zone::Exile);
}

#[test]
fn compost_triggers_on_any_black_card_put_into_an_opponents_graveyard() {
    cr!("603.2", "603.6c", "701.8a", "701.19a");
    ruling!(
        "Compost",
        "The ability triggers on a black card being discarded, milled, or countered, as well as a black card going to the graveyard from the battlefield."
    );
    supported("Compost");
    supported("Tome Scour");
    // Discarded (Mind Rot), milled (Tome Scour), countered (Counterspell), dies (Murder).
    let mut t = TestGame::new(2);
    let compost = t.battlefield(P0, "Compost");
    let mut hand = t.hand_size(P0);
    let mut expect = |t: &mut TestGame, n: usize, what: &str| {
        assert_eq!(triggers_from(t, compost), n, "{what}");
        for _ in 0..n {
            t.answer_yes(P0, true);
        }
        t.resolve_all();
        hand += n;
        assert_eq!(t.hand_size(P0), hand, "{what}");
    };
    t.hand(P1, "Black Knight");
    t.hand(P1, "Grizzly Bears");
    t.answer_choose(P1, &[]);
    cast_new(&mut t, P0, "Mind Rot", &[pl(P1)]);
    t.resolve();
    // (P1 discards both: one black card.)
    assert_eq!(t.hand_size(P1), 0);
    expect(&mut t, 1, "discarded");
    crate::r_s01_common::stack_library(&mut t, P1, &["Black Knight", "Grizzly Bears", "Swamp"]);
    cast_new(&mut t, P0, "Tome Scour", &[pl(P1)]);
    t.resolve();
    expect(&mut t, 1, "milled");
    t.set_step(P1, Step::PrecombatMain);
    let s = cast_new(&mut t, P1, "Black Knight", &[]);
    cast_new(&mut t, P0, "Counterspell", &[obj(s)]);
    t.resolve();
    expect(&mut t, 1, "countered");
    let knight = t.battlefield(P1, "Black Knight");
    cast_new(&mut t, P0, "Murder", &[obj(knight)]);
    t.resolve();
    expect(&mut t, 1, "died");
}

#[test]
fn reap_counts_the_black_permanents_of_the_target_opponent_as_cast() {
    cr!("601.2c", "608.2b", "115.1a");
    ruling!(
        "Reap",
        "The count of the number of black permanents the targeted opponent controls is done during announcement since it only affects target selection. It does not get recounted later."
    );
    ruling!(
        "Reap",
        "This card has received errata so that it now targets a specific opponent."
    );
    supported("Reap");
    let mut t = TestGame::new(3);
    let k1 = t.battlefield(P1, "Black Knight");
    let _k2 = t.battlefield(P1, "Black Knight");
    t.battlefield(P2, "Black Knight");
    let c1 = t.graveyard(P0, "Grizzly Bears");
    let c2 = t.graveyard(P0, "Hill Giant");
    let from = mark(&t);
    lands_for_cost(&mut t, P0, "Reap");
    let card = t.hand(P0, "Reap");
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(c1), obj(c2)]);
    t.cast(P0, card).go();
    // The opponent is a specific target: P1 or P2 were offered.
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&pl(P1)) && c.contains(&pl(P2))));
    // One of P1's black permanents dies in response: both cards still return.
    cast_new(&mut t, P0, "Murder", &[obj(k1)]);
    t.resolve();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Hill Giant"));
}

#[test]
fn knight_of_the_mists_may_destroy_an_opponents_knight() {
    cr!("115.1a", "701.7b");
    ruling!("Knight of the Mists", "Can destroy an opponent’s knight.");
    supported("Knight of the Mists");
    let mut t = TestGame::new(2);
    let white = t.battlefield(P1, "White Knight");
    t.answer_yes(P0, false);
    t.answer_targets(P0, &[obj(white)]);
    t.enter(P0, "Knight of the Mists");
    t.resolve_all();
    assert!(died(&t, white));
}

#[test]
fn outrider_en_kor_chooses_which_simultaneous_source_is_redirected() {
    cr!("615.1", "616.1", "510.2");
    ruling!(
        "Outrider en-Kor",
        "After Outrider en-Kor's activated ability has resolved, if multiple sources would deal damage to Outrider en-Kor simultaneously that turn (which can happen if Outrider en-Kor is blocked by multiple creatures, for example), you choose which source has its 1 damage redirected. If Outrider en-Kor's ability has resolved more than once that turn, you may choose a different source for each redirection effect."
    );
    supported("Outrider en-Kor");
    let mut t = TestGame::new(2);
    let out = t.battlefield(P0, "Outrider en-Kor");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P0, "Wall of Stone");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[out], &[(b1, out), (b2, out)]);
    t.resolve_all();
    // Two shields: one to the Giant, one to the Wall.
    t.activate(P0, out, 0, &[obj(giant)]).unwrap();
    t.resolve();
    t.activate(P0, out, 0, &[obj(wall)]).unwrap();
    t.resolve();
    finish_combat(&mut t);
    // Each Bears dealt 2: 1 of each (or 2 of one) redirected, 2 in all.
    assert_eq!(marked(&t, giant) + marked(&t, wall), 2);
    assert!(marked(&t, giant) >= 1 && marked(&t, wall) >= 1);
}

#[test]
fn toshiro_recasts_the_instant_that_killed_the_creature() {
    cr!("603.3", "603.10", "608.2n", "115.1a");
    ruling!(
        "Toshiro Umezawa",
        "Creatures destroyed by an instant that you cast normally can cause a trigger that allows the instant to be cast again. For example, cast Terror on an opponent’s creature. The creature is destroyed, triggering Toshiro’s ability, and then the Terror is put into your graveyard. You can put Toshiro’s ability on the stack targeting the Terror and then recast it when the ability resolves."
    );
    supported("Toshiro Umezawa");
    supported("Terror");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toshiro Umezawa");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let terror = cast_new(&mut t, P0, "Terror", &[obj(a)]);
    t.answer_targets(P0, &[obj(terror)]);
    t.resolve();
    assert!(died(&t, a));
    assert_eq!(t.stack_len(), 1, "Toshiro's trigger, targeting the Terror");
    t.lands(P0, "Swamp", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.resolve_all();
    assert!(died(&t, b));
    assert_eq!(t.zone(terror), Zone::Exile);
}
