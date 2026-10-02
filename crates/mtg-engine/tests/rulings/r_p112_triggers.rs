//! Rulings batch P112 — triggered abilities and replacement effects: one trigger per
//! creature dealt damage (CR 603.2c), extra triggers (CR 603.2d), delayed triggers that
//! trigger only once (CR 603.7c), Chatterfang's token replacement (CR 614.1a, 111.1),
//! per-opponent combat-damage triggers in Two-Headed Giant (CR 805.10b), intervening "if"
//! clauses (CR 603.4), copies that aren't cast (CR 707.10), "may pay" costs paid once,
//! attack requirements (CR 508.1d) and targets chosen as abilities are put on the stack.

use crate::r_p050_common::two_headed_giant;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s09_common::legal_attack;
use crate::r_s10_common::attacking;
use crate::r_s12_common::attack_target;
use crate::r_s13_common::add;
use crate::r_s19_common::{gain_level, level};
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::{cast_new, keep_copy_targets, spell_copies, targets_of};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn kazarov_triggers_for_each_creature_dealt_damage_at_once() {
    cr!("603.2c");
    ruling!(
        "Kazarov, Sengir Pureblood",
        "Kazarov's triggered ability triggers once for each creature dealt damage at one time."
    );
    supported("Kazarov, Sengir Pureblood");
    let mut t = TestGame::new(2);
    let kazarov = t.battlefield(P0, "Kazarov, Sengir Pureblood");
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(kazarov, counters::PLUS1), 2);
}

#[test]
fn splinter_extra_trigger_chooses_targets_separately() {
    cr!("603.2d", "603.3d");
    ruling!(
        "Splinter, Radical Rat",
        "Splinter's first ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time. Any choices made as you put the ability onto the stack, such as modes and targets, are made separately for each instance of the ability."
    );
    supported("Splinter, Radical Rat");
    supported("Mistblade Shinobi");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Splinter, Radical Rat");
    let ninja = t.battlefield(P0, "Mistblade Shinobi");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[a.into()]);
    t.answer_targets(P0, &[b.into()]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(ninja, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19);
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn old_hob_delayed_trigger_doesnt_trigger_again() {
    cr!("603.7c", "603.7b");
    ruling!(
        "Old Hob, Alleycat Blues",
        "In the case where the Mutant creature token isn't destroyed by the delayed triggered ability (probably because it gained indestructible due to Old Hob's last ability), that delayed triggered ability won't trigger again."
    );
    supported("Old Hob, Alleycat Blues");
    let mut t = TestGame::new(2);
    let hob = t.battlefield(P0, "Old Hob, Alleycat Blues");
    t.lands(P0, "Plains", 2);
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let mutant = toks[0];
    attack_with(&mut t, &[(mutant, Entity::Player(P1))]);
    t.activate(P0, hob, 0, &[mutant.into()]).unwrap();
    t.resolve_all();
    assert!(t
        .obj_now(mutant)
        .chars
        .has_keyword(KeywordKind::Indestructible));
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(mutant));
    // The next end step: no trigger, the token survives without indestructible.
    t.advance_to(P1, Step::End);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(mutant));
    assert!(!t
        .obj_now(mutant)
        .chars
        .has_keyword(KeywordKind::Indestructible));
}

#[test]
fn chatterfang_defending_player_controls_the_attacked_planeswalker() {
    cr!("506.2", "702.14c");
    ruling!(
        "Chatterfang, Squirrel General",
        "In a Commander game, the defending player is the player Chatterfang is attacking or the controller of the planeswalker Chatterfang is attacking."
    );
    supported("Chatterfang, Squirrel General");
    for (attacked_pw_of, forest) in [(P1, true), (P2, false)] {
        let mut t = TestGame::new(3);
        let fang = t.battlefield(P0, "Chatterfang, Squirrel General");
        t.battlefield(P1, "Forest");
        let pw1 = t.battlefield(P1, "Huatli, Dinosaur Knight");
        let pw2 = t.battlefield(P2, "Huatli, Dinosaur Knight");
        let blocker = t.battlefield(attacked_pw_of, "Grizzly Bears");
        let pw = if attacked_pw_of == P1 { pw1 } else { pw2 };
        attack_with(&mut t, &[(fang, Entity::Object(pw))]);
        // Forestwalk looks at the planeswalker's controller only.
        assert_eq!(
            legal_blocks(&mut t, attacked_pw_of, &[(blocker, fang)]),
            !forest,
            "{attacked_pw_of:?}"
        );
    }
}

