//! Rulings batch S08 — flying: the rulings shared by small groups of flying creatures
//! (and a few other cards of the same families): base P/T setting effects in layer 7b,
//! "if you cast it from your hand" on copies, choices made on resolution, gaining
//! vigilance after attacking, forced attacks, Spirit/Arcane spells with {X}, wishes,
//! token copies of attacking creatures, Equipment that are creatures, piles, Adventure
//! spells, "counter unless its controller pays", artifact-only mana, copies of dead
//! creatures, and cost reductions for creature spells with flying.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_base_pt_ability_overwrites_only_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Marsh Flitter",
        "The effect from the ability overwrites other effects that set power and/or toughness if and only if those effects existed before the ability resolved."
    );
    supported("Marsh Flitter");
    supported("Graven Dominator");
    // Marsh Flitter (1/1): "Sacrifice a Goblin: This creature has base power and
    // toughness 3/3 until end of turn." Graven Dominator: "When this creature enters ...,
    // each other creature has base power and toughness 1/1 until end of turn."
    let mut t = TestGame::new(2);
    let flitter = t.battlefield(P0, "Marsh Flitter");
    t.g.add_counters(Entity::Object(flitter), "+1/+1", 1, None);
    attach_new(&mut t, P0, "Bonesplitter", flitter);
    assert_eq!(t.pt(flitter), (4, 2));
    // An earlier setting effect: base 1/1.
    enter(&mut t, P1, "Graven Dominator");
    t.resolve_all();
    assert_eq!(t.pt(flitter), (4, 2));
    // The ability overwrites it; the counter and Bonesplitter still apply: 3/3 + 1/1 +
    // 2/0.
    let goblin = create_token(&mut t, P0, "Goblin");
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    t.activate(P0, flitter, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.g.is_live(goblin));
    assert_eq!(t.pt(flitter), (6, 4));
    // A switch always applies last.
    t.lands(P0, "Island", 1);
    let twist = t.hand(P0, "Twisted Image");
    t.cast(P0, twist).target(flitter).go();
    t.resolve_all();
    assert_eq!(t.pt(flitter), (4, 6));
    // A later setting effect overwrites the ability's: base 1/1 again.
    enter(&mut t, P1, "Graven Dominator");
    t.resolve_all();
    assert_eq!(t.pt(flitter), (2, 4));
}

#[test]
fn a_clone_cast_from_hand_copying_it_gets_the_cast_from_hand_trigger() {
    cr!("707.2", "603.4", "601.2");
    ruling!(
        "Reiver Demon",
        "If a creature (such as Clone) enters the battlefield as a copy of this creature, the copy’s “enters-the-battlefield” ability will still trigger as long as you cast that creature spell from your hand."
    );
    supported("Reiver Demon");
    supported("Clone");
    // Reiver Demon: "When this creature enters, if you cast it from your hand, destroy all
    // nonartifact, nonblack creatures."
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let demon = t.battlefield(P0, "Reiver Demon");
        t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(demon)]);
        if cast {
            t.lands(P0, "Island", 4);
            let clone = t.hand(P0, "Clone");
            t.cast(P0, clone).go();
        } else {
            t.enter(P0, "Clone");
        }
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Reiver Demon").len(), 2);
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), cast, "cast: {cast}");
    }
}

