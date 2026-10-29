//! Rulings batch P118 — miscellaneous power-matters abilities: sets of creatures fixed as
//! an ability resolves (CR 608.2h, 611.2c), mana abilities (CR 605), free casting with X
//! (CR 107.3b, 601.2b), base power and toughness (CR 613.4b), characteristic-defining
//! abilities (CR 604.3), playing lands from the graveyard (CR 305.1), "enters with"
//! counters (CR 614.1c), and replacement effects on zone changes (CR 614.6).

use crate::r_p118_common::*;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Lena, Selfless Champion -------------------------------------------------------------------

#[test]
fn lena_counts_nontoken_creatures_including_itself_as_it_resolves() {
    cr!("608.2h", "111.1");
    ruling!(
        "Lena, Selfless Champion",
        "The number of nontoken creatures you control is counted only as Lena's first ability resolves. If Lena is still on the battlefield (and not somehow a token), it will count itself."
    );
    supported("Lena, Selfless Champion");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    t.enter(P0, "Lena, Selfless Champion");
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1);
    // Another nontoken creature arrives before it resolves.
    t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    // Lena, Bears and Elves: three new Soldiers (plus the one from before).
    assert_eq!(tokens_named(&t, P0, "Soldier"), 4);
}

/// P0's Lena with `n` counters is sacrificed for its ability (with Grizzly Bears, Hill
/// Giant and Craw Wurm on P0's side); returns the game after it resolved.
fn lena_sacrificed(n: u32) -> (TestGame, [ObjectId; 3]) {
    let mut t = TestGame::new(2);
    let lena = with_counters(&mut t, P0, "Lena, Selfless Champion", n);
    let ids = [
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Hill Giant"),
        t.battlefield(P0, "Craw Wurm"),
    ];
    t.activate(P0, lena, 0, &[]).unwrap();
    assert!(!t.on_battlefield(lena));
    t.resolve_all();
    (t, ids)
}

fn indestructible(t: &mut TestGame, id: ObjectId) -> bool {
    has_kw(t, id, KeywordKind::Indestructible)
}

#[test]
fn lena_uses_its_last_known_power() {
    cr!("608.2h", "113.7a", "602.2");
    ruling!(
        "Lena, Selfless Champion",
        "To determine which creatures gain indestructible while resolving Lena's last ability, use Lena's power as it last existed on the battlefield."
    );
    // Lena 5/5: Bears (2) and Hill Giant (3) gain indestructible; Craw Wurm (6) doesn't.
    let (mut t, [b, g, w]) = lena_sacrificed(2);
    assert!(indestructible(&mut t, b));
    assert!(indestructible(&mut t, g));
    assert!(!indestructible(&mut t, w));
    // Lena 3/3: only the Bears.
    let (mut t, [b, g, _]) = lena_sacrificed(0);
    assert!(indestructible(&mut t, b));
    assert!(!indestructible(&mut t, g));
}

#[test]
fn lena_affects_only_the_creatures_there_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Lena, Selfless Champion",
        "Lena's last ability affects only creatures you control with the appropriate power at the time it resolves. Creatures you begin to control later in the turn won't gain indestructible, and a creature you control whose power increases later in the turn won't lose indestructible."
    );
    let (mut t, [b, _, _]) = lena_sacrificed(0);
    giant_growth(&mut t, b);
    assert_eq!(t.pt(b), (5, 5));
    assert!(indestructible(&mut t, b));
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(!indestructible(&mut t, elves));
}

// --- mana abilities ---------------------------------------------------------------------------

