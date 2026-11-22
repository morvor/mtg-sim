# ruling-research-1: adversarial skeptic's verdict

Skeptic (adversarial), 2026-11-19/20. Item: `docs/agents/items/ruling-research-1.md`; entry
`control-lost-before-a-for-as-long-as-you-control-effect-begins` (docs/interpretations/triggers-durations.md,
provisional); investigator's write-up: `target/research/investigator.md`. Research only: no engine code edited,
nothing built, nothing committed. Scratch files: /tmp/claude-0/-home-user-mtg-sim/1c53f6e8-8b42-5b35-8a6f-8e8bc17948ea/scratchpad/skeptic/
(rulings.tsv and oracle.tsv joined from data/, the ormos articles I fetched, and hist.py, the CR-history script).

---------------------------------------------------------------------------------------------------------------------

## 0. Verdict

**UPHELD WITH CORRECTIONS.** I tried to refute the main conclusion and couldn't. Control of the source lost while the
ability is on the stack and regained before it resolves means the "for as long as you control ~" effect does nothing.
A current WotC ruling decides the case: Seasinger, 2007-09-16, source "wotc", still in Scryfall's bulk data of
2026-09-24 and, according to search summaries, on Gatherer. Every CR change since 2007 is consistent with it, and I
found no source anywhere that says otherwise.

The corrections are in four places:
- the engine: the check misses several effect kinds, copies of abilities, and the trigger-time window;
- one sub-case, a duration about another object made false and then true again;
- some wording in the "standard resolution";
- a few citation details.

| Sub-case | Investigator | Verdict | Confidence |
|---|---|---|---|
| Control lost, not regained | nothing | **Upheld** | certain |
| Control lost and regained before resolution | nothing | **Upheld** (but the engine does it only for 3 of the effect kinds that have this duration, see 5) | high |
| "~ remains tapped": untapped and tapped again | nothing | **Upheld** (engine gap for copies of the ability) | certain |
| Source leaves and returns (CR 400.7) | nothing | **Upheld** | certain |
| Source phases out and back in | nothing | **Upheld** (the engine gap also hits "for as long as ~ remains on the battlefield") | high-moderate |
| Duration about another object, false at resolution | nothing | **Upheld** | certain |
| Duration about another object, restored by the resolution itself | applies | **Upheld** (CR text verified; bulletin text only through search summaries) | certain |
| Duration about another object, false and then true again before resolution | nothing (low-moderate) | **Upheld with correction: split it.** (c1) A condition the ability doesn't set up (Old Man's power clause, Garland's monarch, Eriette's Aura, Rootwater's "enchanted"): nothing, **moderate**. Three event-worded rulings support it, one of which (Garland's) the investigator didn't use. (c2) A condition the resolution itself sets up (Xolatoyac, Braided Net, Zygon Infiltrator): **open**. The literal reading gives a perverse result there, so keep the engine's check at resolution. | c1 moderate; c2 open |
| Window opens when the ability triggers, not when it's put on the stack | gap (ii) | **Upheld, restated more broadly** (see 4b) | high |

---------------------------------------------------------------------------------------------------------------------

## 1. Quotes checked against primary data

### 1a. CR history
Checked in a clean clone of github.com/pit142857/mtg-cr (HEAD 8351b94 = origin HEAD, no local changes). Text comes
from each file's own rule line; hist.py prints every change.

- **Confirmed:** the 2024 wording first appears in "CR-2024-11-08.txt", whose header reads "These rules are effective as of
  November 8, 2024." The CR effective September 20, 2024 still has the 2009 text: "If the “for as long as” duration
  never starts, or it ends before the moment the effect would first be applied, the effect does nothing. It doesn’t
  start and immediately stop again, and it doesn’t last forever."
- **Confirmed (dates and texts):**
  - 2002-02-20: "ends between the end of announcing the activated ability or putting the triggered ability onto the
    stack and the moment when the effect would first be applied".
  - 2003-10-01: "announcing" becomes "playing".
  - 2008-02-01: broadened to "ends before the moment the effect would first be applied".
  - 2009-07-08 (610.2b): "for as long as" and "never starts" added.
  - 2009-10-05: renumbered 611.2b.
  - 2011-09-30: Master Thief example.
  - 2024-07-30: "enters" retemplating.
  - 2025-02-07: "this creature".
  - 702.23e, now 702.26f, first appears in CR-2009-07-08.
