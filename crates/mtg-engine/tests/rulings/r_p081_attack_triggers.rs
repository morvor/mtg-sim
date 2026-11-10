//! Rulings batch P081 — abilities that trigger on attacks and punish the attacking
//! player: what they count and when (CR 508.1, 603.2, 603.3), intervening-if conditions
//! (CR 603.4), "your next untap step" after a control change (CR 502.3), creatures that
//! enter during the declare attackers step (CR 603.6a), and the defending player in
//! multiplayer (CR 506.2).

use crate::r_p081_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::mana::ManaType;
use mtg_engine::*;

#[test]
fn lightmine_field_damages_the_declared_attackers() {
    cr!("508.1", "603.3", "603.10", "508.4");
    ruling!(
        "Lightmine Field",
        "Which creatures are dealt damage is based on which creatures were declared as attackers during the declare attackers step."
    );
    supported("Lightmine Field");
    supported("Hero of Bladehold");
    // P1's Hero of Bladehold attacks alone; its tokens enter attacking after the Field's
    // trigger (P0's, on top of P1's) has resolved: only the Hero is dealt damage, 1 (one
    // attacking creature).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightmine Field");
    let hero = t.battlefield(P1, "Hero of Bladehold");
    attacked_by_p1(&mut t, &[hero]);
    // Battle cry, the Soldier tokens and the Field.
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(marked(&t, hero), 1);
    let toks = tokens(&t, P1);
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| marked(&t, *x) == 0));
    // Two declared attackers: each is dealt 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lightmine Field");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    attacked_by_p1(&mut t, &[a, b]);
    t.resolve_all();
    assert_eq!(marked(&t, a), 2);
    assert_eq!(marked(&t, b), 2);
}

#[test]
fn windreader_sphinx_triggers_on_its_own_attack() {
    cr!("603.2", "508.1");
    ruling!(
        "Windreader Sphinx",
        "Windreader Sphinx's ability will trigger when it itself attacks."
    );
    supported("Windreader Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Windreader Sphinx");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(sphinx, pl(P1))]);
    assert_eq!(triggers_from(&t, sphinx), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn sentry_bot_counts_the_attackers_as_its_ability_resolves() {
    cr!("608.2h", "603.3", "702.8a");
    ruling!(
        "Sentry Bot",
        "Use the number of creatures attacking you as Sentry Bot’s third ability resolves to determine the number of energy counters you get."
    );
    supported("Sentry Bot");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a, b], &[]);
    // {4}{W} costs {2}{W} with two attackers.
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let bot = t.hand(P0, "Sentry Bot");
    t.cast(P0, bot).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1, "the enters trigger");
    // One attacker leaves in response.
    cast_new(&mut t, P0, "Unsummon", &[obj(b)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "energy"), 1);
}

#[test]
fn vengeful_pharaoh_must_already_be_in_the_graveyard() {
    cr!("603.2", "603.3", "603.4", "510.2");
    ruling!(
        "Vengeful Pharaoh",
        "Vengeful Pharaoh must be in your graveyard when combat damage is dealt to you or a planeswalker you control in order for its ability to trigger. That is, it can't die and trigger from your graveyard during the same combat damage step."
    );
    supported("Vengeful Pharaoh");
    // The Pharaoh blocks a 6/4 Craw Wurm and dies as P0 is dealt damage by an unblocked
    // Grizzly Bears: no trigger.
    let mut t = TestGame::new(2);
    let ph = t.battlefield(P0, "Vengeful Pharaoh");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[wurm, bears], &[(ph, wurm)]);
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert!(died(&t, ph));
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(bears));
    // Already in the graveyard: it triggers, destroys an attacker and goes on top of the
    // library.
    let mut t = TestGame::new(2);
    let ph = t.graveyard(P0, "Vengeful Pharaoh");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[bears], &[]);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P1, Step::CombatDamage);
    t.resolve_all();
    assert!(died(&t, bears));
    assert_eq!(t.zone(ph), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library.last(), Some(&t.g.current(ph)));
}

