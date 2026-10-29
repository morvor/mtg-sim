//! Rulings batch P118 — combat: "creatures with power less than this creature's power
//! can't block it" and "can't attack or block unless its power is ..." are checked only as
//! attackers or blockers are declared (CR 506.4, 508.1c, 509.1b); "can't attack or block
//! alone" (CR 508.1c, 509.1b, 805.10b); attack requirements and costs (CR 508.1d); and
//! creatures put onto the battlefield attacking (CR 508.4).

use crate::r_p118_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- "creatures with power less than this creature's power can't block it" ----------------

/// P0's `watcher` (with `n` +1/+1 counters, transformed if `back`) makes P1's `blocker`
/// unable to block P0's attacker (the watcher itself, or a Grizzly Bears if `other`).
/// Checks that the unpumped blocker can't block, that a blocker pumped before blocks can,
/// and that changing either power afterwards doesn't undo the block.
fn compared_at_blocks(watcher: &str, back: bool, n: u32, other: bool, blocker: &str) {
    supported(watcher);
    let setup = |t: &mut TestGame| {
        let w = if back {
            enter_with(t, P0, watcher, n, true)
        } else {
            with_counters(t, P0, watcher, n)
        };
        let a = if other {
            t.battlefield(P0, "Grizzly Bears")
        } else {
            w
        };
        let b = t.battlefield(P1, blocker);
        attack_with(t, &[(a, Entity::Player(P1))]);
        (w, a, b)
    };
    let mut t = TestGame::new(2);
    let (_, a, b) = setup(&mut t);
    assert!(!legal_blocks(&mut t, P1, &[(b, a)]), "{watcher}: blocked by {blocker}");
    let mut t = TestGame::new(2);
    let (w, a, b) = setup(&mut t);
    giant_growth(&mut t, b);
    assert!(legal_blocks(&mut t, P1, &[(b, a)]), "{watcher}");
    declare_blocks(&mut t, P1, &[(b, a)]);
    assert!(is_blocked(&t, a), "{watcher}");
    give_plus1(&mut t, w, 5);
    minus_power(&mut t, b, 4);
    assert!(is_blocked(&t, a), "{watcher}: still blocked");
    assert!(blocking(&t, b), "{watcher}");
}

#[test]
fn power_comparisons_for_blocking_are_made_only_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Skarrgan Pit-Skulk",
        "Skarrgan Pit-Skulk compares each blocker's power against its own as blockers are being declared."
    );
    ruling!(
        "Champion of Lambholt",
        "The comparison of power is done only when blockers are declared. Decreasing the power of a blocking creature (or increasing the power of Champion of Lambholt) after this point won't cause any creature to stop blocking or become unblocked."
    );
    ruling!(
        "Wandering Wolf",
        "The comparison of power is only done when blockers are declared. Increasing the power of a blocking creature (or decreasing the power of Wandering Wolf) after this point won’t cause any creature to stop blocking or become unblocked."
    );
    ruling!(
        "Howlgeist",
        "The comparison of power is performed only when blockers are declared. Increasing the power of a blocking creature (or decreasing the power of Howlgeist) after this point won’t cause any creature to stop blocking or become unblocked."
    );
    ruling!(
        "Aura Gnarlid",
        "The power of each potential blocking creature is checked against Aura Gnarlid's power only as blockers are declared. Once a creature blocks Aura Gnarlid, increasing the Gnarlid's power or decreasing the blocking creature's power won't undo that block."
    );
    ruling!(
        "Shrill Howler // Howling Chorus",
        "The power of the blocking creature and that of Shrill Howler are compared only at the moment that blockers are chosen. Changing either creature's power later won't cause Shrill Howler to become unblocked."
    );
    ruling!(
        "Shrill Howler // Howling Chorus",
        "The power of the blocking creature and that of Howling Chorus are compared only at the moment that blockers are chosen. Changing either creature's power later won't cause Howling Chorus to become unblocked."
    );
    // Skarrgan Pit-Skulk 3/3 with two counters vs a Bears (2, pumped to 5).
    compared_at_blocks("Skarrgan Pit-Skulk", false, 2, false, "Grizzly Bears");
    // Champion of Lambholt 4/4 makes a Hill Giant (3, pumped to 6) unable to block Bears.
    compared_at_blocks("Champion of Lambholt", false, 3, true, "Hill Giant");
    compared_at_blocks("Wandering Wolf", false, 0, false, "Llanowar Elves");
    compared_at_blocks("Howlgeist", false, 0, false, "Hill Giant");
    compared_at_blocks("Aura Gnarlid", false, 0, false, "Llanowar Elves");
    // Shrill Howler 3/1 and Howling Chorus 3/5.
    compared_at_blocks("Shrill Howler // Howling Chorus", false, 0, false, "Grizzly Bears");
    compared_at_blocks("Shrill Howler // Howling Chorus", true, 0, false, "Grizzly Bears");
}

