//! Rulings batch P140 — repeatable artifact tokens: Thopter, Servo and Construct makers
//! (Master Trinketeer, Karn, Scion of Urza, Efficient Construction, Oltec Matterweaver,
//! Pinnacle Emissary, Sai, Station Monitor, Saheeli, Siege Veteran, Sly Requisitioner,
//! Hidden Stockpile, Etherium Spinner, Stridehangar Automaton, Sharding Sphinx, Imotekh,
//! Tetravus, Dovin), Artificer Class, Golden Guardian, Myrel, Blink, Roxanne and Master's
//! Manufactory.

use crate::r_p140_common::*;
use crate::r_p141_common::{into_upkeep, yes};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn master_trinketeer_a_servo_thopter_gets_plus_one_once() {
    cr!("613.4c", "205.3m");
    ruling!(
        "Master Trinketeer",
        "A creature that is both a Servo and a Thopter gets +1/+1, not +2/+2."
    );
    supported("Master Trinketeer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Trinketeer");
    let outcast = t.battlefield(P0, "Changeling Outcast");
    assert_eq!(t.pt(outcast), (2, 2));
}

#[test]
fn karns_construct_counts_itself_but_not_karn() {
    cr!("111.3", "306.4");
    ruling!(
        "Karn, Scion of Urza",
        "A token created by Karn's last ability will count itself, so it'll be at least 1/1."
    );
    ruling!(
        "Karn, Scion of Urza",
        "Karn is colorless but not an artifact. The Construct tokens created by Karn's last ability don't count Karn."
    );
    supported("Karn, Scion of Urza");
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Scion of Urza");
    set_loyalty(&mut t, karn, 5);
    t.activate(P0, karn, 2, &[]).expect("−2");
    t.resolve_all();
    let c = tokens_typed(&t, P0, "Construct");
    assert_eq!(c.len(), 1);
    assert_eq!(t.pt(c[0]), (1, 1), "only itself");
    t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(c[0]), (2, 2));
}