#[test]
fn trouble_in_pairs_counts_countered_spells_and_needs_only_the_second() {
    cr!("603.2");
    ruling!(
        "Trouble in Pairs",
        "Trouble in Pairs counts spells that were cast even if they didn't resolve. This means it still counts spells that were countered."
    );
    ruling!(
        "Trouble in Pairs",
        "Trouble in Pairs doesn't have to be on the battlefield to see the first card an opponent draws in a turn or the first spell they cast in a turn. As long as it's already on the battlefield when they draw their second card or cast their second spell in a turn, its last ability will trigger."
    );
    supported("Trouble in Pairs");
    // P1's first spell is countered; the second triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trouble in Pairs");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    let s1 = cast_new(&mut t, P1, "Grizzly Bears", &[]);
    cast_new(&mut t, P0, "Counterspell", &[obj(s1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand);
    cast_new(&mut t, P1, "Hill Giant", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the spell and the trigger");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // It enters after P1's first spell: the second spell still triggers it.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    t.battlefield(P0, "Trouble in Pairs");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Hill Giant", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // And after P1's first draw of the turn (the draw step): the second draw triggers it.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    t.battlefield(P0, "Trouble in Pairs");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn mangara_and_ever_watching_threshold_draw_one_card_per_attack() {
    cr!("603.2c", "508.1", "603.4", "508.3d", "805.10a");
    ruling!(
        "Mangara, the Diplomat",
        "You draw just one card, no matter how many creatures are attacking you and your planeswalkers beyond the second."
    );
    ruling!(
        "Ever-Watching Threshold",
        "You’ll draw only one card for each opponent attacking you or your planeswalkers, no matter how many creatures they’re attacking with. If multiple opponents are attacking you or your planeswalkers at once, you’ll draw one card for each of those opponents."
    );
    for name in ["Mangara, the Diplomat", "Ever-Watching Threshold"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let hand = t.hand_size(P0);
        let at: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        blocked_by_p0(&mut t, &at, &[]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
    // Two-Headed Giant: both opponents (P2 and P3) attack P0 at once: two cards.
    let mut t = crate::r_p044_common::two_headed_giant();
    t.battlefield(P0, "Ever-Watching Threshold");
    let hand = t.hand_size(P0);
    let a = t.battlefield(P2, "Grizzly Bears");
    let b = t.battlefield(P3, "Grizzly Bears");
    t.set_step(P2, Step::BeginningOfCombat);
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, pl(P0)), (b, pl(P0))]),
    );
    t.advance_to(P2, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.g.is_attacking(a) && t.g.is_attacking(b));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn patron_of_the_kitsune_gains_life_per_attacker_of_any_player() {
    cr!("603.2", "508.1");
    ruling!(
        "Patron of the Kitsune",
        "You may gain life when any creatures are declared as attackers, not just for attacking creatures you control."
    );
    ruling!(
        "Patron of the Kitsune",
        "You may gain one life per attacking creature when the ability resolves during each declare attackers step."
    );
    supported("Patron of the Kitsune");
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Kitsune");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attacked_by_p1(&mut t, &[a, b]);
    assert_eq!(triggers_from(&t, patron), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Its own attack too.
    let mut t = TestGame::new(2);
    let patron = t.battlefield(P0, "Patron of the Kitsune");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(patron, pl(P1))]);
    assert_eq!(triggers_from(&t, patron), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn impulsive_maneuvers_flips_a_coin_per_attacker() {
    cr!("603.2", "705.1");
    ruling!(
        "Impulsive Maneuvers",
        "You flip a coin for each creature that attacks, not once per attack."
    );
    supported("Impulsive Maneuvers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impulsive Maneuvers");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    // Both flips are won: both creatures deal double damage.
    crate::r_p017_common::load_coins(&mut t, &[true, true]);
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1))]);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(crate::r_p017_common::flip_results(&t).len(), 2);
    finish_combat(&mut t);
    // 6 from the Giant and 4 from the Bears.
    assert_eq!(t.life(P1), 10);
}

