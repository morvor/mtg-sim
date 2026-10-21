//! Rulings batch P163 — one-sided sweepers: the affected set of a "creatures your
//! opponents control get -N/-N" effect is locked in as it resolves (CR 611.2c), while a
//! rule-changing "can't block" effect isn't (CR 611.2c); choices made on resolution
//! (CR 608.2d); illegal targets (CR 608.2b); simultaneous destruction and sacrifice
//! (CR 701.8, 701.21, 603.10a); state-based actions (CR 704.3, 704.5a, 704.5f, 704.5j);
//! colors of lands and tokens (CR 105.2c, 202.2, 111.4).

use crate::r_p163_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The "-N/-N to creatures your opponents control" spells and abilities whose affected
/// set is locked in: after `effect` resolves for P0, a creature entering under P1's
/// control and a creature P1 gains control of are unaffected.
fn locked_in(name: &str, effect: fn(&mut TestGame), minus: i32) {
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let mine = t.battlefield(P0, "Hill Giant");
    effect(&mut t);
    assert_eq!(t.pt(wurm), (6 - minus, 4 - minus), "{name}");
    let later = t.enter(P1, "Craw Wurm");
    give_control(&mut t, mine, P1);
    assert_eq!(t.g.obj(t.g.current(mine)).controller, P1);
    assert_eq!(t.pt(later), (6, 4), "{name}");
    assert_eq!(t.pt(mine), (3, 3), "{name}");
}

#[test]
fn minus_n_sweepers_lock_in_the_affected_creatures() {
    cr!("611.2c", "613.4c");
    ruling!(
        "Cower in Fear",
        "Cower in Fear affects only creatures your opponents control at the time it resolves. It won’t affect creatures that come under their control later in the turn."
    );
    ruling!(
        "Suffocating Fumes",
        "Suffocating Fumes affects only creatures your opponents control at the time it resolves. Creatures they begin to control later in the turn won’t get -1/-1."
    );
    ruling!(
        "Massacre Wurm",
        "Massacre Wurm's first ability affects only creatures your opponents control at the time it resolves. Creatures they begin to control later in the turn won't get -2/-2."
    );
    ruling!(
        "Make Obsolete",
        "The set of creatures affected by Make Obsolete is determined as the spell resolves. Creatures your opponents begin to control later in the turn and noncreature permanents that become creatures later in the turn won't get -1/-1."
    );
    for name in ["Cower in Fear", "Suffocating Fumes", "Make Obsolete", "Massacre Wurm"] {
        supported(name);
    }
    locked_in("Cower in Fear", |t| cast_resolve(t, P0, "Cower in Fear", &[]), 1);
    locked_in(
        "Suffocating Fumes",
        |t| cast_resolve(t, P0, "Suffocating Fumes", &[]),
        1,
    );
    locked_in("Make Obsolete", |t| cast_resolve(t, P0, "Make Obsolete", &[]), 1);
    locked_in(
        "Massacre Wurm",
        |t| {
            enter(t, P0, "Massacre Wurm");
            t.resolve_all();
        },
        2,
    );
    // Make Obsolete: a land that becomes a creature later isn't affected.
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    cast_resolve(&mut t, P0, "Make Obsolete", &[]);
    activate_resolve(&mut t, P1, vault, 1, &[]);
    assert!(t.obj(vault).is(CardType::Creature));
    assert_eq!(t.pt(vault), (2, 2));
}