#[test]
fn artificer_class_levels_and_class_abilities() {
    cr!("716.2a", "716.2b", "716.2c", "716.3");
    ruling!(
        "Artificer Class",
        "Class is a returning enchantment type. Each Class has five abilities. The three in the major sections of its text box are class abilities."
    );
    supported("Artificer Class");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Artificer Class");
    // Level 1: the first artifact spell each turn costs {1} less.
    t.lands(P0, "Wastes", 1);
    let stone = t.hand(P0, "Millstone");
    assert!(can_cast(&mut t, P0, stone, CastMethod::Normal));
    t.cast(P0, stone).go();
    t.resolve_all();
    // The level 2 ability is an activated ability; becoming level 2 triggers.
    stack_library(&mut t, P0, &["Island", "Ornithopter"]);
    activate_lots(&mut t, P0, class, 0, &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"), "the level 2 trigger");
    assert_eq!(crate::r_s19_common::level(&t, class), 2);
    // Level 3, then the end step trigger copies an artifact.
    activate_lots(&mut t, P0, class, 1, &[]);
    t.resolve_all();
    assert_eq!(crate::r_s19_common::level(&t, class), 3);
    let s = t.g.current(stone);
    t.answer_targets(P0, &[obj(s)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Millstone").len(), 2);
}

#[test]
fn spell_cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.2", "701.6a");
    ruling!(
        "Efficient Construction",
        "Efficient Construction’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Oltec Matterweaver",
        "Oltec Matterweaver’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Pinnacle Emissary",
        "Pinnacle Emissary’s first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Sai, Master Thopterist",
        "Sai's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Station Monitor",
        "Station Monitor’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    let cases = [
        ("Efficient Construction", "Thopter", false),
        ("Oltec Matterweaver", "Gnome", false),
        ("Pinnacle Emissary", "Drone", true),
        ("Sai, Master Thopterist", "Thopter", false),
        ("Station Monitor", "Drone", true),
    ];
    for (name, sub, bounce) in cases {
        supported(name);
        for leave in if bounce { vec![false, true] } else { vec![false] } {
            let mut t = TestGame::new(2);
            t.battlefield(P0, name);
            if name == "Station Monitor" {
                cast_lots(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
                t.resolve_all();
            }
            let spell = cast_lots(&mut t, P0, "Ornithopter", &[]);
            t.settle();
            let top = *t.stack.last().unwrap();
            assert_eq!(t.obj(top).kind, ObjKind::StackAbility, "{name}: on top of it");
            if leave {
                move_to(&mut t, spell, Zone::Hand(P0));
            } else {
                counter_spell(&mut t, spell);
            }
            t.resolve_all();
            assert_eq!(tokens_typed(&t, P0, sub).len(), 1, "{name} leave={leave}");
            assert!(t.named_on_battlefield("Ornithopter").is_empty());
        }
    }
}

#[test]
fn station_monitor_counts_every_spell_cast_this_turn_including_itself() {
    cr!("603.2", "601.2i");
    ruling!(
        "Station Monitor",
        "Station Monitor’s ability will count any spells you’ve cast this turn, which may include Station Monitor itself."
    );
    // Station Monitor first: the next spell is the second.
    let mut t = TestGame::new(2);
    cast_lots(&mut t, P0, "Station Monitor", &[]);
    t.resolve_all();
    cast_lots(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Drone").len(), 1);
    // A countered spell first, then Station Monitor (second), then a third: no trigger.
    let mut t = TestGame::new(2);
    let bolt = cast_lots(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    counter_spell(&mut t, bolt);
    cast_lots(&mut t, P0, "Station Monitor", &[]);
    t.resolve_all();
    cast_lots(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert!(tokens_typed(&t, P0, "Drone").is_empty(), "the third spell");
}

#[test]
fn saheeli_chooses_the_artifact_to_tap_on_resolution() {
    cr!("608.2d", "606.3");
    ruling!(
        "Saheeli, Filigree Master",
        "For the first ability, you don't choose which artifact to tap until the ability resolves."
    );
    supported("Saheeli, Filigree Master");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Saheeli, Filigree Master");
    set_loyalty(&mut t, s, 3);
    let hand = t.hand_size(P0);
    t.activate(P0, s, 0, &[]).expect("+1");
    // An artifact arrives in response; it can be tapped as the ability resolves.
    let o = t.battlefield(P0, "Ornithopter");
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(o)]);
    t.resolve_all();
    assert!(t.obj(o).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn saheeli_emblems_apply_independently() {
    cr!("114.4", "613.4c", "601.2f");
    ruling!(
        "Saheeli, Filigree Master",
        "If you manage to activate Saheeli's last ability multiple times, each emblem you get is applied independently."
    );
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let s = t.battlefield(P0, "Saheeli, Filigree Master");
        set_loyalty(&mut t, s, 4);
        t.activate(P0, s, 2, &[]).expect("−4");
        t.resolve_all();
    }
    let o = t.battlefield(P0, "Ornithopter");
    assert_eq!(t.pt(o), (2, 4));
    // Millstone ({2}) costs nothing.
    let stone = t.hand(P0, "Millstone");
    assert!(can_cast(&mut t, P0, stone, CastMethod::Normal));
    let giant = t.hand(P0, "Hill Giant");
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal), "control");
}

#[test]
fn dies_together_triggers_see_each_other() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Siege Veteran",
        "If Siege Veteran dies at the same time as one or more other nontoken Soldier creatures, its second ability will trigger for each of them."
    );
    ruling!(
        "Sly Requisitioner",
        "If Sly Requisitioner is put into a graveyard at the same time as a nontoken artifact you control, its ability triggers."
    );
    supported("Siege Veteran");
    supported("Sly Requisitioner");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Siege Veteran");
    let a = t.battlefield(P0, "Elite Vanguard");
    let b = t.battlefield(P0, "Elite Vanguard");
    crate::r_p146_common::destroy_together(&mut t, &[v, a, b]);
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Soldier").len(), 2);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sly Requisitioner");
    let o = t.battlefield(P0, "Ornithopter");
    crate::r_p146_common::destroy_together(&mut t, &[s, o]);
    t.resolve_all();
    assert_eq!(tokens_typed(&t, P0, "Servo").len(), 1);
}