#[test]
fn chatterfang_squirrels_share_the_creation_instructions_but_not_the_abilities() {
    cr!("614.1a", "111.1", "508.4");
    ruling!(
        "Chatterfang, Squirrel General",
        "The additional Squirrel tokens won't have any abilities the other tokens were created with. Anything else specified in the effect creating the token (such as tapped, attacking, \"That token gains haste,\" or \"Exile that token at end of combat\") applies to both the original tokens and the Squirrels."
    );
    supported("Hanweir Garrison");
    // Hanweir Garrison: two tapped and attacking Humans, plus two Squirrels the same way.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    let garrison = t.battlefield(P0, "Hanweir Garrison");
    attack_with(&mut t, &[(garrison, Entity::Player(P1))]);
    t.resolve_all();
    let squirrels = with_subtype(&t, P0, "Squirrel")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect::<Vec<_>>();
    let humans = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.obj_now(*id).chars.has_subtype("Human"))
        .collect::<Vec<_>>();
    assert_eq!(squirrels.len(), 2);
    assert_eq!(humans.len(), 2);
    for id in squirrels.iter().chain(&humans) {
        assert!(t.obj_now(*id).tapped);
        assert!(attacking(&t, *id));
    }
    // Geist of Saint Traft: "Exile that token at end of combat" exiles the Squirrel too.
    supported("Geist of Saint Traft");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    t.resolve_all();
    let squirrel = with_subtype(&t, P0, "Squirrel")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect::<Vec<_>>();
    let angel = with_subtype(&t, P0, "Angel");
    assert_eq!((squirrel.len(), angel.len()), (1, 1));
    assert!(t.obj_now(squirrel[0]).tapped && attacking(&t, squirrel[0]));
    t.advance_to_step(Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(angel[0]));
    assert!(!t.on_battlefield(squirrel[0]));
    // A Clue: the Squirrel doesn't get the Clue's ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    let inv = t.battlefield(P0, "Ongoing Investigation");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.activate(P0, inv, 0, &[]).unwrap();
    t.resolve_all();
    let clue = with_subtype(&t, P0, "Clue");
    let squirrel = with_subtype(&t, P0, "Squirrel")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect::<Vec<_>>();
    assert_eq!(clue.len(), 1);
    assert_eq!(squirrel.len(), 1);
    assert!(!t.obj_now(clue[0]).chars.abilities.is_empty());
    assert!(t.obj_now(squirrel[0]).chars.abilities.is_empty());
    assert_eq!(t.pt(squirrel[0]), (1, 1));
}