// --- "can't attack or block unless its power is ..." -----------------------------------------

#[test]
fn lowering_power_after_attacking_doesnt_remove_it_from_combat() {
    cr!("508.1c", "506.4");
    ruling!(
        "Karsus Depthguard",
        "Once Karsus Depthguard has legally attacked, causing its last ability to not apply by reducing its power to 4 or less won’t cause it to stop attacking."
    );
    ruling!(
        "Technodrome",
        "Once Technodrome is attacking or blocking, lowering its power won't cause it to stop attacking or blocking."
    );
    // (card, counters to be able to attack)
    for (name, n) in [("Karsus Depthguard", 1), ("Technodrome", 3)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        to_combat(&mut t, P0);
        assert!(!legal_attack(&mut t, &at(P1, &[c])), "{name}: too small");
        give_plus1(&mut t, c, n);
        attack_with(&mut t, &at(P1, &[c]));
        assert!(attacking(&t, c), "{name}");
        minus_power(&mut t, c, 3);
        assert!(attacking(&t, c), "{name}: still attacking");
        t.advance_to(P0, Step::EndOfCombat);
        assert!(lost(&t, P1) > 0, "{name}: dealt combat damage");
    }
    // Technodrome blocking.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let td = with_counters(&mut t, P0, "Technodrome", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[(td, bears)]);
    minus_power(&mut t, td, 3);
    assert!(blocking(&t, td));
    assert!(is_blocked(&t, bears));
}

// --- Wojek Bodyguard ----------------------------------------------------------------------

#[test]
fn wojek_bodyguard_attack_requirement_forces_a_partner_to_attack() {
    cr!("508.1c", "508.1d");
    ruling!(
        "Wojek Bodyguard",
        "If an effect says that Wojek Bodyguard attacks or blocks if able and you control another creature able to attack or block, you must attack or block with Wojek Bodyguard and another creature."
    );
    supported("Wojek Bodyguard");
    let mut t = TestGame::new(2);
    let wojek = t.battlefield(P0, "Wojek Bodyguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    // It gains Bloodrock Cyclops's "This creature attacks each combat if able."
    supported("Bloodrock Cyclops");
    let must = mtg_engine::card::card("Bloodrock Cyclops").front().chars.abilities[0].clone();
    modify(&mut t, wojek, vec![mtg_engine::ability::Modification::AddAbility(must)]);
    // Not attacking, or attacking with the Bears alone, doesn't obey the requirement;
    // Wojek can't attack alone, so both must attack.
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &at(P1, &[bears])));
    assert!(!legal_attack(&mut t, &at(P1, &[wojek])));
    assert!(legal_attack(&mut t, &at(P1, &[wojek, bears])));
}

#[test]
fn two_wojek_bodyguards_may_attack_or_block_together() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Wojek Bodyguard",
        "If you control more than one Wojek Bodyguard, they can attack or block together, even if no other creatures attack or block."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Wojek Bodyguard");
    let b = t.battlefield(P0, "Wojek Bodyguard");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P1, &[a])));
    assert!(legal_attack(&mut t, &at(P1, &[a, b])));
    // Blocking.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let a = t.battlefield(P0, "Wojek Bodyguard");
    let b = t.battlefield(P0, "Wojek Bodyguard");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    assert!(!legal_blocks(&mut t, P0, &[(a, giant)]));
    assert!(legal_blocks(&mut t, P0, &[(a, giant), (b, giant)]));
}