#[test]
fn hidden_stockpile_counts_a_permanent_that_left_before_it_arrived() {
    cr!("603.4", "207.2c");
    ruling!(
        "Hidden Stockpile",
        "If a permanent you controlled didn't leave the battlefield before your end step begins, Hidden Stockpile's first ability doesn't trigger at all. The permanent may have left before Hidden Stockpile entered the battlefield, however."
    );
    supported("Hidden Stockpile");
    for left in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        if left {
            destroy(&mut t, bears);
        }
        t.battlefield(P0, "Hidden Stockpile");
        t.advance_to(P0, Step::End);
        t.settle();
        let n = t.stack_len();
        t.resolve_all();
        assert_eq!(n, left as usize, "left={left}: triggers");
        assert_eq!(tokens_typed(&t, P0, "Servo").len(), left as usize);
    }
}

#[test]
fn etherium_spinner_counts_x_in_the_mana_value() {
    cr!("202.3e", "107.3");
    ruling!(
        "Etherium Spinner",
        "If a spell on the stack has {X} in its mana cost, X is the value chosen for it when determining that spell's mana value."
    );
    supported("Etherium Spinner");
    for (x, n) in [(3i64, 1usize), (2, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Etherium Spinner");
        lots(&mut t, P0);
        let fb = t.hand(P0, "Fireball");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, fb).x(x).go();
        t.resolve_all();
        assert_eq!(tokens_typed(&t, P0, "Thopter").len(), n, "X={x}");
    }
}

/// Indices of the replacement options offered whose description contains `needle`.
fn pick_replacement(t: &mut TestGame, p: PlayerId, needle: &'static str) {
    use mtg_engine::decision::Agent;
    struct Pick {
        inner: Box<dyn Agent>,
        needle: &'static str,
    }
    impl Agent for Pick {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if let Decision::ChooseReplacement { options, .. } = d {
                if let Some(i) = options.iter().position(|o| o.contains(self.needle)) {
                    return Answer::Index(i);
                }
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Pick { inner, needle });
}

#[test]
fn stridehangar_and_anointed_procession_apply_in_the_order_you_choose() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Stridehangar Automaton",
        "If another effect modifies which or how many tokens you create, you choose the order to apply those effects."
    );
    supported("Stridehangar Automaton");
    for (first, want) in [("Anointed", 3usize), ("Stridehangar", 4)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Stridehangar Automaton");
        t.battlefield(P0, "Anointed Procession");
        pick_replacement(&mut t, P0, first);
        let patrol = t.graveyard(P0, "Fairgrounds Patrol");
        activate_lots(&mut t, P0, patrol, 0, &[]);
        t.resolve_all();
        assert_eq!(tokens_typed(&t, P0, "Thopter").len(), want, "{first} first");
    }
}

#[test]
fn stridehangar_ignores_copies_of_permanent_spells() {
    cr!("707.10f", "111.1");
    ruling!(
        "Stridehangar Automaton",
        "If you copy a permanent spell, that copy becomes a token on the battlefield, but that’s not the same as creating a token. You won’t get an additional Thopter token that way."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    let o = cast_lots(&mut t, P0, "Ornithopter", &[]);
    cast_lots(&mut t, P0, "Double Major", &[obj(o)]);
    t.resolve_all();
    let toks = tokens_of(&t, P0);
    assert_eq!(toks.len(), 1, "the copy only");
    assert_eq!(t.obj(toks[0]).chars.name.as_str(), "Ornithopter");
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 2);
}

