//! Rulings batch P087 — the rest: hate for targeting (Bonecrusher Giant and Stomp, Forsaken
//! Wastes), theft (Tel-Jilad Stylus and ownership), tokens, tutors and shuffling
//! (Stranglehold, Ashiok, Cosi's Trickster), creature types (Halo Hunter, Witch's
//! Vengeance, Eye Gouge, Sigardian Priest, Deathcult Rogue, Goblin Masons, Undead Slayer),
//! colors (Glistening Deluge, Ember Gale), Stormbreath Dragon, Tourach and madness,
//! Blasphemous Act's cost, Leonin Arbiter and Frenzied Baloth.

use crate::r_p087_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const GIANT: &str = "Bonecrusher Giant // Stomp";
const ADVENTURE: mtg_engine::object::CastMethod = mtg_engine::object::CastMethod::Half(1);

fn set_life(t: &mut TestGame, p: PlayerId, n: i32) {
    t.g.players[p.idx()].life = n;
}

/// P0 casts Stomp (paid with {1}{R}) at `target`; it stays on the stack.
fn stomp(t: &mut TestGame, target: Entity) -> ObjectId {
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, GIANT);
    t.cast(P0, card).method(ADVENTURE).target(target).go()
}

// --- Bonecrusher Giant and Stomp ------------------------------------------------------------

#[test]
fn bonecrusher_giant_triggers_once_for_a_spell_targeting_it_several_times() {
    cr!("603.2c", "115.3");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "If a spell targets Bonecrusher Giant more than once, its ability triggers only once."
    );
    supported(GIANT);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, GIANT);
    // Seeds of Strength: "Target creature gets +1/+1 until end of turn." three times.
    cast_new(&mut t, P1, "Seeds of Strength", &[obj(giant), obj(giant), obj(giant)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(giant), (7, 6));
}

#[test]
fn stomp_with_an_illegal_target_leaves_damage_preventable() {
    cr!("608.2b", "702.16e");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "If the chosen target is an illegal target by the time Stomp tries to resolve, the spell won't resolve. Damage can be prevented as usual."
    );
    let mut t = TestGame::new(2);
    let red = t.battlefield(P0, "Raging Goblin");
    let knight = t.battlefield(P1, "Silver Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    stomp(&mut t, obj(bears));
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bonecrusher Giant"));
    // Silver Knight's protection from red still prevents damage.
    t.g.deal_damage(red, obj(knight), 1, false);
    t.g.flush_events();
    assert_eq!(damage_marked(&t, knight), 0);
}

#[test]
fn after_stomp_protection_cant_prevent_damage_but_still_stops_targeting() {
    cr!("702.16b", "702.16e", "615.12");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "Protection prevents damage, so protection will be unable to prevent damage after Stomp has resolved. However, this won't allow a spell or ability to target illegally, even if that spell or ability would cause damage to be dealt."
    );
    let mut t = TestGame::new(2);
    let red = t.battlefield(P0, "Raging Goblin");
    let knight = t.battlefield(P1, "Silver Knight");
    let from = t.asked().len();
    stomp(&mut t, Entity::Player(P1));
    // Stomp itself couldn't target the Knight.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered[0].contains(&obj(knight)) && offered[0].contains(&Entity::Player(P1)));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    t.g.deal_damage(red, obj(knight), 1, false);
    t.g.flush_events();
    assert_eq!(damage_marked(&t, knight), 1);
    // A red spell still can't target it.
    assert!(!spell_targets(&mut t, P0, "Lightning Bolt").contains(&obj(knight)));
}

#[test]
fn stomp_stops_only_effects_that_say_prevent() {
    cr!("615.12", "614.1b", "122.1c");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "Stomp only stops damage from being prevented by effects that specifically use the word \"prevent.\""
    );
    // Palisade Giant ("All damage that would be dealt to you and other permanents you
    // control is dealt to this creature instead.") redirects damage: not a prevention
    // effect, so it still works.
    let mut t = TestGame::new(2);
    let palisade = t.battlefield(P1, "Palisade Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    stomp(&mut t, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_marked(&t, palisade), 2);
    cast_new(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, palisade), 4);
    // Healing Salve ("Prevent the next 3 damage that would be dealt to any target this
    // turn") does use the word: the damage is dealt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stomp(&mut t, Entity::Player(P1));
    t.resolve_all();
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[1]).target(bears).go();
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // So does a shield counter's damage effect (CR 122.1c: "If damage would be dealt to
    // this permanent, prevent that damage and remove a shield counter from it."): the
    // damage is dealt and the counter is still removed (CR 615.12).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    put_counters(&mut t, wurm, counters::SHIELD, 1);
    stomp(&mut t, Entity::Player(P1));
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(damage_marked(&t, wurm), 2);
    assert_eq!(t.counters(wurm, counters::SHIELD), 0);
}

