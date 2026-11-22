//! Rulings batch P118 — triggered abilities of power-matters creatures: abilities that
//! trigger on casting resolve first, even if the spell is countered (CR 603.3, 608.2b);
//! one trigger per spell however many times it targets (CR 603.2c); one trigger per
//! life-gaining event (CR 119.10) and none for a teammate's (CR 810.9); leaves-the-
//! battlefield triggers and simultaneous events (CR 603.10a, 704.3); and loss of the game
//! before triggers are put on the stack (CR 704.3).

use crate::r_p118_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

// --- Myriad Construct ------------------------------------------------------------------------

#[test]
fn myriad_construct_triggers_once_for_a_spell_targeting_it_several_times() {
    cr!("603.2c", "115.3");
    ruling!(
        "Myriad Construct",
        "If a spell targets Myriad Construct more than once, its last ability triggers only once."
    );
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Myriad Construct");
    cast_new(&mut t, P0, "Seeds of Strength", &[obj(c), obj(c), obj(c)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice it"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(c));
    assert_eq!(tokens_named(&t, P0, "Construct"), 4);
}

#[test]
fn myriad_construct_second_trigger_still_creates_tokens_from_its_last_known_power() {
    cr!("603.10a", "608.2h", "701.21a");
    ruling!(
        "Myriad Construct",
        "If you don't sacrifice Myriad Construct as its last ability resolves (perhaps because its ability triggered more than once and you've already sacrificed it), you still create Construct tokens. The number of tokens you create is determined by Myriad Construct's power as it last existed on the battlefield."
    );
    let mut t = TestGame::new(2);
    let c = with_counters(&mut t, P0, "Myriad Construct", 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(c)]);
    t.settle();
    cast_new(&mut t, P0, "Giant Growth", &[obj(c)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice it"), 2);
    t.resolve_all();
    // 5 + 5: the Giant Growths never resolved.
    assert_eq!(tokens_named(&t, P0, "Construct"), 10);
}

// --- triggers on casting resolve first ---------------------------------------------------------

#[test]
fn cast_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "603.2", "701.6a");
    ruling!(
        "Legolas, Master Archer",
        "Legolas, Master Archer's triggered abilities resolve before the spells that caused them to trigger. They resolve even if those spells are countered."
    );
    ruling!(
        "Pollywog Prodigy",
        "Pollywog Prodigy's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Neerdiv, Devious Diver",
        "Neerdiv's last ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered or otherwise leaves the stack without resolving."
    );
    supported("Legolas, Master Archer");
    supported("Pollywog Prodigy");
    supported("Neerdiv, Devious Diver");
    // Legolas: Giant Growth on it, countered.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Legolas, Master Archer");
    let gg = cast_new(&mut t, P0, "Giant Growth", &[obj(l)]);
    t.settle();
    counter_spell(&mut t, gg);
    t.resolve_all();
    assert_eq!(plus1(&t, l), 1);
    assert_eq!(t.pt(l), (2, 5));
    // Pollywog Prodigy (2/4 with a counter): P1's Shock (mana value 1), countered.
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Pollywog Prodigy", 1);
    t.library_top(P0, "Island");
    let hand = t.hand_size(P0);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(stack(&t), 2);
    counter_spell(&mut t, shock);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 20);
    // Neerdiv: Think Twice cast with flashback from the graveyard, countered.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Neerdiv, Devious Diver");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    let hand = t.hand_size(P0);
    let s = t
        .cast(P0, tt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    t.settle();
    assert_eq!(stack(&t), 2);
    counter_spell(&mut t, s);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(plus1(&t, n), 1);
}

#[test]
fn pollywog_prodigy_trigger_resolves_whatever_happens_to_it() {
    cr!("603.3", "113.7a");
    ruling!(
        "Pollywog Prodigy",
        "Once Pollywog Prodigy's last ability has triggered, it doesn't matter what happens to Pollywog Prodigy after that. Reducing its power or removing it from the battlefield won't stop the ability from resolving."
    );
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let p = with_counters(&mut t, P0, "Pollywog Prodigy", 1);
        t.library_top(P0, "Island");
        let hand = t.hand_size(P0);
        cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        t.settle();
        assert_eq!(stack(&t), 2);
        minus_power(&mut t, p, 2);
        if remove {
            exile(&mut t, p);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "remove {remove}");
    }
}