#[test]
fn stridehangars_thopter_shares_the_creation_instructions_but_not_the_abilities() {
    cr!("614.1a", "111.2");
    ruling!(
        "Stridehangar Automaton",
        "The additional Thopter token won’t have any abilities the other tokens were created with."
    );
    supported("Roxanne, Starfall Savant");
    // Roxanne's tapped Meteorite: the Thopter is tapped too, without Meteorite's abilities.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Roxanne, Starfall Savant");
    t.resolve_all();
    let met = controlled_named(&t, P0, "Meteorite");
    assert_eq!(met.len(), 1);
    assert!(t.obj(met[0]).tapped);
    let th: Vec<ObjectId> = tokens_typed(&t, P0, "Thopter");
    assert_eq!(th.len(), 1);
    assert!(t.obj(th[0]).tapped, "created tapped too");
    assert_eq!(t.obj(th[0]).chars.name.as_str(), "Thopter Token");
    assert!(t
        .obj(th[0])
        .chars
        .abilities
        .iter()
        .all(|a| !a.text.contains("damage") && !a.text.contains("Add")));
    assert_eq!(t.life(P1), 18, "only the Meteorite's trigger");
    // Saheeli's "They gain haste": the extra Thopter gains haste too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    let s = t.battlefield(P0, "Saheeli, Filigree Master");
    set_loyalty(&mut t, s, 3);
    t.activate(P0, s, 1, &[]).expect("−2");
    t.resolve_all();
    let th = tokens_typed(&t, P0, "Thopter");
    assert_eq!(th.len(), 3);
    for x in th {
        assert!(t.obj(x).has_keyword(KeywordKind::Haste));
    }
    // Chrome Dome's copy: "That token gains haste. Sacrifice it at the beginning of the
    // next end step." apply to the Thopter as well.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stridehangar Automaton");
    let dome = t.battlefield(P0, "Chrome Dome");
    let o = t.battlefield(P0, "Ornithopter");
    activate_lots(&mut t, P0, dome, 0, &[obj(o)]);
    t.resolve_all();
    let toks = tokens_of(&t, P0);
    assert_eq!(toks.len(), 2);
    for x in &toks {
        assert!(t.obj(*x).has_keyword(KeywordKind::Haste), "{}", t.obj(*x).chars.name);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty(), "both sacrificed");
}

#[test]
fn sharding_sphinx_one_thopter_per_artifact_creature() {
    cr!("510.2", "603.2c");
    ruling!(
        "Sharding Sphinx",
        "If combat damage is dealt to players by multiple artifact creatures you control (including, perhaps, Sharding Sphinx itself), Sharding Sphinx's ability will trigger that many times."
    );
    supported("Sharding Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Sharding Sphinx");
    let sable = t.battlefield(P0, "Bronze Sable");
    let bears = t.battlefield(P0, "Grizzly Bears");
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.attack(
        &[
            (sphinx, Entity::Player(P1)),
            (sable, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 2 - 2);
    assert_eq!(tokens_typed(&t, P0, "Thopter").len(), 2);
}

#[test]
fn imotekh_triggers_once_for_cards_leaving_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Imotekh the Stormlord",
        "If multiple artifact cards leave your graveyard at the same time, the first ability triggers only once, and you will create only two tokens."
    );
    supported("Imotekh the Stormlord");
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Imotekh the Stormlord");
    t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Millstone");
    t.graveyard(P0, "Ichor Wellspring");
    t.enter(P0, "Rest in Peace");
    t.resolve_all();
    assert!(t.g.player(P0).graveyard.is_empty());
    assert_eq!(tokens_typed(&t, P0, "Necron").len(), 2);
}

/// `p`'s upkeep with Tetravus `tv` (and its triggers): it makes `n` Tetravites.
fn tetravites(t: &mut TestGame, tv: ObjectId, n: i64) -> Vec<ObjectId> {
    let before = tokens_of(t, P0);
    // The removing trigger resolves first (the first item ordered is the bottom).
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::Number, Answer::Number(n));
    into_upkeep(t, P0);
    // Only the first upkeep trigger acts; decline the exile one.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.resolve_all();
    let _ = tv;
    tokens_of(t, P0)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect()
}