#[test]
fn the_color_is_chosen_as_the_ability_resolves() {
    cr!("608.2d", "603.3");
    ruling!(
        "Dromar, the Banisher",
        "You choose the color during resolution. This means your opponent does not get to react after knowing the color you chose."
    );
    supported("Dromar, the Banisher");
    // Dromar: "Whenever Dromar deals combat damage to a player, you may pay {2}{U}. If you
    // do, choose a color, then return all creatures of that color to their owners' hands."
    let mut t = TestGame::new(2);
    let dromar = t.battlefield(P0, "Dromar, the Banisher");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    attack_with(&mut t, &[(dromar, Entity::Player(P1))]);
    let ok = t.g.run_until(1000, |g| {
        g.turn.step == Step::CombatDamage && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(t.life(P1), 14);
    let color_asked = |t: &TestGame, from: usize| {
        count_asked_where(t, from, |d| {
            matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Choose a color")
        })
    };
    // The trigger is on the stack and every player gets priority: no color yet.
    assert_eq!(color_asked(&t, from), 0);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.resolve();
    assert_eq!(color_asked(&t, from), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(!t.g.is_live(bears));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(dromar));
}

/// Number of decisions matching `pred` asked since decision `from`.
fn count_asked_where(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> usize {
    t.asked()[from..].iter().filter(|(_, d)| pred(d)).count()
}

#[test]
fn gaining_vigilance_after_attacking_doesnt_untap_the_creature() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Senate Courier",
        "Gaining vigilance any time after the moment you choose to attack with a creature won't cause it to become untapped."
    );
    supported("Senate Courier");
    // Senate Courier: "{1}{W}: This creature gains vigilance until end of turn."
    for before in [true, false] {
        let mut t = TestGame::new(2);
        let courier = t.battlefield(P0, "Senate Courier");
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Wastes", 1);
        t.set_step(P0, Step::BeginningOfCombat);
        if before {
            t.activate(P0, courier, 0, &[]).unwrap();
            t.resolve_all();
        }
        attack_with(&mut t, &[(courier, Entity::Player(P1))]);
        if !before {
            t.activate(P0, courier, 0, &[]).unwrap();
            t.resolve_all();
            assert!(has_kw(
                &t,
                courier,
                mtg_engine::keywords::KeywordKind::Vigilance
            ));
        }
        assert_eq!(is_tapped(&t, courier), !before, "before: {before}");
    }
}

#[test]
fn creatures_that_must_attack_dont_if_tapped_sick_prevented_or_taxed() {
    cr!("508.1d", "302.6");
    ruling!(
        "Warmonger Hellkite",
        "If, during a player's declare attackers step, a creature is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under that player's control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Warmonger Hellkite");
    supported("Ghostly Prison");
    // Warmonger Hellkite: "All creatures attack each combat if able."
    for prison in [false, true] {
        let mut t = TestGame::new(2);
        let hellkite = t.battlefield(P0, "Warmonger Hellkite");
        let elves = t.battlefield(P0, "Llanowar Elves");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[bears.0 as usize].tapped = true;
        let giant = t.battlefield_sick(P0, "Hill Giant");
        let pacified = t.battlefield(P0, "Craw Wurm");
        attach_new(&mut t, P1, "Pacifism", pacified);
        if prison {
            // "Creatures can't attack you unless their controller pays {2} for each
            // creature they control that's attacking you."
            t.battlefield(P1, "Ghostly Prison");
            t.lands(P0, "Wastes", 4);
        }
        t.set_step(P0, Step::BeginningOfCombat);
        // P0 declares no attackers.
        attack_with(&mut t, &[]);
        let attackers = t.g.attackers();
        if prison {
            assert!(attackers.is_empty(), "{attackers:?}");
            assert_eq!(untapped_lands(&t, P0), 4);
        } else {
            assert!(attackers.contains(&hellkite) && attackers.contains(&elves));
            assert_eq!(attackers.len(), 2, "{attackers:?}");
        }
        for id in [bears, giant, pacified] {
            assert!(!t.g.is_attacking(id));
        }
    }
}

#[test]
fn a_spirit_or_arcane_spells_mana_value_includes_x() {
    cr!("202.3e", "107.3");
    ruling!(
        "Bounteous Kirin",
        "If the Spirit or Arcane spell has {X} in the mana cost, then you use the value of {X} on the stack."
    );
    supported("Bounteous Kirin");
    supported("Swallowing Plague");
    // Bounteous Kirin: "Whenever you cast a Spirit or Arcane spell, you may gain life
    // equal to that spell's mana value." Swallowing Plague: {X}{B}{B} Sorcery — Arcane,
    // "deals X damage to target creature and you gain X life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bounteous Kirin");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let plague = t.hand(P0, "Swallowing Plague");
    t.answer_yes(P0, true);
    t.cast(P0, plague).x(2).target(giant).go();
    t.resolve();
    // X = 2: mana value 4.
    assert_eq!(t.life(P0), 24);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.obj_now(giant).damage, 2);
}