#[test]
fn thantis_cant_make_you_attack_yourself() {
    cr!("506.2", "508.1", "508.1d");
    ruling!(
        "Thantis, the Warweaver",
        "You can't attack yourself or your own planeswalkers to give Thantis +1/+1 counters, but Thantis appreciates the way you think."
    );
    supported("Thantis, the Warweaver");
    let mut t = TestGame::new(2);
    let thantis = t.battlefield(P0, "Thantis, the Warweaver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.recompute();
    for (c, targets) in mtg_engine::combat::attack_options(&t.g) {
        assert!(!targets.contains(&pl(P0)), "{c:?}");
        assert!(!targets.contains(&obj(jace)), "{c:?}");
        assert!(targets.contains(&pl(P1)), "{c:?}");
    }
    attack_with(&mut t, &[(thantis, pl(P1)), (bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(thantis, "+1/+1"), 0);
}

#[test]
fn kazuul_is_the_defending_player_for_its_planeswalkers_and_battles_too() {
    cr!("506.2", "310.5", "603.4");
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "You're the defending player if a creature is attacking you, a planeswalker you control, or a battle you protect."
    );
    supported("Kazuul, Tyrant of the Cliffs");
    // Three players: P1 attacks P2 only: P0's Kazuul doesn't trigger.
    let mut t = TestGame::new(3);
    let kaz = t.battlefield(P0, "Kazuul, Tyrant of the Cliffs");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, pl(P2))]);
    assert_eq!(triggers_from(&t, kaz), 0);
    // P1 attacks P0's planeswalker: it triggers (P1 doesn't pay): an Ogre.
    let mut t = TestGame::new(3);
    let kaz = t.battlefield(P0, "Kazuul, Tyrant of the Cliffs");
    let jace = t.battlefield(P0, "Jace Beleren");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, obj(jace))]);
    assert_eq!(triggers_from(&t, kaz), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // P1 attacks a battle (Invasion of Ikoria, P2's Siege) that P0 protects.
    let mut t = TestGame::new(3);
    let kaz = t.battlefield(P0, "Kazuul, Tyrant of the Cliffs");
    t.set_step(P2, Step::PrecombatMain);
    t.answer_choose(P2, &[pl(P0)]);
    let siege = t.enter(P2, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    t.resolve_all();
    assert_eq!(mtg_engine::battle::protector(&t.g, siege), Some(P0));
    let a = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, obj(siege))]);
    assert_eq!(triggers_from(&t, kaz), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn wars_toll_lets_the_opponent_tap_lands_in_response() {
    cr!("605.3a", "603.3", "106.4");
    ruling!(
        "War's Toll",
        "Your opponent can tap lands for mana in response to the triggered ability. The effect won't prevent your opponent from casting spells or activating abilities, but it will make it difficult for that player to cast other spells or activate abilities later in the turn because unspent mana is lost at the end of each step."
    );
    supported("War's Toll");
    let mut t = TestGame::new(2);
    let toll = t.battlefield(P0, "War's Toll");
    let lands = t.lands(P1, "Forest", 3);
    t.set_step(P1, Step::PrecombatMain);
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P1, lands[0], "{G}"));
    t.settle();
    assert_eq!(triggers_from(&t, toll), 1);
    // In response, P1 taps another land for mana, then casts Fog with the two mana.
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P1, lands[1], "{G}"));
    assert_eq!(t.g.player(P1).mana_pool.total(), 2);
    let fog = t.hand(P1, "Fog");
    t.cast(P1, fog).go();
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Fog"));
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(t.g.player(P1).mana_pool.total(), 0, "lost at the end of the step");
    assert!(lands.iter().all(|l| tapped(&t, *l)), "all of P1's lands are tapped");
}

#[test]
fn spectral_bears_checks_for_black_permanents_twice() {
    cr!("603.4", "508.1");
    ruling!(
        "Spectral Bears",
        "The checks for black cards is made both at the time you declare attackers and as the ability resolves. If there are any at the time attackers are declared, it won’t trigger at all. If there are any at the time it resolves, the ability will do nothing."
    );
    supported("Spectral Bears");
    // A black nontoken permanent as attackers are declared: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Spectral Bears");
    t.battlefield(P1, "Black Knight");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(triggers_from(&t, bears), 0);
    // None then, one as it resolves: nothing happens; it untaps next turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Spectral Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert_eq!(triggers_from(&t, bears), 1);
    t.battlefield(P1, "Black Knight");
    t.g.recompute();
    t.resolve_all();
    finish_combat(&mut t);
    assert!(tapped(&t, bears));
    t.advance_to(P0, Step::Upkeep);
    assert!(!tapped(&t, bears), "it untapped");
    // None at either time: it doesn't untap.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Spectral Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    finish_combat(&mut t);
    t.advance_to(P0, Step::Upkeep);
    assert!(tapped(&t, bears));
}