#[test]
fn wojek_bodyguard_may_attack_with_a_teammates_creature() {
    cr!("508.1c", "805.10b");
    ruling!(
        "Wojek Bodyguard",
        "In a Two-Headed Giant game, Wojek Bodyguard can attack or block with a creature controlled by your teammate, even if no other creatures you control are attacking or blocking."
    );
    let mut t = two_headed_giant();
    let w = t.battlefield(P0, "Wojek Bodyguard");
    let mate = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &at(P2, &[w])));
    assert!(legal_attack(&mut t, &at(P2, &[w, mate])));
    attack_with(&mut t, &at(P2, &[w, mate]));
    assert!(attacking(&t, w) && attacking(&t, mate));
}

#[test]
fn wojek_bodyguard_remains_in_combat_alone() {
    cr!("506.4", "508.1c");
    ruling!(
        "Wojek Bodyguard",
        "Once Wojek Bodyguard has attacked or blocked, it will remain in combat even if you no longer control another attacking or blocking creature."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wojek Bodyguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &at(P1, &[w, bears]));
    // Mentor's trigger is on the stack; let it resolve, then remove the Bears.
    t.resolve_all();
    exile(&mut t, bears);
    assert!(attacking(&t, w));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(lost(&t, P1), 3);
}

// --- Legion Warboss -----------------------------------------------------------------------

#[test]
fn legion_warboss_token_isnt_forced_to_attack_if_it_cant_or_if_it_costs() {
    cr!("508.1d");
    ruling!(
        "Legion Warboss",
        "If the token can't attack for any reason (such as becoming tapped), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Legion Warboss");
    // (tap the token, P1 controls Ghostly Prison, not attacking is legal)
    for (tap, prison, ok) in [(false, false, false), (true, false, true), (false, true, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Legion Warboss");
        if prison {
            t.battlefield(P1, "Ghostly Prison");
        }
        beginning_of_combat(&mut t);
        t.resolve_all();
        let tok = t
            .g
            .permanents()
            .find(|o| o.controller == P0 && o.is_token())
            .map(|o| o.id)
            .expect("Goblin token");
        if tap {
            t.g.objects[tok.0 as usize].tapped = true;
        }
        assert_eq!(legal_attack(&mut t, &[]), ok, "tap {tap}, prison {prison}");
    }
}

// --- Krenko's tokens and Gemini Engine's Twin ---------------------------------------------

#[test]
fn krenko_tokens_arent_attacking() {
    cr!("508.1a", "508.4");
    ruling!(
        "Krenko, Tin Street Kingpin",
        "The tokens created by Krenko's triggered ability aren't attacking. Because all attackers are chosen at once, a token created this way can't attack, even if it gains haste."
    );
    supported("Krenko, Tin Street Kingpin");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Krenko, Tin Street Kingpin");
    attack_with(&mut t, &at(P1, &[k]));
    t.resolve_all();
    let goblins: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| o.id)
        .collect();
    assert_eq!(goblins.len(), 2);
    assert!(goblins.iter().all(|g| !attacking(&t, *g)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(lost(&t, P1), 2, "only Krenko dealt damage");
}

/// P0 attacks with Gemini Engine (and `extra` permanents on the battlefield); returns the
/// game after the Twin was created, and the Engine.
fn gemini_attacks(t: &mut TestGame) -> ObjectId {
    supported("Gemini Engine");
    let e = t.battlefield(P0, "Gemini Engine");
    attack_with(t, &at(P1, &[e]));
    t.resolve_all();
    e
}

fn twin(t: &TestGame) -> ObjectId {
    t.g.permanents()
        .find(|o| o.chars.name == "Twin")
        .map(|o| o.id)
        .expect("no Twin")
}

#[test]
fn gemini_engine_twin_copies_its_power_then_is_independent() {
    cr!("111.1", "613.4c", "608.2h");
    ruling!(
        "Gemini Engine",
        "The token's power and toughness are set equal to Gemini Engine's power and toughness. If there's a Glorious Anthem on the battlefield (which gives your creatures +1/+1), Gemini Engine is 4/5. The Twin token starts as a 4/5, and then the Anthem gives it +1/+1, making the Twin a 5/6 creature."
    );
    ruling!(
        "Gemini Engine",
        "Once the token has been created, its power and toughness aren't dependent on Gemini Engine's power and toughness, and they won't change if Gemini Engine's power and toughness change."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    let e = gemini_attacks(&mut t);
    let tw = twin(&t);
    assert_eq!(t.pt(tw), (5, 6));
    give_plus1(&mut t, e, 2);
    assert_eq!(t.pt(e), (6, 7));
    assert_eq!(t.pt(tw), (5, 6));
}

#[test]
fn gemini_engine_twin_attacks_without_restrictions_or_attack_triggers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Gemini Engine",
        "The Twin token enters as an untapped attacking creature. It's not subject to any costs to attack or restrictions on attacking."
    );
    ruling!(
        "Gemini Engine",
        "The Twin token won't trigger any abilities that trigger \"when [a creature] attacks.\""
    );
    // Silent Arbiter: "No more than one creature can attack each combat." Hellrider:
    // "Whenever a creature you control attacks, Hellrider deals 1 damage to the player or
    // planeswalker it's attacking."
    supported("Silent Arbiter");
    supported("Hellrider");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Silent Arbiter");
    t.battlefield(P0, "Hellrider");
    gemini_attacks(&mut t);
    let tw = twin(&t);
    assert!(attacking(&t, tw));
    assert!(!t.obj_now(tw).tapped);
    assert_eq!(lost(&t, P1), 1, "one Hellrider trigger, for the Engine");
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(lost(&t, P1), 1 + 3 + 3);
}

// --- Shadowfax ------------------------------------------------------------------------------

#[test]
fn shadowfax_compares_power_in_hand_not_on_the_battlefield() {
    cr!("508.4", "613.1");
    ruling!(
        "Shadowfax, Lord of Horses",
        "The creature card you put onto the battlefield needs to have lesser power in your hand than the Shadowfax, Lord of Horses on the battlefield. The creature that enters the battlefield doesn't need to have lesser power than Shadowfax (this can happen if it enters with +1/+1 counters on it, for example)."
    );
    supported("Shadowfax, Lord of Horses");
    // Mogg Flunkies is 3/3 in hand and 4/4 on the battlefield under Goblin King.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shadowfax, Lord of Horses");
    t.battlefield(P0, "Goblin King");
    let f = t.hand(P0, "Mogg Flunkies");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(f)]);
    attack_with(&mut t, &at(P1, &[s]));
    t.resolve_all();
    let f = t.g.current(f);
    assert!(t.on_battlefield(f));
    assert_eq!(t.pt(f), (4, 4));
    assert!(attacking(&t, f));
    // Positive control for "lesser": a 4-power card in hand can't be put in.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shadowfax, Lord of Horses");
    let a = t.hand(P0, "Serra Angel");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    attack_with(&mut t, &at(P1, &[s]));
    t.resolve_all();
    assert!(t.in_hand(P0, "Serra Angel"));
}

