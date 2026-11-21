//! roundtrip-clusters-4 (docs/agents/items/roundtrip-clusters-4.md): cards of the 3- and
//! 4-card wording clusters, checked in play where the rendering looked doubtful.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn severance_priest_token_uses_the_card_exiled_by_its_enter_ability() {
    cr!("607.2a");
    // "When this creature enters, target opponent reveals their hand. You may choose a
    // nonland card from it. If you do, exile that card. When this creature leaves the
    // battlefield, the exiled card's owner creates an X/X white Spirit creature token,
    // where X is the mana value of the exiled card."
    let mut t = TestGame::new(2);
    let giant = t.hand(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let priest = t.enter(P0, "Severance Priest");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
    t.g.sacrifice(priest, P0);
    t.settle();
    t.resolve_all();
    let spirits = t.named_on_battlefield("Spirit Token");
    assert_eq!(spirits.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(spirits[0]).controller, P1);
    assert_eq!(t.pt(spirits[0]), (4, 4));
}

#[test]
fn desecration_demon_each_opponent_may_sacrifice_one_counter() {
    cr!("101.4", "608.2c");
    ruling!(
        "Desecration Demon",
        "Each opponent in turn order may choose to sacrifice a creature"
    );
    // "At the beginning of each combat, any opponent may sacrifice a creature. If a player
    // does, tap this creature and put a +1/+1 counter on it."
    let mut t = TestGame::new(3);
    let demon = t.battlefield(P0, "Desecration Demon");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P2, &[Entity::Object(b)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b), "{}", t.dump_log());
    assert!(t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 1);
}

#[test]
fn desecration_demon_untouched_when_no_one_sacrifices() {
    cr!("608.2c");
    let mut t = TestGame::new(3);
    let demon = t.battlefield(P0, "Desecration Demon");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to_step(Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.obj_now(demon).tapped);
    assert_eq!(t.counters(demon, "+1/+1"), 0);
}

#[test]
fn underworld_fires_exiles_a_creature_it_damaged_that_dies() {
    cr!("614.1a");
    ruling!(
        "Underworld Fires",
        "if they would die for any reason that turn"
    );
    // "Underworld Fires deals 1 damage to each creature and each planeswalker. If a
    // permanent dealt damage this way would die this turn, exile it instead."
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let fires = t.hand(P0, "Underworld Fires");
    if let Err(e) = t.cast_with(P0, fires, &[]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert!(t.in_exile("Llanowar Elves"), "{}", t.dump_log());
    // The Giant survived the damage, and still dies into exile this turn.
    t.g.destroy(giant, None);
    t.settle();
    assert!(t.in_exile("Hill Giant"), "{}", t.dump_log());
}

#[test]
fn mammoth_harness_gives_first_strike_to_the_other_creature() {
    cr!("509.3d");
    // "Enchanted creature loses flying. Whenever enchanted creature blocks or becomes
    // blocked by a creature, the other creature gains first strike until end of turn."
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let harness = t.hand(P0, "Mammoth Harness");
    if let Err(e) = t.cast_with(P0, harness, &[Entity::Object(attacker)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    t.attack(&[(attacker, Entity::Player(P1))], &[(blocker, attacker)]);
    t.resolve_all();
    let has_first_strike =
        |t: &TestGame, o| t.obj_now(o).has_keyword(mtg_engine::keywords::KeywordKind::FirstStrike);
    assert!(has_first_strike(&t, blocker), "{}", t.dump_log());
    assert!(!has_first_strike(&t, attacker));
}

#[test]
fn refurbished_familiar_draws_for_each_opponent_who_cant_discard() {
    cr!("701.9a");
    // "When this creature enters, each opponent discards a card. For each opponent who
    // can't, you draw a card."
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    let before = t.hand_size(P0);
    t.enter(P0, "Refurbished Familiar");
    t.settle();
    t.resolve_all();
    // P1 discarded the Bears; P2 had no card to discard.
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), before + 1, "{}", t.dump_log());
}

#[test]
fn trial_of_agony_controller_picks_the_creature_dealt_damage() {
    cr!("115.1", "608.2d");
    // "Choose two target creatures controlled by the same opponent. That player chooses one
    // of those creatures. Trial of Agony deals 5 damage to that creature, and the other
    // can't block this turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let trial = t.hand(P0, "Trial of Agony");
    t.answer_choose(P1, &[Entity::Object(b)]);
    if let Err(e) = t.cast_with(P0, trial, &[Entity::Object(a), Entity::Object(b)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b), "{}", t.dump_log());
}

#[test]
fn gev_counts_each_opponent_who_lost_life_once() {
    cr!("614.1c");
    ruling!(
        "Gev, Scaled Scorch",
        "cares whether opponents lost life this turn"
    );
    // "Other creatures you control enter with an additional +1/+1 counter on them for each
    // opponent who lost life this turn."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Gev, Scaled Scorch");
    t.g.lose_life(P1, 1);
    t.g.lose_life(P1, 2);
    t.g.lose_life(P2, 1);
    t.settle();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.counters(bears, "+1/+1"), 2, "{}", t.dump_log());
}