// --- Forsaken Wastes --------------------------------------------------------------------------

#[test]
fn forsaken_wastes_and_setting_a_life_total() {
    cr!("119.5", "119.7");
    ruling!(
        "Forsaken Wastes",
        "If an effect says to set your life total to a certain number, and that number is higher than your current life total, that effect will normally cause you to gain life equal to the difference. With Forsaken Wastes on the battlefield, that part of the effect won't do anything. (If the number is lower than your current life total, the effect will work as normal.)"
    );
    supported("Forsaken Wastes");
    supported("Blessed Wind");
    // Blessed Wind: "Target player's life total becomes 20."
    for (life, after) in [(10, 10), (25, 20)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Forsaken Wastes");
        set_life(&mut t, P0, life);
        cast_new(&mut t, P0, "Blessed Wind", &[Entity::Player(P0)]);
        t.resolve_all();
        assert_eq!(t.life(P0), after, "{life}");
    }
}

#[test]
fn forsaken_wastes_can_be_countered_without_losing_life() {
    cr!("603.6d", "113.6");
    ruling!(
        "Forsaken Wastes",
        "The loss of life from targeting this card does not work until after this card enters, so you can counter it without losing life."
    );
    let mut t = TestGame::new(2);
    let wastes = cast_new(&mut t, P0, "Forsaken Wastes", &[]);
    cast_new(&mut t, P1, "Counterspell", &[obj(wastes)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forsaken Wastes"));
    assert_eq!(t.life(P1), 20);
    // On the battlefield, targeting it costs 5 life.
    let mut t = TestGame::new(2);
    let wastes = t.battlefield(P0, "Forsaken Wastes");
    cast_new(&mut t, P1, "Disenchant", &[obj(wastes)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

// --- Tel-Jilad Stylus and ownership ---------------------------------------------------------

fn stylus(t: &mut TestGame, target: ObjectId) {
    let s = t.battlefield(P0, "Tel-Jilad Stylus");
    activate(t, P0, s, "bottom", &[obj(target)]);
    t.resolve_all();
}

#[test]
fn tel_jilad_stylus_tokens_and_permanents_you_own() {
    cr!("111.7", "111.8", "108.3", "110.2");
    ruling!(
        "Tel-Jilad Stylus",
        "Tel-Jilad Stylus's ability can put tokens on the bottom of your library (where they then cease to exist)."
    );
    ruling!(
        "Tel-Jilad Stylus",
        "Tel-Jilad Stylus's ability may target a permanent you own but do not control."
    );
    ruling!("Tel-Jilad Stylus", "You own a token if it entered under your control.");
    supported("Tel-Jilad Stylus");
    // A token: bottom of the library, then gone.
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P0, "Soldier");
    let lib = t.library_size(P0);
    stylus(&mut t, tok);
    assert!(!t.on_battlefield(tok));
    assert_eq!(t.library_size(P0), lib);
    assert!(t.g.permanents().all(|o| !o.is_token()));
    // A permanent P0 owns that P1 controls.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    stylus(&mut t, bears);
    assert_eq!(t.g.player(P0).library.first(), Some(&t.g.current(bears)));
    // A token that entered under P0's control because of P1's Forbidden Orchard ("target
    // opponent creates a 1/1 colorless Spirit creature token") is P0's, even when P1 controls
    // it; a token P1 created isn't.
    let mut t = TestGame::new(2);
    let orchard = t.battlefield(P1, "Forbidden Orchard");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    assert!(tap_for_mana(&mut t, P1, orchard, "Add"));
    t.settle();
    t.resolve_all();
    let spirit = crate::r_s05_common::tokens_with_subtype(&t, P0, "Spirit");
    assert_eq!(spirit.len(), 1);
    assert_eq!(t.obj_now(spirit[0]).owner, P0);
    give_control(&mut t, spirit[0], P1);
    let theirs = create_token(&mut t, P1, "Soldier");
    let s = t.battlefield(P0, "Tel-Jilad Stylus");
    let c = ability_targets(&mut t, s, 0);
    assert!(c.contains(&obj(spirit[0])) && !c.contains(&obj(theirs)));
}

#[test]
fn tel_jilad_stylus_cards_from_your_deck_and_wished_cards_are_yours() {
    cr!("108.3", "400.11");
    ruling!(
        "Tel-Jilad Stylus",
        "You own all cards that started the game in your deck, and all cards that you \"Wished\" into the game (with one of the Judgment Wish cards or Ring of Maruf)."
    );
    supported("Living Wish");
    let mut t = TestGame::new(2);
    // Living Wish brings Grizzly Bears from outside the game; P0 casts it; P1 takes it.
    let outside = t.custom(
        P0,
        (*mtg_engine::card::card("Grizzly Bears")).clone(),
        Zone::Outside(P0),
    );
    t.answer_choose(P0, &[obj(outside)]);
    cast_new(&mut t, P0, "Living Wish", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    let bears = t.g.current(outside);
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.obj_now(bears).owner, P0);
    give_control(&mut t, bears, P1);
    // A card from P0's library, cast and stolen the same way.
    let drawn = t.library_top(P0, "Hill Giant");
    t.g.draw_cards(P0, 1);
    t.lands(P0, "Mountain", 4);
    t.cast(P0, t.g.current(drawn)).go();
    t.resolve_all();
    let giant = t.g.current(drawn);
    give_control(&mut t, giant, P1);
    let s = t.battlefield(P0, "Tel-Jilad Stylus");
    let c = ability_targets(&mut t, s, 0);
    assert!(c.contains(&obj(bears)) && c.contains(&obj(giant)));
    stylus(&mut t, bears);
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P0));
}

// --- Tokens ---------------------------------------------------------------------------------

#[test]
fn hallowed_moonlight_exiles_creature_tokens_even_from_cast_spells() {
    cr!("111.7", "614.1c", "601.1");
    ruling!(
        "Hallowed Moonlight",
        "After Hallowed Moonlight resolves, if a creature token would be put onto the battlefield, it’s put into exile instead and then ceases to exist. Creature tokens are never cast, even if the spell that created them was."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Hallowed Moonlight", &[]);
    t.resolve_all();
    // Raise the Alarm, a cast spell, creates two Soldiers: exiled.
    cast_new(&mut t, P1, "Raise the Alarm", &[]);
    t.resolve_all();
    assert!(t.g.permanents().all(|o| !o.is_token()));
    assert!(t.g.objects.iter().all(|o| !o.is_token() || o.zone != Zone::Battlefield));
    // A creature spell still enters.
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn tokens_with_too_little_toughness_enter_die_and_trigger() {
    cr!("704.5f", "603.6a", "603.10a");
    ruling!(
        "Illness in the Ranks",
        "If an effect creates a creature token that normally has toughness 1, it will enter with toughness 0, be put into its owner's graveyard as a state-based action, and then cease to exist. Any abilities that trigger when a creature enters or dies will trigger."
    );
    ruling!(
        "Virulent Plague",
        "If an effect creates a creature token that normally has toughness 2 or less, it will enter the battlefield with toughness 0 or less, be put into its owner’s graveyard, and then cease to exist. Any abilities that trigger when a creature enters the battlefield or dies will trigger."
    );
    for (hate, model) in [
        ("Illness in the Ranks", "Elite Vanguard"),
        ("Virulent Plague", "Grizzly Bears"),
    ] {
        supported(hate);
        let mut t = TestGame::new(2);
        t.battlefield(P1, hate);
        // Soul Warden: "Whenever another creature enters, you gain 1 life." Blood Artist:
        // "Whenever this creature or another creature dies, target player loses 1 life and
        // you gain 1 life."
        t.battlefield(P0, "Soul Warden");
        t.battlefield(P0, "Blood Artist");
        let m = t.battlefield(P0, model);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let before = t.g.objects.len();
        token_copy(&mut t, P0, m);
        t.resolve_all();
        // The token was created, and is gone.
        assert!(t.g.objects[before..].iter().any(|o| o.is_token()), "{hate}");
        assert!(t.g.permanents().all(|o| !o.is_token()), "{hate}");
        assert_eq!(t.life(P0), 22, "{hate}");
        assert_eq!(t.life(P1), 19, "{hate}");
    }
}

#[test]
fn legions_to_ashes_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Legions to Ashes",
        "If the target is illegal as Legions to Ashes tries to resolve, it will be removed from the stack and do nothing, even if there are tokens with that name still on the battlefield."
    );
    supported("Legions to Ashes");
    let mut t = TestGame::new(2);
    let a = create_token(&mut t, P1, "Soldier");
    let b = create_token(&mut t, P1, "Soldier");
    cast_new(&mut t, P0, "Legions to Ashes", &[obj(a)]);
    destroy(&mut t, a);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert!(t.in_graveyard(P0, "Legions to Ashes"));
}

// --- Tutors, shuffles and extra turns ---------------------------------------------------------

#[test]
fn stranglehold_skips_an_extra_turn_unless_it_left_before_the_turn() {
    cr!("614.10", "500.7");
    ruling!(
        "Stranglehold",
        "An “extra turn” means a turn created by a spell or ability. It doesn’t refer to a turn taken in a sanctioned tournament after the time limit for the round has expired."
    );
    ruling!(
        "Stranglehold",
        "If an opponent’s extra turn is created while Stranglehold is on the battlefield, but Stranglehold leaves the battlefield before that turn would begin, that opponent takes the extra turn as normal."
    );
    supported("Stranglehold");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let hold = t.battlefield(P1, "Stranglehold");
        cast_new(&mut t, P0, "Time Walk", &[]);
        t.resolve_all();
        if leaves {
            destroy(&mut t, hold);
        }
        t.advance_to_step(Step::Upkeep);
        assert_eq!(t.g.turn.active, if leaves { P0 } else { P1 }, "{leaves}");
        assert_eq!(t.g.turn.extra, leaves);
    }
}

#[test]
fn search_hate_doesnt_stop_looking_at_or_revealing_the_top_of_a_library() {
    cr!("701.23a", "701.20a");
    ruling!(
        "Ashiok, Dream Render",
        "Effects that instruct your opponents to reveal or look at cards from the top of their libraries will still work. Only effects that use the word “search” are affected."
    );
    ruling!(
        "Stranglehold",
        "Effects that instruct your opponents to reveal or look at cards from the top of their libraries will still work. Only effects that use the word “search” are affected."
    );
    supported("Ashiok, Dream Render");
    for hate in ["Ashiok, Dream Render", "Stranglehold"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, hate);
        t.set_step(P1, Step::PrecombatMain);
        // Impulse: "Look at the top four cards of your library. Put one of them into your
        // hand and the rest on the bottom of your library in any order."
        let bolt = t.library_top(P1, "Lightning Bolt");
        t.answer_choose(P1, &[obj(bolt)]);
        cast_new(&mut t, P1, "Impulse", &[]);
        t.resolve_all();
        assert!(t.in_hand(P1, "Lightning Bolt"), "{hate}");
        // Rampant Growth's search finds nothing.
        let forest = t.library_top(P1, "Forest");
        t.answer_choose(P1, &[obj(forest)]);
        cast_new(&mut t, P1, "Rampant Growth", &[]);
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(forest)), Zone::Library(P1), "{hate}");
    }
}