#[test]
fn legolas_targeting_itself_and_an_opposing_creature_triggers_both() {
    cr!("603.2", "603.3b");
    ruling!(
        "Legolas, Master Archer",
        "If you cast a spell that targets Legolas, Master Archer and at least one creature you don't control, both of Legolas, Master Archer's abilities will trigger. You choose the order in which those triggers go on the stack."
    );
    supported("Ram Through");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Legolas, Master Archer");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Ram Through", &[obj(l), obj(wurm)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "put a +1/+1 counter"), 1);
    assert_eq!(triggers_on_stack(&t, "deals damage equal to its power"), 1);
}

#[test]
fn legolas_last_ability_triggers_once_per_spell() {
    cr!("603.2", "603.2c");
    ruling!(
        "Legolas, Master Archer",
        "Legolas, Master Archer's last ability triggers when you cast a spell that has multiple targets as long as at least one of those targets is a creature you don't control. It doesn't trigger multiple times if you cast a spell that targets a creature you don't control multiple times or that targets multiple creatures you don't control."
    );
    supported("Electrolyze");
    supported("Seeds of Strength");
    // Electrolyze, 1 damage to each of two creatures P1 controls.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Legolas, Master Archer");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_new(&mut t, P0, "Electrolyze", &[obj(a), obj(b)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals damage equal to its power"), 1);
    // Seeds of Strength on the same creature P1 controls three times.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Legolas, Master Archer");
    let a = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Seeds of Strength", &[obj(a), obj(a), obj(a)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals damage equal to its power"), 1);
    // Ram Through: multiple targets, one of them a creature P1 controls.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Legolas, Master Archer");
    let a = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Ram Through", &[obj(bears), obj(a)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals damage equal to its power"), 1);
}

// --- life-gaining events ------------------------------------------------------------------------

#[test]
fn life_gain_for_each_is_one_event() {
    cr!("119.10", "603.2c");
    ruling!(
        "Fiendish Panda",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Fiendish Panda's first ability triggers only once."
    );
    ruling!(
        "Elenda's Hierophant",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and the second ability of Elenda's Hierophant triggers only once."
    );
    ruling!(
        "Elenda's Hierophant",
        "The second ability of Elenda's Hierophant triggers just once for each life-gaining event, no matter how much life was gained."
    );
    supported("Bountiful Harvest");
    supported("Invigorating Falls");
    // Bountiful Harvest: "You gain 1 life for each land you control." Invigorating Falls:
    // "You gain life equal to the number of creature cards in all graveyards."
    for watcher in ["Fiendish Panda", "Elenda's Hierophant"] {
        for spell in ["Bountiful Harvest", "Invigorating Falls"] {
            let mut t = TestGame::new(2);
            let w = t.battlefield(P0, watcher);
            for _ in 0..3 {
                t.graveyard(P1, "Grizzly Bears");
            }
            t.lands(P0, "Forest", 3);
            cast_new(&mut t, P0, spell, &[]);
            t.resolve_all();
            assert!(t.life(P0) >= 23, "{watcher} {spell}");
            assert_eq!(plus1(&t, w), 1, "{watcher} {spell}");
        }
    }
    // One trigger for each event: 4 life, then 1 life.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Elenda's Hierophant");
    gain_life(&mut t, P0, 4);
    t.resolve_all();
    assert_eq!(plus1(&t, h), 1);
    gain_life(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, h), 2);
}