#[test]
fn chatterfang_applies_to_tokens_another_players_effect_creates_for_you() {
    cr!("614.1a", "111.2");
    ruling!(
        "Chatterfang, Squirrel General",
        "You don't need to control the spell or ability that creates the tokens, nor do you have to be the one creating the tokens for Chatterfang's ability to apply. As long as the tokens are being created under your control, Chatterang's replacement effect will apply."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chatterfang, Squirrel General");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1's Beast Within destroys P0's Bears; P0 creates the Beast.
    cast_new(&mut t, P1, "Beast Within", &[bears.into()]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Beast").len(), 1);
    let squirrels = with_subtype(&t, P0, "Squirrel")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .count();
    assert_eq!(squirrels, 1);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn ongoing_investigation_triggers_per_opponent_in_two_headed_giant() {
    cr!("805.10b", "603.2c");
    ruling!(
        "Ongoing Investigation",
        "In a Two-Headed Giant game, if you control more than one creature that can attack, you may have them attack different opponents so that Ongoing Investigation's first ability triggers twice."
    );
    supported("Ongoing Investigation");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Ongoing Investigation");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.attack(&[(a, Entity::Player(P2)), (b, Entity::Player(P3))], &[]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 2);
    // Both at one opponent: once.
    let mut t = two_headed_giant();
    t.battlefield(P0, "Ongoing Investigation");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.attack(&[(a, Entity::Player(P2)), (b, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn toxrill_cares_about_any_slime_counter_and_any_death() {
    cr!("700.4", "122.1");
    ruling!(
        "Toxrill, the Corrosive",
        "Toxrill's third ability triggers when a creature an opponent controls with a slime counter on it dies for any reason, not just due to its toughness being decreased by Toxrill's second ability."
    );
    ruling!(
        "Toxrill, the Corrosive",
        "second and third abilities apply to all creatures you don't control with slime counters, even if those slime counters came from a source other than Toxrill's first ability."
    );
    supported("Toxrill, the Corrosive");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toxrill, the Corrosive");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // A slime counter from another source.
    add(&mut t, wurm, "slime", 1);
    t.g.recompute();
    assert_eq!(t.pt(wurm), (5, 3));
    // Destroyed (not toughness 0): a Slug.
    destroy(&mut t, wurm);
    t.resolve_all();
    let slugs = with_subtype(&t, P0, "Slug")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .count();
    assert_eq!(slugs, 1);
}

/// P0's Hunter's Talent at level 3 (and the given creatures).
fn talent_at_level_3(creatures: &[&str]) -> (TestGame, Vec<ObjectId>) {
    supported("Hunter's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Hunter's Talent");
    t.lands(P0, "Forest", 6);
    gain_level(&mut t, P0, talent, 2).unwrap();
    t.resolve_all();
    gain_level(&mut t, P0, talent, 3).unwrap();
    t.resolve_all();
    assert_eq!(level(&t, talent), 3);
    let ids = creatures.iter().map(|c| t.battlefield(P0, c)).collect();
    (t, ids)
}

/// Moves from `p`'s second main phase into the end step, stopping as soon as an ability
/// that triggered there is on the stack (or when `p` gets priority in the end step).
fn into_end_step(t: &mut TestGame) {
    t.set_step(P0, Step::PostcombatMain);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::End
            && (!g.stack.is_empty()
                || (g.turn.stage == mtg_engine::turn::Stage::Priority
                    && g.turn.priority == Some(P0)))
    });
    assert!(ok);
    t.settle();
}

#[test]
fn hunters_talent_level_three_triggers_once_if_you_control_a_big_creature() {
    cr!("603.4", "716.2a");
    ruling!(
        "Hunter's Talent",
        "Hunter's Talent's level 3 class ability will trigger only once during your end step, no matter how many creatures you control with power 4 or greater. However, if you don't control a creature with power 4 or greater as your end step begins, the ability won't trigger at all."
    );
    // Two big creatures: one trigger, one card.
    let (mut t, _) = talent_at_level_3(&["Craw Wurm", "Colossal Dreadmaw"]);
    let hand = t.hand_size(P0);
    into_end_step(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // None: no trigger.
    let (mut t, _) = talent_at_level_3(&["Grizzly Bears"]);
    let hand = t.hand_size(P0);
    into_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn hunters_talent_level_three_checks_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Hunter's Talent",
        "If you don't control any creatures with power 4 or greater when the ability resolves, the ability won't do anything."
    );
    let (mut t, ids) = talent_at_level_3(&["Craw Wurm"]);
    let hand = t.hand_size(P0);
    into_end_step(&mut t);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ids[0]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn geistblast_copy_isnt_cast_and_resolves_first() {
    cr!("707.10", "707.10c");
    ruling!(
        "Geistblast",
        "When Geistblast's ability resolves, it creates a copy of the instant or sorcery spell. The copy is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Geistblast");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let geist = t.graveyard(P0, "Geistblast");
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve(); // Young Pyromancer's trigger
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, geist, 0, &[bolt.into()]).unwrap();
    assert!(t.in_exile("Geistblast"));
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The copy resolved first; the original is still on the stack.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.stack[0], bolt);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 1);
}

/// P0's Alesha attacks P1 with two small creature cards in P0's graveyard; P0 pays.
#[test]
fn alesha_returns_one_creature_attacking_what_you_choose() {
    cr!("508.4", "506.3b");
    ruling!(
        "Alesha, Who Smiles at Death",
        "While resolving Alesha's triggered ability, you can't pay the cost multiple times to return more than one creature card."
    );
    ruling!(
        "Alesha, Who Smiles at Death",
        "You choose which player or planeswalker the returned creature is attacking. It doesn't have to be attacking the same player or planeswalker Alesha is attacking."
    );
    supported("Alesha, Who Smiles at Death");
    let mut t = TestGame::new(2);
    let alesha = t.battlefield(P0, "Alesha, Who Smiles at Death");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Gray Ogre");
    let pw = t.battlefield(P1, "Huatli, Dinosaur Knight");
    t.lands(P0, "Plains", 6);
    t.answer_targets(P0, &[bears.into()]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(pw)]);
    attack_with(&mut t, &[(alesha, Entity::Player(P1))]);
    t.resolve_all();
    let back = t.g.current(bears);
    assert!(t.on_battlefield(back));
    assert!(t.obj_now(back).tapped);
    assert_eq!(attack_target(&t, back), Some(Entity::Object(pw)));
    assert_eq!(attack_target(&t, alesha), Some(Entity::Player(P1)));
    // Only one card returned, though mana is left.
    assert!(t.in_graveyard(P0, "Gray Ogre"));
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn rodolf_counts_life_gained_regardless_of_life_lost() {
    cr!("119.3", "603.12");
    ruling!(
        "Rodolf Duskbringer",
        "X is the total amount of life you gained this turn, regardless of any life lost. For example, if you gained 3 life this turn and also lost 2 life this turn, X is 3, not 1."
    );
    supported("Rodolf Duskbringer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rodolf Duskbringer");
    let ogre = t.graveyard(P0, "Gray Ogre");
    cast_new(&mut t, P0, "Healing Salve", &[Entity::Player(P0)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Sign in Blood", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    t.lands(P0, "Swamp", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[ogre.into()]);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(ogre));
}

#[test]
fn deathbellow_raider_doesnt_attack_if_unable_or_if_attacking_costs() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Deathbellow Raider",
        "If, during your declare attackers step, Deathbellow Raider is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under your control continuously since the turn began (and doesn't have haste), then it doesn't attack. If there's a cost associated with having a creature attack, you're not forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Deathbellow Raider");
    // Untapped and able: it must attack.
    let mut t = TestGame::new(2);
    let raider = t.battlefield(P0, "Deathbellow Raider");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    // Tapped / summoning sick / Pacifism / Ghostly Prison: no attack is fine.
    for case in 0..4 {
        let mut t = TestGame::new(2);
        let raider2 = if case == 1 {
            t.battlefield_sick(P0, "Deathbellow Raider")
        } else {
            t.battlefield(P0, "Deathbellow Raider")
        };
        match case {
            0 => t.g.objects[raider2.0 as usize].tapped = true,
            2 => {
                crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", raider2);
            }
            3 => {
                t.battlefield(P1, "Ghostly Prison");
            }
            _ => {}
        }
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(legal_attack(&mut t, &[]), "case {case}");
    }
    let _ = raider;
}

#[test]
fn deathbellow_raider_chooses_what_it_attacks() {
    cr!("508.1b", "508.1d");
    ruling!(
        "Deathbellow Raider",
        "You still choose which player or planeswalker Deathbellow Raider attacks."
    );
    let mut t = TestGame::new(2);
    let raider = t.battlefield(P0, "Deathbellow Raider");
    let pw = t.battlefield(P1, "Huatli, Dinosaur Knight");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(raider, Entity::Object(pw))]));
    assert!(legal_attack(&mut t, &[(raider, Entity::Player(P1))]));
    attack_with(&mut t, &[(raider, Entity::Object(pw))]);
    assert_eq!(attack_target(&t, raider), Some(Entity::Object(pw)));
}