#[test]
fn a_wish_gets_a_card_from_outside_the_game() {
    cr!("400.11a", "400.11b");
    ruling!(
        "Glittering Wish",
        "In a casual game, a card you choose from outside the game comes from your personal collection. In a tournament event, a card you choose from outside the game must come from your sideboard."
    );
    supported("Glittering Wish");
    // Glittering Wish: "You may reveal a multicolored card you own from outside the game
    // and put it into your hand. Exile Glittering Wish."
    let mut t = TestGame::new(2);
    let helix = place(&mut t, P0, "Lightning Helix", Zone::Outside(P0));
    let bolt = place(&mut t, P0, "Lightning Bolt", Zone::Outside(P0));
    place(&mut t, P1, "Boros Charm", Zone::Outside(P1));
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let wish = t.hand(P0, "Glittering Wish");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(helix)]);
    t.cast(P0, wish).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Helix"));
    assert!(t.in_exile("Glittering Wish"));
    // Only P0's multicolored card from outside the game was offered.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty());
    for c in offered {
        assert!(!c.contains(&Entity::Object(bolt)));
        assert!(c.iter().all(|e| *e == Entity::Object(helix)), "{c:?}");
    }
}

#[test]
fn a_token_copy_of_an_attacking_creature_copies_only_copiable_values() {
    cr!("707.2", "508.4", "111.10");
    ruling!(
        "Flamerush Rider",
        "It doesn't copy whether that creature has any counters on it or Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Flamerush Rider",
        "If the token isn't a creature as it enters the battlefield, it won't be attacking."
    );
    supported("Flamerush Rider");
    supported("Smuggler's Copter");
    // Flamerush Rider: "Whenever this creature attacks, create a token that's a copy of
    // another target attacking creature and that's tapped and attacking."
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    attach_new(&mut t, P0, "Bonesplitter", bears);
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 6));
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(rider, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    let copies: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(copies.len(), 1);
    let token = copies[0];
    assert_eq!(t.obj(token).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(token), (2, 2));
    assert_eq!(t.counters(token, "+1/+1"), 0);
    assert!(t.g.is_attacking(token) && is_tapped(&t, token));
    // A crewed Vehicle is a creature only because of a non-copy effect: its token copy
    // isn't a creature, so it isn't attacking.
    let mut t = TestGame::new(2);
    let rider = t.battlefield(P0, "Flamerush Rider");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let crewer = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, copter, &[crewer]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    t.answer_targets(P0, &[Entity::Object(copter)]);
    attack_with(
        &mut t,
        &[(rider, Entity::Player(P1)), (copter, Entity::Player(P1))],
    );
    // The Copter's own attack trigger (loot) may be on the stack too.
    t.resolve_all();
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 1);
    let token = copies[0];
    assert_eq!(t.obj(token).chars.name, "Smuggler's Copter");
    assert!(!is_creature(&t, token));
    assert!(!t.g.is_attacking(token));
}

#[test]
fn an_equipment_that_is_a_creature_cant_become_attached() {
    cr!("301.5c", "702.6a");
    ruling!(
        "Abuelo, Ancestral Echo",
        "An Equipment that's also a creature can't be attached to anything. You can activate its equip ability, but it won't become attached."
    );
    supported("March of the Machines");
    // March of the Machines: "Each noncreature artifact is an artifact creature with power
    // and toughness each equal to its mana value." Bonesplitter becomes a 1/1 creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    assert!(is_creature(&t, splitter));
    assert!(can_activate(&mut t, P0, splitter));
    t.activate(P0, splitter, 0, &[Entity::Object(bears)])
        .unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(untapped_lands(&t, P0), 0);
}