#[test]
fn marwyn_and_mona_lisa_mana_abilities_dont_use_the_stack() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Marwyn, the Nurturer",
        "Marwyn's activated ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Mona Lisa, Science Geek",
        "Mona Lisa's last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Marwyn, the Nurturer");
    supported("Mona Lisa, Science Geek");
    let mut t = TestGame::new(2);
    let m = with_counters(&mut t, P0, "Marwyn, the Nurturer", 2);
    t.activate(P0, m, 0, &[]).unwrap();
    assert_eq!(stack(&t), 0);
    assert_eq!(pool_total(&t, P0), 3);
    let mut t = TestGame::new(2);
    let m = with_counters(&mut t, P0, "Mona Lisa, Science Geek", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, m, 0, &[]).unwrap();
    assert_eq!(stack(&t), 0);
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn helga_mana_can_pay_any_part_of_a_spell_with_x() {
    cr!("106.6", "107.3", "601.2g");
    ruling!(
        "Helga, Skittish Seer",
        "If you spend the mana added by Helga's last ability on a creature spell with {X} in its mana cost, the mana can be spent on any part of that spell's total cost. You're not limited to spending it only on the {X} part."
    );
    // Helga 4/6: {G}{G}{G}{G} pays all of Lifeblood Hydra's {X}{G}{G}{G} with X = 1.
    let mut t = TestGame::new(2);
    let h = with_counters(&mut t, P0, "Helga, Skittish Seer", 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, h, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 4);
    let hydra = t.hand(P0, "Lifeblood Hydra");
    t.cast(P0, hydra).x(1).go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    let hydra = t.g.current(hydra);
    assert!(t.on_battlefield(hydra));
    assert_eq!(t.pt(hydra), (1, 1));
}

// --- Narset, Enlightened Exile ------------------------------------------------------------------

#[test]
fn narset_gives_prowess_to_creatures_you_control_including_itself() {
    cr!("702.108a", "611.3a");
    ruling!(
        "Narset, Enlightened Exile",
        "Narset's first ability gives it prowess while it's on the battlefield."
    );
    supported("Narset, Enlightened Exile");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset, Enlightened Exile");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(has_kw(&mut t, n, KeywordKind::Prowess));
    assert!(has_kw(&mut t, bears, KeywordKind::Prowess));
    assert!(!has_kw(&mut t, theirs, KeywordKind::Prowess));
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.pt(n), (4, 5));
}

