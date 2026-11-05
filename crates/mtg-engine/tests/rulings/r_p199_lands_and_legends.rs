//! Rulings batch P199 — legendary lands and other cards with unusual timing: Vesuva and The
//! Mycosynth Gardens copying copies (CR 707.2–707.4), Valakut's intervening "if" (CR
//! 603.4), Reflecting Pool's mana (CR 106.7), Barl's Cage and Magosi's skipped steps and
//! turns, Library of Leng's discard replacement (CR 614), Adagia's legendary token copies
//! (CR 707.9b), Peter Parker's Camera copying linked abilities (CR 607.2), Sindbad's
//! replaced draw (CR 614.6), and Drannith Magistrate's cast restriction.

use crate::r_p079_common::back_face;
use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, create_token};
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{activate_containing, attached_to, give_control};
use crate::r_s20_common::tap_for_mana;
use crate::r_s31_common::put_together;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether `p` could play the card `id` (followed across zone changes) now.
fn playable(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let c = t.g.current(id);
    t.g.legal_actions(p).iter().any(|a| match a {
        Action::PlayLand { card } => *card == c,
        Action::Cast { card, .. } => *card == c,
        _ => false,
    })
}

#[test]
fn nivix_permission_ends_as_your_next_turn_starts() {
    cr!("611.2b", "500.7");
    ruling!(
        "Nivix, Aerie of the Firemind",
        "\"Until your next turn\" means the effect wears off as soon as your next turn starts, even before you untap."
    );
    let mut t = TestGame::new(2);
    let nivix = t.battlefield(P0, "Nivix, Aerie of the Firemind");
    let bolt = t.library_top(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, nivix, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.lands(P0, "Mountain", 1);
    // During the opponent's turn: castable.
    t.advance_to(P1, Step::Upkeep);
    assert!(playable(&mut t, P0, bolt));
    // P0's next turn: not even in its upkeep.
    t.advance_to(P0, Step::Upkeep);
    assert!(!playable(&mut t, P0, bolt));
    assert_eq!(t.zone(bolt), Zone::Exile);
}

#[test]
fn voldaren_estate_mana_casts_spells_with_the_vampire_type() {
    cr!("106.6", "205.3m");
    ruling!(
        "Voldaren Estate",
        "A Vampire spell is a spell with the creature type Vampire on its type line, regardless of whether the word \"Vampire\" is in its name."
    );
    supported("Knight of the Ebon Legion");
    for (name, ok) in [("Knight of the Ebon Legion", true), ("Vampire's Bite", false)] {
        let mut t = TestGame::new(2);
        let estate = t.battlefield(P0, "Voldaren Estate");
        let target = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Option, Answer::Index(2));
        assert!(tap_for_mana(&mut t, P0, estate, "Pay 1 life"));
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 1);
        let card = t.hand(P0, name);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        let r = t.cast(P0, card).target(target).try_go();
        assert_eq!(r.is_ok(), ok, "{name}");
    }
}

#[test]
fn howlpack_alpha_gives_a_werewolf_wolf_only_one_bonus() {
    cr!("613.4c", "702.73a");
    ruling!(
        "Mayor of Avabruck // Howlpack Alpha",
        "A creature that is both a Werewolf and a Wolf will only get +1/+1 from Howlpack Alpha’s first ability."
    );
    let mut t = TestGame::new(2);
    back_face(&mut t, P0, "Mayor of Avabruck // Howlpack Alpha");
    let changeling = t.battlefield(P0, "Changeling Outcast");
    assert_eq!(t.pt(changeling), (2, 2));
}