/// P0's Sphinx of Uthuun enters ("reveal the top five cards of your library. An opponent
/// separates those cards into two piles. Put one pile into your hand and the other into
/// your graveyard."). The top five cards of P0's library.
fn sphinx_enters(t: &mut TestGame) -> Vec<ObjectId> {
    let five = stack_library(
        t,
        P0,
        &[
            "Grizzly Bears",
            "Hill Giant",
            "Lightning Bolt",
            "Island",
            "Ornithopter",
        ],
    );
    enter(t, P0, "Sphinx of Uthuun");
    five
}

/// The players asked the decisions matching `pred` since decision `from`.
fn asked_who(t: &TestGame, from: usize, pred: impl Fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| pred(d))
        .map(|(p, _)| *p)
        .collect()
}

fn is_separation(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("piles"))
}

#[test]
fn you_not_the_opponent_choose_which_pile_goes_to_your_hand() {
    cr!("700.3", "700.3a");
    ruling!(
        "Sphinx of Uthuun",
        "You (not your opponent) choose which pile to put into your hand and which to put into your graveyard."
    );
    supported("Sphinx of Uthuun");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let five = sphinx_enters(&mut t);
    // P1 puts the Bolt alone in the first pile; P0 takes the other four.
    t.answer_choose(P1, &[Entity::Object(five[2])]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(asked_who(&t, from, is_separation), vec![P1]);
    assert_eq!(
        asked_who(&t, from, |d| matches!(
            d,
            Decision::ChooseOption { prompt, .. } if prompt == "Choose a pile"
        )),
        vec![P0]
    );
    assert_eq!(t.hand_size(P0), 4);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn a_pile_can_have_no_cards() {
    cr!("700.3d");
    ruling!(
        "Sphinx of Uthuun",
        "A pile can have no cards in it. In this case, you'll choose whether to put all the revealed cards into your hand or into your graveyard."
    );
    for take_all in [true, false] {
        let mut t = TestGame::new(2);
        let five = sphinx_enters(&mut t);
        let all: Vec<Entity> = five.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P1, &all);
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if take_all { 0 } else { 1 }),
        );
        t.resolve_all();
        if take_all {
            assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (5, 0));
        } else {
            assert_eq!((t.hand_size(P0), t.graveyard_size(P0)), (0, 5));
        }
    }
}

#[test]
fn in_multiplayer_you_choose_the_opponent_who_separates_without_targeting() {
    cr!("700.3", "115.10", "702.11d");
    ruling!(
        "Sphinx of Uthuun",
        "In multiplayer games, you choose an opponent to separate the cards when the ability resolves. This doesn't target that opponent."
    );
    supported("Leyline of Sanctity");
    // P2 has hexproof (Leyline of Sanctity) but can still be chosen to separate.
    let mut t = TestGame::new(3);
    t.battlefield(P2, "Leyline of Sanctity");
    let from = t.asked().len();
    let five = sphinx_enters(&mut t);
    // No target was chosen as the trigger was put on the stack.
    assert!(asked_who(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })).is_empty());
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P2, &[Entity::Object(five[0])]);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(asked_who(&t, from, is_separation), vec![P2]);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn casting_an_adventurer_card_as_a_creature_isnt_casting_an_adventure_spell() {
    cr!("715.3", "715.3b");
    ruling!(
        "Storyteller Pixie",
        "An Adventure spell means an instant or sorcery spell with the Adventure spell type. Permanent spells that you cast which have an Adventure won't cause this ability to trigger."
    );
    supported("Storyteller Pixie");
    supported("Bonecrusher Giant // Stomp");
    // Storyteller Pixie: "Whenever you cast an Adventure spell, draw a card."
    for adventure in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Storyteller Pixie");
        t.lands(P0, "Mountain", 3);
        let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
        let hand = t.hand_size(P0);
        if adventure {
            t.cast(P0, giant)
                .method(CastMethod::Half(1))
                .target(P1)
                .go();
        } else {
            t.cast(P0, giant).go();
        }
        t.resolve_all();
        let drawn = t.hand_size(P0) + 1 - hand;
        assert_eq!(drawn, usize::from(adventure), "adventure: {adventure}");
    }
}

