//! Item `rulings-residue-misc-1`, pass B — "mana value matters" rulings (batch P099): what
//! a mana value is (CR 202.3: tokens, {X}, cards with no mana cost, alternative costs),
//! cards that compare mana values, and the triggers and searches that use them.

use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Gives `p` `n` mana of every type.
fn mana_for(t: &mut TestGame, p: PlayerId, n: u32) {
    fill_pool(t, p, n);
}

#[test]
fn neoform_adds_one_counter() {
    cr!("122.6", "614.1c");
    ruling!(
        "Neoform",
        "A creature put onto the battlefield this way enters the battlefield with one +1/+1 counter if it would otherwise enter with no +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let courser = t.library_top(P0, "Centaur Courser");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(courser)]);
    cast_resolve(&mut t, P0, "Neoform", &[]);
    let courser = t.g.current(courser);
    assert!(t.on_battlefield(courser));
    assert_eq!(p1(&t, courser), 1);
}

#[test]
fn chalice_of_the_void_x_twice() {
    cr!("107.3", "202.3e");
    ruling!(
        "Chalice of the Void",
        "A mana cost of {X}{X} means that you pay twice X."
    );
    let mut t = TestGame::new(2);
    let chalice = t.hand(P0, "Chalice of the Void");
    tap_lands(&mut t, P0);
    mana_for(&mut t, P0, 3);
    let before = pool(&t, P0);
    t.cast(P0, chalice).x(3).go();
    assert_eq!(before - pool(&t, P0), 6);
    t.resolve_all();
    assert_eq!(t.counters(chalice, counters::CHARGE), 3);
}

#[test]
fn gaddock_teeg_looks_at_the_mana_cost_not_the_cost_paid() {
    cr!("202.3b", "118.9c");
    ruling!(
        "Gaddock Teeg",
        "Alternative costs (including casting a spell \"without paying its mana cost\"), additional costs, and cost reductions don't affect a spell's mana value."
    );
    // Force of Will (mana value 5) cast for its alternative cost ("pay 1 life, exile a
    // blue card from your hand") with Lightning Bolt to counter.
    for teeg in [false, true] {
        let mut t = TestGame::new(2);
        if teeg {
            t.battlefield(P1, "Gaddock Teeg");
        }
        t.hand(P0, "Counterspell");
        let fow = t.hand(P0, "Force of Will");
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.g.turn.priority = Some(P1);
        t.cast(P1, bolt).target(P0).go();
        tap_lands(&mut t, P0);
        t.g.turn.priority = Some(P0);
        let alt = t.g.legal_actions(P0).iter().any(|a| {
            matches!(a, Action::Cast { card, method: mtg_engine::object::CastMethod::Alternative(_) } if *card == fow)
        });
        assert_eq!(alt, !teeg, "Teeg {teeg}");
    }
}

#[test]
fn oswald_fiddlebender_token_and_x_artifacts_have_mana_value_0() {
    cr!("202.3b", "202.3e", "111.4");
    ruling!(
        "Oswald Fiddlebender",
        "A token has a mana value of 0 unless it's copying something else."
    );
    ruling!(
        "Oswald Fiddlebender",
        "If its mana cost includes {X}, X is considered to be 0."
    );
    // A Treasure token, and a Chalice of the Void that entered with X = 2.
    for chalice in [false, true] {
        let mut t = TestGame::new(2);
        let oswald = t.battlefield(P0, "Oswald Fiddlebender");
        let sac = if chalice {
            let c = t.battlefield(P0, "Chalice of the Void");
            t.g.objects[c.0 as usize]
                .counters
                .insert(counters::CHARGE.into(), 2);
            c
        } else {
            create_token(&mut t, P0, "Treasure")
        };
        let splitter = t.library_top(P0, "Bonesplitter");
        t.library_top(P0, "Mind Stone");
        t.library_top(P0, "Sol Ring");
        t.lands(P0, "Plains", 1);
        t.answer_choose(P0, &[obj(sac)]);
        t.answer_choose(P0, &[obj(splitter)]);
        t.activate(P0, oswald, 0, &[]).unwrap();
        t.resolve_all();
        assert!(
            t.on_battlefield(t.g.current(splitter)),
            "a mana value 1 artifact (chalice {chalice})"
        );
    }
}

#[test]
fn karns_sylex_stops_paying_life_for_cycling() {
    cr!("602.1", "702.29a");
    ruling!(
        "Karn's Sylex",
        "Some keyword abilities are activated abilities (such as cycling) and will have colons in their reminder text."
    );
    // Street Wraith: "Cycling—Pay 2 life."
    for sylex in [false, true] {
        let mut t = TestGame::new(2);
        if sylex {
            t.battlefield(P1, "Karn's Sylex");
        }
        let wraith = t.hand(P0, "Street Wraith");
        assert_eq!(can_activate(&mut t, P0, wraith), !sylex, "Sylex {sylex}");
    }
}

#[test]
fn sudden_insight_counts_mana_value_0_but_not_lands() {
    cr!("202.3a", "202.3e");
    ruling!(
        "Sudden Insight",
        "Although land cards aren't considered, nonland cards with mana value 0 are."
    );
    let mut t = TestGame::new(2);
    // Endless One ({X}) and Divination ({2}{U}); and a Forest.
    t.graveyard(P0, "Endless One");
    t.graveyard(P0, "Divination");
    t.graveyard(P0, "Forest");
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Sudden Insight", &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn ancient_cellarspawn_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Ancient Cellarspawn",
        "Ancient Cellarspawn's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ancient Cellarspawn");
    // A second Ancient Cellarspawn (a Horror, mana value 3) costs {B}{B}.
    tap_lands(&mut t, P0);
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::B, 2);
    t.answer_targets(P0, &[P1.into()]);
    let second = t.hand(P0, "Ancient Cellarspawn");
    t.cast(P0, second).go();
    assert_eq!(pool(&t, P0), 0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the spell in response.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(second).go();
    t.resolve_all();
    assert_eq!(t.zone(second), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn temporary_lockdown_returning_auras_and_what_stays_behind() {
    cr!("303.4f", "303.4i", "704.5m", "301.5c", "122.2", "610.3c");
    ruling!(
        "Temporary Lockdown",
        "Aura cards exiled this way will return to the battlefield attached to a permanent that they could enchant (based on their enchant ability) chosen by their owner. They cannot enchant any permanents that are entering the battlefield at the same time."
    );
    ruling!(
        "Temporary Lockdown",
        "Auras with mana value 3 or greater attached to an exiled permanent will be put into their owners’ graveyards. Equipment with mana value 3 or greater attached to an exiled creature will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Pacifism (mana value 2) on the Bears: both are exiled and return together.
    let pacifism = attach_new(&mut t, P0, "Pacifism", bears);
    // Faith's Fetters (4) and Loxodon Warhammer (3) on the Bears too, and a counter.
    let fetters = attach_new(&mut t, P0, "Faith's Fetters", bears);
    let hammer = attach_new(&mut t, P1, "Loxodon Warhammer", bears);
    t.g.objects[bears.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 1);
    let lockdown = cast_new(&mut t, P0, "Temporary Lockdown", &[]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(pacifism), Zone::Exile);
    assert_eq!(t.zone(fetters), Zone::Graveyard(P0));
    assert!(t.on_battlefield(hammer));
    assert_eq!(attached_to(&t, hammer), None);
    // Lockdown leaves: the Bears return with no counters; Pacifism can't enchant them as
    // they enter at the same time, so its owner puts it on the Hill Giant.
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(giant)]);
    destroy(&mut t, lockdown);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(p1(&t, bears), 0);
    assert_eq!(attached_to(&t, pacifism), Some(obj(giant)));
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty());
    assert!(
        offered.iter().all(|c| !c.contains(&obj(bears))),
        "{offered:?}"
    );
}