#[test]
fn mishras_factory_that_entered_this_turn_cant_attack_or_tap() {
    cr!("302.6", "302.1");
    ruling!(
        "Mishra's Factory",
        "A noncreature permanent that becomes a creature can attack, and its {T} abilities can be activated, only if its controller has continuously controlled that permanent since the beginning of their most recent turn."
    );
    let mut t = TestGame::new(2);
    let f = t.enter(P0, "Mishra's Factory");
    t.settle();
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, f, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(f).is(CardType::Creature));
    assert!(t.activate(P0, f, 2, &[obj(f)]).is_err());
    assert!(t.activate(P0, f, 0, &[]).is_err());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, f));
    // Controlled since the turn began: it can.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Mishra's Factory");
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, f, 1, &[]).unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, f));
}

#[test]
fn aladdin_takes_one_artifact_per_activation() {
    cr!("602.1", "613.1b");
    ruling!(
        "Aladdin",
        "Aladdin's ability can take control of more than one artifact, although only one each time the ability is used."
    );
    let mut t = TestGame::new(2);
    let aladdin = t.battlefield(P0, "Aladdin");
    let a = t.battlefield(P1, "Mind Stone");
    let b = t.battlefield(P1, "Ornithopter");
    for target in [a, b] {
        mana(&mut t, P0, ManaType::R, 2);
        mana(&mut t, P0, ManaType::C, 1);
        t.activate(P0, aladdin, 0, &[obj(target)]).unwrap();
        t.resolve_all();
        t.g.objects[aladdin.0 as usize].tapped = false;
    }
    assert_eq!(t.obj_now(a).controller, P0);
    assert_eq!(t.obj_now(b).controller, P0);
    // Losing Aladdin ends both.
    move_to(&mut t, aladdin, Zone::Graveyard(P0));
    assert_eq!(t.obj_now(a).controller, P1);
    assert_eq!(t.obj_now(b).controller, P1);
}

/// The mana type options offered when P0 taps Reflecting Pool for mana.
fn pool_options(t: &mut TestGame, pool: ObjectId) -> Vec<String> {
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    assert!(tap_for_mana(t, P0, pool, "Add"));
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn reflecting_pool_sees_land_type_changes() {
    cr!("106.7", "305.7");
    ruling!(
        "Reflecting Pool",
        "Any change to a land's type or splicing of text into a land can affect the types of mana a land can produce."
    );
    supported("Spreading Seas");
    let mut t = TestGame::new(2);
    let pool = t.battlefield(P0, "Reflecting Pool");
    let forest = t.battlefield(P0, "Forest");
    let seas = t.battlefield(P1, "Spreading Seas");
    assert!(t.g.attach(seas, obj(forest)));
    t.g.recompute();
    t.g.players[P0.idx()].mana_pool = Default::default();
    assert!(tap_for_mana(&mut t, P0, pool, "Add"));
    let p = &t.g.player(P0).mana_pool;
    assert_eq!((p.count(ManaType::U), p.count(ManaType::G)), (1, 0));
}

#[test]
fn reflecting_pool_considers_replacement_effects() {
    cr!("106.7");
    ruling!(
        "Reflecting Pool",
        "Any replacement effects are considered by Reflecting Pool when determining the types of mana a land can produce."
    );
    // Under Ritual of Subdual, a Forest could produce only colorless mana: so can the
    // Pool (its mana is colorless either way, and no color is offered).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ritual of Subdual");
    let pool = t.battlefield(P0, "Reflecting Pool");
    t.battlefield(P0, "Forest");
    let opts = pool_options(&mut t, pool);
    assert!(opts.iter().all(|o| !o.contains('G')), "{opts:?}");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
}

#[test]
fn vesuva_copying_a_land_gets_its_enters_abilities() {
    cr!("707.2", "614.1c", "603.6a");
    ruling!(
        "Vesuva",
        "Any enters-the-battlefield abilities of the copied land will trigger if Vesuva is entering the battlefield. Any \"As [this land] enters the battlefield\" or \"[This land] enters the battlefield with\" abilities of the copied land will also work."
    );
    let mut t = TestGame::new(2);
    let garden = t.battlefield(P1, "Khalni Garden");
    t.answer_choose(P0, &[obj(garden)]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(v).chars.name, "Khalni Garden");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P1, "Gemstone Mine");
    t.answer_choose(P0, &[obj(mine)]);
    let v = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.counters(v, "mining"), 3);
}