#[test]
fn tetravites_outlive_their_tetravus_and_only_return_to_it() {
    cr!("607.2a", "113.7a");
    ruling!(
        "Tetravus",
        "If the Tetravus is destroyed when the Tetravites are off the card, they are not destroyed, they are just orphaned."
    );
    ruling!(
        "Tetravus",
        "Only Tetravites from this specific Tetravus may be used for the ability. Ones from a different Tetravus can't."
    );
    supported("Tetravus");
    let mut t = TestGame::new(2);
    let tv = t.battlefield(P0, "Tetravus");
    put_counters(&mut t, tv, counters::PLUS1, 3);
    let made = tetravites(&mut t, tv, 2);
    assert_eq!(made.len(), 2, "{}", t.dump_log());
    assert_eq!(t.counters(tv, counters::PLUS1), 1);
    // Destroyed: its Tetravites stay.
    destroy(&mut t, tv);
    assert!(!t.on_battlefield(tv));
    for x in &made {
        assert!(t.on_battlefield(*x), "orphaned, not destroyed");
    }
    // Another Tetravus can't take them back.
    let other = t.battlefield(P0, "Tetravus");
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::Entities, Answer::Entities(made.iter().map(|x| obj(*x)).collect()));
    into_upkeep(&mut t, P0);
    t.resolve_all();
    for x in &made {
        assert!(t.on_battlefield(*x), "not exiled by another Tetravus");
    }
    assert_eq!(t.counters(other, counters::PLUS1), 0);
}

#[test]
fn golden_guardians_fight_needs_both() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Golden Guardian // Gold-Forge Garrison",
        "If the target of Golden Guardian's activated ability isn't a legal target as that ability resolves, or if Golden Guardian has left the battlefield, neither creature will deal or be dealt damage."
    );
    supported("Golden Guardian // Gold-Forge Garrison");
    for gone in ["target", "guardian"] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Golden Guardian // Gold-Forge Garrison");
        let giant = t.battlefield(P0, "Hill Giant");
        activate_lots(&mut t, P0, g, 0, &[obj(giant)]);
        if gone == "target" {
            move_to(&mut t, giant, Zone::Hand(P0));
        } else {
            move_to(&mut t, g, Zone::Hand(P0));
        }
        t.resolve_all();
        if gone == "target" {
            assert_eq!(t.obj(g).damage, 0);
        } else {
            assert_eq!(t.obj(giant).damage, 0);
        }
    }
}

#[test]
fn dovin_with_a_short_library_takes_three_or_all() {
    cr!("401.5", "608.2c");
    ruling!(
        "Dovin, Grand Arbiter",
        "If you have fewer than ten cards in your library as Dovin's last ability resolves, look at however many cards you have, put three of them into your hand (or all of them if you have fewer than three cards in your library), and then put the rest back in a random order."
    );
    for (lib, to_hand) in [(5usize, 3usize), (2, 2)] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Dovin, Grand Arbiter");
        set_loyalty(&mut t, d, 10);
        library_of(&mut t, P0, lib);
        let hand = t.hand_size(P0);
        t.activate(P0, d, 2, &[]).expect("−7");
        t.resolve_all();
        assert_eq!(t.hand_size(P0) - hand, to_hand, "{lib} cards");
        assert_eq!(t.library_size(P0), lib - to_hand);
    }
}

#[test]
fn myrel_doesnt_stop_abilities_of_cards_in_other_zones() {
    cr!("602.5", "113.6");
    ruling!(
        "Myrel, Shield of Argive",
        "It doesn't prevent activated abilities from cards in any zone other than the battlefield."
    );
    supported("Myrel, Shield of Argive");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Myrel, Shield of Argive");
    let skel = t.graveyard(P1, "Reassembling Skeleton");
    let vs = t.battlefield(P1, "Void Stalker");
    lots(&mut t, P1);
    assert!(!can_activate(&mut t, P1, vs), "a creature on the battlefield");
    assert!(can_activate(&mut t, P1, skel), "a card in a graveyard");
    // On P1's turn, the battlefield one works again.
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    assert!(can_activate(&mut t, P1, vs));
}