#[test]
fn crippling_fear_locks_in_the_creatures_and_their_types() {
    cr!("611.2c", "608.2d");
    ruling!(
        "Crippling Fear",
        "Crippling Fear affects only creatures that aren't of the chosen type that are on the battlefield as it resolves. It won't affect creatures that enter the battlefield later in the turn, regardless of creature type. If a creature changes creature types later in the turn, Crippling Fear won't start or stop affecting that creature."
    );
    supported("Crippling Fear");
    supported("Amoeboid Changeling");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let ch1 = t.battlefield(P0, "Amoeboid Changeling");
    let ch2 = t.battlefield(P0, "Amoeboid Changeling");
    pick_options(&mut t, P0, &["Bear"]);
    cast_resolve(&mut t, P0, "Crippling Fear", &[]);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(wurm), (3, 1));
    // The Changelings (every creature type, so Bears) are unaffected.
    assert_eq!(t.pt(ch1), (1, 1));
    // Later: a Giant entering, the Bears losing all creature types, the Wurm gaining them.
    let giant = t.enter(P1, "Hill Giant");
    activate_resolve(&mut t, P0, ch1, 1, &[obj(bears)]);
    activate_resolve(&mut t, P0, ch2, 0, &[obj(wurm)]);
    assert!(!t.obj(bears).chars.has_subtype("Bear"));
    assert!(t.obj(wurm).chars.has_subtype("Bear"));
    assert_eq!(t.pt(giant), (3, 3));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(wurm), (3, 1));
}

/// The options of the "choose a creature type" decisions since `from` must be creature
/// types only.
fn assert_creature_types_only(t: &TestGame, from: usize) {
    let opts = creature_type_options(t, P0, from);
    assert_eq!(opts.len(), 1);
    for ok in ["Fish", "Artificer", "Bear"] {
        assert!(opts[0].iter().any(|o| o == ok), "{ok}");
    }
    for bad in ["Artifact", "Snow", "Legendary", "Creature", "Equipment", "Forest"] {
        assert!(!opts[0].iter().any(|o| o.eq_ignore_ascii_case(bad)), "{bad}");
    }
}

/// Records, at each "choose an option" decision P0 is asked, whether `watched` is on the
/// battlefield and the stack size.
fn watch_choice(t: &mut TestGame) -> crate::r_s01_common::Seen<(bool, usize)> {
    crate::r_s01_common::watch(
        t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| {
            (
                g.permanents().any(|o| o.chars.name == "Grizzly Bears"),
                g.stack.len(),
            )
        },
    )
}

#[test]
fn creature_type_choice_is_made_on_resolution_with_no_chance_to_respond() {
    cr!("608.2d", "608.2c", "205.3m");
    ruling!(
        "Crippling Fear",
        "You choose the creature type as Crippling Fear resolves, and then creatures not of that type get -3/-3. Once Crippling Fear starts to resolve, players can't respond to the choice or take any actions until Crippling Fear finishes resolving."
    );
    ruling!(
        "Kindred Dominance",
        "Once Kindred Dominance begins to resolve, no players may take other actions until it's done. Notably, players can't try to save their creatures after you've chosen a creature type."
    );
    ruling!(
        "Crippling Fear",
        "You must choose an existing creature type, such as Fish or Artificer. You can't choose card types (e.g., artifact) or supertypes (e.g., snow)."
    );
    for name in ["Crippling Fear", "Kindred Dominance"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let seen = watch_choice(&mut t);
        pick_options(&mut t, P0, &["Goblin"]);
        let spell = cast_new(&mut t, P0, name, &[]);
        // Nothing is chosen as it's cast.
        assert!(seen.lock().unwrap().is_empty(), "{name}");
        let from = n_asked(&t);
        t.resolve();
        // Chosen while the spell is resolving (still on the stack), with the Bears on the
        // battlefield; nobody got priority before the Bears were affected.
        assert_eq!(*seen.lock().unwrap(), vec![(true, 1)], "{name}");
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::Priority { .. })));
        assert!(!t.on_battlefield(bears), "{name}");
        assert!(resolved(&t, spell));
        if name == "Crippling Fear" {
            assert_creature_types_only(&t, from);
        }
    }
}

#[test]
fn sudden_demise_x_on_casting_color_on_resolution() {
    cr!("601.2b", "608.2d", "105.2a");
    ruling!(
        "Sudden Demise",
        "You choose the value for X as you cast Sudden Demise. You don’t choose the color until it resolves."
    );
    ruling!(
        "Sudden Demise",
        "Sudden Demise will deal damage to a multicolored creature if one of its colors is the chosen color."
    );
    supported("Sudden Demise");
    let mut t = TestGame::new(2);
    let gold = t.battlefield(P1, "Watchwolf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let white = t.battlefield(P1, "Savannah Lions");
    t.lands(P0, "Mountain", 3);
    let sd = t.hand(P0, "Sudden Demise");
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| g.stack.len(),
    );
    pick_options(&mut t, P0, &["green"]);
    let from = n_asked(&t);
    t.cast(P0, sd).x(2).go();
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    assert!(seen.lock().unwrap().is_empty());
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    // Watchwolf (green and white) and the Bears were dealt 2; the Lions weren't.
    assert_eq!(marked(&t, gold), 2);
    assert!(!t.on_battlefield(bears));
    assert_eq!(marked(&t, white), 0);
}

