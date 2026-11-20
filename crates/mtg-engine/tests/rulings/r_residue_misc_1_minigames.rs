//! Item `rulings-residue-misc-1` (pass C) — guesses, flips, votes, piles and the values
//! abilities look at as they resolve: Master of Predicaments (casting without paying its
//! mana cost as an effect resolves, CR 608.2g, 118.9, 601.3e), Friendly Fire and {X}
//! (CR 107.3e), Mana Clash, Faramir, Prince of Ithilien (attacking a planeswalker or battle
//! isn't attacking you, CR 506.2), Raging River's piles and creatures that arrive later,
//! Vault 11 and Prisoner's Dilemma (choices made on resolution, CR 608.2c), Psychic Battle
//! (changing targets, CR 115.7), Rebound, Goliath Hatchery's total toxic value
//! (CR 702.164c), Altar of the Pantheon (CR 700.5), Luck Bobblehead's X, Woodcaller
//! Automaton's and Phyrexian Totem's last known information (CR 608.2h), Infuse and
//! Strixhaven Stadium.

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Master of Predicaments
// ---------------------------------------------------------------------------------------

/// P1 guesses whether the card's mana value is greater than 4.
fn guess(t: &mut TestGame, p: PlayerId, greater: bool) {
    t.answer(
        p,
        DecisionKind::Option,
        Answer::Index(if greater { 0 } else { 1 }),
    );
}

/// P0's Master of Predicaments (4/4) deals combat damage to P1: P0 chooses `card` (put into
/// P0's hand), P1 guesses `greater`, and P0 answers `cast` to "you may cast it". `more`
/// queues further answers before combat. Returns the game and the chosen card.
fn predicament(
    card: &str,
    greater: bool,
    cast: bool,
    more: impl FnOnce(&mut TestGame),
) -> (TestGame, ObjectId) {
    supported("Master of Predicaments");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Predicaments");
    let c = t.hand(P0, card);
    t.answer_choose(P0, &[obj(c)]);
    guess(&mut t, P1, greater);
    t.answer_yes(P0, cast);
    more(&mut t);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(master, pl(P1))], &[]);
    t.resolve_all();
    (t, c)
}

#[test]
fn master_of_predicaments_cant_play_a_chosen_land() {
    cr!("305.9", "608.2g");
    ruling!(
        "Master of Predicaments",
        "Although you may choose a land card in your hand, you won't be able to play it even if the player guessed wrong."
    );
    // Forest: mana value 0, P1 guesses "greater than 4" (wrong).
    let (t, forest) = predicament("Forest", true, true, |_| {});
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn master_of_predicaments_and_friendly_fire_x_is_0() {
    cr!("107.3e", "202.3e", "601.2b");
    ruling!(
        "Master of Predicaments",
        "If the card has {X} in its mana cost, X is 0."
    );
    ruling!(
        "Friendly Fire",
        "If the card has {X} in its mana cost, X is 0."
    );
    // Fireball cast without paying its mana cost: X is 0, no damage.
    let (t, fireball) = predicament("Fireball", true, true, |t| {
        t.answer_targets(P0, &[pl(P1)]);
    });
    assert_eq!(t.zone(fireball), Zone::Graveyard(P0), "it was cast");
    assert_eq!(t.life(P1), 16, "only the Master's 4");
    // Friendly Fire revealing a Fireball in hand: its mana value counts X as 0, so 1.
    supported("Friendly Fire");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P1, "Fireball");
    assert_eq!(t.hand_size(P1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Friendly Fire", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(damage_on(&t, giant), 1);
}

#[test]
fn master_of_predicaments_uses_a_split_cards_combined_mana_value() {
    cr!("709.4", "709.3", "702.102a");
    ruling!(
        "Master of Predicaments",
        "If the card is a split card, its mana value is determined by the combined mana cost of its two halves. If the player guesses wrong and you cast a split card this way, you may cast either half (or both halves if the card has fuse), no matter what its mana value is."
    );
    // Turn // Burn: {2}{U} and {1}{R}, mana value 5. P1 guesses "not greater" (wrong);
    // P0 may cast either half or both fused: P0 casts Burn (2 damage to any target).
    let (t, _) = predicament("Turn // Burn", false, true, |t| {
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        t.answer_targets(P0, &[pl(P1)]);
    });
    let methods: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseCastingMethod { options, .. } if p == P0 => Some(options),
            Decision::ChooseOption { options, .. } if p == P0 && options.len() >= 2 => {
                Some(options)
            }
            _ => None,
        })
        .collect();
    assert!(
        methods.iter().any(|o| o.len() >= 3),
        "either half or both: {methods:?}"
    );
    assert!(t.life(P1) < 16, "a half was cast: {}", t.life(P1));
}