#[test]
fn gorma_counts_creatures_that_died_under_your_control() {
    cr!("614.1c", "700.4");
    // "Nontoken creatures you control enter with an additional +1/+1 counter on them for
    // each creature that died under your control this turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorma, the Gullet");
    let mine = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Llanowar Elves");
    t.g.sacrifice(mine, P0);
    t.g.sacrifice(theirs, P1);
    t.settle();
    t.resolve_all();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.counters(bears, "+1/+1"), 1, "{}", t.dump_log());
}

#[test]
fn visions_flashback_costs_less_by_your_commanders_mana_value() {
    cr!("601.2f", "702.34a");
    // Visions of Glory: "Flashback {8}{W}{W}. This spell costs {X} less to cast this way,
    // where X is the greatest mana value of a commander you own on the battlefield or in
    // the command zone."
    let mut t = TestGame::new(2);
    let cmd = t.command(P0, "Hill Giant");
    t.g.obj_mut(cmd).is_commander = true;
    let visions = t.graveyard(P0, "Visions of Glory");
    t.lands(P0, "Plains", 6);
    t.battlefield(P0, "Grizzly Bears");
    let r = t
        .cast(P0, visions)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .try_go();
    assert!(r.is_ok(), "{{4}}{{W}}{{W}} with six Plains: {r:?}\n{}", t.dump_log());
    t.resolve_all();
    // One Human for each creature you control (the Bears).
    assert_eq!(t.named_on_battlefield("Human Token").len(), 1, "{}", t.dump_log());
}

#[test]
fn siege_striker_gets_a_bonus_for_each_creature_tapped_this_way() {
    cr!("701.26a");
    // "Whenever this creature attacks, you may tap any number of untapped creatures you
    // control. This creature gets +1/+1 until end of turn for each creature tapped this
    // way."
    let mut t = TestGame::new(2);
    let striker = t.battlefield(P0, "Siege Striker");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let already = t.battlefield(P0, "Llanowar Elves");
    t.g.obj_mut(already).tapped = true;
    let (p, q) = t.pt(striker);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.attack(&[(striker, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped, "{}", t.dump_log());
    assert_eq!(t.pt(striker), (p + 2, q + 2));
}

#[test]
fn phytohydra_turns_damage_into_counters() {
    cr!("614.1a");
    // "If damage would be dealt to this creature, put that many +1/+1 counters on it
    // instead."
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Phytohydra");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    if let Err(e) = t.cast_with(P1, bolt, &[Entity::Object(hydra)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 3);
    assert_eq!(t.obj_now(hydra).damage, 0);
}

#[test]
fn heart_of_light_prevents_damage_dealt_by_and_to_the_creature() {
    cr!("615.1a");
    // "Prevent all damage that would be dealt to and dealt by enchanted creature."
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let heart = t.battlefield(P0, "Heart of Light");
    assert!(t.g.attach(heart, Entity::Object(pyro)));
    let life = t.life(P1);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P1), life, "{}", t.dump_log());
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    if let Err(e) = t.cast_with(P1, bolt, &[Entity::Object(pyro)]) {
        panic!("cast: {e:?}\n{}", t.dump_log());
    }
    t.resolve_all();
    assert!(t.on_battlefield(pyro));
}

