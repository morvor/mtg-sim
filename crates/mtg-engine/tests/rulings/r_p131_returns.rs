//! Rulings batch P131 — spells and abilities that return cards from a graveyard: untargeted
//! choices made on resolution, random choices, what "you've gained life this turn" checks,
//! cost reductions, votes, and creature cards' characteristics in the graveyard.

use crate::r_p131_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn targets_asked_since(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count()
}

fn names_in(t: &TestGame, zone: Zone) -> Vec<String> {
    let ids = match zone {
        Zone::Hand(p) => t.g.player(p).hand.clone(),
        Zone::Graveyard(p) => t.g.player(p).graveyard.clone(),
        _ => unreachable!(),
    };
    let mut v: Vec<String> = ids
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    v.sort();
    v
}

// --- Random returns -------------------------------------------------------------------------

#[test]
fn make_a_wish_cant_return_itself() {
    cr!("608.2n");
    ruling!(
        "Make a Wish",
        "Make a Wish isn’t put into its owner’s graveyard until it is finished resolving, so it can’t be returned by its own effect."
    );
    supported("Make a Wish");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    cast(&mut t, P0, "Make a Wish", &[]);
    t.resolve_all();
    assert_eq!(names_in(&t, Zone::Hand(P0)), vec!["Grizzly Bears"]);
    assert_eq!(names_in(&t, Zone::Graveyard(P0)), vec!["Make a Wish"]);
}

#[test]
fn make_a_wish_chooses_the_cards_as_it_resolves() {
    cr!("115.10");
    ruling!(
        "Make a Wish",
        "The cards aren’t randomly chosen until Make a Wish resolves."
    );
    let mut t = TestGame::new(2);
    cast(&mut t, P0, "Make a Wish", &[]);
    // Cards put into the graveyard in response are there to be chosen.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(
        names_in(&t, Zone::Hand(P0)),
        vec!["Grizzly Bears", "Hill Giant"]
    );
}

#[test]
fn fossil_find_reordering_doesnt_make_new_objects() {
    cr!("404.2");
    ruling!(
        "Fossil Find",
        "Except for the card that you return to your hand, reordering your graveyard won’t affect anything that’s currently targeting or enchanting a card in your graveyard."
    );
    supported("Fossil Find");
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant", "Craw Wurm"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    cast(&mut t, P0, "Fossil Find", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    let returned = t.g.player(P0).hand[0];
    let stayed: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|c| t.g.obj(*c).chars.name != t.g.obj(returned).chars.name)
        .collect();
    assert_eq!(stayed.len(), 2);
    for c in stayed {
        assert!(
            t.g.is_live(c),
            "the cards left in the graveyard are the same objects"
        );
        assert_eq!(t.g.obj(c).zone, Zone::Graveyard(P0));
    }
}

// --- Untargeted returns -----------------------------------------------------------------------