#[test]
fn cosis_trickster_sees_shuffle_instructions_but_not_cascade() {
    cr!("701.24a", "702.85a", "603.2");
    ruling!(
        "Cosi's Trickster",
        "Cosi’s Trickster’s ability triggers when an opponent shuffles their library because that player was instructed to do so by a spell or ability that specifically contains the word “shuffle” in its text or (in the case of a keyword ability) in its rules."
    );
    ruling!(
        "Cosi's Trickster",
        "The cascade ability doesn’t cause a player to shuffle their library, even if that player revealed their entire library. The revealed cards are put on the bottom of their owner’s library in a random order, but they’re not “shuffled.”"
    );
    supported("Cosi's Trickster");
    // Rampant Growth: search, then shuffle.
    let mut t = TestGame::new(2);
    let cosi = t.battlefield(P0, "Cosi's Trickster");
    t.set_step(P1, Step::PrecombatMain);
    t.library_top(P1, "Forest");
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Rampant Growth", &[]);
    t.resolve_all();
    assert_eq!(t.counters(cosi, counters::PLUS1), 1);
    // Bloodbraid Elf's cascade, revealing P1's entire library: no shuffle, no trigger.
    let mut t = TestGame::new(2);
    let cosi = t.battlefield(P0, "Cosi's Trickster");
    t.set_step(P1, Step::PrecombatMain);
    while let Some(c) = t.g.player(P1).library.last().copied() {
        t.g.exile_object(c, None);
    }
    crate::r_s01_common::stack_library(&mut t, P1, &["Mountain", "Forest"]);
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Bloodbraid Elf", &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 2);
    assert_eq!(t.counters(cosi, counters::PLUS1), 0);
}