#[test]
fn master_of_predicaments_pays_additional_costs_but_no_alternative_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f", "702.103b");
    ruling!(
        "Master of Predicaments",
        "If you cast a card \"without paying its mana cost,\" you can't pay any alternative costs. Notably, you can't cast a card with bestow as an Aura this way. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    // Kicker: Burst Lightning kicked ({4}) deals 4.
    let (t, _) = predicament("Burst Lightning", true, true, |t| {
        t.lands(P0, "Wastes", 4);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P0, &[pl(P1)]);
    });
    assert_eq!(
        t.life(P1),
        12,
        "the Master's 4 and the kicked Burst Lightning's 4"
    );
    // A mandatory additional cost: Bone Splinters sacrifices the Bears.
    let mut bears = None;
    let (t, _) = predicament("Bone Splinters", true, true, |t| {
        let b = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        bears = Some(b);
        t.answer_choose(P0, &[obj(b)]);
        t.answer_targets(P0, &[obj(giant)]);
    });
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let _ = bears;
    // Bestow: Boon Satyr is cast as a creature, not an Aura.
    let (t, satyr) = predicament("Boon Satyr", true, true, |t| {
        t.battlefield(P0, "Grizzly Bears");
    });
    let satyr = t.g.current(satyr);
    assert!(t.on_battlefield(satyr));
    assert!(t.obj_now(satyr).is_creature());
    assert_eq!(attached_to(&t, satyr), None);
    let offered_bestow = t.asked().into_iter().any(|(_, d)| {
        matches!(&d, Decision::ChooseCastingMethod { options, .. }
            if options.iter().any(|o| o.to_lowercase().contains("bestow")))
    });
    assert!(!offered_bestow);
}

#[test]
fn master_of_predicaments_doesnt_reveal_a_card_it_doesnt_cast() {
    cr!("701.20a", "608.2c");
    ruling!(
        "Master of Predicaments",
        "If you don't cast the card, either because the player guessed right or because you choose not to cast it, you don't have to reveal it. Your opponent won't know the reason you didn't cast the card."
    );
    // Guessed right (Grizzly Bears, "not greater"), then guessed wrong but not cast.
    for (greater, cast) in [(false, true), (true, false)] {
        let (t, bears) = predicament("Grizzly Bears", greater, cast, |_| {});
        assert_eq!(t.zone(bears), Zone::Hand(P0));
        assert!(!mtg_engine::reveal::is_revealed(&t.g, bears));
    }
}

#[test]
fn master_of_predicaments_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "601.3e", "307.5");
    ruling!(
        "Master of Predicaments",
        "You cast the card in your hand as part of the resolution of the triggered ability. Timing restrictions based on the card's type (such as creature or sorcery) are ignored. Other restrictions, such as \"Cast [this card] only during combat,\" are not."
    );
    // A sorcery is cast in the combat damage step.
    let (t, div) = predicament("Divination", true, true, |_| {});
    assert_eq!(t.zone(div), Zone::Graveyard(P0));
    // Berserk ("Cast this spell only before the combat damage step.") can't be.
    let (t, berserk) = predicament("Berserk", true, true, |t| {
        t.battlefield(P0, "Grizzly Bears");
    });
    assert_eq!(t.zone(berserk), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------------------
// Mana Clash, Faramir
// ---------------------------------------------------------------------------------------

#[test]
fn mana_clash_flips_for_you_and_targets_an_opponent() {
    cr!("705.1", "115.1a");
    ruling!(
        "Mana Clash",
        "Always affects the player and targets one of their opponents."
    );
    supported("Mana Clash");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    lands_for_cost(&mut t, P0, "Mana Clash");
    let c = t.hand(P0, "Mana Clash");
    // P0 tails and P1 heads, then both heads: only P0 is dealt 1.
    t.g.dice.loaded_coins.extend([false, true, true, true]);
    let from = t.asked().len();
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, c).go();
    // It can't target its caster.
    assert_eq!(
        crate::r_s02_common::target_candidates(&t, P0, from),
        vec![vec![pl(P1)]]
    );
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (19, 20));
}

