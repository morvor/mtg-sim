//! Rulings batch P118 — "where X is this creature's power": X is determined only once, as
//! the ability resolves (CR 608.2h). Changing the source's power while the ability is on
//! the stack changes X; changing it after the ability has resolved doesn't change the
//! effect (CR 611.2c).

use crate::r_p118_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[derive(Clone, Copy, Debug)]
enum Trig {
    /// At the beginning of combat on your turn.
    Combat,
    /// Whenever it attacks (or becomes tapped, by attacking); `bool`: the target attacks
    /// too.
    Attack(bool),
}

/// P0's `card` (pumped by `pump` +1/+1 counters in response to its trigger) affects P0's
/// `target`; returns the target's power and toughness after the ability resolved, and
/// again after the source got two more counters.
fn fixed_x(card: &str, trig: Trig, target: &str, yes: bool, pump: u32) -> ((i32, i32), (i32, i32)) {
    supported(card);
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, card);
    let tgt = t.battlefield(P0, target);
    t.answer_targets(P0, &[obj(tgt)]);
    if yes {
        t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    }
    match trig {
        Trig::Combat => beginning_of_combat(&mut t),
        Trig::Attack(both) => {
            let mut a = vec![(src, Entity::Player(P1))];
            if both {
                a.push((tgt, Entity::Player(P1)));
            }
            attack_with(&mut t, &a);
        }
    }
    assert!(stack(&t) >= 1, "{card}: no trigger");
    give_plus1(&mut t, src, pump);
    t.resolve_all();
    let after = t.pt(tgt);
    give_plus1(&mut t, src, 2);
    (after, t.pt(tgt))
}

#[test]
fn x_is_its_power_as_the_ability_resolves_and_doesnt_change_later() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Brambleguard Captain",
        "The value of X is calculated only once, as Brambleguard Captain’s ability resolves."
    );
    ruling!(
        "Hemosymbic Mite",
        "The value of X is calculated only once, as Hemosymbic Mite’s ability resolves."
    );
    ruling!(
        "Ouroboroid",
        "The value of X is calculated only once, as Ouroboroid’s ability resolves."
    );
    ruling!(
        "Gruul Beastmaster",
        "The value of X is determined only as Gruul Beastmaster’s triggered ability resolves. Once that happens, the value of X won’t change later in the turn even if Gruul Beastmaster’s power changes."
    );
    ruling!(
        "Syr Faren, the Hengehammer",
        "The value of X is determined only as Syr Faren's triggered ability resolves. Once that happens, the value of X won't change later in the turn even if Syr Faren's power changes."
    );
    ruling!(
        "Ashroot Animist",
        "The value of X is determined only once, as Ashroot Animist's last ability resolves. The value of the bonus won't change later in the turn if Ashroot Animist's power changes."
    );
    ruling!(
        "Maester Seymour",
        "The value of X is determined only once, as Maester Seymour's last ability resolves."
    );
    ruling!(
        "Unruly Krasis",
        "The value of X is determined only once, as Unruly Krasis's second ability resolves. Further changes to Unruly Krasis's power that turn won't cause the target creature's base power and toughness to change."
    );
    ruling!(
        "Jyoti, Moag Ancient",
        "The value of X in Jyoti's last ability is calculated only once, as Jyoti's ability resolves."
    );
    // (card, trigger, target, "you may", counters in response, target's P/T after)
    let cases: [(&str, Trig, &str, bool, u32, (i32, i32)); 9] = [
        ("Brambleguard Captain", Trig::Combat, "Grizzly Bears", false, 2, (6, 2)),
        ("Hemosymbic Mite", Trig::Attack(false), "Grizzly Bears", false, 2, (5, 5)),
        // X = 3 counters on each creature P0 controls.
        ("Ouroboroid", Trig::Combat, "Grizzly Bears", false, 2, (5, 5)),
        ("Gruul Beastmaster", Trig::Attack(false), "Grizzly Bears", false, 2, (6, 2)),
        ("Syr Faren, the Hengehammer", Trig::Attack(true), "Grizzly Bears", false, 2, (6, 6)),
        ("Ashroot Animist", Trig::Attack(false), "Grizzly Bears", false, 1, (7, 7)),
        // X = 3 counters on the target.
        ("Maester Seymour", Trig::Combat, "Grizzly Bears", false, 2, (5, 5)),
        // Base P/T 6/6.
        ("Unruly Krasis", Trig::Attack(false), "Grizzly Bears", true, 2, (6, 6)),
        // Dryad Arbor (1/1 land creature) gets +4/+4.
        ("Jyoti, Moag Ancient", Trig::Combat, "Dryad Arbor", false, 2, (5, 5)),
    ];
    for (card, trig, target, yes, pump, expect) in cases {
        let (after, later) = fixed_x(card, trig, target, yes, pump);
        assert_eq!(after, expect, "{card}");
        assert_eq!(later, expect, "{card}: changed later");
    }
}

