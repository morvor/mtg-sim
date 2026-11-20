//! Item `rulings-residue-shapechange-typal` — interaction rulings about the top of the
//! library (Conspicuous Snoop's revealed top card and the abilities it grants, casting from
//! the library, Quest for Ula's Temple's look), cost increases against cost reductions
//! (Gloom, CR 601.2f) and putting a land onto the battlefield (CR 305.4).

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The texts of the object's activated abilities now.
fn activated(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .collect()
}

/// The index of the object's activated ability whose text contains `text`.
fn index_of(t: &mut TestGame, id: ObjectId, text: &str) -> usize {
    activated(t, id)
        .iter()
        .position(|a| a.contains(text))
        .unwrap_or_else(|| panic!("no activated ability with {text:?}"))
}

// ---------------------------------------------------------------------------
// Conspicuous Snoop
// ---------------------------------------------------------------------------

#[test]
fn conspicuous_snoop_gains_keyword_activated_abilities() {
    cr!("602.1", "702.177a", "613.1f");
    ruling!(
        "Conspicuous Snoop",
        "Activated abilities contain a colon. They're generally written “[Cost]: [Effect].” Some keyword abilities are activated abilities and will have colons in their reminder text."
    );
    supported("Conspicuous Snoop");
    supported("Prowcatcher Specialist");
    let mut t = main_phase(2);
    let snoop = t.battlefield(P0, "Conspicuous Snoop");
    // Prowcatcher Specialist: "Exhaust — {3}{R}: Put two +1/+1 counters on this creature."
    t.library_top(P0, "Prowcatcher Specialist");
    t.lands(P0, "Mountain", 4);
    let i = index_of(&mut t, snoop, "two +1/+1 counters");
    t.activate(P0, snoop, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, snoop), 2);
    // Its static "haste" isn't gained.
    assert!(!crate::r_p119_common::has_kw(
        &mut t,
        snoop,
        KeywordKind::Haste
    ));
}

#[test]
fn conspicuous_snoop_casts_goblins_from_the_library_with_normal_timing_and_costs() {
    cr!("601.2b", "601.2f", "601.3", "307.1", "702.109a");
    ruling!(
        "Conspicuous Snoop",
        "You must follow the normal timing permissions and restrictions of the Goblin spells you cast from your library."
    );
    ruling!(
        "Conspicuous Snoop",
        "You still pay all costs for a Goblin spell you cast from your library, including additional costs. You may also pay alternative costs."
    );
    supported("Goblin Bushwhacker");
    supported("Goblin Heelcutter");
    supported("Tarfire");
    // A Goblin creature can't be cast from the library during combat; a kindred Goblin
    // instant can.
    let mut t = main_phase(2);
    t.battlefield(P0, "Conspicuous Snoop");
    let gob = t.library_top(P0, "Raging Goblin");
    t.lands(P0, "Mountain", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, gob).try_go().is_err());
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, gob).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(gob)));
    let tar = t.library_top(P0, "Tarfire");
    t.set_step(P0, Step::BeginningOfCombat);
    t.cast(P0, tar).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Kicker (an additional cost) paid for Goblin Bushwhacker cast from the library.
    let mut t = main_phase(2);
    t.battlefield(P0, "Conspicuous Snoop");
    let bush = t.library_top(P0, "Goblin Bushwhacker");
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, bush).kicked(true).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, bush).kicked(true).go();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, bears), (3, 2));
    // Dash (an alternative cost) for Goblin Heelcutter.
    let mut t = main_phase(2);
    t.battlefield(P0, "Conspicuous Snoop");
    let heel = t.library_top(P0, "Goblin Heelcutter");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, heel)
        .method(CastMethod::Keyword(KeywordKind::Dash))
        .go();
    t.resolve_all();
    let heel = t.g.current(heel);
    assert!(t.on_battlefield(heel));
    assert!(crate::r_p119_common::has_kw(
        &mut t,
        heel,
        KeywordKind::Haste
    ));
}