#[test]
fn faramir_attacking_your_planeswalker_or_battle_isnt_attacking_you() {
    cr!("506.2", "508.1b", "310.8");
    ruling!(
        "Faramir, Prince of Ithilien",
        "Attacking a planeswalker you control or battle you're protecting doesn't count as attacking you."
    );
    supported("Faramir, Prince of Ithilien");
    for battle in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Faramir, Prince of Ithilien");
        let what = if battle {
            let id = t.battlefield(P1, "Invasion of Azgol");
            t.g.objects[id.0 as usize].choices.player = Some(P0);
            t.g.dirty = true;
            id
        } else {
            let id = t.battlefield(P0, "Liliana of the Veil");
            t.g.add_counters(obj(id), counters::LOYALTY, 3, None);
            id
        };
        let bears = t.battlefield(P1, "Grizzly Bears");
        through_end_step(&mut t, P0);
        t.advance_to(P1, Step::BeginningOfCombat);
        t.attack(&[(bears, obj(what))], &[]);
        let hand = t.hand_size(P0);
        through_end_step(&mut t, P1);
        assert_eq!(t.hand_size(P0), hand + 1, "battle {battle}: P0 draws");
        assert_eq!(tokens_of(&t, P0, "Soldier"), 0, "battle {battle}");
    }
}

// ---------------------------------------------------------------------------------------
// Raging River
// ---------------------------------------------------------------------------------------

#[test]
fn raging_river_a_creature_arriving_later_cant_block_the_earlier_attackers() {
    cr!("509.1b", "506.4");
    ruling!(
        "Raging River",
        "If a creature enters (or something becomes a creature) after the ability resolves, that creature will not be able to block any creature that was attacking at the time the ability resolved."
    );
    supported("Raging River");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging River");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    // P1: the Giant left; P0: the Bears left.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(bear, pl(P1))]);
    t.resolve_all();
    // A creature that enters now isn't in either pile.
    let late = t.battlefield(P1, "Craw Wurm");
    block_and_finish(&mut t, P1, &[(late, bear)]);
    assert_eq!(t.life(P1), 18, "the Wurm couldn't block");
}

#[test]
fn raging_river_an_attacker_arriving_later_can_be_blocked_normally() {
    cr!("509.1b", "508.4");
    ruling!(
        "Raging River",
        "If a creature is put onto the battlefield attacking after the ability resolves, it can be blocked by any creature that could normally block it (including creatures that entered after the ability resolved)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging River");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(bear, pl(P1))]);
    t.resolve_all();
    let late_attacker = crate::r_p079_common::enter_attacking(&mut t, P0, "Savannah Lions", pl(P1));
    let late_blocker = t.battlefield(P1, "Craw Wurm");
    // The Giant (left) blocks the late Lions; so could the late Wurm, which blocks... the
    // Bears can't be blocked by the Giant (it's on the right).
    block_and_finish(&mut t, P1, &[(giant, late_attacker)]);
    assert!(
        !t.on_battlefield(late_attacker),
        "the Giant blocked the Lions"
    );
    let _ = late_blocker;
    let mut t2 = TestGame::new(2);
    t2.battlefield(P0, "Raging River");
    let bear = t2.battlefield(P0, "Grizzly Bears");
    t2.battlefield(P1, "Hill Giant");
    t2.answer(P1, DecisionKind::Option, Answer::Index(0));
    t2.answer(P0, DecisionKind::Option, Answer::Index(1));
    t2.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t2, &[(bear, pl(P1))]);
    t2.resolve_all();
    let late_attacker =
        crate::r_p079_common::enter_attacking(&mut t2, P0, "Savannah Lions", pl(P1));
    let late_blocker = t2.battlefield(P1, "Craw Wurm");
    block_and_finish(&mut t2, P1, &[(late_blocker, late_attacker)]);
    assert!(
        !t2.on_battlefield(late_attacker),
        "a creature that entered later blocked it"
    );
}