#[test]
fn the_spells_controller_chooses_whether_to_pay_as_the_ability_resolves() {
    cr!("608.2d", "118.12");
    ruling!(
        "Spiketail Hatchling",
        "The spell’s controller gets the option to pay when this ability resolves."
    );
    supported("Spiketail Hatchling");
    // Spiketail Hatchling: "Sacrifice this creature: Counter target spell unless its
    // controller pays {1}."
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let hatchling = t.battlefield(P0, "Spiketail Hatchling");
        t.lands(P1, "Mountain", 2);
        let bolt = t.hand(P1, "Lightning Bolt");
        t.cast(P1, bolt).target(P0).go();
        let from = t.asked().len();
        t.activate(P0, hatchling, 0, &[Entity::Object(bolt)])
            .unwrap();
        t.settle();
        assert!(t.in_graveyard(P0, "Spiketail Hatchling"));
        let pay_asked = |t: &TestGame| {
            asked_who(t, from, |d| matches!(d, Decision::YesNo { .. }))
                .into_iter()
                .filter(|p| *p == P1)
                .count()
        };
        // Not asked yet: the ability is on the stack.
        assert_eq!(pay_asked(&t), 0);
        t.answer_yes(P1, pay);
        t.resolve();
        assert_eq!(pay_asked(&t), 1);
        t.resolve_all();
        assert_eq!(t.life(P0), if pay { 17 } else { 20 });
    }
}

#[test]
fn artifact_source_mana_can_pay_for_abilities_of_artifact_cards_in_hand() {
    cr!("106.6", "702.29a");
    ruling!(
        "Oaken Siren",
        "An \"artifact source\" is any object with the card type artifact. This means you could spend the mana to activate an ability of an artifact you control or an artifact card in your hand or graveyard, for example."
    );
    supported("Oaken Siren");
    supported("Indatha Crystal");
    // Oaken Siren: "{T}: Add {U}. Spend this mana only to cast an artifact spell or
    // activate an ability of an artifact source."
    let mut t = TestGame::new(2);
    let siren = t.battlefield(P0, "Oaken Siren");
    t.lands(P0, "Wastes", 1);
    let think = t.hand(P0, "Think Twice");
    let crystal = t.hand(P0, "Indatha Crystal");
    // Its mana can't pay for Think Twice.
    assert!(t.cast(P0, think).try_go().is_err());
    assert!(t.in_hand(P0, "Think Twice"));
    assert!(!is_tapped(&t, siren));
    // It can pay for the cycling ability of an artifact card in hand (Cycling {2}).
    assert!(can_cycle(&mut t, P0, crystal));
    let hand = t.hand_size(P0);
    cycle(&mut t, P0, crystal, 0).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Indatha Crystal"));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn the_token_copies_the_creature_as_it_last_existed_on_the_battlefield() {
    cr!("707.2", "603.10a", "608.2h");
    ruling!(
        "Nightmare Shepherd",
        "The token copies the creature as it last existed on the battlefield before it died, not as it existed in the graveyard before it was exiled."
    );
    supported("Nightmare Shepherd");
    // Nightmare Shepherd: "Whenever another nontoken creature you control dies, you may
    // exile it. If you do, create a token that's a copy of that creature, except it's 1/1
    // and it's a Nightmare in addition to its other types." A Clone copying Hill Giant
    // dies: in the graveyard it's Clone, but the token is a Hill Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightmare Shepherd");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let clone = t.enter(P0, "Clone");
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    t.answer_yes(P0, true);
    destroy(&mut t, clone);
    t.resolve_all();
    assert!(t.in_exile("Clone"));
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 1);
    let token = copies[0];
    let c = &t.obj(token).chars;
    assert_eq!(c.name, "Hill Giant");
    assert!(c.has_subtype("Giant") && c.has_subtype("Nightmare"));
    assert_eq!(t.pt(token), (1, 1));
}