#[test]
fn leonin_arbiter_payer_may_search_any_library() {
    cr!("116.2f", "701.23a");
    ruling!(
        "Leonin Arbiter",
        "A player who has paid {2} may search any library when instructed to do so that turn, not just their own library."
    );
    supported("Leonin Arbiter");
    supported("Praetor's Grasp");
    // Praetor's Grasp: "Search target opponent's library for a card and exile it face down.
    // ..."
    for paid in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Leonin Arbiter");
        let bolt = t.library_top(P1, "Lightning Bolt");
        if paid {
            t.lands(P0, "Wastes", 2);
            let ignore = crate::r_s08_common::actions_of(&mut t, P0)
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        mtg_engine::decision::Action::Special(
                            mtg_engine::decision::SpecialAction::Static { .. }
                        )
                    )
                })
                .expect("the special action to ignore Leonin Arbiter");
            t.g.perform_action(P0, ignore).unwrap();
            t.g.flush_events();
            t.settle();
            t.resolve_all();
        }
        t.answer_choose(P0, &[obj(bolt)]);
        cast_new(&mut t, P0, "Praetor's Grasp", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(bolt)) == Zone::Exile, paid, "{paid}");
    }
}

// --- Creature types ---------------------------------------------------------------------------

#[test]
fn halo_hunter_must_target_your_own_angel_if_its_the_only_one() {
    cr!("603.3d", "115.1");
    ruling!(
        "Halo Hunter",
        "Halo Hunter's triggered ability isn't optional. When it enters, if you're the only player that controls an Angel, you'll have to target an Angel you control."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "Halo Hunter");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
}

