//! Rulings batch P142 — the rest: creatures that stay blocked when the condition of their
//! evasion changes, Hans Eriksson's creature and fight, Kellan's Adventure, lords and
//! anthems that don't grant abilities, values locked in on resolution, delayed triggers
//! that outlive their source, "you may pay ... If you do" paid once, and Grand Master of
//! Flowers as a creature.

use crate::r_p142_common::*;
use crate::r_p009_common::gain;
use crate::r_p146_common::lose_life;
use crate::r_s01_common::watch;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// P0 attacks with `attacker`, P1 blocks with `blocker`; `change` runs after blockers are
/// declared; combat finishes.
fn blocked_then(
    t: &mut TestGame,
    attacker: ObjectId,
    blocker: ObjectId,
    change: impl FnOnce(&mut TestGame),
) {
    attack_with(t, &[(attacker, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(attacker));
    change(t);
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn creatures_stay_blocked_when_their_evasion_condition_changes() {
    cr!("509.1b", "506.4");
    ruling!(
        "Carnage Interpreter",
        "Once Carnage Interpreter has been blocked, causing it to gain menace by removing cards from your hand won't cause it to stop being blocked."
    );
    ruling!(
        "Locke, Treasure Hunter",
        "Once Locke has been blocked, changing its power or the power of the blocking creature won't cause Locke to become unblocked."
    );
    ruling!(
        "Ghost of Ramirez DePietro",
        "Once a creature with toughness 2 or less has blocked this creature, changing the toughness of the blocking creature won't cause this creature to become unblocked."
    );
    supported("Carnage Interpreter");
    supported("Locke, Treasure Hunter");
    // Carnage Interpreter, with three cards in hand: gets menace after the block.
    let mut t = TestGame::new(2);
    let ci = t.battlefield(P0, "Carnage Interpreter");
    let cards = fill_hand(&mut t, P0, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(ci), (3, 3));
    blocked_then(&mut t, ci, bears, |t| {
        discard(t, P0, cards[0]);
        discard(t, P0, cards[1]);
        assert!(t.obj_now(ci).has_keyword(KeywordKind::Menace));
        assert_eq!(t.pt(ci), (5, 5));
    });
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Locke: the blocker's power grows past Locke's.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Locke, Treasure Hunter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_then(&mut t, l, bears, |t| pump(t, bears, 3, 0));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Locke, Treasure Hunter"));
    // Ghost of Ramirez DePietro: the blocker's toughness grows to 5.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Ghost of Ramirez DePietro");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_then(&mut t, g, bears, |t| pump(t, bears, 0, 3));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 2);
}

#[test]
fn hans_eriksson_reflexive_fight_can_be_responded_to() {
    cr!("603.12", "701.14a");
    ruling!(
        "Hans Eriksson",
        "Putting a creature card onto the battlefield this way causes a second reflexive triggered ability to trigger. Players may respond to this ability before it resolves."
    );
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Hans Eriksson");
        t.library_top(P0, "Hill Giant");
        attack_with(&mut t, &[(h, pl(P1))]);
        t.resolve();
        assert_eq!(stacked_triggers(&t), 1, "the reflexive trigger");
        if respond {
            pump(&mut t, h, 3, 0);
        }
        t.resolve_all();
        assert_eq!(t.in_graveyard(P0, "Hill Giant"), respond, "{respond}");
    }
}

#[test]
fn hans_eriksson_creature_attacks_the_defending_player_or_their_planeswalker() {
    cr!("506.2", "508.4");
    ruling!(
        "Hans Eriksson",
        "The defending player is the player Hans Eriksson is attacking or the controller of the planeswalker Hans Eriksson is attacking."
    );
    let mut t = TestGame::new(3);
    let h = t.battlefield(P0, "Hans Eriksson");
    let pw = t.battlefield(P2, "Liliana, the Last Hope");
    t.battlefield(P1, "Liliana, the Necromancer");
    t.library_top(P0, "Hill Giant");
    let from = n_asked(&t);
    attack_with(&mut t, &[(h, obj(pw))]);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    let target = t.g.combat.as_ref().unwrap().attack_target(g).expect("attacking");
    assert!(target == pl(P2) || target == obj(pw), "{target:?}");
    // Nothing of P1's was offered.
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseEntities { candidates, .. } | Decision::ChooseTargets { candidates, .. } = d {
            assert!(!candidates.contains(&pl(P1)), "{d:?}");
        }
    }
}

#[test]
fn journey_on_counts_opponents_with_artifacts() {
    cr!("715.3", "102.2");
    ruling!(
        "Kellan, Daring Traveler // Journey On",
        "Journey On counts the number of opponents who control at least one artifact, not the number of artifacts opponents control."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Ornithopter");
    let k = t.hand(P0, "Kellan, Daring Traveler // Journey On");
    lots(&mut t, P0);
    t.cast(P0, k).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Map").len(), 2);
}

#[test]
fn kellan_has_an_adventure_when_cast_as_a_creature() {
    cr!("715.5");
    ruling!(
        "Kellan, Daring Traveler // Journey On",
        "Some cards in other sets refer to a card, spell, or permanent that \"has an Adventure.\" This refers to a card, spell, or permanent that has an adventurer card's set of alternative characteristics, even if they're not being used and even if that card was never cast as an Adventure."
    );
    supported("Edgewall Innkeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgewall Innkeeper");
    let hand = t.hand_size(P0);
    let k = t.hand(P0, "Kellan, Daring Traveler // Journey On");
    lots(&mut t, P0);
    t.cast(P0, k).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kellan, Daring Traveler").len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1, "Innkeeper drew a card");
}

#[test]
fn lords_and_anthems_without_granted_abilities() {
    cr!("613.4c", "113.3");
    ruling!(
        "Lord of the Undead",
        "Lord of the Undead now has the Zombie creature type and its first ability has been reworded to affect *other* Zombies. This means that if two Lord of the Undead are on the battlefield, each gives the other a bonus."
    );
    ruling!(
        "Ruxa, Patient Professor",
        "Notably, neither of Ruxa's last two abilities give any creatures an ability."
    );
    supported("Lord of the Undead");
    supported("Ruxa, Patient Professor");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Lord of the Undead");
    assert_eq!(t.pt(a), (2, 2));
    let b = t.battlefield(P0, "Lord of the Undead");
    assert_eq!((t.pt(a), t.pt(b)), ((3, 3), (3, 3)));
    // Ruxa: Grizzly Bears gets +1/+1 and still has no abilities (so it keeps counting).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruxa, Patient Professor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(bears).chars.abilities.is_empty());
}

#[test]
fn tezzeret_ultimate_control_lasts_indefinitely() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Tezzeret, Master of Metal",
        "The effect of Tezzeret’s third ability lasts indefinitely."
    );
    let mut t = TestGame::new(2);
    let tz = t.battlefield(P0, "Tezzeret, Master of Metal");
    loyalty(&mut t, tz, 8);
    let g = t.battlefield(P1, "Hill Giant");
    let o = t.battlefield(P1, "Ornithopter");
    let f = t.battlefield(P1, "Forest");
    t.activate(P0, tz, 2, &[pl(P1)]).unwrap();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(g).controller, P0);
    assert_eq!(t.obj_now(o).controller, P0);
    assert_eq!(t.obj_now(f).controller, P1);
}