#[test]
fn celestial_judgment_chooses_on_resolution_any_creature_of_each_power() {
    cr!("608.2d", "702.12b");
    ruling!(
        "Celestial Judgment",
        "If there are multiple creatures with the same power on the battlefield, you may choose any of them, even if one of them wouldn't be destroyed. For example, if you control a 2-power creature with indestructible and a 2-power creature with no abilities, you may choose the second one even though the first one can't be destroyed by Celestial Judgment."
    );
    ruling!(
        "Celestial Judgment",
        "No creatures are targeted by Celestial Judgment, so all choices are made as the spell resolves. Notably, no players may respond or take any actions between the creatures being chosen and the other creatures being destroyed."
    );
    supported("Celestial Judgment");
    let mut t = TestGame::new(2);
    // Power 0: an indestructible Darksteel Myr and an Ornithopter (chosen).
    let myr = t.battlefield(P0, "Darksteel Myr");
    let thopter = t.battlefield(P0, "Ornithopter");
    // Power 3: two Hill Giants (the second is chosen).
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    // Power 2: only the Bears.
    let bears = t.battlefield(P1, "Grizzly Bears");
    prefer(&mut t, P0, &[thopter, g2]);
    let from = n_asked(&t);
    let spell = cast_new(&mut t, P0, "Celestial Judgment", &[]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. } | Decision::ChooseTargets { .. })));
    let from = n_asked(&t);
    t.resolve();
    assert!(resolved(&t, spell));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.on_battlefield(myr));
    assert!(t.on_battlefield(thopter));
    assert!(!t.on_battlefield(g1));
    assert!(t.on_battlefield(g2));
    assert!(t.on_battlefield(bears));
}

#[test]
fn expel_the_interlopers_numbers_include_0_and_10() {
    cr!("107.1a", "608.2d");
    ruling!("Expel the Interlopers", "The numbers you may choose include 0 and 10.");
    supported("Expel the Interlopers");
    for (n, thopter_dies, colossus_dies) in [(0, true, true), (10, false, true)] {
        let mut t = TestGame::new(2);
        let thopter = t.battlefield(P1, "Ornithopter");
        let big = t.battlefield(P1, "Craw Wurm");
        put_counters(&mut t, big, counters::PLUS1, 4);
        let from = n_asked(&t);
        t.answer(P0, DecisionKind::Number, Answer::Number(n));
        cast_resolve(&mut t, P0, "Expel the Interlopers", &[]);
        let range: Vec<_> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseNumber { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .collect();
        assert_eq!(range, vec![(0, 10)]);
        assert_eq!(!t.on_battlefield(thopter), thopter_dies, "{n}");
        assert_eq!(!t.on_battlefield(big), colossus_dies, "{n}");
    }
}

#[test]
fn sunblast_angel_destroys_tapped_creatures_including_itself() {
    cr!("608.2h", "603.2");
    ruling!(
        "Sunblast Angel",
        "The second ability destroys each creature that's tapped at the time it resolves, including creatures you control. If Sunblast Angel has become tapped by the time its ability resolves, it will be destroyed too."
    );
    supported("Sunblast Angel");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let untapped = t.battlefield(P1, "Craw Wurm");
    let angel = enter(&mut t, P0, "Sunblast Angel");
    tap(&mut t, mine);
    tap(&mut t, theirs);
    tap(&mut t, angel);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert!(!t.on_battlefield(angel));
    assert!(t.on_battlefield(untapped));
}

#[test]
fn calming_verse_checks_for_an_untapped_land_on_resolution() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Calming Verse",
        "It checks if you control an untapped land during resolution."
    );
    supported("Calming Verse");
    for tap_it in [false, true] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Glorious Anthem");
        let theirs = t.battlefield(P1, "Glorious Anthem");
        let spell = cast_new(&mut t, P0, "Calming Verse", &[]);
        // One more land, untapped as the spell is cast; maybe tapped in response.
        let land = t.battlefield(P0, "Forest");
        if tap_it {
            tap(&mut t, land);
        }
        t.resolve_all();
        assert!(resolved(&t, spell));
        assert!(!t.on_battlefield(theirs));
        assert_eq!(t.on_battlefield(mine), tap_it);
    }
}