#[test]
fn blood_for_bones_chooses_on_resolution_and_triggers_wait() {
    cr!("115.10", "603.3", "601.2h");
    ruling!(
        "Blood for Bones",
        "Blood for Bones doesn’t target the creature cards to return from your graveyard."
    );
    ruling!(
        "Blood for Bones",
        "Any abilities that trigger as the creature card is returned to the battlefield won’t resolve until after Blood for Bones is finished resolving."
    );
    ruling!(
        "Blood for Bones",
        "Once you begin to cast Blood for Bones, no player may take actions until you’re done. Notably, opponents can’t try to remove the creature you wish to sacrifice."
    );
    supported("Blood for Bones");
    let mut t = TestGame::new(2);
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let ew = t.graveyard(P0, "Eternal Witness");
    let giant = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Blood for Bones");
    let card = t.hand(P0, "Blood for Bones");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(fodder)]);
    let bfb = t.cast(P0, card).go();
    // Sacrificed while casting, with no priority in between: it's already in the
    // graveyard as the spell is on the stack, and nothing was targeted.
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.g.stack, vec![bfb]);
    assert_eq!(targets_asked_since(&t, from), 0);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.answer_choose(P0, &[obj(ew)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    // Blood for Bones finished resolving before the Witness's trigger went on the stack.
    assert!(t.in_graveyard(P0, "Blood for Bones"));
    assert_eq!(t.named_on_battlefield("Eternal Witness").len(), 1);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert_eq!(targets_asked_since(&t, from), 1);
}

#[test]
fn grim_captains_call_chooses_on_resolution_and_a_card_counts_for_one_type() {
    cr!("115.10", "205.3m");
    ruling!(
        "Grim Captain's Call",
        "Grim Captain’s Call doesn’t target the cards to return. You choose them while it’s resolving."
    );
    ruling!(
        "Grim Captain's Call",
        "If a card in your graveyard has more than one of these types (such as a Dinosaur that’s a Vampire Dinosaur due to Arcane Adaptation’s effect), you may choose to return it for one of its types and return another card of the other type."
    );
    supported("Grim Captain's Call");
    supported("Arcane Adaptation");
    let mut t = TestGame::new(2);
    choose_vampire(&mut t, P0);
    t.enter(P0, "Arcane Adaptation");
    t.settle();
    let a = t.graveyard(P0, "Ripjaw Raptor");
    let b = t.graveyard(P0, "Colossal Dreadmaw");
    assert!(t.g.obj(a).chars.has_subtype("Vampire"));
    let from = t.asked().len();
    let spell = cast(&mut t, P0, "Grim Captain's Call", &[]);
    assert!(targets_of(&t, spell).is_empty());
    // Vampire: the Raptor; Dinosaur: the Dreadmaw.
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert_eq!(targets_asked_since(&t, from), 0);
    assert_eq!(
        names_in(&t, Zone::Hand(P0)),
        vec!["Colossal Dreadmaw", "Ripjaw Raptor"]
    );
}

#[test]
fn desecrator_hag_sees_star_power_in_the_graveyard() {
    cr!("604.3", "208.2");
    ruling!(
        "Desecrator Hag",
        "If the creature card has “*” in its power or toughness, the ability that defines “*” works in all zones."
    );
    supported("Desecrator Hag");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Nightmare");
    // Four Swamps (two pay for the Hag): Nightmare is 4/4 in the graveyard.
    t.lands(P0, "Swamp", 2);
    cast(&mut t, P0, "Desecrator Hag", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Nightmare"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn macabre_waltz_may_discard_a_returned_card() {
    cr!("608.2c");
    ruling!(
        "Macabre Waltz",
        "If you have no other cards in hand, you'll have to discard one of the creature cards you return to your hand."
    );
    supported("Macabre Waltz");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Macabre Waltz");
    let c = t.hand(P0, "Macabre Waltz");
    t.cast(P0, c).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn the_underworld_cookbook_returns_any_creature_card() {
    cr!("115.1");
    ruling!(
        "The Underworld Cookbook",
        "The Underworld Cookbook's last ability can target any creature card in your graveyard, not just one discarded with its first ability."
    );
    supported("The Underworld Cookbook");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "The Underworld Cookbook");
    let gargoyle = t.battlefield(P0, "Granite Gargoyle");
    destroy(&mut t, gargoyle);
    let card = in_yard(&t, P0, "Granite Gargoyle");
    t.lands(P0, "Wastes", 4);
    act(&mut t, P0, book, "Return target", &[obj(card)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Granite Gargoyle"));
}

#[test]
fn soul_manipulation_cant_return_the_creature_spell_it_counters() {
    cr!("601.2c");
    ruling!(
        "Soul Manipulation",
        "If you choose both modes, you choose their targets at the same time. You can’t counter your own creature spell and then return that card from your graveyard to your hand."
    );
    supported("Soul Manipulation");
    let mut t = TestGame::new(2);
    let bears_card = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    let bears = t.cast(P0, bears_card).go();
    let giant = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Soul Manipulation");
    let sm = t.hand(P0, "Soul Manipulation");
    let from = t.asked().len();
    t.cast(P0, sm)
        .modes(&[0, 1])
        .targets(&[obj(bears)])
        .targets(&[obj(giant)])
        .go();
    // The second target was chosen while the Bears were still a spell.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 2);
    assert_eq!(offered[1], vec![obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
}

// --- "If you gained life this turn" --------------------------------------------------------

#[test]
fn courier_bat_and_sanguine_indulgence_ignore_life_lost() {
    cr!("603.4", "601.2f");
    ruling!(
        "Courier Bat",
        "Courier Bat's ability will trigger if you gained any amount of life this turn. It doesn't matter if you also lost that much life or more."
    );
    ruling!(
        "Sanguine Indulgence",
        "Sanguine Indulgence's first ability looks at how much life you've gained in the turn. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    supported("Courier Bat");
    supported("Sanguine Indulgence");
    for gained in [false, true] {
        let mut t = TestGame::new(2);
        if gained {
            t.g.gain_life(P0, 3);
        }
        t.g.lose_life(P0, 5);
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, "Courier Bat");
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), gained);
        // Sanguine Indulgence for {B}.
        let mut t2 = TestGame::new(2);
        if gained {
            t2.g.gain_life(P0, 3);
        }
        t2.g.lose_life(P0, 5);
        mana(&mut t2, P0, ManaType::B, 1);
        let si = t2.hand(P0, "Sanguine Indulgence");
        assert_eq!(can_cast(&mut t2, P0, si, CastMethod::Normal), gained);
    }
}

// --- Finality ----------------------------------------------------------------------------------

fn cast_finality(t: &mut TestGame) -> ObjectId {
    supported("Find // Finality");
    mana(t, P0, ManaType::B, 1);
    mana(t, P0, ManaType::G, 1);
    mana(t, P0, ManaType::C, 4);
    let c = t.hand(P0, "Find // Finality");
    t.cast(P0, c).method(CastMethod::Half(1)).go()
}

#[test]
fn finality_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Find // Finality",
        "Finality affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't get -4/-4."
    );
    let mut t = TestGame::new(2);
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    cast_finality(&mut t);
    t.resolve_all();
    assert_eq!(t.pt(dreadmaw), (2, 2));
    let later = t.battlefield(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.pt(later), (3, 3));
}

#[test]
fn finality_can_be_cast_with_no_creatures() {
    cr!("115.10");
    ruling!(
        "Find // Finality",
        "Finality doesn't target the creature to receive +1/+1 counters. You can cast it even if you control no creatures."
    );
    let mut t = TestGame::new(2);
    let s = cast_finality(&mut t);
    assert!(targets_of(&t, s).is_empty());
    // With a creature: chosen on resolution, no target.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let s = cast_finality(&mut t);
    assert!(targets_of(&t, s).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.counters(wurm, counters::PLUS1), 2);
    assert_eq!(t.pt(wurm), (4, 2));
}

// --- Honest Rutstein ---------------------------------------------------------------------------

#[test]
fn honest_rutstein_reduces_only_generic_mana_and_not_mana_value() {
    cr!("601.2f", "202.3");
    ruling!(
        "Honest Rutstein",
        "Honest Rutstein’s last ability can’t reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell’s cost."
    );
    ruling!(
        "Honest Rutstein",
        "Honest Rutstein’s last ability doesn’t change the mana cost or mana value of any spell. It changes only the total cost you pay to cast creature spells."
    );
    supported("Honest Rutstein");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Honest Rutstein");
    // Llanowar Elves ({G}) still needs its {G}.
    mana(&mut t, P0, ManaType::C, 1);
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves, CastMethod::Normal));
    // Grizzly Bears ({1}{G}) costs {G}, but its mana value is still 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Honest Rutstein");
    mana(&mut t, P0, ManaType::G, 1);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 2);
    assert_eq!(
        format!("{}", t.obj(spell).chars.mana_cost.clone().unwrap()),
        "{1}{G}"
    );
}

