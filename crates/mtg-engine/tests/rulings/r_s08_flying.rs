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