#[test]
fn witchs_vengeance_creature_type_choice_and_affected_creatures() {
    cr!("205.3m", "608.2c", "611.2c", "702.11b", "702.16b");
    ruling!(
        "Witch's Vengeance",
        "To choose a creature type, you must choose an existing creature type, such as Human or Knight. You can't choose multiple creature types, such as “Human Knight,” but choosing “Human” is enough to affect a Human Knight. Card types such as artifact can't be chosen; nor can supertypes such as legendary; nor can subtypes that aren't creature types, such as Vehicle."
    );
    ruling!(
        "Witch's Vengeance",
        "Witch's Vengeance affects only creatures of the chosen type that are on the battlefield at the time it resolves. This includes creatures you control, creatures with hexproof, and creatures with protection from black. Creatures that enter the battlefield or become the chosen type later in the turn won't get -3/-3."
    );
    ruling!(
        "Witch's Vengeance",
        "You choose a creature type as Witch's Vengeance resolves, not as you cast it. No player may take actions between the time you choose a creature type and the time those creatures get -3/-3."
    );
    supported("Witch's Vengeance");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Benalish Knight");
    let stalker = t.battlefield(P1, "Invisible Stalker");
    let white = t.battlefield(P1, "White Knight");
    let mine = t.battlefield(P0, "Squire");
    let bears = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Witch's Vengeance", &[]);
    // Nothing chosen while casting.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    assert!(t.g.stack.contains(&spell));
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Human");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| g.stack.len(),
    );
    let from = t.asked().len();
    t.resolve();
    // Chosen while it resolved (it was still on the stack); the options are creature
    // types only.
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    let options = t.asked()[from..]
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .expect("a creature type choice");
    for ok in ["Human", "Knight", "Goat"] {
        assert!(options.iter().any(|o| o == ok), "{ok}");
    }
    for bad in ["Artifact", "Legendary", "Vehicle", "Human Knight", "Equipment"] {
        assert!(!options.iter().any(|o| o == bad), "{bad}");
    }
    // Every Human, including P0's, the hexproof one and the one with protection from
    // black; not the Wurm.
    assert!(!t.on_battlefield(knight) && !t.on_battlefield(stalker));
    assert!(!t.on_battlefield(white) && !t.on_battlefield(mine));
    assert_eq!(t.pt(bears), (6, 4));
    // A Human entering later, or a creature becoming Human later, isn't affected.
    let later = t.battlefield(P1, "Elite Vanguard");
    assert_eq!(t.pt(later), (2, 1));
    let amoeboid = t.battlefield(P0, "Amoeboid Changeling");
    activate(&mut t, P0, amoeboid, "gains all", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 4));
}