#[test]
fn conspicuous_snoop_doesnt_reveal_the_new_top_card_while_casting() {
    cr!("401.5", "601.2", "613.1f");
    interp!("unrevealed-top-card-grants-no-abilities");
    ruling!(
        "Conspicuous Snoop",
        "If the top card of your library changes while you're casting a spell, playing a land, or activating an ability, you don't reveal the new top card until you finish doing so. This means that if you cast the top card of your library, you can't look at the next one until you're done paying for that spell. While the top card isn't revealed, Conspicuous Snoop doesn't have any abilities of that card."
    );
    let mut t = main_phase(2);
    let snoop = t.battlefield(P0, "Conspicuous Snoop");
    // Goblin Balloon Brigade ("{R}: This creature gains flying") under Tarfire.
    t.library_top(P0, "Goblin Balloon Brigade");
    let tar = t.library_top(P0, "Tarfire");
    t.lands(P0, "Mountain", 1);
    // While Tarfire's target is chosen, the Snoop doesn't have the Brigade's ability.
    let seen =
        crate::r_s01_common::watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::ChooseTargets { .. }),
            |g| {
                let snoop = g
                    .permanents()
                    .find(|o| o.chars.name == "Conspicuous Snoop")
                    .unwrap();
                snoop.chars.abilities.iter().any(|a| {
                    matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("flying")
                })
            },
        );
    t.cast(P0, tar).target(Entity::Player(P1)).go();
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    // Once Tarfire is cast, the Brigade is revealed and the Snoop has its ability.
    assert!(activated(&mut t, snoop)
        .iter()
        .any(|a| a.contains("flying")));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn conspicuous_snoop_linked_abilities_last_only_while_it_has_them() {
    cr!("607.2a", "607.5", "613.1f");
    ruling!(
        "Conspicuous Snoop",
        "If Conspicuous Snoop gains a set of linked activated abilities (for example, one ability that exiles a card and another that refers to the card “exiled with” the object), that link only lasts as long as Conspicuous Snoop has those abilities. If it loses the abilities and then regains them, the link is lost."
    );
    supported("Izzet Chemister");
    // Izzet Chemister: "{R}, {T}: Exile target instant or sorcery card from your
    // graveyard." and "{1}{R}, {T}, Sacrifice this creature: Cast any number of cards
    // exiled with this creature without paying their mana costs."
    let run = |lose: bool| {
        let mut t = main_phase(2);
        let snoop = t.battlefield(P0, "Conspicuous Snoop");
        let chem = t.library_top(P0, "Izzet Chemister");
        let bolt = t.graveyard(P0, "Lightning Bolt");
        t.lands(P0, "Mountain", 3);
        let i = index_of(&mut t, snoop, "Exile target instant");
        t.activate(P0, snoop, i, &[obj(bolt)]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Lightning Bolt"));
        if lose {
            // Another card goes on top, then the Chemister is back on top.
            let filler = t.library_top(P0, "Grizzly Bears");
            assert!(!activated(&mut t, snoop)
                .iter()
                .any(|a| a.contains("Exile target instant")));
            let c = t.g.current(chem);
            let lib = &mut t.g.players[0].library;
            lib.retain(|x| *x != c && *x != filler);
            lib.push(filler);
            lib.push(c);
            t.g.dirty = true;
        }
        t.g.objects[snoop.0 as usize].tapped = false;
        t.answer_choose_moved(P0, &[obj(bolt)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let i = index_of(&mut t, snoop, "Cast any number");
        t.activate(P0, snoop, i, &[]).unwrap();
        t.resolve_all();
        t.life(P1)
    };
    assert_eq!(run(false), 17);
    assert_eq!(run(true), 20);
}

// ---------------------------------------------------------------------------
// Goblin Recruiter
// ---------------------------------------------------------------------------

#[test]
fn goblin_recruiter_finds_cards_with_the_goblin_type() {
    cr!("205.3m", "308.3", "701.23a");
    ruling!(
        "Goblin Recruiter",
        "You can search for any card with the creature type Goblin, including creature or kindred cards. Older cards of type \"Summon Goblin\" and \"Summon Goblins\" also count. You can't search for cards that have Goblin in the name but don't have the creature type Goblin."
    );
    supported("Goblin Recruiter");
    let mut t = main_phase(2);
    let raging = t.library_top(P0, "Raging Goblin");
    let tar = t.library_top(P0, "Tarfire");
    let bomb = t.library_top(P0, "Goblin Bombardment");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(raging), obj(tar)]);
    t.enter(P0, "Goblin Recruiter");
    t.resolve_all();
    let cands: Vec<Entity> = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("searched");
    assert!(cands.contains(&obj(raging)));
    assert!(cands.contains(&obj(tar)));
    assert!(!cands.contains(&obj(bomb)));
    // The two found are on top.
    let lib = &t.g.players[0].library;
    let top2: Vec<String> = lib[lib.len() - 2..]
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    assert!(top2.contains(&"Raging Goblin".to_string()));
    assert!(top2.contains(&"Tarfire".to_string()));
}

// ---------------------------------------------------------------------------
// Quest for Ula's Temple
// ---------------------------------------------------------------------------

/// Quest for Ula's Temple on P0's battlefield with `top` on top of P0's library; P0's
/// upkeep trigger is on the stack.
fn quest_upkeep(t: &mut TestGame, top: &str) -> (ObjectId, ObjectId) {
    supported("Quest for Ula's Temple");
    let q = t.battlefield(P0, "Quest for Ula's Temple");
    t.advance_to(P1, Step::End);
    let c = t.library_top(P0, top);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    (q, c)
}

#[test]
fn quest_for_ulas_temple_look_and_reveal_are_optional() {
    cr!("603.5", "701.20a", "401.3");
    ruling!(
        "Quest for Ula's Temple",
        "When the first ability resolves, you may look at the top card of your library. If you don’t, nothing happens. If you look at it and it’s not a creature card, it simply stays hidden. If you look at it and it is a creature card, you may reveal it and put a quest counter on Quest for Ula’s Temple, or you may choose to leave it hidden. In all cases, the card remains on top of your library."
    );
    for (look, reveal, top, counters) in [
        (false, false, "Grizzly Bears", 0),
        (true, false, "Grizzly Bears", 0),
        (true, true, "Grizzly Bears", 1),
        (true, true, "Lightning Bolt", 0),
    ] {
        let mut t = TestGame::new(2);
        t.answer_yes(P0, look);
        t.answer_yes(P0, reveal);
        let (q, c) = quest_upkeep(&mut t, top);
        t.resolve_all();
        assert_eq!(
            t.counters(q, "quest"),
            counters,
            "look {look} reveal {reveal} {top}"
        );
        assert_eq!(*t.g.players[0].library.last().unwrap(), c);
        t.clear_answers();
    }
}

#[test]
fn quest_for_ulas_temple_with_the_top_card_already_revealed() {
    cr!("401.3", "603.5");
    ruling!(
        "Quest for Ula's Temple",
        "If an ability (from Future Sight or Oracle of Mul Daya, for example) causes you to play with the top card of your library revealed, you may still put a quest counter on Quest for Ula’s Temple when the first ability resolves if the top card of your library is a creature card."
    );
    supported("Future Sight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Future Sight");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let (q, _) = quest_upkeep(&mut t, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(q, "quest"), 1);
}

#[test]
fn quest_for_ulas_temple_end_step_ability_triggers_each_turn() {
    cr!("513.1a", "603.4");
    ruling!(
        "Quest for Ula's Temple",
        "Although the first ability triggers only on your turn, the second ability may trigger each turn."
    );
    supported("Kraken Hatchling");
    let mut t = main_phase(2);
    let q = t.battlefield(P0, "Quest for Ula's Temple");
    t.g.add_counters(Entity::Object(q), "quest", 3, None);
    let k = t.hand(P0, "Kraken Hatchling");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(k)]);
    // The opponent's end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(k)));
    // The opponent's upkeep doesn't trigger the first ability.
    let mut t = main_phase(2);
    t.battlefield(P0, "Quest for Ula's Temple");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "look at the top"), 0);
}