// ---------------------------------------------------------------------------------------
// Choices made as the spell or ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn vault_11_votes_only_as_its_chapter_ability_resolves() {
    cr!("701.38a", "608.2c", "714.2b");
    ruling!(
        "Vault 11: Voter's Dilemma",
        "No player votes until Vault 11's last ability resolves. Any responses to the ability must be made without knowing the outcome of the vote."
    );
    supported("Vault 11: Voter's Dilemma");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let saga = t.battlefield(P0, "Vault 11: Voter's Dilemma");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(bears)]);
    let from = t.asked().len();
    t.g.add_counters(obj(saga), "lore", 1, None);
    t.settle();
    // The chapter ability waits on the stack; nobody has voted.
    assert_eq!(triggers_on_stack(&t, "votes"), 1);
    assert!(!was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseEntities { .. }
    )));
    // A response: P1 gives the Bears +3/+3; the vote still destroys them.
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseEntities { .. }
    )));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn prisoners_dilemma_choices_are_made_on_resolution() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Prisoner's Dilemma",
        "Choices are made during the resolution of Prisoner's Dilemma, so any responses to Prisoner's Dilemma must be made without knowing the outcome of those choices. Have fun iterating!"
    );
    supported("Prisoner's Dilemma");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Prisoner's Dilemma", &[]);
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(!was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseOption { .. }
    )));
    t.resolve_all();
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::ChooseOption { .. }
    )));
    assert_eq!(t.life(P1), 16, "P1 chose silence");
}

// ---------------------------------------------------------------------------------------
// Psychic Battle, Rebound
// ---------------------------------------------------------------------------------------

/// P0 casts Fire (2 damage divided 1/1 between P1's Hill Giant and P1) with Psychic Battle
/// out; P1 reveals the greater card and gives `answers` for each target in turn. Returns
/// the game, the Giant, P0's Grizzly Bears and the spell.
fn psychic_fire(answers: &[&[Entity]]) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported("Psychic Battle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychic Battle");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.library_top(P1, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let fi = t.hand(P0, "Fire // Ice");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    for a in answers {
        t.answer_targets(P1, a);
    }
    let fire = t
        .cast(P0, fi)
        .method(mtg_engine::object::CastMethod::Half(0))
        .targets(&[obj(giant), pl(P1)])
        .go();
    t.settle();
    (t, giant, bears, fire)
}

#[test]
fn psychic_battle_may_change_all_some_or_none_of_the_targets() {
    cr!("115.7c", "115.7d");
    ruling!(
        "Psychic Battle",
        "You can change all, some, or none of the targets."
    );
    for (case, expect) in [
        ("none", (0, 1, 19, 20)),
        ("some", (1, 0, 19, 20)),
        ("all", (1, 0, 20, 19)),
    ] {
        let (mut t, giant, bears, _fire) = psychic_fire(&[]);
        t.clear_answers();
        let (a, b): (Vec<Entity>, Vec<Entity>) = match case {
            "none" => (vec![], vec![]),
            "some" => (vec![obj(bears)], vec![]),
            _ => (vec![obj(bears)], vec![pl(P0)]),
        };
        t.answer_targets(P1, &a);
        t.answer_targets(P1, &b);
        t.resolve_all();
        let got = (
            damage_on(&t, bears),
            damage_on(&t, giant),
            t.life(P1),
            t.life(P0),
        );
        assert_eq!(got, expect, "{case}");
    }
}