- **Omission.** The investigator's table skips the earliest version of the rule: CR "current as of November 1, 1999"
  (CR-1999-11-01.txt line 857), 418.3d: "If an effect's duration expires before the spell or ability creating it
  resolves, then the effect does nothing-it doesn't start and immediately stop again, and it doesn't last forever."
  The first CR (04/23/99) has no such rule. The 2001 wording ("is over before it would begin") is already in
  CR-2001-07-23, not only 2001-09-24.
- **Not a CR.** CR-1999-04-21.txt is D'Angelo's Rulings Summary, not a CR. Its T.2.7 [WotC Rules Team 08/05/97],
  E.12.2 [D'Angelo 09/12/95], E.12.Ruling.2 [WotC Rules Team 08/05/97] and E.12.Ruling.4 [D'Angelo 10/15/96] are
  quoted verbatim and attributed correctly.

### 1b. Rulings
Source: data/rulings.jsonl.gz, Scryfall bulk updated 2026-09-24T09:00:33 per data/manifest.json. I joined it to
oracle_cards by oracle_id.

- **Verbatim, source "wotc", dates as stated:**
  - Seasinger: 2004-10-04, and both 2007-09-16 rulings.
  - Preacher 2007-09-16.
  - Willow Satyr, Rubinia Soulsinger, Old Man of the Sea: all 2009-10-01.
  - Entrancing Lyre 2020-01-24.
  - Tidebinder Mage 2013-07-01, Olivia Voldaren 2017-03-14, Master Thief 2011-09-22.
  - Time of Ice 2018-04-27; Ty Lee, Chi Blocker 2025-10-02; Spectral Adversary 2021-09-24.
  - Vedalken Shackles 2004-12-01, Grand Marshal Macie 2022-10-07.
  - Dungeon Geists 2019-07-12 (both rulings), Wall of Stolen Identity 2019-08-23.
  - Rootwater Matriarch 2004-10-04 and 2009-02-01.
  - Xolatoyac 2023-11-10, Wellgabber Apothecary 2007-10-01, Teferi's Protection 2017-08-25.
  - Eriette 2024-04-12, Sower of Temptation 2007-10-01.
  - The other "lost control" and "regaining control" rulings listed in investigator §3a.