// --- Votes ---------------------------------------------------------------------------------------

#[test]
fn custodi_squire_votes_even_if_it_left_the_battlefield() {
    cr!("701.38a", "113.7a");
    ruling!(
        "Custodi Squire",
        "If Custodi Squire is not on the battlefield when its ability resolves, the vote will still take place and the card(s) that had the most votes will still be returned to your hand."
    );
    supported("Custodi Squire");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let sq = t.enter(P0, "Custodi Squire");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, sq);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn custodi_squire_returns_nothing_without_votes() {
    cr!("701.38a");
    ruling!(
        "Custodi Squire",
        "If a choice has no votes, it can't have the most votes. If all choices have zero votes, the voting ability has no effect."
    );
    // A card nobody voted for isn't returned.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(bears)]);
    t.enter(P0, "Custodi Squire");
    t.resolve_all();
    assert_eq!(names_in(&t, Zone::Hand(P0)), vec!["Grizzly Bears"]);
    // Nothing to vote for: no effect.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.enter(P0, "Custodi Squire");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn the_initial_vote_is_made_even_if_the_additional_one_is_declined() {
    cr!("701.38a", "701.38d");
    ruling!(
        "Ballot Broker",
        "You must make your initial vote, even if you decline to vote an additional time."
    );
    ruling!(
        "Tivit, Seller of Secrets",
        "You must make your initial vote, even if you decline to vote an additional time."
    );
    use mtg_engine::events::Event;
    use mtg_engine::kwa::vote::VOTED;
    let voters = |t: &TestGame| -> Vec<PlayerId> {
        t.g.turn_events
            .iter()
            .chain(t.g.events.iter())
            .filter_map(|e| match e {
                Event::Custom { name, player, .. } if name == VOTED => *player,
                _ => None,
            })
            .collect()
    };
    supported("Ballot Broker");
    supported("Tivit, Seller of Secrets");
    // Ballot Broker: Plea for Power.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ballot Broker");
    t.answer_yes(P1, false);
    cast(&mut t, P0, "Plea for Power", &[]);
    t.resolve_all();
    assert_eq!(voters(&t), vec![P0, P1]);
    // Tivit's own vote.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, false);
    t.enter(P0, "Tivit, Seller of Secrets");
    t.resolve_all();
    assert_eq!(voters(&t), vec![P0, P1]);
    assert_eq!(
        tokens_with(&t, P0, "Clue") + tokens_with(&t, P0, "Treasure"),
        2
    );
}