#[test]
fn a_spell_cast_with_yennett_affects_blocks_not_attacks() {
    cr!("508.1", "509.1a", "509.1b");
    ruling!(
        "Yennett, Cryptic Sovereign",
        "Because all attackers are chosen at once, a spell cast this way can't affect which creatures you can attack with, but it can affect which creatures will be able to block."
    );
    // Panic ({R}, mana value 1): "Cast this spell only during combat before blockers are
    // declared. Target creature can't block this turn. ..."
    let mut t = TestGame::new(2);
    let yennett = t.battlefield(P0, "Yennett, Cryptic Sovereign");
    let blocker = t.battlefield(P1, "Wall of Air");
    let panic = t.library_top(P0, "Panic");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(blocker)]);
    t.attack(&[(yennett, P1.into())], &[(blocker, yennett)]);
    assert_eq!(t.zone(panic), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 17, "the Wall of Air couldn't block");
    // Expedite ({R}): "Target creature gains haste until end of turn. Draw a card." A
    // creature given haste by it isn't attacking: attackers were already declared.
    let mut t = TestGame::new(2);
    let yennett = t.battlefield(P0, "Yennett, Cryptic Sovereign");
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    t.library_top(P0, "Expedite");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(sick)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(yennett, P1.into())]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    assert!(has(&mut t, sick, keywords::KeywordKind::Haste));
    assert!(!is_attacking(&t, sick));
}