#[test]
fn ogre_savant_returns_itself_if_its_the_only_creature() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Ogre Savant",
        "If {U} was spent to cast Ogre Savant and no other creatures are on the battlefield, the Ogre will have to return itself."
    );
    supported("Ogre Savant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 4);
    let card = t.hand(P0, "Ogre Savant");
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Ogre Savant"));
    assert!(t.named_on_battlefield("Ogre Savant").is_empty());
}

#[test]
fn bartz_and_boko_targets_one_creature_for_all_birds() {
    cr!("115.1", "601.2c");
    ruling!(
        "Bartz and Boko",
        "Bartz and Boko's last ability targets only one creature, regardless of how many Birds you control."
    );
    supported("Bartz and Boko");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Storm Crow");
    t.battlefield(P0, "Storm Crow");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[giant.into()]);
    let bb = t.enter(P0, "Bartz and Boko");
    t.g.flush_events();
    t.settle();
    let trig = t.g.stack[0];
    assert_eq!(targets_of(&t, trig), vec![Entity::Object(giant)]);
    let slots = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(slots, 1);
    t.resolve_all();
    // Two Storm Crows (1/2 each) deal 1 each.
    assert_eq!(crate::r_s07_common::damage_on(&t, giant), 2);
    assert_eq!(crate::r_s07_common::damage_on(&t, bb), 0);
}