// --- Misc ----------------------------------------------------------------------------------------

#[test]
fn atzocan_seers_sacrifice_ability_works_while_tapped() {
    cr!("602.2");
    ruling!(
        "Atzocan Seer",
        "You can activate that ability even if it's already been tapped, perhaps because you activated its first ability."
    );
    supported("Atzocan Seer");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Atzocan Seer");
    let raptor = t.graveyard(P0, "Ripjaw Raptor");
    t.g.tap(seer);
    assert!(can_activate(&mut t, P0, seer));
    act(&mut t, P0, seer, "Sacrifice", &[obj(raptor)]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Ripjaw Raptor"));
}

#[test]
fn confluence_modes_need_legal_targets_and_may_repeat_them() {
    cr!("700.2a", "700.2d");
    ruling!(
        "Wretched Confluence",
        "If a mode requires a target, you can select that mode only if there’s a legal target available. Ignore the targeting requirements for modes you don’t choose. Each time you select that mode, you can choose a different target, or you can choose the same target."
    );
    supported("Wretched Confluence");
    // No creature card in the graveyard: the third mode isn't available.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Wretched Confluence");
    let c = t.hand(P0, "Wretched Confluence");
    let from = t.asked().len();
    let s = t
        .cast(P0, c)
        .modes(&[0, 1, 1])
        .targets(&[Entity::Player(P1)])
        .targets(&[obj(giant)])
        .targets(&[obj(giant)])
        .go();
    let available = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseModes { available, .. } => Some(available.clone()),
        _ => None,
    });
    assert_eq!(available, Some(vec![0, 1]));
    assert_eq!(
        targets_of(&t, s),
        vec![Entity::Player(P1), obj(giant), obj(giant)]
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "-4/-4 in all");
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_confluence_with_every_target_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Righteous Confluence",
        "If all targets for the chosen modes become illegal before the Confluence resolves, the spell won’t resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    supported("Righteous Confluence");
    for keep_one in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Pacifism");
        let b = t.battlefield(P1, "Glorious Anthem");
        lands_for_cost(&mut t, P0, "Righteous Confluence");
        let c = t.hand(P0, "Righteous Confluence");
        t.cast(P0, c)
            .modes(&[1, 1, 2])
            .targets(&[obj(a)])
            .targets(&[obj(b)])
            .go();
        destroy(&mut t, a);
        if !keep_one {
            destroy(&mut t, b);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if keep_one { 25 } else { 20 });
        assert_eq!(t.in_exile("Glorious Anthem"), keep_one);
    }
}