#[test]
fn narset_leaving_doesnt_stop_a_prowess_trigger() {
    cr!("603.3", "702.108a", "611.2c");
    ruling!(
        "Narset, Enlightened Exile",
        "Once a prowess ability triggers, causing the creature to lose prowess by removing Narset, Enlightened Exile won't affect that ability. The creature will still get +1/+1 until end of turn."
    );
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset, Enlightened Exile");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    exile(&mut t, n);
    assert!(!has_kw(&mut t, bears, KeywordKind::Prowess));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn free_casts_of_x_spells_choose_zero() {
    cr!("107.3b", "601.2b");
    ruling!(
        "Narset, Enlightened Exile",
        "If the spell you cast has {X} in its mana cost, you must choose 0 as the value of X."
    );
    ruling!(
        "Velomachus Lorehold",
        "If you’re casting a spell with {X} in its cost without paying its mana cost, you must choose 0 for the value of X."
    );
    supported("Velomachus Lorehold");
    supported("Blaze");
    // Narset (3/4): exile Blaze (mana value 1) from a graveyard; cast the copy.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset, Enlightened Exile");
    let blaze = t.graveyard(P1, "Blaze");
    t.answer_targets(P0, &[obj(blaze)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &at(P1, &[n]));
    t.resolve();
    assert!(t.in_exile("Blaze"));
    // (Narset's prowess trigger for the copy is above it.)
    let top = *t
        .g
        .stack
        .iter()
        .find(|id| t.g.obj(**id).chars.name == "Blaze")
        .expect("the copy was cast");
    assert_ne!(x_of(&t, top), Some(5));
    t.resolve_all();
    assert_eq!(lost(&t, P1), 0, "X = 0");
    // Velomachus (5/5): Blaze among the top seven.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Velomachus Lorehold");
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let blaze = t.library_top(P0, "Blaze");
    t.answer_choose(P0, &[obj(blaze)]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &at(P1, &[v]));
    t.resolve();
    let top = *t.g.stack.last().expect("Blaze was cast");
    assert_eq!(t.g.obj(top).chars.name, "Blaze");
    assert_ne!(x_of(&t, top), Some(5));
    t.resolve_all();
    assert_eq!(lost(&t, P1), 0, "X = 0");
}

#[test]
fn velomachus_casting_nothing_puts_all_seven_on_the_bottom() {
    cr!("401.4", "701.24a");
    ruling!(
        "Velomachus Lorehold",
        "If you choose not to cast a spell this way, all of the cards will be put on the bottom of the library in a random order."
    );
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Velomachus Lorehold");
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    t.library_top(P0, "Shock");
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    t.answer_choose(P0, &[]);
    t.answer_yes(P0, false);
    attack_with(&mut t, &at(P1, &[v]));
    t.resolve_all();
    // The library from the top (the last element of the list is the top card).
    let lib: Vec<String> = t.g.players[0]
        .library
        .iter()
        .rev()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    let n = lib.len();
    assert!(lib[..3].iter().all(|n| n == "Forest"), "{lib:?}");
    let bottom = &lib[n - 7..];
    assert!(bottom.contains(&"Shock".to_string()), "{lib:?}");
    assert_eq!(bottom.iter().filter(|n| *n == "Island").count(), 6, "{lib:?}");
}

// --- sets of creatures, base power and toughness ---------------------------------------------------

#[test]
fn sigardian_zealot_may_choose_a_single_creature_as_it_resolves() {
    cr!("608.2c", "115.10");
    ruling!(
        "Sigardian Zealot",
        "No creatures are targeted by Sigardian Zealot's ability. The set of creatures is chosen as the ability resolves. You may choose a single creature as a valid set. If you do, that creature will get +X/+X and gain vigilance until end of turn."
    );
    supported("Sigardian Zealot");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigardian Zealot");
    let bears = t.battlefield(P0, "Grizzly Bears");
    beginning_of_combat(&mut t);
    assert_eq!(stack(&t), 1);
    // A creature with hexproof arriving before it resolves can be chosen.
    let hexy = t.battlefield(P1, "Gladecover Scout");
    t.answer_choose(P0, &[obj(hexy)]);
    t.resolve_all();
    assert_eq!(t.pt(hexy), (4, 4));
    assert!(has_kw(&mut t, hexy, KeywordKind::Vigilance));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn obuun_overwrites_earlier_pt_setting_and_keeps_abilities() {
    cr!("613.4b", "613.7");
    ruling!(
        "Obuun, Mul Daya Ancestor",
        "Obuun's first ability can target a land that's already a creature. This overwrites any previous effects that set the land creature's power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    ruling!(
        "Obuun, Mul Daya Ancestor",
        "Obuun's first ability doesn't remove any abilities the land creature has."
    );
    supported("Obuun, Mul Daya Ancestor");
    let set = |t: &mut TestGame, id: ObjectId, n: i32| {
        modify(t, id, vec![Modification::SetPT(Some(Value::c(n)), Some(Value::c(n)))]);
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Obuun, Mul Daya Ancestor");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    set(&mut t, arbor, 5);
    modify(
        &mut t,
        arbor,
        vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))],
    );
    assert_eq!(t.pt(arbor), (5, 5));
    t.answer_targets(P0, &[obj(arbor)]);
    beginning_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.pt(arbor), (3, 3));
    assert!(has_kw(&mut t, arbor, KeywordKind::Flying));
    assert!(has_kw(&mut t, arbor, KeywordKind::Trample));
    assert!(t.obj_now(arbor).chars.has_subtype("Forest"));
    set(&mut t, arbor, 1);
    assert_eq!(t.pt(arbor), (1, 1));
}