#[test]
fn golden_tail_trainer_x_is_determined_as_it_resolves() {
    cr!("608.2h", "611.2c", "700.9");
    ruling!(
        "Golden-Tail Trainer",
        "The value of X in Golden-Tail Trainer's last ability is determined only as the ability resolves."
    );
    supported("Golden-Tail Trainer");
    let mut t = TestGame::new(2);
    let trainer = t.battlefield(P0, "Golden-Tail Trainer");
    let bears = with_counters(&mut t, P0, "Grizzly Bears", 1);
    attack_with(&mut t, &[(trainer, Entity::Player(P1))]);
    assert_eq!(stack(&t), 1);
    giant_growth(&mut t, trainer);
    t.resolve_all();
    // X = 1 + 3.
    assert_eq!(t.pt(bears), (3 + 4, 3 + 4));
    give_plus1(&mut t, trainer, 2);
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn jor_kadeen_x_is_fixed_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Jor Kadeen, First Goldwarden",
        "The value of X is determined only once, as the triggered ability resolves. If the number of equipped creatures you control changes after that time, it won't change the bonus granted."
    );
    supported("Jor Kadeen, First Goldwarden");
    let mut t = TestGame::new(2);
    let jor = t.battlefield(P0, "Jor Kadeen, First Goldwarden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let e1 = t.battlefield(P0, "Short Sword");
    let e2 = t.battlefield(P0, "Short Sword");
    t.g.attach(e1, obj(bears));
    t.g.recompute();
    attack_with(&mut t, &[(jor, Entity::Player(P1))]);
    // A second creature becomes equipped while the trigger is on the stack: X = 2.
    t.g.attach(e2, obj(elves));
    t.g.recompute();
    let hand = t.hand_size(P0);
    t.library_top(P0, "Island");
    t.resolve_all();
    assert_eq!(t.pt(jor), (4, 4));
    assert_eq!(t.hand_size(P0), hand + 1, "power 4: draws");
    // Unequipping afterwards doesn't change the bonus.
    destroy(&mut t, e1);
    destroy(&mut t, e2);
    assert_eq!(t.pt(jor), (4, 4));
}

#[test]
fn prime_speaker_zegana_x_is_fixed_as_it_enters() {
    cr!("614.1c", "608.2h");
    ruling!(
        "Prime Speaker Zegana",
        "The value of X is calculated only once, as Prime Speaker Zegana's first ability resolves. If you control no other creatures at that time, X is 0."
    );
    supported("Prime Speaker Zegana");
    // No other creatures: X is 0; it's a 1/1 and draws one card.
    let mut t = TestGame::new(2);
    for _ in 0..8 {
        t.library_top(P0, "Island");
    }
    t.battlefield(P1, "Craw Wurm");
    let hand = t.hand_size(P0);
    let z = t.enter(P0, "Prime Speaker Zegana");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(z), (1, 1));
    assert_eq!(t.hand_size(P0), hand + 1);
    // A Hill Giant: 4/4; pumping the Giant later doesn't change it.
    let mut t = TestGame::new(2);
    for _ in 0..8 {
        t.library_top(P0, "Island");
    }
    let giant = t.battlefield(P0, "Hill Giant");
    let z = t.enter(P0, "Prime Speaker Zegana");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(z), (4, 4));
    give_plus1(&mut t, giant, 3);
    assert_eq!(t.pt(z), (4, 4));
}

#[test]
fn famished_worldsire_x_is_its_power_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Famished Worldsire",
        "The value of X is calculated only once, as Famished Worldsire’s last ability resolves."
    );
    supported("Famished Worldsire");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Forest");
    }
    let top = crate::r_s01_common::stack_library(&mut t, P0, &["Forest", "Forest", "Forest"]);
    let lands = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.name == "Forest")
            .count()
    };
    // It enters with no counters (nothing devoured): its trigger would look at 0 cards.
    let w = t.enter(P0, "Famished Worldsire");
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1);
    give_plus1(&mut t, w, 3);
    t.answer_choose(P0, &top.iter().map(|i| obj(*i)).collect::<Vec<_>>());
    t.resolve_all();
    assert_eq!(lands(&t), 3);
}

#[test]
fn rip_reveals_its_power_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Rip, Spawn Hunter",
        "The value of X is determined only once, as Rip's ability resolves."
    );
    supported("Rip, Spawn Hunter");
    let mut t = TestGame::new(2);
    let rip = t.battlefield(P0, "Rip, Spawn Hunter");
    t.g.objects[rip.0 as usize].tapped = true;
    let names = [
        "Llanowar Elves",
        "Grizzly Bears",
        "Hill Giant",
        "Serra Angel",
        "Air Elemental",
        "Craw Wurm",
    ];
    let ids = crate::r_s01_common::stack_library(&mut t, P0, &names);
    t.answer_choose(P0, &ids.iter().map(|i| obj(*i)).collect::<Vec<_>>());
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to_step(Step::PostcombatMain);
    t.settle();
    assert_eq!(stack(&t), 1);
    // Power 6 as it resolves: the top six cards (powers 1, 2, 3, 4, 4, 6).
    give_plus1(&mut t, rip, 2);
    t.resolve_all();
    assert!(t.in_hand(P0, "Craw Wurm"));
    assert_eq!(t.hand_size(P0), 5);
}