- **Typography.** The data has *straight* apostrophes in the Time of Ice ruling ("Time of Ice's first two chapter
  abilities expire if you lose control of it, even if you immediately regain control of it or cast another Time of
  Ice.") and in Rubinia's ("you won't gain control"). investigator.md prints them curly; Willow Satyr's ("won’t") is
  curly in the data. **Copy `ruling!` substrings from the data, not from investigator.md.**
- **Seasinger's Oracle text today:** "{T}: Gain control of target creature whose controller controls an Island for as
  long as you control this creature and this creature remains tapped."
  - Unchanged in what matters. The 2004 and 2007 rulings themselves show both halves were there ("You only lose
    control if Seasinger leaves your control or becomes untapped"). The 2009 "as long as" to "for as long as" rewording
    was "Nothing functionally changed here." (July 2009 Update Bulletin).
  - The oracle_cards printing is me1 (Masters Edition, `digital: true`, games ["mtgo"]). That explains
    docs/agents/items/cards-digital-alchemy.texts.tsv:877 calling it "digital". The card is paper (Fallen Empires,
    Fifth Edition) and legal in Legacy and Vintage, so the investigator is right that this is a misclassification.

### 1c. Wizards articles
Source: github.com/maxmakesmagic/ormos, fetched file by file from raw.githubusercontent.com; copies are in the
scratchpad's skeptic/ormos/. All verbatim:
- July 2009 Update Bulletin (Mark L. Gottlieb, 2009-07-08; Wayback capture 2016-04-14): "In the other kind, "as long
  as" represents a duration that's set up by an earlier part of the same ability or by the ability's activation cost.
  That kind of "as long as" effect is true from the moment its ability goes on the stack or starts applying until the
  "as long as" condition stops being true. ... Nothing functionally changed here."
- Morningtide Update Bulletin (Gottlieb, 2008-01-16): "**418.3d** This rule discusses what happens if an "as long as"
  duration ends before the effect that cares about it even starts. However, it was overly specific about the window of
  time, and there were still cases in which an "as long as" duration could end before the stated window began. So the
  rule's being broadened."
- Saturday School #4 (2002-11-16), #37 (2003-08-02), #73 (2004-04-10) and #79 (2004-05-22), all by Rune Horvik.
  The #79 path is archive/en/articles/archive/2004/05/..., not .../feature/....
- "May Days" (John Carter, 2005-04-30): "“As long as” sets a duration, and if a duration ends before the effect begins,
  the effect does nothing [CR 418.3d]."
- Theros Release Notes, line 2427 (Kraken: "between when its last ability triggers and when that ability resolves").
- Theros Beyond Death Release Notes, line 1478 (Entrancing Lyre).

### 1d. Read only through WebSearch summaries
magic.wizards.com is blocked: WebFetch fails with "getaddrinfo ENOTFOUND".
- **Foundations Update Bulletin** (https://magic.wizards.com/en/news/announcements/foundations-update-bulletin).
  - The summaries consistently give:
    - "published November 14, 2024";
    - "The only change here is that this rule now explicitly handles the rare case where the duration for a
      continuous effect starts during the resolution of the spell or ability which created that effect.";
    - "If another effect removes the existing flood counter before Xolatoyac's ability resolves, the previous wording
      of 611.2b could have been read to mean that Xolatoyac's ability doesn't make that land an Island on
      resolution".
  - New, not in investigator.md: the new wording "makes it clearer that the land does indeed become an Island and that
    similar effects work as players would expect them to".
  - **The authors ("Jess Dunks and Eric Levine") could not be verified**; a search for them found nothing.
  - Treat every one of these sentences as unverified page text.
- **Gatherer.** A search restricted to gatherer.wizards.com summarizes gatherer.wizards.com/MED/en-us/49/seasinger as
  listing the "9/16/2007" ruling ("Whether you control Seasinger is checked continually, starting when the ability is
  activated ..."), so the investigator's claim holds as far as search summaries go.
- **parun-mtg/parun #6027** (2026-11-03, open) and **PR #6029** (closed) exist as described. Read through the
  api.github.com search endpoint.

---------------------------------------------------------------------------------------------------------------------

## 2. Is the 2007 Seasinger ruling outdated? No.

**No later change moved the check to the resolution or changed when the duration starts.**
- The ruling paraphrases the 2002–2007 418.3d window, which started "[at] the end of playing the activated ability".
- 2008 broadened that window to *earlier* starts (Morningtide: durations "could end before the stated window began").
- 2009 renamed the durations: "Nothing functionally changed here."
- The Masters Edition III rulings of 2009-10-01 (Willow Satyr, Rubinia, Old Man) came *after* M10 and restate the
  model ("even if it becomes tapped again right away").
- 2024 is, in the bulletin's words, "The only change here": it handles a duration that starts "during the resolution".

**611.2b's own structure supports "checked continually, starting when the ability is activated".** Since 2009,
"never starts" has been its own case. The second case, a duration that "ends before the moment the effect would
first be applied", therefore assumes a duration already running while the ability waits on the stack. Under a
state-at-resolution reading, that sentence would apply to almost nothing. The CR's own Master Thief example
describes control lost while the ability waits as "its duration ... was over before the effect began", not as
"never started". The 2002–2007 text ("ends between [activation or stacking] and the moment ...") is explicitly about
a period.

**Can a duration "end" before it started? No.** investigator.md §2 answers "Yes", which is wrong as worded, though
its next sentence is right. A duration that never started is 611.2b's "never starts" case. What can happen is that a
duration starts (at activation or triggering), ends, and both happen before *the effect* begins. The CR has always
put it that way:
- 1999: "expires before the spell or ability creating it resolves".
- 2001: "is over before it [the effect] would begin".
- Master Thief example: "was over before the effect began".

The register entry must not say a duration "ends before it starts".

**The best counter-arguments, and why they fail:**
- **Plain English** ("for as long as you control ~" sounds like a state). The CR has defined these durations as
  periods since 2002, and the 2009 bulletin sets them against "as long as" on/off switches. No official source reads
  them as a state.
- **608.2h** (an effect uses the source's current information). That rule covers information an effect needs, "the
  answer is determined only once, when the effect is applied". A duration is checked over time by definition
  (611.2a), and 611.2b is the specific rule for durations.
- **The 2024 clause shows durations can "begin again", so regaining control is a restart.** The clause counts a
  restart only "during that spell or ability's resolution". Nothing in it saves a restart before the resolution,
  and the bulletin calls the clause "The only change here".
- **Seasinger's duration is a conjunction.** The ruling takes the control half on its own ("Whether you control
  Seasinger is checked continually ... The same is true regarding whether Seasinger remains tapped"). 611.2b's
  Master Thief example shows that a plain "for as long as you control ~" is the same kind of duration.

---------------------------------------------------------------------------------------------------------------------

## 3. Contrary sources: none found

Searched in the data, for every card:
- "checked continually", "any point", "by the time ... resolves";
- "as the ability resolves", "when the ability resolves", "on resolution";
- "before ... resolves" on all 259 cards with "for as long as" (884 rulings);
- "regain(ing) control";
- phasing together with stack timing;
- copies of abilities.

- **The closest thing to a contrary ruling: Jin-Gitaxias // The Great Synthesis (wotc 2023-04-14).** "For chapter I of
  The Great Synthesis, the number of cards you draw is determined as the ability resolves. If you no longer control
  The Great Synthesis at that time, your maximum hand size remains unchanged." It is worded as a state at
  resolution, but it covers only control lost and not regained, which both readings agree on. Not contrary.
- **Support the investigator didn't use: Garland, Royal Kidnapper (wotc 2025-10-02).** "While Garland's second ability
  is on the stack, if that opponent stops being the monarch before it resolves, you won't gain control of the target
  creature at all." This is a duration about a *player*, worded as an event, with "at all". It sits with Eriette
  (2024-04-12, "becomes unattached ... before ... resolves, you won’t gain control ... at all") and Old Man (2009-10-01,
  "The same is true if the targeted creature’s power becomes greater ... before the ability resolves").
- Tidebinder's own ruling is also worded as an event: "If you lose control of Tidebinder Mage between when it entered
  the battlefield and when the ability resolves ...". Read literally, it covers losing and regaining control.
- **Web.** Searched: regaining before resolution, Seasinger, Tidebinder, untap-and-retap, "begin again". The only
  argument for the state reading is the 2014 Magic Judges forum *question* (apps.magicjudges.org/forum/post/61148,
  Amber Prison), which is a question, not an answer. Its reply couldn't be retrieved. MTG Salvation thread 265243
  (Rubinia) gives the period reading.
- **A weak lead the other way: the bulletin's "work as players would expect them to"**, which is aimed at the
  during-resolution case only.

---------------------------------------------------------------------------------------------------------------------

## 4. The sub-cases challenged

### 4a. Phasing: upheld, high-moderate
- **The rules.** 702.26b: a phased-out permanent "is treated as though it does not exist". 702.26d: "treated as though
  it’s not on the battlefield and not under its controller’s control". 702.26f: "effects with “for as long as”
  durations that track that permanent (see rule 611.2b) end when that permanent phases out because they can no longer
  see it". 17 phasing rulings on about 40 cards say "the duration will expire".
- **Counter-argument from 702.26d's last sentence** ("Effects that check a phased-in permanent’s history won’t treat
  the phasing event as having caused the permanent to leave ... its controller’s control"). One could argue that the
  611.2b check is a history check. 702.26f is the more specific rule for "for as long as" durations and says they
  *end*, so it wins.
- **A real scenario exists** (the sources give none): a mid-turn phase-in, e.g. The Moment ("phases out until The
  Moment leaves the battlefield") followed by destroying The Moment while the source's ability waits. No ruling
  covers it.
- **The same reasoning covers "for as long as ~ remains on the battlefield"** (Sower of Temptation and about 35
  others).

### 4b. When the window opens: upheld, restated
- **Evidence for trigger time:**
  - Tidebinder: "between when it entered the battlefield and when the ability resolves".
  - Kraken: "between when its last ability triggers and when that ability resolves".
  - CR 610.3b and 610.4c: "after that ability triggered" (activated abilities, 610.3a and 610.4b: "after that spell or
    ability was put onto the stack").
  - The 2008 broadening, made precisely because durations "could end before the stated window began", the stated
    window having begun at "putting the triggered ability onto the stack".
- **Activated abilities.** The engine's window opens at `Event::AbilityActivated` (casting.rs ~3439, "602.2i: becomes
  activated", after the costs are paid). That matches Seasinger's "starting when the ability is activated".
- **Restated gap.** The engine misses more than "a control change and change back between the trigger event and the
  trigger being put on the stack". The missed case is control **lost between triggering and being put on the stack,
  then regained at any time before resolution**. `changed_while_on_stack` only looks for a `ControlChanged` *from*
  the controller after the stacking event; the regain is a change *to* the controller.
- **Still very rare:** something in the same resolution must change control after the trigger event.
- **Reflexive abilities** (Nihiloor) are separate abilities, each with its own window (603.12). The engine's per-ability
  window is right there.

### 4c. Durations about another object made false and then true again: split the sub-case
- **(c1) A condition the ability doesn't set up**: Old Man's power clause, Garland's "they're the monarch", Eriette's
  "that Aura is attached to it", Rootwater's "that creature is enchanted", Assimilation Aegis. **Nothing, moderate**,
  because:
  - the literal 611.2b says so;
  - three WotC rulings are worded as events, with "at all": Old Man 2009-10-01, Eriette 2024-04-12, Garland
    2025-10-02.
  - Today the engine checks only the state at resolution (gap iii), so it lets these begin.
  - Garland's test (tests/rulings/r_p061_misc.rs `garland_gains_nothing_if_the_opponent_is_no_longer_the_monarch`)
    covers only "stops being the monarch and stays that way".
- **(c2) A condition the resolution itself sets up** (Xolatoyac's counter, Braided Net's and Zygon Infiltrator's
  tap): **open**.
  - The literal 2024 text gives a perverse result. Take a land whose flood counter is removed and then replaced by
    another effect before the ability resolves: the duration "began again" *before* the resolution, not during it, so
    the effect does nothing. That happens even though Xolatoyac's ability then puts a further counter on the land.
  - This runs against the bulletin's stated aim ("the land does indeed become an Island") and against the 2009
    bulletin's "or starts applying".
  - Recommendation: keep the engine's check at resolution for c2 and register it as provisional.

### 4d. Theoretical over-breadth (no printed card)
- 611.2b lets a duration "begin again during that spell or ability's resolution".
- `changed_while_on_stack` also counts events from the ability's own resolution. So control lost and regained
  *during* the resolution (for example through 800.4a, a player leaving the game) would end the duration in the
  engine, where 611.2b says the effect applies.
- Both the investigator and parun #6027 found no card that produces this. Mention it in the entry; no fix needed.

---------------------------------------------------------------------------------------------------------------------

## 5. The engine

The investigator's gaps (i) phasing, (ii) window and (iii) other-object durations are **confirmed**, with (ii)
restated as in 4b. Gap (i) also covers `Duration::WhileSourceOnBattlefield`: `ended_before_it_began` has no arm for
it, and `effect_expired` sees only the phased-in state. Gaps the investigator didn't find:

- **(iv) Effect kinds the check never reaches.** `ended_before_it_began` has exactly three call sites:
  - resolve.rs ~1056, `Effect::Modify`;
  - resolve.rs ~1169, `Effect::AddRestriction`;
  - control_change.rs ~50, `plan_gain_control`.

  Two other effect kinds take this duration but create it **with no 611.2b check** and expire it only by state:
  - `Effect::GrantPlayPermission` (resolve.rs ~2722, then `casting::grant_play_permission`; expiry by state in
    permissions.rs ~233 and in the layers.rs cleanup);
  - `Effect::AddPlayerEffect` (resolve.rs ~1186).

  `Effect::AddReplacement` has the same hole, though no card with this duration uses it today. The cards affected:
  - **Lightning, Security Sergeant** (verified tier);
  - Gwen Stacy // Ghost-Spider and Victor Mancha, Runaway (round-trip);
  - Kotose, the Silent Spider, Hama, the Bloodbender and Taster of Wares (compiled);
  - Jin-Gitaxias // The Great Synthesis: "You have no maximum hand size for as long as you control this Saga", which
    compiles to `AddPlayerEffect` (hand_graveyard_grammar.rs `p_no_max_hand_size`).

  With control lost and regained while the trigger waits, the engine still grants the permission or the unlimited
  hand size. That contradicts the entry's own claim of covering "every 'for as long as you control ~'". When control
  is lost and not regained, the outcome is right (the grant can't be used and is removed at the next recompute).
- **(v) Copies of abilities have no window.**
  - copy.rs `copy_spell_quiet` emits no event when it copies a `StackAbility` (it emits only `SpellCopied`, and only
    for spells).
  - So `changed_while_on_stack` never sets `put_on_stack` for the copy, and returns false.
  - A copy made by Strionic Resonator, Rings of Brighthearth or Lithoform Engine then lets the effect begin after
    control is lost and regained, or after the source is untapped and tapped again, while the copy waits. The copy's
    source is the original's (Strionic Resonator 2018-03-16, Lithoform Engine 2020-09-25).
- **(vi) A latent fragility.**
  - `rule_effect_expired` (layers.rs ~864) evaluates the condition with a fresh `Ctx::new` that has no stack object.
  - `ended_before_it_began` matches only a bare `Condition::Custom`.
  - So a *restriction* whose remains-tapped duration is a conjunction would lose the history check. Today only Old
    Man of the Sea has such a conjunction, and it takes `plan_gain_control`, which keeps `ctx.stack_obj`, so it works.
- **Over-breadth checks, all clean:**
  - Only " for as long as you control ~" produces `WhileYouControlSource` (effects.rs ~1282, durations_5.rs ~239,
    permission_grammar.rs ~332/~961).
  - `of_src` excludes spells.
  - Static "as long as" abilities never reach the check.
  - `ControlChanged` comes from recompute for any cause.
  - Effects that already exist are re-checked on every recompute (expire_dependent_effects runs twice), and once
    ended they're removed for good, as Time of Ice's and Ty Lee's rulings require.
- **The tests in r_p086_untap_locks.rs** drive real `ControlChanged` events (r_s06_common.rs `give_control` runs exec,
  then recompute, then flush). They are sound, and they cite 611.2b and the interp entry.

---------------------------------------------------------------------------------------------------------------------

## 6. Corrections for the implementer

1. **Register entry.**
   - Quote the Seasinger ruling (2007-09-16) as the deciding source in Rules and Why. Add Preacher (2007-09-16),
     Entrancing Lyre (2020-01-24), Time of Ice (2018-04-27), the 2002–2007 418.3d window text, the Morningtide
     (2008-01-16) and July 2009 bulletins, and the Foundations bulletin, marked as known only through search summaries
     with authorship unverified.
   - Status can become decided on the strength of the ruling. That is the coordinator's and the user's call: every
     decided entry so far says "Decided by the user".
2. **Rewrite the entry's scope sentence.** It currently says "Effects of spells, and those whose duration is about
   another object, aren't affected". Instead, say what the engine does for each sub-case, including the split of
   4c (c1 nothing, c2 the state at resolution, provisional).
3. **Standard-resolution wording.**
   - The period starts "when the ability becomes activated (602.2i, after its costs are paid) or, for a triggered
     ability, when it triggers" (not "and put on the stack").
   - "An ended duration never restarts *before the resolution*; 611.2b lets it begin again during the resolution."
   - Never write that a duration "ends before it starts" (see 2).
4. **Engine (general, no card names):**
   - (a) Run the same pre-resolution check for `GrantPlayPermission` (including `WithPlayTerms`), `AddPlayerEffect`
     and `AddReplacement`. Test with Lightning, Security Sergeant and The Great Synthesis.
   - (b) Give copies of abilities a window, via an event or a recorded start in `copy_spell_quiet` for a
     `StackAbility`. Test with a Strionic Resonator copy of Tidebinder Mage's trigger.
   - (c) Open the window for triggered abilities at the trigger event.
   - (d) Count the source phasing out (`Event::PhasedOut`) as ending `WhileYouControlSource`, the remains-tapped
     conditions and `WhileSourceOnBattlefield`.
   - (e) Optional: a history check for c1 durations.
   - (f) Optional: make `ended_before_it_began` walk `And` conditions.
5. **Tests.**
   - Add a Seasinger test citing `ruling!("Seasinger", "Whether you control Seasinger is checked continually")`
     (P177, OPEN), covering both halves: lost and regained, and untapped and retapped. TestGame doesn't check tiers.
   - If Old Man's P117 ruling is cited for its power clause, decide c1 first.
6. **History table:** add the 1999-11-01 418.3d text; date the 2001 wording from 2001-07-23.

---------------------------------------------------------------------------------------------------------------------

## 7. Hosts

- **Blocked:** magic.wizards.com (WebFetch "getaddrinfo ENOTFOUND", once, for the Foundations bulletin). I didn't
  retry the hosts the investigator lists as blocked (investigator.md §9); that list stands untested by me.
- **Reachable:**
  - WebSearch (summaries only);
  - github.com (git fetch / ls-remote of pit142857/mtg-cr);
  - raw.githubusercontent.com (maxmakesmagic/ormos files);
  - api.github.com search endpoints. `repos/maxmakesmagic/ormos` answered "GitHub access to this repository is not
    enabled for this session", which is a session scope message, not the proxy.

Status: COMPLETE.