#[test]
fn blinks_chapters_trigger_out_of_order() {
    cr!("714.2b", "714.2c");
    ruling!(
        "Blink",
        "The first ability triggers when the first and third lore counters are added. The last ability triggers when the second and fourth lore counters are placed on it."
    );
    let def = card("Blink");
    assert!(def.unsupported_text().is_empty(), "{:?}", def.unsupported_text());
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Blink");
    let mut seen = vec![];
    for lore in 1..=4u32 {
        let b = t.battlefield(P1, "Grizzly Bears");
        if lore % 2 == 1 {
            t.answer_targets(P0, &[obj(b)]);
        }
        crate::r_s19_common::add_lore(&mut t, saga, 1);
        assert_eq!(t.counters(t.g.current(saga), counters::LORE), lore);
        let angels = tokens_typed(&t, P0, "Angel").len();
        t.resolve_all();
        let shuffled = zone_of(&t, b) == Zone::Library(P1);
        let angel = tokens_typed(&t, P0, "Angel").len() > angels;
        seen.push((lore, shuffled, angel));
        if lore == 4 {
            break;
        }
    }
    assert_eq!(
        seen,
        vec![(1, true, false), (2, false, true), (3, true, false), (4, false, true)]
    );
}

#[test]
fn roxannes_extra_mana_is_hers_and_unrestricted() {
    cr!("106.6", "106.12a", "605.4a");
    ruling!(
        "Roxanne, Starfall Savant",
        "Roxanne doesn't care about any restrictions or riders the artifact token put on the mana it produced."
    );
    ruling!(
        "Roxanne, Starfall Savant",
        "The additional mana is produced by Roxanne, not by the artifact token that you tapped for mana."
    );
    let mut t = TestGame::new(2);
    let rox = t.battlefield(P0, "Roxanne, Starfall Savant");
    let stone = crate::r_s02_common::create_token(&mut t, P0, "Powerstone");
    t.activate(P0, stone, 0, &[]).expect("tap for mana");
    t.settle();
    let pool = &t.g.player(P0).mana_pool.mana;
    assert_eq!(pool.len(), 2, "{pool:?}");
    let from_stone: Vec<_> = pool.iter().filter(|m| m.source == Some(stone)).collect();
    let from_rox: Vec<_> = pool.iter().filter(|m| m.source == Some(rox)).collect();
    assert_eq!(from_stone.len(), 1);
    assert!(from_stone[0].restriction.is_some(), "Powerstone mana is restricted");
    assert_eq!(from_rox.len(), 1, "Roxanne produced the extra mana");
    assert!(from_rox[0].restriction.is_none() && from_rox[0].rider.is_none());
}

#[test]
fn masters_manufactory_only_needs_the_artifact_to_have_entered() {
    cr!("602.5b", "603.10");
    ruling!(
        "Master's Guide-Mural // Master's Manufactory",
        "The ability of Master's Manufactory can be activated no matter what happened to the artifact after it entered under your control."
    );
    supported("Master's Guide-Mural // Master's Manufactory");
    for what in ["left", "control", "none"] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Master's Guide-Mural // Master's Manufactory");
        crate::r_s17_common::transform(&mut t, m);
        let m = t.g.current(m);
        assert_eq!(t.obj(m).chars.name.as_str(), "Master's Manufactory");
        if what != "none" {
            let o = t.enter(P0, "Ornithopter");
            t.settle();
            if what == "left" {
                destroy(&mut t, o);
            } else {
                give_control(&mut t, o, P1);
            }
        }
        assert_eq!(can_activate(&mut t, P0, m), what != "none", "{what}");
    }
}