#[test]
fn spectral_force_stolen_for_a_turn_untaps_with_its_owner() {
    cr!("502.3", "603.3", "608.2h");
    ruling!(
        "Spectral Force",
        "”Your next untap…” refers to the player who controls Spectral Force when the triggered ability resolves. That means if it’s temporarily stolen (such as with Word of Seizing) and attacked with, it will untap in its previous controller’s untap step."
    );
    supported("Spectral Force");
    let mut t = TestGame::new(2);
    let force = t.battlefield(P0, "Spectral Force");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Act of Treason", &[obj(force)]);
    t.resolve_all();
    assert_eq!(t.obj_now(force).controller, P1);
    attack_with(&mut t, &[(force, pl(P0))]);
    assert_eq!(triggers_from(&t, force), 1);
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 12);
    assert!(tapped(&t, force));
    // Back with P0 at end of turn; it untaps in P0's untap step.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(force).controller, P0);
    assert!(!tapped(&t, force));
}

#[test]
fn whirling_dervish_counts_damage_to_its_current_controllers_opponents_once() {
    cr!("603.4", "603.2", "603.3");
    ruling!(
        "Whirling Dervish",
        "Whirling Dervish's triggered ability checks to see if it dealt damage during the turn to an opponent of its current controller. If it dealt damage to player A, then player A takes control of it, the ability doesn't trigger."
    );
    ruling!(
        "Whirling Dervish",
        "Whirling Dervish's triggered ability gives it at most one +1/+1 counter each turn, regardless of how many opponents were dealt damage beyond the first."
    );
    supported("Whirling Dervish");
    supported("Control Magic");
    // It damages P1; then P1 takes control of it (Control Magic): no counter.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Whirling Dervish");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(d, pl(P1))]);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 19);
    crate::r_s06_common::attach_new(&mut t, P1, "Control Magic", d);
    t.g.recompute();
    assert_eq!(t.obj_now(d).controller, P1);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, d), 0);
    t.resolve_all();
    assert_eq!(t.counters(d, "+1/+1"), 0);
    // Three players: it pings P1 (Hermetic Study), is untapped (Vitalize) and pings P2:
    // one counter.
    supported("Hermetic Study");
    supported("Vitalize");
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Whirling Dervish");
    crate::r_s06_common::attach_new(&mut t, P0, "Hermetic Study", d);
    t.g.recompute();
    t.activate(P0, d, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    cast_new(&mut t, P0, "Vitalize", &[]);
    t.resolve_all();
    assert!(!tapped(&t, d));
    t.activate(P0, d, 0, &[pl(P2)]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_from(&t, d), 1);
    t.resolve_all();
    assert_eq!(t.counters(d, "+1/+1"), 1);
}

#[test]
fn gabriel_angelfire_keeps_the_ability_through_the_untap_step() {
    cr!("611.2a", "500.4", "503.1");
    ruling!(
        "Gabriel Angelfire",
        "Gabriel Angelfire will have the ability it gains through your next untap step, but will no longer have it during your next upkeep step."
    );
    supported("Gabriel Angelfire");
    let mut t = TestGame::new(2);
    let gabe = t.battlefield(P0, "Gabriel Angelfire");
    t.set_step(P0, Step::Untap);
    // Flying (the first option).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(has_keyword(&t, gabe, KeywordKind::Flying));
    t.advance_to(P1, Step::PrecombatMain);
    assert!(has_keyword(&t, gabe, KeywordKind::Flying));
    t.advance_to(P0, Step::Upkeep);
    // The upkeep has begun; the trigger hasn't resolved yet: no flying.
    assert!(!has_keyword(&t, gabe, KeywordKind::Flying));
    // First strike this time.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert!(has_keyword(&t, gabe, KeywordKind::FirstStrike));
    assert!(!has_keyword(&t, gabe, KeywordKind::Flying));
}