// ---------------------------------------------------------------------------
// Gloom
// ---------------------------------------------------------------------------

#[test]
fn gloom_increase_is_paid_with_the_cost_and_can_be_reduced() {
    cr!("601.2f", "602.2b", "118.7", "202.3");
    ruling!(
        "Gloom",
        "The additional mana is paid at the same time the spell/ability is cast/activated."
    );
    ruling!(
        "Gloom",
        "The extra mana is not considered part of the activation cost. It is a separate cost."
    );
    ruling!(
        "Gloom",
        "The extra mana is part of the total cost, so cost reducers can be applied to this cost."
    );
    supported("Gloom");
    supported("Savannah Lions");
    supported("Helm of Awakening");
    supported("Gerrard's Battle Cry");
    // Savannah Lions ({W}) costs {3}{W}: not castable with {2}{W}.
    let mut t = main_phase(2);
    t.battlefield(P1, "Gloom");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let lions = t.hand(P0, "Savannah Lions");
    assert!(t.cast(P0, lions).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, lions).go();
    // All four lands were tapped as it was cast; its mana value is still 1.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
    assert_eq!(t.g.obj(spell).chars.mana_value(), 1);
    t.resolve_all();
    // With Helm of Awakening ({1} less): {2}{W}.
    let mut t = main_phase(2);
    t.battlefield(P1, "Gloom");
    t.battlefield(P1, "Helm of Awakening");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let lions = t.hand(P0, "Savannah Lions");
    t.cast(P0, lions).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    // A white enchantment's ability ({2}{W}) costs {5}{W}, paid as it's activated.
    let mut t = main_phase(2);
    t.battlefield(P1, "Gloom");
    let cry = t.battlefield(P0, "Gerrard's Battle Cry");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    assert!(t.activate(P0, cry, 0, &[]).is_err());
    t.lands(P0, "Wastes", 1);
    t.activate(P0, cry, 0, &[]).unwrap();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 6);
}

// ---------------------------------------------------------------------------
// Yuna's Decision
// ---------------------------------------------------------------------------

#[test]
fn yunas_decision_land_isnt_a_land_play() {
    cr!("305.4", "305.2");
    ruling!(
        "Yuna's Decision",
        "Putting a land card onto the battlefield with the first mode of Yuna's Decision doesn't count as playing a land. You can put a land card onto the battlefield this way even if you've already played a land for the turn."
    );
    supported("Yuna's Decision");
    for land_first in [true, false] {
        let mut t = main_phase(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let f1 = t.hand(P0, "Forest");
        let f2 = t.hand(P0, "Forest");
        if land_first {
            t.play_land(P0, f1).unwrap();
        }
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        t.answer_choose(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(f2)]);
        t.answer_yes(P0, true);
        cast_resolve(&mut t, P0, "Yuna's Decision", &[]);
        assert!(t.on_battlefield(t.g.current(f2)), "land first {land_first}");
        assert_eq!(t.g.players[0].lands_played_this_turn, u32::from(land_first));
        if !land_first {
            // The land play is still available.
            t.play_land(P0, f1).unwrap();
            assert!(t.on_battlefield(t.g.current(f1)));
        }
    }
}