#[test]
fn eye_gouge_gives_a_cyclops_minus_one_before_destroying_it() {
    cr!("608.2c", "701.19a");
    ruling!(
        "Eye Gouge",
        "If the target creature is a Cyclops, it will get -1/-1 before it’s destroyed."
    );
    supported("Eye Gouge");
    // Polis Crusher, a 4/4 Cyclops, regenerates: the -1/-1 stays.
    let mut t = TestGame::new(2);
    let crusher = t.battlefield(P1, "Polis Crusher");
    cast_new(&mut t, P1, "Regenerate", &[obj(crusher)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Eye Gouge", &[obj(crusher)]);
    t.resolve_all();
    assert!(t.on_battlefield(crusher));
    assert!(is_tapped(&t, crusher));
    assert_eq!(t.pt(crusher), (3, 3));
    // A non-Cyclops just gets -1/-1.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Eye Gouge", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn creature_type_targets() {
    cr!("205.3m", "115.1");
    ruling!(
        "Sigardian Priest",
        "A creature that is Human and has other types, such as a Human Werewolf, isn’t a non-Human creature."
    );
    ruling!(
        "Undead Slayer",
        "Undead Slayer’s ability must target a permanent (not a card in the graveyard) with at least one of the listed creature types."
    );
    supported("Undead Slayer");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Sigardian Priest");
    let slayer = t.battlefield(P0, "Undead Slayer");
    let werewolf = t.battlefield(P1, "Instigator Gang // Wildblood Pack");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let zombie = t.battlefield(P1, "Gravecrawler");
    let vampire = t.battlefield(P1, "Vampire Nighthawk");
    let skeleton = t.battlefield(P1, "Drudge Skeletons");
    let dead = t.graveyard(P1, "Gravecrawler");
    let c = ability_targets(&mut t, priest, 0);
    assert!(!c.contains(&obj(werewolf)) && c.contains(&obj(bears)));
    let c = ability_targets(&mut t, slayer, 0);
    for ok in [zombie, vampire, skeleton] {
        assert!(c.contains(&obj(ok)));
    }
    assert!(!c.contains(&obj(dead)) && !c.contains(&obj(bears)) && !c.contains(&obj(werewolf)));
}

#[test]
fn deathcult_rogue_stays_blocked_when_the_blocker_stops_being_a_rogue() {
    cr!("509.1h", "506.4");
    ruling!(
        "Deathcult Rogue",
        "If a creature already assigned to block Deathcult Rogue stops being a Rogue, it will continue to block."
    );
    supported("Deathcult Rogue");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Deathcult Rogue");
    let goblin = t.battlefield(P1, "Goatnapper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Amoeboid Changeling");
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, rogue)]));
    assert!(legal_blocks(&mut t, P1, &[(goblin, rogue)]));
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Deathcult Rogue");
    let goblin = t.battlefield(P1, "Goatnapper");
    let amoeboid = t.battlefield(P1, "Amoeboid Changeling");
    to_blockers(&mut t, &[(rogue, Entity::Player(P1))], &[(goblin, rogue)]);
    activate(&mut t, P1, amoeboid, "loses all", &[obj(goblin)]);
    t.resolve_all();
    assert!(!t.obj_now(goblin).chars.has_subtype("Rogue"));
    assert_eq!(blocks_now(&t), vec![(goblin, rogue)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn goblin_masons_without_a_wall_is_removed_from_the_stack() {
    cr!("603.3d");
    ruling!(
        "Goblin Masons",
        "If there is no Wall to target, then the ability just gets removed from the stack before any player gets priority."
    );
    supported("Goblin Masons");
    let mut t = TestGame::new(2);
    let masons = t.battlefield(P0, "Goblin Masons");
    t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, masons);
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let masons = t.battlefield(P0, "Goblin Masons");
    let wall = t.battlefield(P1, "Wall of Stone");
    destroy(&mut t, masons);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(wall));
}