#[test]
fn vesuva_cant_copy_a_land_entering_with_it() {
    cr!("614.12", "707.2");
    ruling!(
        "Vesuva",
        "If Vesuva somehow enters the battlefield at the same time as another land, Vesuva can't become a copy of that land. You may choose only a land that's already on the battlefield."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let v = t.hand(P0, "Vesuva");
    let m = t.hand(P0, "Mountain");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(forest)]);
    let ids = put_together(&mut t, &[v, m]);
    let cands = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(!cands.is_empty());
    let mountain = ids[1].unwrap();
    assert!(cands.iter().all(|c| !c.contains(&obj(mountain)) && !c.contains(&obj(m))));
    assert_eq!(t.obj_now(ids[0].unwrap()).chars.name, "Forest");
}

#[test]
fn vesuva_and_mycosynth_gardens_copy_what_a_copy_copies() {
    cr!("707.3", "707.2");
    ruling!(
        "Vesuva",
        "If the copied land is copying something else, Vesuva uses the copiable values of that land. In most cases, Vesuva will be a copy of whatever that land is copying."
    );
    ruling!(
        "The Mycosynth Gardens",
        "If the copied artifact is copying something else, then The Mycosynth Gardens becomes a copy of whatever that artifact copied."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    let a = t.enter(P0, "Vesuva");
    t.settle();
    t.answer_choose(P0, &[obj(a)]);
    let b = t.enter(P0, "Vesuva");
    t.settle();
    assert_eq!(t.obj_now(b).chars.name, "Forest");
    assert!(t.obj_now(b).chars.has_subtype("Forest"));
    // The Mycosynth Gardens copying Sculpting Steel that copies Mind Stone.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    t.answer_choose(P0, &[obj(stone)]);
    let steel = t.enter(P0, "Sculpting Steel");
    t.settle();
    assert_eq!(t.obj_now(steel).chars.name, "Mind Stone");
    let gardens = t.battlefield(P0, "The Mycosynth Gardens");
    mana(&mut t, P0, ManaType::C, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, gardens, "becomes a copy").unwrap();
    t.answer_targets(P0, &[obj(steel)]);
    t.resolve_all();
    assert_eq!(t.obj_now(gardens).chars.name, "Mind Stone");
}

#[test]
fn magosi_skips_your_next_turn_even_without_the_counter() {
    cr!("608.2b", "614.10");
    ruling!(
        "Magosi, the Waterveil",
        "As Magosi’s third ability resolves, you’ll skip your next turn even if you don’t put an eon counter on Magosi (because it’s left the battlefield by then, perhaps)."
    );
    let mut t = TestGame::new(2);
    let magosi = t.battlefield(P0, "Magosi, the Waterveil");
    mana(&mut t, P0, ManaType::U, 1);
    activate_containing(&mut t, P0, magosi, "Skip your next turn").unwrap();
    move_to(&mut t, magosi, Zone::Hand(P0));
    t.resolve_all();
    // Turn 2 is P1's; P0's turn 3 is skipped, so turn 3 is P1's again.
    t.advance_to(P1, Step::Upkeep);
    let n = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| g.turn.number == n + 1);
    assert!(ok);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn barls_cage_follows_the_creature_not_its_controller() {
    cr!("502.3", "611.2a");
    ruling!(
        "Barl's Cage",
        "Barl’s Cage doesn’t track the creature’s controller. If the affected creature changes controllers before its old controller’s next untap step, this ability will prevent it from being untapped during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let cage = t.battlefield(P0, "Barl's Cage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, cage, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    give_control(&mut t, bears, P0);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn barls_cage_doesnt_stop_other_untapping() {
    cr!("502.3", "701.26b");
    ruling!(
        "Barl's Cage",
        "Does not prevent a creature from untapping at other times. It just stops the “normal” untap during the untap step."
    );
    let mut t = TestGame::new(2);
    let cage = t.battlefield(P0, "Barl's Cage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, cage, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    let twiddle = t.hand(P1, "Twiddle");
    mana(&mut t, P1, ManaType::U, 1);
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.cast(P1, twiddle).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn bloodbat_summoner_bats_are_still_blood_tokens() {
    cr!("613.1d", "111.10g");
    ruling!(
        "Voldaren Bloodcaster // Bloodbat Summoner",
        "Blood tokens that become Bats with Bloodbat Summoner’s ability are still Blood artifact tokens and still have “{1}, {T}, Discard a card, Sacrifice this token: Draw a card.”"
    );
    let mut t = TestGame::new(2);
    back_face(&mut t, P0, "Voldaren Bloodcaster // Bloodbat Summoner");
    let blood = create_token(&mut t, P0, "Blood");
    t.answer_targets(P0, &[obj(blood)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let o = t.obj_now(blood);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Bat") && o.chars.has_subtype("Blood"));
    assert_eq!((o.power(), o.toughness()), (2, 2));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Draw a card")));
}

#[test]
fn deaths_shadow_is_13_13_off_the_battlefield() {
    cr!("611.3a", "208.1");
    ruling!(
        "Death's Shadow",
        "Death's Shadow's ability applies only while Death's Shadow is on the battlefield. In all other zones, its power and toughness are 13."
    );
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Death's Shadow");
    let gy = t.graveyard(P0, "Death's Shadow");
    assert_eq!(t.pt(hand), (13, 13));
    assert_eq!(t.pt(gy), (13, 13));
    t.g.players[P0.idx()].life = 5;
    let bf = t.battlefield(P0, "Death's Shadow");
    assert_eq!(t.pt(bf), (8, 8));
    assert_eq!(t.pt(hand), (13, 13));
}

#[test]
fn south_wind_avatar_triggers_once_per_lifelink_creature() {
    cr!("702.15b", "120.3f", "603.2c");
    ruling!(
        "South Wind Avatar",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    supported("Vampire Nighthawk");
    // Two lifelink attackers unblocked: two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "South Wind Avatar");
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    // 4 combat damage plus 1 per trigger.
    assert_eq!(t.life(P1), 14);
    // One lifelink creature dealing damage to two blockers at once: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "South Wind Avatar");
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let x = t.battlefield(P1, "Ornithopter");
    let y = t.battlefield(P1, "Ornithopter");
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 1]));
    t.attack(&[(hawk, Entity::Player(P1))], &[(x, hawk), (y, hawk)]);
    assert!(!t.on_battlefield(x) && !t.on_battlefield(y));
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn drannith_magistrate_doesnt_stop_other_zones_otherwise() {
    cr!("101.2", "305.1", "602.1");
    ruling!(
        "Drannith Magistrate",
        "Effects may put cards from other zones onto the battlefield under an opponent's control."
    );
    ruling!(
        "Drannith Magistrate",
        "If a rule or effect allows them to do so, your opponents may play lands from zones other than their hand and activate abilities of cards in zones other than their hand."
    );
    supported("Dregscape Zombie");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drannith Magistrate");
    t.set_step(P1, Step::PrecombatMain);
    // Zombify returns a creature card from P1's graveyard.
    let giant = t.graveyard(P1, "Hill Giant");
    let z = t.hand(P1, "Zombify");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 3);
    t.cast(P1, z).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).controller, P1);
    // Unearth from the graveyard.
    let zombie = t.graveyard(P1, "Dregscape Zombie");
    mana(&mut t, P1, ManaType::B, 1);
    t.g.turn.priority = Some(P1);
    let unearth = t
        .g
        .legal_actions(P1)
        .into_iter()
        .find(|a| matches!(a, Action::Activate { source, .. } if *source == zombie));
    assert!(unearth.is_some());
    // A land from the graveyard with Crucible of Worlds.
    t.battlefield(P1, "Crucible of Worlds");
    let land = t.graveyard(P1, "Forest");
    t.g.turn.priority = Some(P1);
    assert!(t.play_land(P1, land).is_ok());
}

#[test]
fn emeria_checks_its_plains_as_it_triggers_and_resolves() {
    cr!("603.4");
    ruling!(
        "Emeria, the Sky Ruin",
        "Emeria, the Sky Ruin's triggered ability has an \"intervening 'if' clause.\""
    );
    // Six Plains: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emeria, the Sky Ruin");
    t.lands(P0, "Plains", 6);
    t.graveyard(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Seven, one gone before it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emeria, the Sky Ruin");
    let plains = t.lands(P0, "Plains", 7);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, plains[0], Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    // Seven throughout: returned.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emeria, the Sky Ruin");
    t.lands(P0, "Plains", 7);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn huntmaster_liger_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "702.140d");
    ruling!(
        "Huntmaster Liger",
        "Huntmaster Liger’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +X/+X."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let liger = t.hand(P0, "Huntmaster Liger");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, liger)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 4));
    let later = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
}