#[test]
fn exuberant_wolfbear_copies_actual_power_and_toughness() {
    cr!("613.4b", "611.2c");
    ruling!(
        "Exuberant Wolfbear",
        "The Human’s new base power and toughness are set to Exuberant Wolfbear’s actual power and toughness, not just its base power and toughness. If Exuberant Wolfbear’s power or toughness changes later in the turn, the Human isn’t affected."
    );
    let mut t = TestGame::new(2);
    let w = with_counters(&mut t, P0, "Exuberant Wolfbear", 2);
    let human = t.battlefield(P0, "Elite Vanguard");
    t.answer_targets(P0, &[obj(human)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &at(P1, &[w]));
    t.resolve_all();
    assert_eq!(t.pt(human), (6, 6));
    give_plus1(&mut t, w, 2);
    assert_eq!(t.pt(human), (6, 6));
}

#[test]
fn aurelia_bonuses_by_color_and_kept_after_color_changes() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Aurelia, Exemplar of Justice",
        "The target creature gains trample and vigilance if it’s both red and white. It gets +2/+0 only once, even if it’s both red and white, and even if it’s neither red nor white."
    );
    ruling!(
        "Aurelia, Exemplar of Justice",
        "Once Aurelia’s last ability has resolved, the creature keeps whatever bonuses it got from that ability even if its colors change."
    );
    supported("Aurelia, Exemplar of Justice");
    // (creature, P/T after, trample, vigilance)
    let cases = [
        ("Boros Recruit", (3, 1), true, true),
        ("Raging Goblin", (3, 1), true, false),
        ("Savannah Lions", (4, 1), false, true),
        ("Grizzly Bears", (4, 2), false, false),
    ];
    for (name, pt, trample, vigilance) in cases {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aurelia, Exemplar of Justice");
        let c = t.battlefield(P0, name);
        t.answer_targets(P0, &[obj(c)]);
        beginning_of_combat(&mut t);
        t.resolve_all();
        assert_eq!(t.pt(c), pt, "{name}");
        assert_eq!(has_kw(&mut t, c, KeywordKind::Trample), trample, "{name}");
        assert_eq!(has_kw(&mut t, c, KeywordKind::Vigilance), vigilance, "{name}");
        if trample && vigilance {
            modify(
                &mut t,
                c,
                vec![Modification::SetColors(mtg_engine::types::ColorSet::single(
                    mtg_engine::types::Color::Blue,
                ))],
            );
            assert!(has_kw(&mut t, c, KeywordKind::Trample), "{name}");
            assert!(has_kw(&mut t, c, KeywordKind::Vigilance), "{name}");
            assert_eq!(t.pt(c), pt);
        }
    }
}

#[test]
fn packsong_pup_gets_only_one_counter() {
    cr!("603.4", "122.1");
    ruling!(
        "Packsong Pup",
        "Packsong Pup gets only one +1/+1 counter as its first ability resolves, no matter how many other Wolf or Werewolf creatures you control."
    );
    supported("Packsong Pup");
    let mut t = TestGame::new(2);
    let pup = t.battlefield(P0, "Packsong Pup");
    t.battlefield(P0, "Wandering Wolf");
    t.battlefield(P0, "Wandering Wolf");
    beginning_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(plus1(&t, pup), 1);
}

// --- Pyrogoyf -----------------------------------------------------------------------------------

#[test]
fn pyrogoyf_counts_card_types_in_all_zones() {
    cr!("604.3", "205.2a");
    ruling!(
        "Pyrogoyf",
        "The ability that defines Pyrogoyf's power and toughness counts card types, not cards. If the only card in all graveyards is a single artifact creature card, Pyrogoyf will be a 2/3. If the cards in all graveyards are ten artifact cards and ten creature cards, Pyrogoyf will still be a 2/3."
    );
    ruling!(
        "Pyrogoyf",
        "The ability that defines Pyrogoyf's power and toughness works in all zones, not just the battlefield."
    );
    supported("Pyrogoyf");
    let mut t = TestGame::new(2);
    let on = t.battlefield(P0, "Pyrogoyf");
    let in_hand = t.hand(P0, "Pyrogoyf");
    t.graveyard(P1, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(on), (2, 3));
    let h = t.obj_now(in_hand);
    assert_eq!((h.chars.power, h.chars.toughness), (Some(2), Some(3)));
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P1, "Bonesplitter");
    }
    t.g.recompute();
    assert_eq!(t.pt(on), (2, 3));
    // A land adds a type.
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(on), (3, 4));
}

// --- Szarel, Genesis Shepherd ---------------------------------------------------------------------