#[test]
fn psychic_battle_cant_change_the_number_or_make_illegal_or_duplicate_targets() {
    cr!("115.7e", "115.3");
    ruling!(
        "Psychic Battle",
        "You can't change the number of targets, choose an illegal target, or make something get targeted more than once."
    );
    let (mut t, giant, _bears, fire) = psychic_fire(&[]);
    t.clear_answers();
    let ring = t.battlefield(P0, "Sol Ring");
    // An illegal target (an artifact) for the Giant's, and the Giant again for P1's.
    t.answer_targets(P1, &[obj(ring)]);
    t.answer_targets(P1, &[obj(giant)]);
    t.resolve();
    let targets = crate::r_s25_common::targets_of(&t, fire);
    assert_eq!(targets, vec![obj(giant), pl(P1)], "unchanged, still two");
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 1);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn rebound_targets_only_a_spell_that_targets_a_player() {
    cr!("115.10a", "115.7", "303.4a");
    ruling!(
        "Rebound",
        "You can’t use this on a spell unless the target spell is an Aura or uses the word “target” in its text. If it isn’t an Aura and the word “target” is not there, then the spell does not target a player at all—it just affects a player without targeting them."
    );
    supported("Rebound");
    for (spell, targets, ok) in [
        ("Flame Rift", vec![], false),
        ("Lava Spike", vec![pl(P0)], true),
        ("Curse of the Pierced Heart", vec![pl(P0)], true),
    ] {
        let mut t = TestGame::new(3);
        t.advance_to(P1, Step::PrecombatMain);
        let s = cast_new(&mut t, P1, spell, &targets);
        lands_for_cost(&mut t, P0, "Rebound");
        let r = t.hand(P0, "Rebound");
        t.g.turn.priority = Some(P0);
        let res = t.cast(P0, r).target(obj(s)).try_go();
        assert_eq!(res.is_ok(), ok, "{spell}");
        if ok {
            t.answer_targets(P0, &[pl(P2)]);
            t.resolve();
            assert_eq!(
                crate::r_s25_common::targets_of(&t, s),
                vec![pl(P2)],
                "{spell}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------
// Goliath Hatchery
// ---------------------------------------------------------------------------------------

/// P0's Goliath Hatchery at the beginning of P0's upkeep with P1 at three poison counters
/// (the trigger on the stack).
fn hatchery_upkeep(t: &mut TestGame) {
    supported("Goliath Hatchery");
    let h = t.custom(
        P0,
        (*mtg_engine::card::card("Goliath Hatchery")).clone(),
        Zone::Battlefield,
    );
    let _ = h;
    t.g.add_counters(pl(P1), "poison", 3, None);
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
}

#[test]
fn goliath_hatchery_adds_up_every_instance_of_toxic() {
    cr!("702.164c", "702.164a");
    ruling!(
        "Goliath Hatchery",
        "The \"total toxic value\" of a creature is usually just the number after the word toxic on that creature. However, if a creature has gained multiple instances of toxic, the total toxic value of that creature is the sum of those numbers. For example, if a creature that has toxic 2 later gains toxic 1 due to an effect, it has both toxic 1 and toxic 2, and its total toxic value is 3."
    );
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        custom_card(
            "Toxic Two",
            "Creature — Horror",
            "{2}",
            Some((2, 2)),
            "Toxic 2",
        ),
        Zone::Battlefield,
    );
    hatchery_upkeep(&mut t);
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::with_n(
                KeywordKind::Toxic,
                1,
            ))],
            duration: Duration::EndOfTurn,
        },
        &[obj(c)],
    );
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(c)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn goliath_hatchery_may_choose_a_creature_without_toxic() {
    cr!("702.164c", "608.2c");
    ruling!(
        "Goliath Hatchery",
        "You may choose any creature you control, including one that doesn't have toxic, if you don't want to draw cards. If you don't control any creatures, you won't draw any cards."
    );
    for with_bears in [true, false] {
        let mut t = TestGame::new(2);
        let bears = with_bears.then(|| t.battlefield(P0, "Grizzly Bears"));
        hatchery_upkeep(&mut t);
        let hand = t.hand_size(P0);
        if let Some(b) = bears {
            t.answer_choose(P0, &[obj(b)]);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "bears {with_bears}");
    }
}

// ---------------------------------------------------------------------------------------
// Altar of the Pantheon
// ---------------------------------------------------------------------------------------

#[test]
fn altar_of_the_pantheon_increases_devotion_to_two_colors_by_one() {
    cr!("700.5", "700.5a");
    ruling!(
        "Altar of the Pantheon",
        "Your devotion to two colors (for example, your devotion to red and green) is increased by one, not by two or three."
    );
    supported("Xenagos, God of Revels");
    // Xenagos ({3}{R}{G}) is a creature while devotion to red and green is seven or more.
    // Symbols: Xenagos 2, Burning-Tree Emissary 2, Mogg Fanatic 1 = 5; the Altar adds one.
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, "Xenagos, God of Revels");
    t.battlefield(P0, "Burning-Tree Emissary");
    t.battlefield(P0, "Mogg Fanatic");
    t.battlefield(P0, "Altar of the Pantheon");
    t.recompute();
    assert!(!t.obj_now(x).is_creature(), "six, not seven");
    // One more symbol makes seven.
    t.battlefield(P0, "Mogg Fanatic");
    t.recompute();
    assert!(t.obj_now(x).is_creature());
}