#[test]
fn attack_reselectors_dont_trigger_outside_the_declare_attackers_step() {
    cr!("603.2", "603.6a");
    ruling!(
        "Misleading Signpost",
        "You may cast Misleading Signpost outside of a declare attackers step. If Misleading Signpost enters the battlefield outside of a declare attackers step, its ability simply doesn't trigger."
    );
    ruling!(
        "Portal Mage",
        "You may cast Portal Mage outside of a declare attackers step. If Portal Mage enters the battlefield outside of a declare attackers step, its ability simply doesn’t trigger."
    );
    ruling!(
        "Windshaper Planetar",
        "You may cast Windshaper Planetar outside of a declare attackers step. If Windshaper Planetar enters the battlefield outside of a declare attackers step, its ability simply doesn't trigger."
    );
    for name in ["Misleading Signpost", "Portal Mage", "Windshaper Planetar"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        assert!(can_cast_now(&mut t, P0, name), "{name}");
        let spell = cast_new(&mut t, P0, name, &[]);
        t.resolve();
        assert!(t.on_battlefield(t.g.current(spell)), "{name}");
        assert_eq!(t.stack_len(), 0, "{name}: no trigger");
    }
}

#[test]
fn tamiyo_transformed_in_combat_waits_for_a_main_phase() {
    cr!("606.3", "117.3a");
    ruling!(
        "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar",
        "You can activate one of Tamiyo, Seasoned Scholar's loyalty abilities the turn she enters the battlefield. However, you may do so only during one of your main phases when the stack is empty. For example, if Tamiyo, Seasoned Scholar enters the battlefield during combat, there will be an opportunity for your opponent to remove her before you can activate one of her abilities."
    );
    supported("Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar");
    let mut t = TestGame::new(2);
    let tam = t.battlefield(P0, "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar");
    t.set_step(P0, Step::BeginningOfCombat);
    // Three Opts during combat: the third card drawn this turn transforms her.
    for _ in 0..3 {
        cast_new(&mut t, P0, "Opt", &[]);
        t.resolve_all();
    }
    assert!(t.obj_now(tam).is(CardType::Planeswalker));
    assert!(!crate::r_p001_common::can_activate_nth(&mut t, P0, tam, 0));
    // P1 gets priority in combat before P0's main phase and can bolt her (loyalty 2).
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(tam)]);
    t.resolve_all();
    assert!(!t.on_battlefield(t.g.current(tam)));
    // Left alone, she can be activated in the postcombat main phase.
    let mut t = TestGame::new(2);
    let tam = t.battlefield(P0, "Tamiyo, Inquisitive Student // Tamiyo, Seasoned Scholar");
    t.set_step(P0, Step::BeginningOfCombat);
    for _ in 0..3 {
        cast_new(&mut t, P0, "Opt", &[]);
        t.resolve_all();
    }
    t.advance_to(P0, Step::PostcombatMain);
    assert!(crate::r_p001_common::can_activate_nth(&mut t, P0, tam, 0));
}

#[test]
fn unstable_glyphbridge_triggers_only_if_cast() {
    cr!("603.2", "601.2i");
    ruling!(
        "Unstable Glyphbridge // Sandswirl Wanderglyph",
        "Unstable Glyphbridge's first ability triggers if you cast it from any zone. It doesn't trigger if you put Unstable Glyphbridge onto the battlefield without casting it."
    );
    supported("Unstable Glyphbridge // Sandswirl Wanderglyph");
    let name = "Unstable Glyphbridge // Sandswirl Wanderglyph";
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // Cast: it triggers and destroys the creatures not chosen (none with power 2 or less).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, name, &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(died(&t, a) && died(&t, b));
}