#[test]
fn szarel_lets_you_play_lands_from_the_graveyard_only_normally() {
    cr!("305.1", "305.2", "602.1");
    ruling!(
        "Szarel, Genesis Shepherd",
        "Szarel’s second ability doesn’t allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    ruling!(
        "Szarel, Genesis Shepherd",
        "Szarel’s second ability doesn’t change the times when you can play those land cards. You can still play lands only during your main phase when you have priority and the stack is empty."
    );
    supported("Szarel, Genesis Shepherd");
    supported("Secluded Steppe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Szarel, Genesis Shepherd");
    t.lands(P0, "Plains", 1);
    let steppe = t.graveyard(P0, "Secluded Steppe");
    let in_hand = t.hand(P0, "Secluded Steppe");
    assert!(can_activate(&mut t, P0, in_hand), "cycling from hand");
    assert!(!can_activate(&mut t, P0, steppe), "no cycling from the graveyard");
    assert!(can_play_land(&mut t, P0, steppe));
    // Not with something on the stack, nor outside a main phase.
    let szarel = t.named_on_battlefield("Szarel, Genesis Shepherd")[0];
    cast_new(&mut t, P0, "Giant Growth", &[obj(szarel)]);
    assert!(!can_play_land(&mut t, P0, steppe));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, steppe));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, steppe));
}

// --- Temple of Power -------------------------------------------------------------------------------

#[test]
fn temple_of_power_adds_up_noncombat_damage_from_several_red_sources() {
    cr!("602.5b", "120.1");
    ruling!(
        "Ojer Axonil, Deepest Might // Temple of Power",
        "The 4 or more noncombat damage may be dealt by multiple red sources you controlled. For example, if you cast a red spell that dealt 3 damage to a creature and then a red artifact you controlled dealt 1 noncombat damage to a creature, you could activate Temple of Power's last ability."
    );
    supported("Ojer Axonil, Deepest Might // Temple of Power");
    let mut t = TestGame::new(2);
    let temple = enter_with(&mut t, P0, "Ojer Axonil, Deepest Might // Temple of Power", 0, true);
    assert_eq!(t.obj_now(temple).chars.name, "Temple of Power");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Shock", &[obj(wurm)]);
    t.resolve_all();
    // (The Temple may have been tapped for the Shock's mana.)
    t.g.objects[temple.0 as usize].tapped = false;
    t.lands(P0, "Mountain", 3);
    assert!(t.activate(P0, temple, 1, &[]).is_err(), "only 2 damage so far");
    t.g.players[0].mana_pool = Default::default();
    cast_new(&mut t, P0, "Shock", &[obj(giant)]);
    t.resolve_all();
    t.g.objects[temple.0 as usize].tapped = false;
    t.lands(P0, "Mountain", 3);
    t.activate(P0, temple, 1, &[]).unwrap();
    t.resolve_all();
    let t2 = t.g.current(temple);
    assert_eq!(t.obj_now(t2).chars.name, "Ojer Axonil, Deepest Might");
}

// --- Thought Sponge --------------------------------------------------------------------------------

fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    for _ in 0..n {
        t.library_top(p, "Island");
    }
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

fn sponge(t: &mut TestGame) -> ObjectId {
    supported("Thought Sponge");
    let s = t.enter(P0, "Thought Sponge");
    t.g.flush_events();
    t.settle();
    s
}

#[test]
fn thought_sponge_counts_the_greatest_total_drawn_by_one_opponent() {
    cr!("614.1c", "121.2", "800.4a");
    ruling!(
        "Thought Sponge",
        "Thought Sponge checks how many cards were drawn by opponents this turn, even opponents who have left the game. The greatest number drawn by a single opponent is how many +1/+1 counters it enters with."
    );
    ruling!(
        "Thought Sponge",
        "Thought Sponge checks the total number of cards drawn by each opponent, not how many were drawn due to a single instruction."
    );
    // P1 draws one card, then two: three in total.
    let mut t = TestGame::new(3);
    draw(&mut t, P1, 1);
    draw(&mut t, P1, 2);
    draw(&mut t, P2, 2);
    draw(&mut t, P0, 5);
    let s = sponge(&mut t);
    assert_eq!(plus1(&t, s), 3);
    // P1 draws three cards and leaves the game.
    let mut t = TestGame::new(3);
    draw(&mut t, P1, 3);
    draw(&mut t, P2, 1);
    crate::r_p053_common::concede(&mut t, P1);
    assert!(t.has_lost(P1));
    let s = sponge(&mut t);
    assert_eq!(plus1(&t, s), 3);
}