#[test]
fn altar_of_the_pantheon_gains_just_1_life() {
    cr!("106.12", "605.1a");
    ruling!(
        "Altar of the Pantheon",
        "You gain just 1 life, no matter how many Gods, Demigods, and/or legendary enchantments you control beyond the first."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Purphoros, God of the Forge");
    t.battlefield(P0, "Xenagos, God of Revels");
    t.battlefield(P0, "Oath of Teferi");
    let altar = t.battlefield(P0, "Altar of the Pantheon");
    t.activate(P0, altar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// ---------------------------------------------------------------------------------------
// Values determined as the ability resolves; last known information
// ---------------------------------------------------------------------------------------

#[test]
fn luck_bobblehead_counts_bobbleheads_as_its_ability_resolves() {
    cr!("608.2h", "706.2");
    ruling!(
        "Luck Bobblehead",
        "The value of X is determined only once, as Luck Bobblehead’s last ability resolves."
    );
    supported("Luck Bobblehead");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Luck Bobblehead");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, b, 1, &[]).unwrap();
    // A second Bobblehead arrives in response: two dice.
    t.battlefield(P0, "Luck Bobblehead");
    t.g.dice.loaded.extend([2, 4]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Treasure"), 2);
}

#[test]
fn woodcaller_automaton_uses_its_power_and_toughness_on_resolution_or_last_known() {
    cr!("608.2h", "613.4b");
    ruling!(
        "Woodcaller Automaton",
        "Use Woodcaller Automaton's power and toughness at the time its triggered ability resolves to determine the base power and toughness of the Treefolk creature land. If Woodcaller Automaton is no longer on the battlefield, use the power and toughness it had the last time it was on the battlefield."
    );
    supported("Woodcaller Automaton");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let forest = t.battlefield(P0, "Forest");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_targets(P0, &[obj(forest)]);
        let w = cast_new(&mut t, P0, "Woodcaller Automaton", &[]);
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "Treefolk"), 1);
        let w = t.g.current(w);
        cast_new(&mut t, P0, "Giant Growth", &[obj(w)]);
        t.resolve();
        if leaves {
            crate::r_p046_common::bounce(&mut t, w);
        }
        t.resolve_all();
        assert!(t.obj_now(forest).is_creature());
        assert_eq!(pt_now(&mut t, forest), (11, 11), "leaves {leaves}");
    }
}

#[test]
fn phyrexian_totem_sacrifices_that_many_including_itself_or_after_it_left() {
    cr!("603.4", "608.2h", "701.21a");
    ruling!(
        "Phyrexian Totem",
        "When damage is dealt to Phyrexian Totem, its triggered ability will trigger. A number of permanents equal to the amount of damage dealt will be sacrificed when the ability resolves. It doesn't matter if Phyrexian Totem isn't on the battlefield when the ability resolves, as long as Phyrexian Totem was a creature as it left the battlefield. If one of the permanents sacrificed is Phyrexian Totem itself, the requisite number of other permanents must still be sacrificed."
    );
    supported("Phyrexian Totem");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let totem = t.battlefield(P0, "Phyrexian Totem");
        let lands = t.lands(P0, "Swamp", 6);
        t.advance_to(P0, Step::PrecombatMain);
        t.activate(P0, totem, 1, &[]).unwrap();
        t.resolve_all();
        assert!(t.obj_now(totem).is_creature());
        t.g.turn.priority = Some(P1);
        cast_new(&mut t, P1, "Lightning Bolt", &[obj(totem)]);
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
        let untapped: Vec<ObjectId> = lands
            .iter()
            .copied()
            .filter(|l| !t.obj_now(*l).tapped)
            .collect();
        if gone {
            destroy(&mut t, totem);
            t.answer_choose(P0, &[obj(untapped[0]), obj(untapped[1]), obj(untapped[2])]);
        } else {
            t.answer_choose(P0, &[obj(totem), obj(untapped[0]), obj(untapped[1])]);
        }
        let before = t.g.permanents().filter(|o| o.controller == P0).count();
        t.resolve_all();
        let after = t.g.permanents().filter(|o| o.controller == P0).count();
        assert_eq!(before - after, 3, "gone {gone}");
        assert!(!t.on_battlefield(totem));
    }
}

#[test]
fn infuse_on_an_untapped_permanent_still_draws_next_upkeep() {
    cr!("701.26b", "603.7");
    ruling!(
        "Infuse",
        "You may target any artifact, creature, or land, not just a tapped one. If the targeted permanent is already untapped as Infuse resolves, Infuse won’t affect it, but you’ll still draw a card at the beginning of the next turn’s upkeep."
    );
    supported("Infuse");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Infuse", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn strixhaven_stadium_that_player_is_the_damaged_opponent() {
    cr!("104.3a", "510.2");
    ruling!(
        "Strixhaven Stadium",
        "“That player” refers to the opponent who was dealt damage."
    );
    supported("Strixhaven Stadium");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Strixhaven Stadium");
    crate::r_p108_common::put_counters(&mut t, s, "point", 9);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, pl(P2))], &[]);
    t.resolve_all();
    assert!(t.has_lost(P2));
    assert!(!t.has_lost(P1));
    assert_eq!(t.counters(s, "point"), 0);
}