#[test]
fn satsuki_adds_lore_counters_and_triggers_only_new_chapters() {
    cr!("714.2b");
    ruling!(
        "Satsuki, the Living Lore",
        "Putting a lore counter on a Saga will usually cause the next chapter ability to trigger. If it already has a number of lore counters on it equal to its final chapter number however, no abilities trigger."
    );
    supported("Satsuki, the Living Lore");
    let mut t = TestGame::new(2);
    let satsuki = t.battlefield(P0, "Satsuki, the Living Lore");
    // Sagas with one and with three (its final chapter) lore counters, placed without
    // putting the counters on as an event (no chapter triggers, no state-based actions
    // checked yet).
    let one = t.battlefield(P0, "The Eldest Reborn");
    let three = t.battlefield(P0, "The Eldest Reborn");
    for (id, n) in [(one, 1), (three, 3)] {
        t.g.objects[id.0 as usize]
            .counters
            .insert(counters::LORE.into(), n);
    }
    act(&mut t, P0, satsuki, "lore counter", &[]).unwrap();
    t.g.resolve_top();
    t.g.flush_events();
    assert_eq!(t.counters(one, counters::LORE), 2);
    assert_eq!(t.counters(three, counters::LORE), 4);
    t.settle();
    // Only chapter II of the first one triggered.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(abilities_of(&t, one), 1);
}

fn abilities_of(t: &TestGame, source: ObjectId) -> usize {
    crate::r_s25_common::abilities_from(t, source).len()
}

#[test]
fn a_chosen_creature_type_is_one_existing_creature_type() {
    cr!("205.3m");
    ruling!(
        "Arcane Adaptation",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    ruling!(
        "Pillar of Origins",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    for name in ["Arcane Adaptation", "Pillar of Origins"] {
        supported(name);
        let mut t = TestGame::new(2);
        let from = t.asked().len();
        t.enter(P0, name);
        let options = t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseOption {
                    prompt, options, ..
                } if prompt == "Choose a creature type" => Some(options.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name}: no creature type choice"));
        // One creature type is two words long (CR 205.3m).
        for ok in ["Vampire", "Knight", "Assembly-Worker", "Time Lord"] {
            assert!(options.iter().any(|o| o == ok), "{name}: {ok}");
        }
        for bad in [
            "Vampire Knight",
            "Artifact",
            "Jace",
            "Vehicle",
            "Treasure",
            "Equipment",
        ] {
            assert!(!options.iter().any(|o| o == bad), "{name}: {bad}");
        }
    }
}