#[test]
fn thought_sponge_enters_with_its_counters() {
    cr!("614.1c", "614.12");
    ruling!(
        "Thought Sponge",
        "Thought Sponge gets its +1/+1 counters at the moment it enters the battlefield. There's not a time that it's on the battlefield before its replacement effect has applied."
    );
    // Garruk's Packleader: "Whenever another creature you control with power 3 or greater
    // enters, you may draw a card."
    supported("Garruk's Packleader");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garruk's Packleader");
    draw(&mut t, P1, 3);
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    let s = sponge(&mut t);
    assert_eq!(t.pt(s), (4, 4));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// --- Grenzo, Dungeon Warden ------------------------------------------------------------------------

/// P0's Grenzo (2/2) activates with Grizzly Bears as the only card of P0's library (with
/// P1's Leyline of the Void if `leyline`).
fn grenzo(leyline: bool) -> TestGame {
    let mut t = TestGame::new(2);
    if leyline {
        t.battlefield(P1, "Leyline of the Void");
    }
    let g = t.battlefield(P0, "Grenzo, Dungeon Warden");
    // The bottom card (the first of the list).
    let bears = t.library_top(P0, "Grizzly Bears");
    let lib = &mut t.g.players[0].library;
    lib.retain(|id| *id != bears);
    lib.insert(0, bears);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, g, 0, &[]).unwrap();
    t
}

#[test]
fn grenzo_puts_the_creature_onto_the_battlefield_from_the_graveyard() {
    cr!("608.2", "117.2");
    ruling!(
        "Grenzo, Dungeon Warden",
        "If the card is a creature card with power less than or equal to Grenzo's power, it's put onto the battlefield from your graveyard, not from your library. Players can't attempt to exile it from your graveyard in response."
    );
    let mut t = grenzo(false);
    let from = n_asked(&t);
    t.g.resolve_top();
    t.g.flush_events();
    assert!(!priority_asked_since(&t, from));
    t.settle();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    let came_from_graveyard = t.g.turn_events.iter().any(|e| {
        matches!(e, mtg_engine::events::Event::ZoneChange { from: Zone::Graveyard(_), to: Zone::Battlefield, .. })
    });
    assert!(came_from_graveyard, "{}", t.dump_log());
}

#[test]
fn grenzo_puts_the_creature_onto_the_battlefield_from_exile_under_leyline() {
    cr!("614.6", "608.2c");
    ruling!(
        "Grenzo, Dungeon Warden",
        "If a replacement effect (such as that of Leyline of the Void) causes the card to be put from your library directly into exile, you'll put it onto the battlefield from exile if it's a creature card with appropriate power."
    );
    supported("Leyline of the Void");
    let mut t = grenzo(true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{}", t.dump_log());
    assert!(!t.in_exile("Grizzly Bears"));
}

// --- Jyoti, Moag Ancient ------------------------------------------------------------------------

#[test]
fn jyoti_counts_casts_of_each_of_your_commanders() {
    cr!("903.8", "608.2h");
    ruling!(
        "Jyoti, Moag Ancient",
        "If you have multiple commanders, Jyoti's first ability will create a 1/1 green Forest Dryad land creature token for each time you've cast either of them. For example, if you've cast one commander once and one twice, you'll create three Forest Dryads."
    );
    let mut t = TestGame::new(2);
    let p = &mut t.g.players[0];
    p.commander_names = vec!["Tymna the Weaver".into(), "Thrasios, Triton Hero".into()];
    p.commander_casts.insert("Tymna the Weaver".into(), 1);
    p.commander_casts.insert("Thrasios, Triton Hero".into(), 2);
    t.enter(P0, "Jyoti, Moag Ancient");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Dryad"), 3);
}