#[test]
fn a_creature_spell_that_gains_flying_only_on_the_battlefield_doesnt_cost_less() {
    cr!("601.2f", "702.9a");
    ruling!(
        "Warden of Evos Isle",
        "A creature spell that doesn't have flying won't cost less even if an effect will cause the creature to have flying once on the battlefield."
    );
    supported("Warden of Evos Isle");
    supported("Kitesail Corsair");
    // Warden of Evos Isle: "Creature spells with flying you cast cost {1} less to cast."
    // Kitesail Corsair ({1}{U}): "This creature has flying as long as it's attacking."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Warden of Evos Isle");
    t.lands(P0, "Island", 1);
    let corsair = t.hand(P0, "Kitesail Corsair");
    let crow = t.hand(P0, "Storm Crow");
    assert!(!can_cast(&mut t, P0, corsair, CastMethod::Normal));
    // Storm Crow ({1}{U}, flying) costs {U}.
    assert!(can_cast(&mut t, P0, crow, CastMethod::Normal));
    t.lands(P0, "Island", 1);
    assert!(can_cast(&mut t, P0, corsair, CastMethod::Normal));
    t.cast(P0, corsair).go();
    assert_eq!(untapped_lands(&t, P0), 0);
}

/// Casts the real card `name` from P0's hand (with lands for its mana cost, targeting P1
/// if it needs a target) and returns how many triggered abilities are then on the stack.
fn triggers_from_casting(t: &mut TestGame, name: &str) -> usize {
    give_mana_for(t, P0, name);
    let card = t.hand(P0, name);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, card).go();
    t.settle();
    let n = t.stack_len() - 1;
    t.clear_answers();
    t.resolve_all();
    n
}

#[test]
fn a_wizard_spell_has_the_creature_type_wizard() {
    cr!("603.2", "205.3m", "601.2i");
    ruling!(
        "Umara Mystic",
        "A Wizard spell is one with the creature type Wizard. Spells that are Wizard-themed (such as Relic Amulet) aren’t Wizard spells."
    );
    ruling!(
        "Umara Wizard // Umara Skyfalls",
        "A Wizard spell is one with the creature type Wizard. Spells that are Wizard-themed (such as Relic Amulet) aren't Wizard spells."
    );
    supported("Umara Mystic");
    supported("Umara Wizard // Umara Skyfalls");
    supported("Prodigal Sorcerer");
    // "Whenever you cast an instant, sorcery, or Wizard spell, this creature gets +2/+0
    // (gains flying) until end of turn."
    for name in ["Umara Mystic", "Umara Wizard // Umara Skyfalls"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        // Prodigal Sorcerer is a Human Wizard Sorcerer; Lightning Bolt is an instant.
        assert_eq!(triggers_from_casting(&mut t, "Prodigal Sorcerer"), 1, "{name}");
        assert_eq!(triggers_from_casting(&mut t, "Lightning Bolt"), 1, "{name}");
        // Relic Amulet (an artifact) and Grizzly Bears (a Bear) aren't Wizard spells.
        assert_eq!(triggers_from_casting(&mut t, "Relic Amulet"), 0, "{name}");
        assert_eq!(triggers_from_casting(&mut t, "Grizzly Bears"), 0, "{name}");
    }
    // Umara Mystic got +2/+0 twice.
    let mut t = TestGame::new(2);
    let mystic = t.battlefield(P0, "Umara Mystic");
    triggers_from_casting(&mut t, "Prodigal Sorcerer");
    triggers_from_casting(&mut t, "Lightning Bolt");
    assert_eq!(t.pt(mystic), (5, 3));
}