#[test]
fn electropotence_creature_deals_the_damage() {
    cr!("702.16b", "702.16e", "702.15b", "702.2b", "608.2h");
    ruling!(
        "Electropotence",
        "Electropotence is the source of the ability, but the creature is the source of the damage. The ability couldn't target a creature with protection from red, for example. It could target a creature with protection from creatures, but all the damage would be prevented. Since damage is dealt by the creature, lifelink, deathtouch, and wither are taken into account, even if the creature has left the battlefield by the time it deals damage."
    );
    supported("Electropotence");
    // Protection from red: not a legal target.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Electropotence");
    let priest = t.battlefield(P1, "Soltari Priest");
    t.lands(P0, "Mountain", 3);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    assert!(offered
        .iter()
        .all(|c| !c.contains(&Entity::Object(priest))));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Protection from creatures: targetable, the damage is prevented.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Electropotence");
    let chaplain = t.battlefield(P1, "Beloved Chaplain");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[chaplain.into()]);
    t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(chaplain));
    assert_eq!(crate::r_s07_common::damage_on(&t, chaplain), 0);
    // Lifelink and deathtouch count, even after the creature has left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Electropotence");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[maw.into()]);
    let hawk = t.enter(P0, "Vampire Nighthawk");
    t.g.flush_events();
    t.settle();
    destroy(&mut t, hawk);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Colossal Dreadmaw"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn tazri_beacon_of_unity_up_to_two_cards_total() {
    cr!("608.2d");
    ruling!(
        "Tazri, Beacon of Unity",
        "Tazri's activated ability lets you put up to two cards total into your hand, not two cards of each of the listed creature types."
    );
    supported("Tazri, Beacon of Unity");
    let mut t = TestGame::new(2);
    let tazri = t.battlefield(P0, "Tazri, Beacon of Unity");
    let top = stack_library(
        &mut t,
        P0,
        &[
            "Alesha, Who Smiles at Death",
            "Keldon Strike Team",
            "Najeela, the Blade-Blossom",
            "Kels, Fight Fixer",
            "Phyrexian Infiltrator",
            "Ogre Savant",
        ],
    );
    for l in ["Island", "Swamp", "Mountain", "Forest"] {
        t.lands(P0, l, 1);
    }
    let from = t.asked().len();
    t.answer_choose(P0, &[top[0].into(), top[1].into(), top[5].into()]);
    t.activate(P0, tazri, 0, &[]).unwrap();
    t.resolve_all();
    let maxes: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![2]);
    // Warriors and a Wizard among the six; at most two go to hand.
    let hand: Vec<_> = t.g.player(P0).hand.clone();
    assert!(hand.len() <= 2);
}

#[test]
fn tazri_beacon_of_unity_chooses_a_cost_for_each_symbol() {
    cr!("601.2f", "107.4e");
    ruling!(
        "Tazri, Beacon of Unity",
        "To determine the cost of Tazri's activated ability, you choose which cost you'll pay for each of the four mana symbols (either {2} or one mana of the appropriate color), add up the total cost, apply any additional costs, then apply any cost reductions."
    );
    // {U} for one symbol and {2} for each of the other three.
    let mut t = TestGame::new(2);
    let tazri = t.battlefield(P0, "Tazri, Beacon of Unity");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 6);
    t.activate(P0, tazri, 0, &[]).unwrap();
    assert_eq!(tapped_lands(&t, P0), 7);
    // Five Wastes can't pay it.
    let mut t = TestGame::new(2);
    let tazri = t.battlefield(P0, "Tazri, Beacon of Unity");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 5);
    assert!(t.activate(P0, tazri, 0, &[]).is_err());
}