#[test]
fn grand_master_of_flowers_becomes_a_creature_as_the_cost_is_paid() {
    cr!("606.4", "306.5b", "602.2b");
    ruling!(
        "Grand Master of Flowers",
        "The first ability of Grand Master of Flowers doesn't use the stack and can't be responded to. Similarly, players can't respond to paying the cost of adding loyalty counters for an ability. Therefore, if Grand Master of Flowers has six loyalty and you activate one of his abilities, he will be a 7/7 Dragon God with flying and indestructible before players can respond to the loyalty ability."
    );
    supported("Grand Master of Flowers");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    loyalty(&mut t, gm, 6);
    assert!(!t.obj_now(gm).is(CardType::Creature));
    t.activate(P0, gm, 1, &[]).unwrap();
    assert_eq!(t.stack_len(), 1, "the ability is still on the stack");
    let o = t.obj_now(gm);
    assert!(o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Dragon") && o.chars.has_subtype("God"));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(o.has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.pt(gm), (7, 7));
}

#[test]
fn grand_master_of_flowers_as_a_creature_isnt_a_planeswalker() {
    cr!("306.5b", "506.3", "120.3d", "606.3");
    ruling!(
        "Grand Master of Flowers",
        "While Grand Master of Flowers is a creature, he is not a planeswalker. He cannot be attacked and damage dealt to him while he is a creature will not reduce the number of loyalty counters he has. However, you may still activate his loyalty abilities as normal."
    );
    ruling!(
        "Grand Master of Flowers",
        "You can activate the last ability of Grand Master of Flowers even if there isn't a card named Monk of the Open Hand in your library or graveyard."
    );
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    loyalty(&mut t, gm, 7);
    assert!(t.obj_now(gm).is(CardType::Creature));
    assert!(!t.obj_now(gm).is(CardType::Planeswalker));
    // Damage marks the creature; the loyalty stays.
    t.g.deal_damage(gm, Entity::Object(gm), 3, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.counters(gm, "loyalty"), 7);
    assert_eq!(t.obj_now(gm).damage, 3);
    // It can't be attacked.
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!mtg_engine::combat::attack_targets(&t.g).contains(&obj(gm)));
    assert!(mtg_engine::combat::attack_targets(&t.g).contains(&pl(P0)));
    // Its loyalty ability, with no Monk of the Open Hand anywhere.
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, gm, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(gm, "loyalty"), 8);
    // Positive control: a Monk in the graveyard is found.
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Grand Master of Flowers");
    let m = t.graveyard(P0, "Monk of the Open Hand");
    t.answer_choose(P0, &[obj(m)]);
    t.activate(P0, gm, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Monk of the Open Hand"));
}