// --- Soltari Guerrillas ------------------------------------------------------------------------

#[test]
fn soltari_guerrillas_redirects_to_the_defending_player_whenever_activated() {
    cr!("614.1a", "615.1", "510.2");
    ruling!(
        "Soltari Guerrillas",
        "Regardless of when the ability is activated, it will apply to the defending player at the time Soltari Guerrillas tries to deal combat damage to them."
    );
    supported("Soltari Guerrillas");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.battlefield(P0, "Soltari Guerrillas");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Activated in the main phase, before there's a defending player.
    t.activate(P0, s, 0, &[obj(wurm)]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &at(P1, &[s]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(lost(&t, P1), 0);
    assert_eq!(damage(&t, wurm), 3);
}

// --- Vincent ------------------------------------------------------------------------------

#[test]
fn vincent_chaos_damage_isnt_combat_damage() {
    cr!("510.2", "120.2b", "603.2");
    ruling!(
        "Vincent, Vengeful Atoner",
        "The damage dealt by Vincent as a result of its last ability is not combat damage and therefore doesn't cause the ability to trigger again."
    );
    supported("Vincent, Vengeful Atoner");
    let mut t = TestGame::new(3);
    let v = with_counters(&mut t, P0, "Vincent, Vengeful Atoner", 4);
    attack_with(&mut t, &at(P1, &[v]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 7);
    assert_eq!(lost(&t, P2), 7);
}

#[test]
fn truefire_captain_triggers_even_when_dealt_lethal_damage() {
    cr!("603.10a", "120.3", "704.5g");
    ruling!(
        "Truefire Captain",
        "Truefire Captain’s last ability will trigger even if it is dealt lethal damage. For example, if it’s dealt damage by a 7/7 creature it’s blocking, its ability will trigger and Truefire Captain will deal 7 damage to the target player."
    );
    supported("Truefire Captain");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let tc = t.battlefield(P0, "Truefire Captain");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(wurm, Entity::Player(P0))]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    block_and_finish(&mut t, P0, &[(tc, wurm)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Truefire Captain"));
    assert_eq!(lost(&t, P1), 6);
}

#[test]
fn truefire_captain_controller_loses_before_the_trigger_is_put_on_the_stack() {
    cr!("704.5a", "117.5", "104.3b");
    ruling!(
        "Truefire Captain",
        "If your life total is brought to 0 or less at the same time that Truefire Captain is dealt damage, you lose the game before its triggered ability goes on the stack."
    );
    // Earthquake X=3: 3 damage to each creature without flying and each player.
    for (life, p1_lost) in [(3, 3), (20, 6)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Truefire Captain");
        set_life(&mut t, P0, life);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.lands(P1, "Mountain", 4);
        let eq = t.hand(P1, "Earthquake");
        t.cast(P1, eq).x(3).go();
        t.resolve_all();
        assert_eq!(t.has_lost(P0), life == 3);
        assert_eq!(lost(&t, P1), p1_lost, "life {life}");
    }
}

// --- Iron Mastiff ----------------------------------------------------------------------------

#[test]
fn iron_mastiff_resolves_without_anyone_getting_priority() {
    cr!("608.2", "117.2", "706.2");
    ruling!(
        "Iron Mastiff",
        "Once Iron Mastiff's triggered ability begins to resolve, no player may cast spells or activate abilities until it finishes resolving. Notably, this means that players may not take actions that would change Iron Mastiff's power after they know the result of the rolls but before players are dealt damage."
    );
    supported("Iron Mastiff");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Iron Mastiff");
    attack_with(&mut t, &at(P1, &[m]));
    assert_eq!(stack(&t), 1);
    let from = n_asked(&t);
    t.g.resolve_top();
    assert!(!priority_asked_since(&t, from));
    assert_eq!(lost(&t, P0) + lost(&t, P1), 4, "4 damage to one player");
}

#[test]
fn ravenous_tyrannosaurus_deals_excess_damage_simultaneously_and_modifiably() {
    cr!("120.4a", "120.10", "614.1a");
    ruling!(
        "Ravenous Tyrannosaurus",
        "Once you've determined how much damage is excess, Ravenous Tyrannosaurus simultaneously deals damage to the target creature and its controller. This damage may be modified by replacement or prevention effects."
    );
    supported("Ravenous Tyrannosaurus");
    // Furnace of Rath doubles both: 2 lethal + 4 excess becomes 4 and 8.
    for (furnace, to_bears, to_p1) in [(false, 2, 4), (true, 4, 8)] {
        let mut t = TestGame::new(2);
        if furnace {
            t.battlefield(P1, "Furnace of Rath");
        }
        let r = t.battlefield(P0, "Ravenous Tyrannosaurus");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        attack_with(&mut t, &at(P1, &[r]));
        assert_eq!(stack(&t), 1);
        t.resolve();
        let dealt: u32 = t
            .g
            .turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::Damage { amount, target, .. }
                    if matches!(target, Entity::Object(_)) =>
                {
                    Some(*amount)
                }
                _ => None,
            })
            .sum();
        assert_eq!(dealt, to_bears, "furnace {furnace}");
        assert_eq!(lost(&t, P1), to_p1, "furnace {furnace}");
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
    }
}

#[test]
fn bloodshot_trainee_deals_damage_even_if_its_power_drops() {
    cr!("602.5b", "608.2");
    ruling!(
        "Bloodshot Trainee",
        "Once Bloodshot Trainee's ability has been activated, it will deal damage even if Bloodshot Trainee's power is less than 4 by the time the ability resolves."
    );
    supported("Bloodshot Trainee");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bloodshot Trainee");
    let wurm = t.battlefield(P1, "Craw Wurm");
    assert!(t.activate(P0, b, 0, &[obj(wurm)]).is_err(), "power 2");
    give_plus1(&mut t, b, 2);
    t.activate(P0, b, 0, &[obj(wurm)]).unwrap();
    minus_power(&mut t, b, 3);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}