const KIANNE: &str = "Kianne, Dean of Substance // Imbraham, Dean of Theory";

#[test]
fn kianne_exiles_face_up_and_the_study_counters_stay() {
    cr!("406.3", "122.2", "400.7");
    ruling!(
        "Kianne, Dean of Substance // Imbraham, Dean of Theory",
        "Cards are exiled face up unless an effect states otherwise."
    );
    ruling!(
        "Kianne, Dean of Substance // Imbraham, Dean of Theory",
        "Exiled cards with study counters on them retain their study counters even if Kianne or Imbraham leave the battlefield."
    );
    let mut t = TestGame::new(2);
    let kianne = t.battlefield(P0, KIANNE);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, kianne, 0, &[]).unwrap();
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert!(!t.obj_now(bolt).face_down);
    assert_eq!(t.counters(bolt, "study"), 1);
    destroy(&mut t, kianne);
    t.resolve_all();
    assert_eq!(t.counters(bolt, "study"), 1);
}

#[test]
fn desecrate_reality_can_exile_a_land() {
    cr!("202.3a", "115.1");
    ruling!(
        "Desecrate Reality",
        "Cards with no mana cost, like land cards, have a mana value of zero."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    cast_resolve(&mut t, P0, "Desecrate Reality", &[obj(forest)]);
    assert_eq!(t.zone(forest), Zone::Exile);
}

#[test]
fn eidolon_of_the_great_revel_doesnt_see_itself_cast() {
    cr!("603.2", "603.6a");
    ruling!(
        "Eidolon of the Great Revel",
        "Casting Eidolon of the Great Revel doesn't cause its own ability to trigger."
    );
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Eidolon of the Great Revel", &[]);
    assert_eq!(t.life(P0), 20);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[P1.into()]);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn chalice_of_the_void_must_be_there_as_the_spell_is_cast() {
    cr!("601.2i", "603.2", "603.10a");
    ruling!(
        "Chalice of the Void",
        "If you sacrifice Chalice of the Void as a cost to cast a spell, its ability can't trigger. However, if it leaves the battlefield once its ability has triggered, that ability will still counter the spell."
    );
    // Sacrificed to Shrapnel Blast (mana value 2) with two charge counters.
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    t.g.objects[chalice.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 2);
    t.answer_choose(P0, &[obj(chalice)]);
    cast_resolve(&mut t, P0, "Shrapnel Blast", &[P1.into()]);
    assert_eq!(t.life(P1), 15);
    // With one counter, Lightning Bolt triggers it; destroyed in response, it still
    // counters the Bolt.
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P1, "Chalice of the Void");
    t.g.objects[chalice.0 as usize]
        .counters
        .insert(counters::CHARGE.into(), 1);
    cast_new(&mut t, P0, "Lightning Bolt", &[P1.into()]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, chalice);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn wrath_of_the_skies_paying_no_energy_destroys_mana_value_0() {
    cr!("202.3", "107.14");
    ruling!(
        "Wrath of the Skies",
        "Choosing not to pay any {E} will have the same result as choosing to pay zero {E}."
    );
    for answer in [None, Some(0)] {
        let mut t = TestGame::new(2);
        let orni = t.battlefield(P1, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if let Some(n) = answer {
            t.answer(P0, DecisionKind::Number, Answer::Number(n));
        }
        lands_for_cost(&mut t, P0, "Wrath of the Skies");
        let wrath = t.hand(P0, "Wrath of the Skies");
        t.cast(P0, wrath).x(0).go();
        t.resolve_all();
        assert!(!t.on_battlefield(orni), "{answer:?}");
        assert!(t.on_battlefield(bears), "{answer:?}");
    }
}

#[test]
fn in_search_of_greatness_with_only_lands() {
    cr!("608.2h", "202.3a");
    ruling!(
        "In Search of Greatness",
        "If the only other permanents you control are lands, or in the unlikely event you don't control any other permanents, you may cast a permanent spell with mana value 1 from your hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "In Search of Greatness");
    t.battlefield(P0, "Forest");
    let elves = t.hand(P0, "Llanowar Elves");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::Untap);
    t.answer_choose(P0, &[obj(elves)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(elves)));
}

#[test]
fn droning_bureaucrats_with_x_0_stops_tokens_and_land_creatures() {
    cr!("202.3b", "202.3e", "111.4");
    ruling!(
        "Droning Bureaucrats",
        "Creature lands and most token creatures have mana value 0. If a creature on the battlefield has {X} in its mana cost, treat X as 0."
    );
    let mut t = TestGame::new(2);
    let bureaucrats = t.battlefield(P0, "Droning Bureaucrats");
    let token = create_token(&mut t, P0, "Soldier");
    t.g.objects[token.0 as usize].summoning_sick = false;
    let wastes = t.battlefield(P0, "Wastes");
    animate(&mut t, wastes, 2, 2);
    let endless = t.battlefield(P0, "Endless One");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, bureaucrats, 0, &[]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, token));
    assert!(!can_attack(&mut t, wastes));
    assert!(!can_attack(&mut t, endless));
    assert!(can_attack(&mut t, bears));
}