#[test]
fn swarmyard_massacre_counts_a_rat_squirrel_once() {
    cr!("205.3", "608.2h");
    ruling!(
        "Swarmyard Massacre",
        "If you control a creature that has more than one of the listed types (for example, Daggerfang Duo, which is a Rat Squirrel), it's only counted once for Swarmyard Massacre."
    );
    supported("Swarmyard Massacre");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Daggerfang Duo");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Swarmyard Massacre", &[]);
    // The Duo and two Squirrels: -3/-3.
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn two_headed_giant_each_opponent_effects_hit_the_team_twice() {
    cr!("810.9", "810.9a", "120.3a");
    ruling!(
        "Dagger Caster",
        "In a Two-Headed Giant game, Dagger Caster’s ability causes the opposing team to lose 2 life."
    );
    supported("Dagger Caster");
    let mut t = two_headed_giant();
    let life = t.life(P2);
    let a = t.battlefield(P2, "Grizzly Bears");
    let b = t.battlefield(P3, "Grizzly Bears");
    let mine = t.battlefield(P1, "Grizzly Bears");
    enter(&mut t, P0, "Dagger Caster");
    t.resolve_all();
    assert_eq!(t.life(P2), life - 2);
    assert_eq!(t.life(P3), life - 2);
    assert_eq!((marked(&t, a), marked(&t, b), marked(&t, mine)), (1, 1, 0));
}