/// Adagia, Windswept Bastion with twelve charge counters.
fn adagia(t: &mut TestGame) -> ObjectId {
    let a = t.battlefield(P0, "Adagia, Windswept Bastion");
    t.g.add_counters(obj(a), "charge", 12, None);
    mana(t, P0, ManaType::W, 1);
    mana(t, P0, ManaType::C, 3);
    a
}

fn adagia_copy(t: &mut TestGame, a: ObjectId, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, a, "copy of target").unwrap();
    t.resolve_all();
}

#[test]
fn adagia_aura_copy_chooses_what_it_enchants_without_targeting() {
    cr!("303.4f", "702.11b", "702.16b");
    ruling!(
        "Adagia, Windswept Bastion",
        "If Adagia creates a token copy of an Aura, you choose what the token is enchanting as it enters the battlefield. This doesn’t target the player or permanent it will enchant, so a player or permanent with hexproof may be chosen this way. The chosen recipient must be able to legally be enchanted by the Aura, so a player or permanent with protection from one of the Aura’s qualities can’t be chosen this way. If there’s nothing legal for the Aura token to enchant, the token is never created."
    );
    supported("Gladecover Scout");
    supported("Mistcutter Hydra");
    // Anti-Magic Aura (blue): a hexproof creature can be chosen, a creature with protection
    // from blue can't, nor the creature already enchanted (no "other Auras").
    let mut t = TestGame::new(2);
    let a = adagia(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Anti-Magic Aura");
    assert!(t.g.attach(aura, obj(bears)));
    let scout = t.battlefield(P1, "Gladecover Scout");
    let hydra = t.battlefield(P1, "Mistcutter Hydra");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(scout)]);
    adagia_copy(&mut t, a, aura);
    let cands = crate::r_s03_common::choice_candidates(&t, from, "");
    assert!(cands.iter().any(|c| c.contains(&obj(scout))), "{cands:?}");
    assert!(cands.iter().all(|c| !c.contains(&obj(hydra)) && !c.contains(&obj(bears))));
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(attached_to(&t, toks[0]), Some(obj(scout)));
    // Nothing else to enchant: no token.
    let mut t = TestGame::new(2);
    let a = adagia(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Anti-Magic Aura");
    assert!(t.g.attach(aura, obj(bears)));
    t.battlefield(P1, "Mistcutter Hydra");
    adagia_copy(&mut t, a, aura);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn adagia_copy_of_a_nonlegendary_permanent_is_the_only_legendary_one() {
    cr!("707.9b", "704.5j");
    ruling!(
        "Adagia, Windswept Bastion",
        "If Adagia’s last ability creates a token copy of a nonlegendary permanent, you’ll control two permanents with the same name, but only one of them (the newly created token) will be legendary. You won’t put any of them into their owners’ graveyards."
    );
    let mut t = TestGame::new(2);
    let a = adagia(&mut t);
    let stone = t.battlefield(P0, "Mind Stone");
    adagia_copy(&mut t, a, stone);
    let stones = t.named_on_battlefield("Mind Stone");
    assert_eq!(stones.len(), 2);
    let legendary: Vec<_> = stones
        .iter()
        .map(|s| t.g.obj(*s).chars.supertypes.contains(Supertype::Legendary))
        .collect();
    assert_eq!(legendary.iter().filter(|l| **l).count(), 1);
    assert!(!t.g.obj(stone).chars.supertypes.contains(Supertype::Legendary));
}

#[test]
fn adagia_copy_of_a_token_uses_its_original_characteristics() {
    cr!("707.2", "111.3", "707.9b");
    ruling!(
        "Adagia, Windswept Bastion",
        "If the copied permanent is a token, the token that’s created copies the original characteristics of that token as stated by the effect that created the token, with the listed exception."
    );
    // A Blood token made a 2/2 Bat creature by Bloodbat Summoner: the copy is a plain
    // (legendary) Blood token.
    let mut t = TestGame::new(2);
    back_face(&mut t, P0, "Voldaren Bloodcaster // Bloodbat Summoner");
    let blood = create_token(&mut t, P0, "Blood");
    t.answer_targets(P0, &[obj(blood)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(blood).is(CardType::Creature));
    t.g.combat = None;
    t.set_step(P0, Step::PostcombatMain);
    let a = adagia(&mut t);
    adagia_copy(&mut t, a, blood);
    let copy = tokens(&t, P0)
        .into_iter()
        .find(|id| *id != t.g.current(blood))
        .expect("copy");
    let o = t.g.obj(copy);
    assert!(o.chars.has_subtype("Blood") && !o.chars.has_subtype("Bat"));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
}

#[test]
fn valakut_counts_other_mountains_as_it_resolves() {
    cr!("603.4");
    ruling!(
        "Valakut, the Molten Pinnacle",
        "If a Mountain you control leaves the battlefield between the time Valakut's second ability triggers and the time it resolves, be aware if that was the Mountain that caused Valakut's ability to trigger or not. If it was, Valakut's count isn't affected; if it wasn't, Valakut's count goes down by one."
    );
    for (remove_new, damage) in [(true, 3), (false, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Valakut, the Molten Pinnacle");
        let old = t.lands(P0, "Mountain", 5);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let new = t.enter(P0, "Mountain");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        let gone = if remove_new { new } else { old[0] };
        move_to(&mut t, gone, Zone::Hand(P0));
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - damage, "{remove_new}");
    }
}

#[test]
fn valakut_counts_mountains_entering_together() {
    cr!("603.6a", "603.4");
    ruling!(
        "Valakut, the Molten Pinnacle",
        "If multiple Mountains enter under your control at the same time, Valakut's second ability could trigger that many times. Each ability takes into consideration the other Mountains that entered at the same time as the one that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valakut, the Molten Pinnacle");
    t.lands(P0, "Mountain", 4);
    let a = t.hand(P0, "Mountain");
    let b = t.hand(P0, "Mountain");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    put_together(&mut t, &[a, b]);
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn peter_parkers_camera_copy_of_a_linked_ability_stays_linked() {
    cr!("607.2a", "707.10", "607.3");
    ruling!(
        "Peter Parker's Camera",
        "If an ability is linked to a second ability, copies of that ability are also linked to that second ability."
    );
    let mut t = TestGame::new(2);
    let camera = t.battlefield(P0, "Peter Parker's Camera");
    t.g.add_counters(obj(camera), "film", 3, None);
    let a = t.battlefield(P1, "Mind Stone");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.graveyard(P1, "Hill Giant");
    let d = t.graveyard(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(c)]);
    let spot = t.enter(P0, "The Spot, Living Portal");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trigger = top_of_stack(&t);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, camera, 0, &[obj(trigger)]).unwrap();
    // The copy: new targets.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.answer_targets(P0, &[obj(d)]);
    t.resolve_all();
    for x in [a, b, c, d] {
        assert_eq!(t.zone(x), Zone::Exile);
    }
    t.answer_yes(P0, true);
    move_to(&mut t, spot, Zone::Graveyard(P0));
    t.resolve_all();
    for x in [a, b, c, d] {
        assert_eq!(t.zone(x), Zone::Hand(P1), "{x:?}");
    }
}

#[test]
fn library_of_leng_decides_for_each_discarded_card() {
    cr!("614.1a", "616.1");
    ruling!(
        "Library of Leng",
        "If more than one card is discarded due to a single effect, the Library allows you to decide whether or not to use it on each of the cards. You get to decide the order the cards are placed on the library if more than one goes there."
    );
    supported("Mind Rot");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    let x = t.hand(P0, "Grizzly Bears");
    let y = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P1, rot).target(Entity::Player(P0)).go();
    t.resolve_all();
    let on_top = [x, y]
        .iter()
        .filter(|c| matches!(t.zone(**c), Zone::Library(_)))
        .count();
    let in_gy = [x, y]
        .iter()
        .filter(|c| matches!(t.zone(**c), Zone::Graveyard(_)))
        .count();
    assert_eq!((on_top, in_gy), (1, 1));
    // Both on top: the top card is one of them.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    let x = t.hand(P0, "Grizzly Bears");
    let y = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P1, rot).target(Entity::Player(P0)).go();
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    let top2: Vec<ObjectId> = lib[lib.len() - 2..].to_vec();
    assert!(top2.contains(&t.g.current(x)) && top2.contains(&t.g.current(y)));
}