#[test]
fn life_gained_by_a_teammate_doesnt_trigger() {
    cr!("810.9", "119.10");
    ruling!(
        "Fiendish Panda",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Fiendish Panda's first ability to trigger, even though it caused your team's life total to increase."
    );
    ruling!(
        "Elenda's Hierophant",
        "In a Two-Headed Giant game, life gained by your teammate won't cause the second ability of Elenda's Hierophant to trigger even though it caused your team's life total to increase."
    );
    for watcher in ["Fiendish Panda", "Elenda's Hierophant"] {
        let mut t = two_headed_giant();
        let w = t.battlefield(P0, watcher);
        let life = t.life(P0);
        cast_new(&mut t, P1, "Sacred Nectar", &[]);
        t.resolve_all();
        assert_eq!(t.life(P0), life + 4);
        assert_eq!(plus1(&t, w), 0, "{watcher}: teammate");
        cast_new(&mut t, P0, "Sacred Nectar", &[]);
        t.resolve_all();
        assert_eq!(plus1(&t, w), 1, "{watcher}: you");
    }
}

// --- leaving the battlefield -------------------------------------------------------------------

#[test]
fn light_of_the_legion_dying_with_another_white_creature_doesnt_save_it() {
    cr!("704.3", "603.10a", "704.5g");
    ruling!(
        "Light of the Legion",
        "If another white creature you control is dealt lethal damage at the same time as Light of the Legion, they're destroyed at the same time. That creature won't receive a counter from Light of the Legion's ability in time to save it."
    );
    supported("Light of the Legion");
    let mut t = TestGame::new(2);
    let light = t.battlefield(P0, "Light of the Legion");
    t.battlefield(P0, "Savannah Lions");
    let angel = t.battlefield(P0, "Serra Angel");
    // 3 damage marked first; then Pyroclasm's 2 is lethal to Light and to the Lions.
    let src = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, src, 3, light);
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Light of the Legion"));
    assert!(t.in_graveyard(P0, "Savannah Lions"));
    assert_eq!(plus1(&t, angel), 1, "the survivor gets its counter");
}

#[test]
fn carmen_sacrificed_still_triggers() {
    cr!("603.10a", "701.21a");
    ruling!(
        "Carmen, Cruel Skymarcher",
        "If you sacrifice Carmen, Cruel Skymarcher, its second ability will still trigger."
    );
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Carmen, Cruel Skymarcher");
    t.g.sacrifice(c, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(stack(&t), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn juri_doesnt_let_you_sacrifice_anything() {
    cr!("701.21a", "113.3b");
    ruling!(
        "Juri, Master of the Revue",
        "Juri's first ability doesn't allow you to sacrifice any permanents. You'll have to find another way to do that."
    );
    supported("Juri, Master of the Revue");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Juri, Master of the Revue");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, j));
    assert!(!can_activate(&mut t, P0, bears));
    // Sacrificing another way does trigger it.
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(plus1(&t, j), 1);
}

#[test]
fn syr_vondam_sees_only_creatures_exiled_from_the_battlefield() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Syr Vondam, Sunstar Exemplar",
        "Syr Vondam, Sunstar Exemplar’s first triggered ability refers to creatures you control being put into exile from the battlefield. It won’t trigger if a creature card is put into exile from another zone."
    );
    supported("Syr Vondam, Sunstar Exemplar");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Syr Vondam, Sunstar Exemplar");
    let card = t.graveyard(P0, "Grizzly Bears");
    exile(&mut t, card);
    assert_eq!(stack(&t), 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    exile(&mut t, bears);
    assert_eq!(stack(&t), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, v), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn felisa_counts_every_kind_of_counter() {
    cr!("603.10a", "122.1");
    ruling!(
        "Felisa, Fang of Silverquill",
        "The last ability counts the number of all kinds of counters, not just +1/+1 counters."
    );
    supported("Felisa, Fang of Silverquill");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Felisa, Fang of Silverquill");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    put_counters(&mut t, bears, "flying", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Inkling"), 2);
}