#[test]
fn feather_spells_may_have_other_targets() {
    cr!("115.1", "601.2c");
    ruling!(
        "Feather, the Redeemed",
        "The spell may have any other targets in addition to a creature you control."
    );
    supported("Feather, the Redeemed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feather, the Redeemed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let e = t.hand(P0, "Electrolyze");
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P0, e).targets(&[obj(bears), pl(P1)]).go();
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_exile("Electrolyze"));
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Electrolyze"));
}

#[test]
fn famished_foragers_counts_any_life_lost() {
    cr!("119.3", "603.4");
    ruling!(
        "Famished Foragers",
        "The triggered ability adds {R}{R}{R} if any opponent lost any amount of life this turn, even if they also gained life at some point."
    );
    supported("Famished Foragers");
    for lost in [true, false] {
        let mut t = TestGame::new(2);
        if lost {
            lose_life(&mut t, P1, 1);
        }
        gain(&mut t, P1, 3);
        enter(&mut t, P0, "Famished Foragers");
        t.resolve_all();
        assert_eq!(
            t.g.player(P0).mana_pool.count(ManaType::R),
            if lost { 3 } else { 0 },
            "{lost}"
        );
    }
}

#[test]
fn values_locked_in_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Ajani, Valiant Protector",
        "The value of X for Ajani’s third ability is determined only as it resolves. The number of +1/+1 counters on the creature won’t change later if your life total changes."
    );
    ruling!(
        "Ashcoat of the Shadow Swarm",
        "The value of X is locked in as the first ability resolves. The bonus it grants won't change after that point, even if the number of Rats you control does."
    );
    supported("Ashcoat of the Shadow Swarm");
    // Ajani's −11 at 20 life; then P0 gains life.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ajani, Valiant Protector");
    loyalty(&mut t, a, 11);
    let b = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, a, 2, &[obj(b)]).unwrap();
    lose_life(&mut t, P0, 5);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 15);
    gain(&mut t, P0, 5);
    assert_eq!(t.counters(b, "+1/+1"), 15);
    // Ashcoat attacks with two other Rats: X = 3; then a Rat dies.
    let mut t = TestGame::new(2);
    let ash = t.battlefield(P0, "Ashcoat of the Shadow Swarm");
    let r1 = t.battlefield(P0, "Typhoid Rats");
    let r2 = t.battlefield(P0, "Typhoid Rats");
    attack_with(&mut t, &[(ash, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(r1), (4, 4));
    destroy(&mut t, r2);
    assert_eq!(t.pt(r1), (4, 4));
}

#[test]
fn tiana_delayed_trigger_outlives_tiana() {
    cr!("603.7c");
    ruling!(
        "Tiana, Ship's Caretaker",
        "Tiana's last ability triggers and creates a delayed triggered ability that will let you return the Aura or Equipment during the next end step. It will do so even if Tiana leaves the battlefield before the next end step."
    );
    supported("Tiana, Ship's Caretaker");
    let mut t = TestGame::new(2);
    let tiana = t.battlefield(P0, "Tiana, Ship's Caretaker");
    let b = t.battlefield(P0, "Grizzly Bears");
    let s = attach_new(&mut t, P0, "Bonesplitter", b);
    yes(&mut t, P0);
    yes(&mut t, P0);
    destroy(&mut t, s);
    t.resolve_all();
    destroy(&mut t, tiana);
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Bonesplitter"));
}

#[test]
fn optional_costs_in_triggers_are_paid_once_and_in_full() {
    cr!("118.12", "603.5");
    ruling!(
        "Dreamshaper Shaman",
        "While resolving Dreamshaper Shaman's ability, you can't pay {2}{R} more than once or sacrifice multiple permanents."
    );
    ruling!(
        "Dreamshaper Shaman",
        "You can't pay {2}{R} and not sacrifice a nonland permanent, or vice versa. The cost must be paid in full or not at all."
    );
    ruling!(
        "Eternal Taskmaster",
        "You can't pay {2}{B} more than once each time Eternal Taskmaster's ability resolves."
    );
    ruling!(
        "Veinwitch Coven",
        "You may only pay {B} once for each life gain event."
    );
    supported("Eternal Taskmaster");
    supported("Veinwitch Coven");
    // Dreamshaper Shaman with plenty of mana: one sacrifice, one card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreamshaper Shaman");
    let o = t.battlefield(P0, "Ornithopter");
    let bs = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Mountain", 6);
    stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm"]);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(o)]);
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Ornithopter"), "{}", t.dump_log());
    assert!(t.on_battlefield(bs));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    assert_eq!(tapped_lands(&t, P0), 3);
    // Without the mana: nothing is sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreamshaper Shaman");
    let o = t.battlefield(P0, "Ornithopter");
    t.library_top(P0, "Hill Giant");
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(o)]);
    end_step(&mut t, P0);
    assert!(t.on_battlefield(o));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    // Eternal Taskmaster with six Swamps: one card.
    let mut t = TestGame::new(2);
    let et = t.battlefield(P0, "Eternal Taskmaster");
    t.lands(P0, "Swamp", 6);
    let g = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    yes(&mut t, P0);
    yes(&mut t, P0);
    t.answer_targets(P0, &[obj(g)]);
    attack_with(&mut t, &[(et, pl(P1))]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tapped_lands(&t, P0), 3);
    // Veinwitch Coven: one life gain event, one {B}; two events, two.
    for events in [1u32, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Veinwitch Coven");
        t.lands(P0, "Swamp", 3);
        let g = t.graveyard(P0, "Hill Giant");
        let b = t.graveyard(P0, "Grizzly Bears");
        for _ in 0..3 {
            yes(&mut t, P0);
        }
        t.answer_targets(P0, &[obj(g)]);
        t.answer_targets(P0, &[obj(b)]);
        for _ in 0..events {
            gain(&mut t, P0, 2);
            t.resolve_all();
        }
        assert_eq!(tapped_lands(&t, P0), events as usize, "{events}");
        assert!(t.in_hand(P0, "Hill Giant"));
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), events == 2);
    }
}

fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tapped_lands(t, p)
}

#[test]
fn illuna_exiles_face_up() {
    cr!("406.3", "702.140d");
    ruling!(
        "Illuna, Apex of Wishes",
        "Illuna's last ability exiles the cards face up. All players will see what the card is if you put it into your hand."
    );
    supported("Illuna, Apex of Wishes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    let p1_saw = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { options, .. } if options.len() == 2),
        |g| {
            g.exile.iter().any(|id| {
                g.obj(*id).chars.name == "Hill Giant"
                    && !g.obj(*id).face_down
                    && mtg_engine::player_control::can_see(g, PlayerId(1), *id)
            })
        },
    );
    let i = t.hand(P0, "Illuna, Apex of Wishes");
    lots(&mut t, P0);
    lots(&mut t, P0);
    // {R/G} either way; over the Bears; then the Giant goes to the hand.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, i)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(p1_saw.lock().unwrap().iter().any(|x| *x), "face up in exile");
}

#[test]
fn thran_tome_targets_an_opponent() {
    cr!("115.1", "602.2b", "702.11b");
    ruling!(
        "Thran Tome",
        "It does not target the cards, but it targets the opponent. If you can’t target an opponent, you can’t activate this ability."
    );
    supported("Thran Tome");
    supported("Leyline of Sanctity");
    for leyline in [true, false] {
        let mut t = TestGame::new(2);
        let tome = t.battlefield(P0, "Thran Tome");
        t.lands(P0, "Wastes", 5);
        if leyline {
            t.battlefield(P1, "Leyline of Sanctity");
        }
        assert_eq!(can_activate(&mut t, P0, tome), !leyline);
    }
}