#[test]
fn dauntless_dismantler_destroys_exactly_x() {
    cr!("107.3m");
    ruling!(
        "Dauntless Dismantler",
        "Dauntless Dismantler's last ability destroys only artifacts whose mana values are exactly X."
    );
    let mut t = TestGame::new(2);
    let dismantler = t.battlefield(P0, "Dauntless Dismantler");
    let orni = t.battlefield(P1, "Ornithopter");
    let splitter = t.battlefield(P1, "Bonesplitter");
    let stone = t.battlefield(P1, "Mind Stone");
    mana_for(&mut t, P0, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, dismantler, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(orni));
    assert!(!t.on_battlefield(splitter));
    assert!(t.on_battlefield(stone));
}

#[test]
fn electrodominance_doesnt_let_you_play_a_land() {
    cr!("305.9", "601.1");
    ruling!(
        "Electrodominance",
        "Effects that allow you to cast a spell don't allow you to play a land."
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let orni = t.hand(P0, "Ornithopter");
    let from = t.asked().len();
    lands_for_cost(&mut t, P0, "Electrodominance");
    let e = t.hand(P0, "Electrodominance");
    t.cast(P0, e).x(0).target(P1).go();
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(offered.contains(&obj(orni)), "{offered:?}");
    assert!(!offered.contains(&obj(forest)));
    assert_eq!(t.zone(forest), Zone::Hand(P0));
}

#[test]
fn capitoline_triads_base_9_9_and_earlier_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "The Capitoline Triad",
        "Effects that modify a creature’s power and/or toughness without setting it will apply to creatures you control no matter when they started to take effect."
    );
    let mut t = TestGame::new(2);
    let triad = t.battlefield(P0, "The Capitoline Triad");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bears.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 1);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(pt_now(&mut t, bears), (6, 6));
    // Historic cards with total mana value 35: Blightsteel Colossus (12) twice and
    // Darksteel Colossus (11).
    let a = t.graveyard(P0, "Blightsteel Colossus");
    let b = t.graveyard(P0, "Darksteel Colossus");
    let f = t.graveyard(P0, "Blightsteel Colossus");
    t.answer_choose(P0, &[obj(a), obj(b), obj(f)]);
    t.activate(P0, triad, 0, &[]).unwrap();
    t.resolve_all();
    // Base 9/9, then +1/+1 from the counter and +3/+3 from Giant Growth.
    assert_eq!(pt_now(&mut t, bears), (13, 13));
}