#[test]
fn balduvian_trading_post_sacrifices_a_mountain_however_it_enters_and_doubles_either_type() {
    cr!("614.1a", "614.12", "605.1a");
    ruling!(
        "Balduvian Trading Post",
        "You sacrifice the Mountain before this card is put onto the battlefield, and do so no matter how it is put onto the battlefield."
    );
    ruling!(
        "Balduvian Trading Post",
        "This card produces two different types of mana. If an effect adds one mana of the same type as the mana produced by this card, you choose which of the two types it produces."
    );
    supported("Balduvian Trading Post");
    supported("Soldevi Excavations");
    supported("Mana Flare");
    // Played from hand: a Mountain is sacrificed first.
    let mut t = TestGame::new(2);
    let mtn = t.battlefield(P0, "Mountain");
    let post = t.hand(P0, "Balduvian Trading Post");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mtn)]);
    t.play_land(P0, post).unwrap();
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(t.on_battlefield(t.g.current(post)));
    // Put onto the battlefield by an effect (Crop Rotation, sacrificing a Forest and
    // searching for it): the same.
    supported("Crop Rotation");
    let mut t = TestGame::new(2);
    let mtn = t.battlefield(P0, "Mountain");
    let forest = t.battlefield(P0, "Forest");
    let post = t.library_top(P0, "Balduvian Trading Post");
    t.lands(P0, "Forest", 1);
    t.answer_choose(P0, &[obj(forest)]);
    t.answer_choose_moved(P0, &[obj(post)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mtn)]);
    let rot = t.hand(P0, "Crop Rotation");
    t.cast(P0, rot).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(t.on_battlefield(t.g.current(post)));
    // Without an untapped Mountain it goes to the graveyard instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    let post = t.hand(P0, "Balduvian Trading Post");
    t.play_land(P0, post).unwrap();
    assert!(!t.on_battlefield(t.g.current(post)));
    assert!(t.in_graveyard(P0, "Balduvian Trading Post"));
    // Mana Flare: "one additional mana of any type that land produced": {C} or {R}.
    for (choice, other) in [(ManaType::C, ManaType::R), (ManaType::R, ManaType::C)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mana Flare");
        let post = t.battlefield(P0, "Balduvian Trading Post");
        t.answer(P0, DecisionKind::Option, Answer::Index(if choice == ManaType::R { 1 } else { 0 }));
        assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, post, "Add"));
        t.resolve_all();
        let pool = &t.g.player(P0).mana_pool;
        assert_eq!(pool.total(), 3);
        assert_eq!(pool.count(choice), 2, "{choice:?}");
        assert_eq!(pool.count(other), 1, "{choice:?}");
    }
}

#[test]
fn entrapment_maneuver_uses_the_toughness_as_the_creature_last_existed() {
    cr!("608.2h", "115.1a");
    ruling!(
        "Entrapment Maneuver",
        "To determine how many Soldier tokens are created, use the toughness of the sacrificed creature as it last existed on the battlefield."
    );
    supported("Entrapment Maneuver");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    blocked_by_p0(&mut t, &[a], &[]);
    cast_new(&mut t, P1, "Giant Growth", &[obj(a)]);
    t.resolve();
    assert_eq!(t.pt(a), (6, 6));
    cast_new(&mut t, P0, "Entrapment Maneuver", &[pl(P1)]);
    t.resolve_all();
    assert!(died(&t, a));
    assert_eq!(tokens(&t, P0).len(), 6);
}

#[test]
fn wing_shards_targets_the_player_who_then_chooses_the_creature() {
    cr!("115.1a", "701.21a", "608.2c");
    ruling!(
        "Wing Shards",
        "Wing Shards doesn’t target the creature to be sacrificed. The target player chooses the creature as it resolves."
    );
    supported("Wing Shards");
    supported("Silhana Ledgewalker");
    let mut t = TestGame::new(2);
    let hex = t.battlefield(P1, "Silhana Ledgewalker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[hex, bears], &[]);
    let from = mark(&t);
    // The only targets are players; P1 sacrifices the hexproof Ledgewalker.
    t.answer_choose(P1, &[obj(hex)]);
    cast_new(&mut t, P0, "Wing Shards", &[pl(P1)]);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(cands[0].iter().all(|e| matches!(e, Entity::Player(_))));
    t.resolve_all();
    assert!(died(&t, hex));
    assert!(t.on_battlefield(bears));
}

#[test]
fn goblin_kaboomist_creates_the_mine_before_the_flip() {
    cr!("608.2c", "705.1");
    ruling!(
        "Goblin Kaboomist",
        "You flip the coin after the Land Mine token is created, not after you sacrifice a Land Mine. The token is created even if you lose the flip."
    );
    supported("Goblin Kaboomist");
    let mut t = TestGame::new(2);
    let kab = t.battlefield(P0, "Goblin Kaboomist");
    crate::r_p017_common::load_coins(&mut t, &[false]);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.obj_now(tokens(&t, P0)[0]).chars.name, "Land Mine");
    assert!(died(&t, kab), "2 damage to the 1/2");
}