// --- Colors -----------------------------------------------------------------------------------

#[test]
fn glistening_deluge_locks_in_its_creatures_and_their_colors() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Glistening Deluge",
        "All effects of Glistening Deluge are determined as it resolves. Creatures that enter the battlefield later in the turn won’t get -1/-1 or -3/-3, nor will noncreature permanents that become creatures. A creature that gets -1/-1 won’t get an additional -2/-2 if it later becomes green and/or white. A green and/or white creature that gets -3/-3 won’t have the -2/-2 rescinded if it later stops being green and white."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let factory = t.battlefield(P1, "Mishra's Factory");
    cast_new(&mut t, P0, "Glistening Deluge", &[]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.pt(wurm), (3, 1));
    // Entering later.
    let later = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(later), (2, 2));
    // A land becoming a creature later ("{1}: becomes a 2/2 Assembly-Worker artifact
    // creature until end of turn").
    t.lands(P1, "Wastes", 1);
    activate(&mut t, P1, factory, "becomes", &[]);
    t.resolve_all();
    assert_eq!(t.pt(factory), (2, 2));
    // Purelace: "Target spell or permanent becomes white." The Hill Giant doesn't get the
    // additional -2/-2.
    cast_new(&mut t, P0, "Purelace", &[obj(giant)]);
    t.resolve_all();
    assert!(t.obj_now(giant).chars.colors.contains(Color::White));
    assert_eq!(t.pt(giant), (2, 2));
    // Cerulean Wisps: "Target creature becomes blue until end of turn. Untap that
    // creature." The Wurm keeps its -3/-3.
    cast_new(&mut t, P0, "Cerulean Wisps", &[obj(wurm)]);
    t.resolve_all();
    assert!(!t.obj_now(wurm).chars.colors.contains(Color::Green));
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn ember_gale_damages_at_resolution_but_stops_every_later_blocker() {
    cr!("611.2c", "608.2h", "509.1b");
    ruling!(
        "Ember Gale",
        "Ember Gale will deal damage only to white and/or blue creatures the targeted player controls at the time it resolves. However, it will prevent all creatures that player controls from blocking that turn, even if those creatures enter later."
    );
    supported("Ember Gale");
    let mut t = TestGame::new(2);
    let white = t.battlefield(P1, "Benalish Knight");
    let green = t.battlefield(P1, "Grizzly Bears");
    let attacker = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Ember Gale", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(damage_marked(&t, white), 1);
    assert_eq!(damage_marked(&t, green), 0);
    // A blue creature entering later isn't damaged, and can't block either.
    let blue = t.battlefield(P1, "Coral Merfolk");
    assert_eq!(damage_marked(&t, blue), 0);
    attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    for b in [white, green, blue] {
        assert!(!legal_blocks(&mut t, P1, &[(b, attacker)]));
    }
}