#[test]
fn massacre_wurm_triggers_for_deaths_from_its_own_ability() {
    cr!("704.5f", "603.2", "603.10a");
    ruling!(
        "Massacre Wurm",
        "Massacre Wurm's triggered ability triggers if a creature an opponent controls dies for any reason, including from its first ability reducing its toughness to 0, as long as Massacre Wurm remains on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Craw Wurm");
    enter(&mut t, P0, "Massacre Wurm");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn massacre_wurm_sees_a_creature_dying_at_the_same_time() {
    cr!("603.10a", "701.8a");
    ruling!(
        "Massacre Wurm",
        "If a creature an opponent controls dies at the same time as Massacre Wurm, Massacre Wurm's last ability triggers for that creature."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Massacre Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy_together(&mut t, &[w, bears]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn massacre_wurm_controller_at_zero_loses_before_the_trigger() {
    cr!("704.3", "704.5a", "704.5g", "117.5");
    ruling!(
        "Massacre Wurm",
        "If your life total is brought to 0 or less at the same time that a creature an opponent controls is dealt lethal damage, you lose the game before Massacre Wurm's last ability goes on the stack."
    );
    supported("Earthquake");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Massacre Wurm");
    t.battlefield(P1, "Savannah Lions");
    t.g.players[0].life = 1;
    t.g.players[1].life = 2;
    lands_for_cost(&mut t, P0, "Earthquake");
    t.lands(P0, "Mountain", 1);
    let eq = t.hand(P0, "Earthquake");
    t.cast(P0, eq).x(1).go();
    t.resolve();
    assert!(t.has_lost(P0));
    assert!(!t.has_lost(P1));
    assert_eq!(t.life(P1), 1);
    assert!(t.in_graveyard(P1, "Savannah Lions"));
}

#[test]
fn second_elesh_norn_dies_to_the_legend_rule_at_the_same_time() {
    cr!("704.3", "704.5j", "704.5f");
    ruling!(
        "Elesh Norn, Grand Cenobite",
        "If a second Elesh Norn, Grand Cenobite comes under your control, you'll put one into its owner's graveyard due to the “legend rule” at the same time that any of your opponents' creatures getting -4/-4 are put into their owner's graveyard for having 0 or less toughness."
    );
    supported("Elesh Norn, Grand Cenobite");
    let mut t = TestGame::new(2);
    let n1 = t.battlefield(P0, "Elesh Norn, Grand Cenobite");
    let angel = t.battlefield(P1, "Serra Angel");
    assert_eq!(t.pt(angel), (2, 2));
    let n2 = t.enter(P0, "Elesh Norn, Grand Cenobite");
    t.answer_choose(P0, &[obj(n1)]);
    t.g.flush_events();
    t.settle();
    // The 4/4 died: the legend rule wasn't applied before the toughness check.
    assert!(t.in_graveyard(P1, "Serra Angel"));
    assert!(t.on_battlefield(n1) != t.on_battlefield(n2));
    assert!(t.in_graveyard(P0, "Elesh Norn, Grand Cenobite"));
}

#[test]
fn dont_move_misses_creatures_entering_tapped() {
    cr!("701.26a", "603.2e", "611.2c");
    ruling!(
        "Don't Move",
        "Don't Move won't affect a creature that enters the battlefield tapped."
    );
    supported("Don't Move");
    let mut t = TestGame::new(2);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, tapped);
    cast_resolve(&mut t, P0, "Don't Move", &[]);
    assert!(!t.on_battlefield(tapped));
    let entered = enter_tapped(&mut t, P1, "Hill Giant");
    t.resolve_all();
    assert!(t.on_battlefield(entered));
    assert!(t.obj(entered).tapped);
    // One that becomes tapped later is destroyed.
    let later = t.battlefield(P1, "Craw Wurm");
    tap(&mut t, later);
    t.resolve_all();
    assert!(!t.on_battlefield(later));
}

#[test]
fn cosmotronic_wave_cant_block_applies_to_creatures_gained_later() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Cosmotronic Wave",
        "Because Cosmotronic Wave doesn’t change the characteristics of any permanents, the set of creatures affected by its second effect is constantly updated. Creatures that come under an opponent’s control later in the turn can’t block."
    );
    supported("Cosmotronic Wave");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let mine = t.battlefield(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Cosmotronic Wave", &[]);
    give_control(&mut t, mine, P1);
    let entered = t.enter(P1, "Hill Giant");
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    let from = n_asked(&t);
    let mine = t.g.current(mine);
    block_and_finish(&mut t, P1, &[(mine, attacker)]);
    let options: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::DeclareBlockers { options } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(options
        .iter()
        .all(|o| o.iter().all(|(b, _)| *b != mine && *b != entered)));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sagittars_volley_hits_every_opponents_flyers_and_needs_its_target() {
    cr!("608.2b", "702.12b", "802.4a");
    ruling!(
        "Sagittars' Volley",
        "Each creature with flying controlled by any of your opponents is dealt damage, even if that opponent didn’t control the target creature with flying."
    );
    ruling!(
        "Sagittars' Volley",
        "If the target creature is an illegal target by the time Sagittars’ Volley tries to resolve, the spell doesn’t resolve. No creatures are dealt damage. If the target is legal but not destroyed (most likely because it has indestructible), creatures with flying your opponents control are dealt damage, possibly including the target creature."
    );
    supported("Sagittars' Volley");
    // Three players: the target is P1's; P2's flyer is dealt damage too, P0's isn't.
    let mut t = TestGame::new(3);
    let target = t.battlefield(P1, "Wind Drake");
    let p2 = t.battlefield(P2, "Wind Drake");
    let mine = t.battlefield(P0, "Wind Drake");
    cast_resolve(&mut t, P0, "Sagittars' Volley", &[obj(target)]);
    assert!(!t.on_battlefield(target));
    assert_eq!(marked(&t, p2), 1);
    assert_eq!(marked(&t, mine), 0);
    // The target leaves: no damage.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Wind Drake");
    let other = t.battlefield(P1, "Wind Drake");
    let spell = cast_new(&mut t, P0, "Sagittars' Volley", &[obj(target)]);
    move_to(&mut t, target, Zone::Hand(P1));
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(marked(&t, other), 0);
    // An indestructible target survives and is dealt damage.
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P1, "Avacyn, Angel of Hope");
    cast_resolve(&mut t, P0, "Sagittars' Volley", &[obj(avacyn)]);
    assert!(t.on_battlefield(avacyn));
    assert_eq!(marked(&t, avacyn), 1);
}

#[test]
fn fell_the_mighty_spares_the_target_and_needs_it() {
    cr!("608.2b", "608.2h");
    ruling!("Fell the Mighty", "Fell the Mighty won't destroy the target creature.");
    ruling!(
        "Fell the Mighty",
        "If the target creature is an illegal target when Fell the Mighty tries to resolve, Fell the Mighty won't resolve and none of its effects will happen. No creatures will be destroyed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Fell the Mighty", &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(other));
    assert!(!t.on_battlefield(giant));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Fell the Mighty", &[obj(bears)]);
    move_to(&mut t, bears, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn flames_of_the_raze_boar_target_and_lethal_damage() {
    cr!("608.2b", "704.3", "608.2h");
    ruling!(
        "Flames of the Raze-Boar",
        "If the chosen target becomes an illegal target for Flames of the Raze-Boar, the spell doesn’t resolve. No creatures are dealt damage."
    );
    ruling!(
        "Flames of the Raze-Boar",
        "If the target is dealt lethal damage this way, it will still be on the battlefield while Flames of the Raze-Boar deals damage to other creatures, if appropriate. Creatures dealt lethal damage won’t die until after Flames of the Raze-Boar is entirely finished resolving."
    );
    supported("Flames of the Raze-Boar");
    // The target leaves: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Flames of the Raze-Boar", &[obj(target)]);
    move_to(&mut t, target, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(marked(&t, other), 0);
    // The target is dealt lethal damage: it isn't dealt the 2 (it's not an "other"
    // creature), and it dies only after the spell resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Flames of the Raze-Boar", &[obj(target)]);
    t.g.resolve_top();
    assert!(t.on_battlefield(target));
    assert_eq!(marked(&t, target), 4);
    assert_eq!(marked(&t, other), 2);
    t.settle();
    assert!(!t.on_battlefield(target));
    assert!(t.on_battlefield(other));
}

#[test]
fn winds_of_abandon_counts_a_token_that_ceased_to_exist() {
    cr!("111.7", "704.5d", "608.2c");
    ruling!(
        "Winds of Abandon",
        "If a creature is exiled but ends up in another zone, it's still a \"creature exiled this way\" for Winds of Abandon."
    );
    supported("Winds of Abandon");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Raise the Alarm", &[]);
    t.set_step(P0, Step::PrecombatMain);
    let soldier = tokens(&t, P1)[0];
    t.library_top(P1, "Forest");
    cast_resolve(&mut t, P0, "Winds of Abandon", &[obj(soldier)]);
    assert!(!t.g.is_live(soldier) || !t.on_battlefield(soldier));
    let forests: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.controller == P1 && o.chars.name == "Forest")
        .collect();
    assert_eq!(forests.len(), 1);
    assert!(forests[0].tapped);
}

#[test]
fn sudden_disappearance_countered_delayed_trigger_leaves_cards_exiled() {
    cr!("603.7", "603.7c", "701.6b");
    ruling!(
        "Sudden Disappearance",
        "If Sudden Disappearance’s delayed triggered ability is countered or otherwise removed from the stack after triggering, the exiled cards will remain in exile indefinitely. The delayed triggered ability will only trigger once."
    );
    supported("Sudden Disappearance");
    supported("Stifle");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Forest");
    cast_resolve(&mut t, P0, "Sudden Disappearance", &[Entity::Player(P1)]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Glorious Anthem"));
    t.advance_to(P0, Step::End);
    t.settle();
    let trigger = *t.g.stack.last().expect("the delayed trigger");
    lands_for_cost(&mut t, P1, "Stifle");
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(obj(trigger)).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // It doesn't trigger again at the next end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Glorious Anthem"));
}