#[test]
fn atemsis_counts_mana_value_0() {
    cr!("202.3a", "104.3e");
    ruling!(
        "Atemsis, All-Seeing",
        "A card with no mana cost has a mana value of 0, and 0 can be one of the six mana values that count toward Atemsis's ability."
    );
    for with_land in [false, true] {
        let mut t = TestGame::new(2);
        let atemsis = t.battlefield(P0, "Atemsis, All-Seeing");
        for name in [
            "Llanowar Elves",
            "Grizzly Bears",
            "Centaur Courser",
            "Hill Giant",
            "Air Elemental",
        ] {
            t.hand(P0, name);
        }
        if with_land {
            t.hand(P0, "Forest");
        }
        t.answer_yes(P0, true);
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(atemsis, P1.into())]),
        );
        t.g.run_until(10_000, |g| g.turn.step == Step::EndOfCombat || g.is_over());
        assert_eq!(t.has_lost(P1), with_land, "land {with_land}");
    }
}

#[test]
fn fires_of_invention_doesnt_restrict_lands_or_abilities() {
    cr!("305.2", "602.5");
    ruling!(
        "Fires of Invention",
        "Fires of Invention doesn't impose any restrictions on playing lands or activating abilities."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fires of Invention");
    cast_resolve(&mut t, P0, "Ornithopter", &[]);
    cast_resolve(&mut t, P0, "Ornithopter", &[]);
    let third = t.hand(P0, "Ornithopter");
    assert!(!can_cast(
        &mut t,
        P0,
        third,
        mtg_engine::object::CastMethod::Normal
    ));
    let forest = t.hand(P0, "Forest");
    assert!(t.play_land(P0, forest).is_ok());
    let seal = t.battlefield(P0, "Seal of Fire");
    assert!(t.activate(P0, seal, 0, &[P1.into()]).is_ok());
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tariff_asks_the_active_player_first_and_sacrifices_at_once() {
    cr!("101.4", "608.2e", "603.2c");
    ruling!(
        "Tariff",
        "First the active player identifies the creature they control with the highest mana cost (and chooses one if there's a tie) and chooses whether to sacrifice it or pay mana. Then each other player in turn order does the same. All creatures sacrificed this way are sacrificed at the same time."
    );
    let mut t = TestGame::new(2);
    // Chainsaw: "Whenever one or more creatures die, put a rev counter on this Equipment."
    let saw = t.battlefield(P0, "Chainsaw");
    let b0 = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Llanowar Elves");
    let b1 = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Raging Goblin");
    // Each could pay: P0 has {1}{G} after Tariff, P1 {3}{R}.
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Mountain", 4);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Tariff", &[]);
    assert!(!t.on_battlefield(b0));
    assert!(!t.on_battlefield(b1));
    assert_eq!(t.counters(saw, "rev"), 1, "one batch of deaths");
    // P0 chooses its creature and whether to pay, then P1 does.
    let who: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(who, vec![P0, P0, P1, P1]);
}

#[test]
fn lukka_exiled_creatures_castable_with_any_lukka() {
    cr!("611.2a", "113.6");
    ruling!(
        "Lukka, Coppercoat Outcast",
        "For each creature card exiled with Lukka's first ability, you can cast it as long as you control any Lukka planeswalker, not just the one that exiled the card."
    );
    let mut t = TestGame::new(2);
    let lukka = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, lukka, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    assert!(can_cast(
        &mut t,
        P0,
        bears,
        mtg_engine::object::CastMethod::Normal
    ));
    destroy(&mut t, lukka);
    t.resolve_all();
    assert!(!can_cast(
        &mut t,
        P0,
        bears,
        mtg_engine::object::CastMethod::Normal
    ));
    t.battlefield(P0, "Lukka, Bound to Ruin");
    assert!(can_cast(
        &mut t,
        P0,
        bears,
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn agadeems_awakening_one_card_of_each_mana_value_up_to_x() {
    cr!("601.2c", "107.3");
    ruling!(
        "Agadeem's Awakening // Agadeem, the Undercrypt",
        "if X is 3, you may target up to four creature cards: one with mana value 3, one with mana value 2, one with mana value 1, and one with mana value 0."
    );
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = [
        "Centaur Courser",
        "Grizzly Bears",
        "Llanowar Elves",
        "Ornithopter",
    ]
    .iter()
    .map(|n| t.graveyard(P0, n))
    .collect();
    let spell = t.hand(P0, "Agadeem's Awakening // Agadeem, the Undercrypt");
    mana_for(&mut t, P0, 3);
    let targets: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.cast(P0, spell).x(3).targets(&targets).go();
    t.resolve_all();
    for c in cards {
        assert!(t.on_battlefield(t.g.current(c)));
    }
}