#[test]
fn stormbreath_dragon_counts_each_hand_on_resolution() {
    cr!("608.2h", "701.37c");
    ruling!(
        "Stormbreath Dragon",
        "For each opponent, count the number of cards in that player's hand when the last ability resolves to determine how much damage Stormbreath Dragon does to them."
    );
    supported("Stormbreath Dragon");
    let mut t = TestGame::new(3);
    let dragon = t.battlefield(P0, "Stormbreath Dragon");
    for _ in 0..2 {
        t.hand(P1, "Grizzly Bears");
    }
    for _ in 0..4 {
        t.hand(P2, "Grizzly Bears");
    }
    t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::C, 5);
    add_mana(&mut t, P0, mtg_engine::mana::ManaType::R, 2);
    activate(&mut t, P0, dragon, "Monstrosity", &[]);
    t.resolve();
    t.settle();
    // The "becomes monstrous" trigger is on the stack; P1 draws a card in response.
    assert_eq!(t.stack_len(), 1);
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn tourach_and_madness_triggers_go_on_the_stack_in_apnap_order() {
    cr!("101.4", "603.3b", "702.35a");
    ruling!(
        "Tourach, Dread Cantor",
        "Triggered abilities are put on the stack in turn order, starting with the player whose turn it is. For example, if an opponent discards a card with madness during your turn, the madness trigger will resolve before Tourach's ability can put a +1/+1 counter on it. If they discard it during their turn, Tourach's ability puts a +1/+1 counter on it before they can cast the spell."
    );
    for p1_turn in [false, true] {
        let mut t = TestGame::new(2);
        if p1_turn {
            t.set_step(P1, Step::PrecombatMain);
        }
        t.battlefield(P0, "Tourach, Dread Cantor");
        let temper = t.hand(P1, "Fiery Temper");
        t.answer_yes(P1, true);
        t.g.discard(P1, temper, None);
        t.g.flush_events();
        t.settle();
        let items = crate::r_s04_common::stack_items(&t);
        let tourach_text = "ability: Whenever an opponent discards a card, put a +1/+1 counter on ~";
        let expected = if p1_turn {
            // P1's madness trigger first (bottom), P0's Tourach trigger on top.
            vec!["ability: Madness".to_string(), tourach_text.to_string()]
        } else {
            vec![tourach_text.to_string(), "ability: Madness".to_string()]
        };
        assert_eq!(items, expected);
        // The top one resolves first.
        let tourach = t.named_on_battlefield("Tourach, Dread Cantor")[0];
        t.resolve();
        assert_eq!(t.counters(tourach, counters::PLUS1), u32::from(p1_turn));
    }
}

#[test]
fn blasphemous_act_cost_is_locked_in_before_mana_abilities_sacrifice_a_creature() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Blasphemous Act",
        "Although players may respond to Blasphemous Act once it's been cast, once it's announced, they can't respond before the cost is calculated and paid."
    );
    ruling!(
        "Blasphemous Act",
        "The total cost to cast Blasphemous Act is locked in before you pay that cost. For example, if there are three creatures on the battlefield, including one you can sacrifice to add {C}, the total cost of Blasphemous Act is {5}{R}. Then you can sacrifice the creature when you activate mana abilities just before paying the cost."
    );
    supported("Blood Pet");
    let mut t = TestGame::new(2);
    // Three creatures, one of them Blood Pet ("Sacrifice this creature: Add {B}."): the
    // total cost is {5}{R}, paid with five lands and Blood Pet's mana.
    let pet = t.battlefield(P0, "Blood Pet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let act = t.hand(P0, "Blasphemous Act");
    let from = t.asked().len();
    let s = t.cast(P0, act).go();
    // Nobody had priority between announcing it and paying for it.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.in_graveyard(P0, "Blood Pet"));
    assert!(!t.on_battlefield(pet));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    // Once cast, P1 may respond.
    assert!(t.g.stack.contains(&s));
    assert!(could_cast(&mut t, P1, "Lightning Bolt"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blasphemous Act"));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
}

#[test]
fn frenzied_baloth_can_be_targeted_by_counterspells_whose_other_effects_happen() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Frenzied Baloth",
        "A spell or ability that counters spells can still target Frenzied Baloth. When that spell or ability resolves, Frenzied Baloth won’t be countered, but any additional effects of the countering spell or ability will still happen. The same is true for creature spells you control while you control Frenzied Baloth."
    );
    // Exclude: "Counter target creature spell. Draw a card."
    for name in ["Frenzied Baloth", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        if name == "Grizzly Bears" {
            t.battlefield(P0, "Frenzied Baloth");
        }
        let s = cast_new(&mut t, P0, name, &[]);
        let hand = t.hand_size(P1);
        lands_for_cost(&mut t, P1, "Exclude");
        let ex = t.hand(P1, "Exclude");
        t.cast_with(P1, ex, &[obj(s)]).expect("Exclude can target it");
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(name).len(), 1, "{name}");
        assert_eq!(t.hand_size(P1), hand + 1, "{name}");
    }
}