#[test]
fn a_trigger_condition_with_a_comma_list_is_read_whole() {
    cr!("603.1", "603.2");
    supported("God-Pharaoh's Faithful");
    supported("Rockslide Sorcerer");
    // God-Pharaoh's Faithful: "Whenever you cast a blue, black, or red spell, you gain 1
    // life." Rockslide Sorcerer: "Whenever you cast an instant, sorcery, or Wizard spell,
    // this creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "God-Pharaoh's Faithful");
    t.battlefield(P0, "Rockslide Sorcerer");
    // Lightning Bolt: red and an instant. Both trigger.
    assert_eq!(triggers_from_casting(&mut t, "Lightning Bolt"), 2);
    assert_eq!(t.life(P0), 21);
    // Grizzly Bears: green, and a Bear creature spell. Neither triggers.
    assert_eq!(triggers_from_casting(&mut t, "Grizzly Bears"), 0);
    // Prodigal Sorcerer: a blue Wizard. Both trigger.
    assert_eq!(triggers_from_casting(&mut t, "Prodigal Sorcerer"), 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_kirin_uses_the_value_of_x_on_the_stack_for_the_spells_mana_value() {
    cr!("202.3e", "115.1d");
    ruling!(
        "Skyfire Kirin",
        "If the Spirit or Arcane spell has {X} in the mana cost, then you use the value of {X} on the stack. For example, Shining Shoal costs {X}{W}{W}. If you choose X = 2, then Shining Shoal's mana value is 4."
    );
    supported("Skyfire Kirin");
    // Skyfire Kirin: "Whenever you cast a Spirit or Arcane spell, you may gain control of
    // target creature with that spell's mana value until end of turn." Swallowing Plague
    // ({X}{B}{B} Sorcery — Arcane) with X = 2 has mana value 4: Hill Giant ({3}{R}) can be
    // targeted, Grizzly Bears ({1}{G}) can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skyfire Kirin");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 4);
    let plague = t.hand(P0, "Swallowing Plague");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.cast(P0, plague).x(2).go();
    t.settle();
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 2, "{cands:?}");
    assert!(cands[1].contains(&Entity::Object(giant)));
    assert!(!cands[1].contains(&Entity::Object(bears)));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn an_aura_spell_has_a_target_and_triggers_a_spell_with_targets_ability() {
    cr!("115.1b", "115.9a", "702.5a");
    ruling!(
        "Voracious Bibliophile",
        "An Aura spell requires a target, as defined by its enchant ability."
    );
    supported("Voracious Bibliophile");
    supported("Fire // Ice");
    // Voracious Bibliophile: "Whenever you cast a spell with one or more targets, draw
    // that many cards."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Voracious Bibliophile");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Mountain", 2);
    let hand = t.hand_size(P0);
    // Rancor (Aura, "Enchant creature"): one target.
    let rancor = t.hand(P0, "Rancor");
    t.cast(P0, rancor).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Grizzly Bears: no targets, no trigger.
    let mine = t.hand(P0, "Grizzly Bears");
    t.cast(P0, mine).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Fire with two targets: two cards.
    let fire = t.hand(P0, "Fire // Ice");
    t.answer(
        P0,
        DecisionKind::Divide,
        Answer::Numbers(vec![1, 1]),
    );
    t.cast(P0, fire)
        .method(CastMethod::Half(0))
        .targets(&[Entity::Player(P1), Entity::Object(bears)])
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn finality_counters_work_on_any_permanent() {
    cr!("122.1h", "614.1a");
    ruling!(
        "Mirko, Obsessive Theorist",
        "Finality counters work on any permanent, not only creatures. If a permanent with a finality counter on it would go to a graveyard from the battlefield, exile it instead."
    );
    supported("Mirko, Obsessive Theorist");
    // Mirko (1/3): "At the beginning of your end step, you may return target creature card
    // with power less than Mirko's from your graveyard to the battlefield with a finality
    // counter on it." With two +1/+1 counters it's 3/5: Grizzly Bears (2) can come back,
    // Hill Giant (3) can't.
    let mut t = TestGame::new(2);
    let mirko = t.battlefield(P0, "Mirko, Obsessive Theorist");
    t.g.add_counters(Entity::Object(mirko), "+1/+1", 2, None);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].contains(&Entity::Object(bears)));
    assert!(!cands[0].contains(&Entity::Object(giant)));
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "finality"), 1);
    // It would die: it's exiled instead.
    destroy(&mut t, back[0]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    // A noncreature permanent with a finality counter is exiled too.
    let millstone = t.battlefield(P1, "Millstone");
    t.g.add_counters(Entity::Object(millstone), "finality", 1, None);
    destroy(&mut t, millstone);
    assert!(t.in_exile("Millstone"));
    assert!(!t.in_graveyard(P1, "Millstone"));
}