/// P0's hand size after their cleanup step with seven cards in hand and `first` then
/// `second` entering the battlefield in that order.
fn hand_after_cleanup(first: &str, second: &str) -> usize {
    let mut t = TestGame::new(2);
    t.enter(P0, first);
    t.settle();
    t.enter(P0, second);
    t.settle();
    for _ in 0..7 {
        t.hand(P0, "Grizzly Bears");
    }
    t.advance_to(P1, Step::Upkeep);
    t.hand_size(P0)
}

#[test]
fn library_of_leng_and_null_profusion_apply_in_timestamp_order() {
    cr!("402.2", "613.7", "514.1");
    ruling!(
        "Library of Leng",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    assert_eq!(hand_after_cleanup("Null Profusion", "Library of Leng"), 7);
    assert_eq!(hand_after_cleanup("Library of Leng", "Null Profusion"), 2);
}

#[test]
fn sindbad_does_nothing_more_when_its_draw_is_replaced() {
    cr!("614.6", "702.52a");
    ruling!(
        "Sindbad",
        "If the draw is replaced by another effect, none of the rest of Sindbad’s ability applies, even if the draw is replaced by another draw (such as with Enduring Renewal)."
    );
    let mut t = TestGame::new(2);
    let sindbad = t.battlefield(P0, "Sindbad");
    let imp = t.graveyard(P0, "Stinkweed Imp");
    t.answer_yes(P0, true);
    t.activate(P0, sindbad, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(imp), Zone::Hand(P0));
    // Nothing discarded (the five milled cards aside).
    assert!(t.in_hand(P0, "Stinkweed Imp"));
    assert!(!t.in_graveyard(P0, "Stinkweed Imp"));
    // A normal draw of a nonland card: discarded.
    let mut t = TestGame::new(2);
    let sindbad = t.battlefield(P0, "Sindbad");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, sindbad, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
}