#[test]
fn scandalmonger_any_player_activates_on_their_own_turn() {
    cr!("602.5d");
    // "{2}: Target player discards a card. Any player may activate this ability but only
    // as a sorcery."
    let mut t = TestGame::new(2);
    let monger = t.battlefield(P0, "Scandalmonger");
    t.lands(P1, "Swamp", 2);
    t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    assert!(
        t.activate(P1, monger, 0, &[Entity::Player(P0)]).is_err(),
        "not on another player's turn"
    );
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, monger, 0, &[Entity::Player(P0)])
        .expect("its own main phase");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn thousand_year_elixir_lets_new_creatures_use_tap_abilities() {
    cr!("302.6");
    // "You may activate abilities of creatures you control as though those creatures had
    // haste."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Elixir");
    let pyro = t.battlefield_sick(P0, "Prodigal Pyromancer");
    let life = t.life(P1);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("as though it had haste");
    t.resolve_all();
    assert_eq!(t.life(P1), life - 1);
}

const BLITZWING: &str = "Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant";

/// Blitzwing, Cruel Tormentor: "At the beginning of your end step, target opponent loses
/// life equal to the life that player lost this turn. If no life is lost this way, convert
/// Blitzwing." `lost_before` life is lost by P1 earlier in the turn; `locked` gives P1
/// Platinum Emperion ("Your life total can't change.").
fn blitzwing_end_step(lost_before: u32, locked: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let blitzwing = t.battlefield(P0, BLITZWING);
    t.set_step(P0, Step::PrecombatMain);
    if lost_before > 0 {
        t.g.lose_life(P1, lost_before);
    }
    if locked {
        t.battlefield(P1, "Platinum Emperion");
    }
    t.settle();
    t.advance_to_step(Step::End);
    t.resolve_all();
    (t, blitzwing)
}

#[test]
fn blitzwing_converts_when_the_life_loss_doesnt_happen() {
    cr!("119.3", "608.2c", "701.28a");
    use mtg_engine::object::FaceState;
    // P1 lost 3 life this turn, so Blitzwing tells them to lose 3, but their life total
    // can't change: no life is lost this way.
    let (t, blitzwing) = blitzwing_end_step(3, true);
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
    assert_eq!(t.obj_now(blitzwing).face, FaceState::Back, "{}", t.dump_log());
    // Without the lock the 3 life is lost again, and Blitzwing stays as it is.
    let (t, blitzwing) = blitzwing_end_step(3, false);
    assert_eq!(t.life(P1), 14, "{}", t.dump_log());
    assert_eq!(t.obj_now(blitzwing).face, FaceState::Front);
    // No life lost this turn: zero life is lost this way.
    let (t, blitzwing) = blitzwing_end_step(0, false);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(blitzwing).face, FaceState::Back);
}

#[test]
fn nashi_gets_a_counter_when_no_legendary_card_is_put_into_hand() {
    cr!("608.2c");
    // Nashi, Searcher in the Dark: "Whenever Nashi deals combat damage to a player, you
    // mill that many cards. You may put any number of legendary and/or enchantment cards
    // from among them into your hand. If you put no cards into your hand this way, put a
    // +1/+1 counter on Nashi." Choosing none of the milled cards ("any number") puts no
    // cards into your hand.
    let mut t = TestGame::new(2);
    let nashi = t.battlefield(P0, "Nashi, Searcher in the Dark");
    for _ in 0..4 {
        t.library_top(P0, "Grizzly Bears");
    }
    t.library_top(P0, "Isamaru, Hound of Konda");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[]);
    t.attack(&[(nashi, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"), "{}", t.dump_log());
    assert!(!t.in_hand(P0, "Isamaru, Hound of Konda"));
    assert_eq!(t.counters(nashi, "+1/+1"), 1, "{}", t.dump_log());
}
