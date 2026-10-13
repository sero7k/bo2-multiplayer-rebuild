# Call of Duty Trickshotting Techniques: Evidence-Based Catalogue
### MW2 (2009), Black Ops (2010), MW3 (2011), Black Ops II (2012), Ghosts (2013)

Compiled 2026-10-13 for the BO2 Rust recreation project. Every claim carries a citation `[n]` (see **Sources**) or an explicit evidence tag.

---

## 0. Read this first: method, evidence grades, and research limits

### Evidence tags
| Tag | Meaning |
|---|---|
| **[S]** | Corroborated: two or more independent retrieved sources agree. |
| **[s]** | Single retrieved source. Plausible but not cross-checked. |
| **[U]** | **Unverified.** Widely repeated community knowledge or a requested term that no retrievable source in this session confirmed. Do not treat as fact. Validate against original-game footage or by testing the real game before implementing. |
| **[H]** | **Engine hypothesis.** A mechanical explanation inferred from general IW/Treyarch-engine modding knowledge (weapon-file timers such as `fireTime`, `rechamberTime`, `reloadTime`, `reloadAddTime`, `dropTime`, `raiseTime`, `quickDropTime`, `quickRaiseTime`, `altRaiseTime`, `altDropTime`). These field names come from IW-engine mod tools and were **not** retrieved in this session. The explanations are reasoned, not documented. |

### Research limits (important)
- **Most primary trickshot community sources were unavailable.** Reddit (r/CoDTrickshots and others), YouTube tutorial descriptions, and most Call of Duty Fandom wiki pages (Elevator, Bounce, Select Fire, One Man Army main page) were blocked by fetch permissions or not indexed by the search tool. Only pages surfaced by web search could be fetched.
- As a result, community slang (NAC variants, Mala, Shax, Canswap, stalls, glides) is **thinly sourced**: often one hobbyist blog, a Weebly clan site, and Urban Dictionary. Official patch notes and journalism are well sourced, but they cover glitches (elevators, care packages, javelin, knife lunge), not swap techniques.
- **Ghosts (2013)** has almost no technique-level documentation in the retrieved material beyond the developer's statements about the sniper/quickscope redesign.
- The community uses inconsistent terms. Disagreements between sources are recorded explicitly instead of resolved by guesswork.

---

## 1. Shared engine background (applies to all swap/cancel techniques) [H]

Swap and cancel tricks in all five titles exploit the same pattern. The viewmodel weapon runs a state machine (roughly: *ready → firing → rechambering → ready*, *ready → reloading → ready*, *ready → dropping → (switch) → raising → ready*, plus alt-mode and offhand/equipment states). Each state has a timer from the weapon definition. A weapon-switch request is accepted from most states, including firing recovery, rechamber, and reload. Accepting it **abandons the remaining timer** of the current state. Consequences:

1. **Reload cancel.** If the switch arrives after the point where ammo is committed to the magazine (`reloadAddTime`-style point), the player keeps the ammo but skips the rest of the reload animation. This matches the community note that you cancel "once the clip fills" [8] and the condition "rounds in the magazine or a chambered round" [7].
2. **Fire/rechamber cancel.** A switch issued during a bolt-action rechamber or post-fire recovery ends that state immediately. Switching back (double-tap) can be faster than letting the bolt cycle. Retrieved evidence for this is from an analogous Source-engine game only [39]. It is unverified for CoD.
3. **Drop/raise cancel ("instant" swaps).** If a second switch input arrives during the *dropping* phase of the outgoing weapon, the engine may resolve the switch with a reduced or skipped raise of the incoming weapon. The community calls the result an "instant" or "nac" swap [1][2][3]. The exact frame windows per title are **not documented** in any retrieved source.
4. **Pickup/equipment/alt-mode states** (weapon pickup, equipment placement, select-fire toggle, OMA bag) are separate animation states with their own timers and transitions. Interrupting them can cause visual artefacts, such as a replayed pickup animation [1] or equipment held while firing [20].

> **Implementation implication for the Rust recreation:** to preserve these tricks faithfully, the weapon state machine must (a) accept switch inputs during fire, rechamber, reload, and drop states, (b) commit reload ammo at a discrete time point instead of at animation end, and (c) model alt-mode toggles and pickups as interruptible states. **Exact per-weapon timings must be measured from the original BO2 weapon definitions or footage. They are not available from community sources.**

---

## 2. Technique catalogue

Each entry uses this format: **Aliases | Titles (origin) | Setup | Input | Timing | Animation | Mechanic | Result | Limits/patches | Evidence**.

### 2.1 NAC / Nac swap / Nacswap
- **Aliases:** nac swap, nacswap, "instaswap" (per [1]; see disagreement). The expansion "Next-After-Cancel" appears in community usage but **no retrieved source states what NAC stands for** [U].
- **Titles:** MW2, where [1] calls it "most commonly referred to in Modern Warfare 2". One clan site claims it is "available in all Call of Duty titles" [2]. One Urban Dictionary definer also applies it to Garry's Mod [3].
- **Setup:** Two weapons. The classic pairing is a non-sniper (shotgun, pistol) to a sniper [2][3].
- **Input (sources disagree):**
  - [2]: fire the shotgun, then "press the shoot button, X [reload], and Y [switch] at the same time" so the weapon switches to the sniper.
  - [3] (2015 definer): "press Y at a specific point in an animation to change weapons instantly." [3] (2023 definer): "you reload, then switch at a specific moment"; can be done mid-air.
  - [1]: a nacswap is "instant", the opposite of a canswap.
- **Timing:** Precise and frame-sensitive [3]. No source gives a frame or millisecond window.
- **Animation behaviour:** The incoming weapon appears with no visible (or a truncated) raise animation [1][3].
- **Mechanic:** [H] see §1(3). The reload or fire input creates a state, and the switch input issued inside that state's interruptible window ends it. The raise of the next weapon is shortened or skipped. The cited sources do not explain why including reload helps.
- **Result:** The kill-feed and killcam show the target weapon (often the sniper) "coming out" instantly before the shot [1][3].
- **Variants:**
  - **Reload NAC** and **Hybrid NAC.** Named by [2] as BO2-only and "hard to do", with **no description** given. [U] beyond the name.
  - **"Moni NAC".** Mentioned by [3] as an exception to the usual timing rule, with no description [U].
  - **NAC to same weapon, infinite NAC.** **No retrieved source** [U].
- **Limits/patches:** No patch documented.
- **Evidence:** [S] that the term exists and means a timing-based instant switch (MW2) [1][2][3]. [s]/[U] for the exact inputs.

### 2.2 Instaswap
- **Aliases:** nacswap (per [1]).
- **Disagreement:**
  - [1]: "A nacswap is just the expert term for an instaswap, most commonly referred to in Modern Warfare 2." Instaswap = NAC.
  - [2]: an instaswap is a distinct technique: "switch your weapon but you are covering it with a[n] equipment", which "only works in Call of Duty Black Ops 2 and Call of Duty Advanced Warfare".
- **Titles:** MW2 (as NAC synonym) [1]; BO2 (equipment-covered variant) [2].
- **Setup ([2] variant):** Equipment (lethal/tactical) plus two weapons.
- **Input ([2] variant):** Switch weapons quickly while the equipment animation covers the swap. The exact order is not given.
- **Mechanic:** [H] the offhand/equipment animation hides the weapon raise. If the primary switch completes underneath it, the new weapon is ready as soon as the equipment animation ends.
- **Evidence:** [s] for each competing definition. **Definitions conflict.**

### 2.3 YY (double-tap switch)
- **Aliases:** "double tap Y/Triangle", "YY sequences" [4], `weapnext; wait; weapnext` bind [7].
- **Titles:** CoD4 and MW2 [1][6][7][8]; BO2 [4]. Reportedly "a bit buggy for BO[1]", where it sometimes fails to switch back [6].
- **Uses documented:**
  1. **Reload cancel** (see 2.4) [6][7][8].
  2. **Shot styling in MW2** [1]:
     - *Temperrr:* jump off, scope in "but not all the way", double-tap Y and release ADS, spin 360, shoot, then press Y once immediately to create a "fading effect".
     - *Parallel:* jump off, start a 360, ADS during the spin, double-tap Y while holding ADS, then hardscope.
  3. **Commercial BO2 binds** list "YY sequences" next to reload cancels and swaps [4].
- **Timing:** The PC bind uses `wait 2` between presses [7]. A GIMX macro used 13 ms, and a user recommended about 45 ms between presses [6].
- **Mechanic:** [H] The first switch press enters the dropping state and interrupts whatever state was active (reload, ADS, fire recovery). The second press re-targets the original weapon before the drop completes, so the player returns to the same weapon with the interrupted state cleared.
- **Result:** The same weapon is kept with the reload, ADS or animation interrupted. In [1] it also produces visual "fade" effects on the scope.
- **Evidence:** [S] (≥3 independent sources for the input; uses in [1] are single-source).

### 2.4 Reload cancel (RC)
- **Aliases:** "Reloadus Interruptus" (CoD4 bind name) [7], reload skip [36], reload cancel [4][9].
- **Titles:** CoD4 and MW2 [6][7]; BO1 (buggy per one user) [6]; BO2 [4]. [9] states reload cancels "have been a feature in all Call of Duty games except in the latest installment" (written about the 2022 era; a weak general claim). MW2 campaign speedrunners list "Reloadskipping" as a core technique [36] (title only).
- **Input variants:**
  - *Switch-cancel:* Y-Y / `weapnext; wait 2; weapnext` [7][8].
  - *Sprint-cancel:* "once the clip fills, sprint or double-tap Y" (CoD4, unverified user answer) [8].
  - *ADS-cancel:* **no retrieved source** [U].
- **Condition:** Works "only if you still have rounds in the magazine or a chambered round" [7]. The ammo must already be committed: "once the clip fills" [8].
- **Animation:** The reload animation is cut off and the gun returns "nearly instantaneous[ly]" to ready [7].
- **Mechanic:** [H] see §1(1). Ammo is added at a mid-animation commit point, and the tail of the animation is cosmetic and cancellable.
- **Counter-evidence:** One CoD4 respondent claimed that apparent early fire on killcams was a killcam visual artefact [8].
- **Evidence:** [S].

### 2.5 Canswap
- **Aliases:** "can swap" [U as alias].
- **Titles:** Source [1] uses BO2 examples. Other titles [U].
- **Setup:** A weapon on the ground to pick up, plus your sniper.
- **Input [1]:** Pick up a gun, quickly switch back to your sniper, then switch to the picked-up gun.
- **Animation:** "The game replays the pickup animation on the switch" [1]. Canswaps "take the longest to perform", the opposite of a nacswap [1].
- **Mechanic:** [H] The pickup animation is a distinct first-raise state. Leaving it before completion apparently leaves a flag set, so the next raise of that weapon replays the pickup or first-raise animation instead of the normal raise.
- **"Canswap with equipment":** **no retrieved source** [U].
- **Origin of the name:** not found [U].
- **Evidence:** [s].

### 2.6 Mid-air swap
- **Titles:** MW2 [1][3]; generic.
- **Description:** Any of the above swaps performed while airborne, typically after jumping off a high point and during a spin. [3] (2023) notes nac swaps are "also do[ne] midair". [1] has the 360, Temperrr and Parallel all start with "jump off".
- **Mechanic:** [H] Weapon state is independent of movement state, so all swaps and cancels work in the air. Spins are pure view-angle input.
- **Evidence:** [S] for existence; inputs as per the underlying swap.

### 2.7 Mala / Mala-NAC
- **Status:** **[U] No retrievable source defines "Mala".** The task asked to check whether it relates to the MW2 Tactical Knife or OMA. Neither link could be confirmed.
- **Closest documented MW2 equipment trick:** the **Double Equipment Switch** (see 2.9) [20]. Do **not** assume it is the Mala.
- **Recommendation:** Treat as an open question. Verify against original MW2 footage or community members before implementing anything named "Mala".

### 2.8 OMA (One Man Army) tricks (MW2 only)
- **Perk facts:** Tier 1 perk. It replaces the secondary with a bag; switching to the bag opens the class menu. There is a 5 s class change (3 s with Pro), during which the player can only move, crouch, jump and go prone [18][19]. "C4 or Claymores planted before one uses OMA will not disappear", and they can still be detonated by double-tapping reload or use [18]. Players used OMA swaps instead of reloading LMGs [18]. With Danger Close, OMA allowed continual explosive resupply [11].
- **Titles:** MW2 only. In MW3, "OMA" survives only as the name of a Specialist-bonus challenge and as bag props on the Faceoff maps Lookout and U-Turn [18]. It is not a perk in BO1, BO2 or Ghosts. The perk's absence from those titles is consistent with [18]; non-existence is not separately sourced.
- **Trickshot use:** Class-swap mid-air or mid-sequence lets a player change the loadout (e.g. to obtain different equipment or a sniper) while keeping placed equipment [18]. This is the basis of the Double Equipment Switch [20].
- **Mechanic:** [H] The OMA bag is a pseudo-weapon whose "raise" opens the class UI. Applying the class swaps inventory without respawn, and entities already placed by the player are not cleaned up [18].
- **Evidence:** [S] for perk behaviour. [s] for trickshot-specific use.

### 2.9 Double Equipment Switch (MW2)
- **Titles:** MW2 (requires OMA).
- **Input [20]:** "1. Have OMA Class with T.I. 2. Swap to Claymore class 3. Put Claymore Down 4. Pull out T.I. and Shoot." A "Fakie" variant also exists, with no inputs given [20].
- **Mechanic:** [H] Combines OMA's non-cleanup of equipment [18] with an equipment-held state, so equipment appears in hand while the kill comes from the gun.
- **Evidence:** [s] (video-title aggregator quoting a tutorial).

### 2.10 Select Fire swap / Select Fire NAC (BO2)
- **Attachment facts:** The BO2 Select Fire attachment lets ARs and SMGs "switch fire modes between semi-auto, burst-fire, and fully-automatic". It does not change damage [29]. Unlocks at AR level 10 and SMG level 14 [29]. A 2013 patch fixed Select Fire reverting to default after round transitions; it "now remembers previous setting" [30].
- **Use as animation cancel or swap:** **[U] No retrieved source documents Select Fire being used as a NAC or swap.** The toggle input (the attachment's D-pad/alt-weapon button) was also not confirmed in the retrieved text.
- **Mechanic:** [H] In Treyarch's engine, fire-mode toggles are plausibly implemented like alt-weapon switching (separate weapon variants joined by `altRaiseTime`/`altDropTime`). If so, the toggle would be accepted as a weapon change. It would interrupt fire recovery or reload exactly like a Y press, but with much shorter drop/raise times. This would make it a faster cancel than a full switch. **Verify against original BO2 before implementing.**
- **Titles:** The attachment is confirmed in BO2 [29][30]. It was not found as an attachment for MW2, BO1, MW3 or Ghosts in retrieved sources [U].
- **Evidence:** [S] attachment exists and is stateful. [U] trickshot use.

### 2.11 Knife lunge
- **MW2:** The Commando perk "let players make melee attacks from a much greater distance" and was paired with Marathon and the Tactical Knife [31]. Gameinformer lists "super speed care package knife deaths" among MW2 bugs [44].
- **BO1:** A Treyarch update planned to "limit the swipe range so you'll not be able to make so many lunge knife kills as before" [25].
- **MW3:** Commando was removed: it "would not be making an appearance" [31].
- **BO2:** Patch notes: "Fixed knife lunge launching players on the Express map" [30]. A lunge-launch movement exploit existed and was patched on that map.
- **Ghosts:** [U].
- **Mechanic:** [H] Melee lunge applies a velocity toward the target. Collision with geometry during the lunge can convert it into a launch, which is the source of the BO2 Express issue [30].
- **Evidence:** [S] (per-title facts each single-sourced, but the overall picture is consistent).

### 2.12 Elevators
- **Titles:** MW2 (origin for these titles; famous) [10][11][12]. Mentioned as returning in Vanguard (2021) [10][12]. BO1, MW3, BO2 and Ghosts: [U].
- **Input:** Get "trapped between two walls", then use "a series of movements such as wriggling around, crouching, and standing" until the game slowly raises the player [10].
- **Result:** A slow ascent with "a clear, aerial view of the map", used for sniper trickshots [10][11].
- **Mechanic:** [H] The player bounding box is wedged between brushes. Stance changes (box height changes) trigger the engine's stuck-player resolution, which nudges the player upward each frame.
- **Patches:** The MW2 1.08 / Jan-2010 PC patch: "Improved player collision removing cases of getting into geometry and 'elevators'" [13][14]. Whether any MW2 elevators survived the patch is [U].
- **Evidence:** [S].

### 2.13 Bounces (MW2)
- **Titles:** CoD4 ("Bouncetage", CoDJumper mod) [22]; MW2 ("MW2 New Bounce Techni[q]ue and Bouncetage 2" [22]; "MW2 Estate Bounce Tutorial", "useful to do amazing trickshots" [21]). Other titles: [U].
- **Input:** Not described in retrieved sources. They show only "getting to the spots to bounce from and hitting the bounce's" [22].
- **Mechanic:** [H] In Quake-3-derived movement, landing at speed on certain sloped or brush-edge surfaces can clip the velocity into an upward component instead of zeroing it. The result is a vertical bounce, gaining height without jump input.
- **Evidence:** [S] that bounces exist in MW2. [U] mechanics and inputs.

### 2.14 Care-package tricks
- **Speed glitch (MW2):** "Quickly toggling between their primary weapon and the care package marker" let players "sprint at speeds far beyond the game's intended limits" [11]. Holding the marker made you faster, "more so with Lightweight" [17]. **Patched:** "Care Package, Emergency Airdrop, and Sentry Gun marker grenades sprint speed normalized" (MW2 1.08 / PC Jan 2010) [13][14].
- **Infinite care packages (MW2):** fixed in 1.06 [15][16] and again noted in 1.08 [14].
- **MW3:** A separate "care package glitch" on Terminal was under investigation in July 2012 [37] (nature not described).
- **"Carepackage stall" (trickshot):** **[U] no retrieved source.**
- **Mechanic:** [H] The marker is a weapon whose movement-speed scale was set higher than standard weapons. Switching to it applied that scale; the fix normalised it [13].
- **Evidence:** [S] speed glitch and patches. [U] stall.

### 2.15 "Teleport" / Shax swap
- **[U] No retrievable source defines "Shax" or a "teleport swap".** Open question.

### 2.16 Drop-shot
- **Definition:** "Successfully going prone while shooting an opponent" [32] (CoD Mobile guide; the mechanic is generic). No retrieved source covers it for the five target titles specifically.
- **Titles:** Prone and firing exist in all five titles, so the mechanic works natively in all [H/general].
- **Input:** Press prone and fire together [32].
- **Evidence:** [s] definition. Applicability across titles is by game design.

### 2.17 Quick-scope
- **Definition:** Firing "right after zooming in" with a sniper [23][24]. Tapping ADS triggers aim-assist snap to the chest or head [24]. Popularised by CoD4 [23][24].
- **MW2:** Works natively (era of the Highrise 360 no-scopes [33]; the trickshotting blog assumes Intervention FMJ quickscopes [1]).
- **BO1:** Deliberately suppressed. An update aimed to "stop players using the quickscoping technique" [25]. Treyarch's Vonderhaar admitted that a sniper "not shooting straight until fully scoped" caused confusion [26].
- **MW3:** Infinity Ward "embraced" quickscoping [26].
- **BO2:** Treyarch sought "balance" between quickscopers and hardscopers. They proposed a "0.4 ADS raise time" and removal of the "random bullet" [26]. A BO2 quickscope tutorial dates to 13 Nov 2012 [23].
- **Ghosts:** "Sway timing" changes meant quickscoping "certainly won't be viable in the way it previously was" [27][28].
- **Mechanic:** [H] ADS raise fraction, ADS aim-assist snap, and sway/accuracy-during-ADS-transition determine viability. BO1 forced spread until fully scoped [26]. Ghosts changed sway timing [27].
- **Evidence:** [S].

### 2.18 No-scope / 360 & 720 spins / Hardscope
- **No-scope:** hip-fire sniper kill. **360 no-scope:** spin 360 then hip-fire [23].
- **360 trickshot (MW2) [1]:** Jump off, spin once mid-air, shoot as the spin stops. "Most standards are 720 and above (2 or more spins)".
- **NSLing:** spinning *before* jumping off the platform. [1] warns against it.
- **Hardscope:** fully scoped "for more than a second or two" [1][23].
- **Raging:** a term, not a technique: getting a hitmarker on the trickshot and finishing the kill without actually hitting the shot [1].
- **Titles:** Spinning and hip-fire exist in all five titles. The BO1 hip-fire and partial-scope accuracy penalty [26] limits no-scopes there. BO2 Plutonium servers are "mostly 'spinner only'" [42].
- **Evidence:** [S].

### 2.19 Bolt cancel
- **Description:** Cancelling the bolt-action rechamber after a shot via a weapon switch (usually YY).
- **Evidence:** **[U] for CoD.** The only retrieved description is from Day of Defeat: Source, where double-tapping the switch key is "faster than letting the bolt cycle" [39]. That is an analogous engine, not CoD.
- **Mechanic:** [H] see §1(2).

### 2.20 Sniper swap
- **Description:** A generic term for any swap ending on the sniper before the kill. In practice it equals the final step of NAC, canswap or instaswap [1][2].
- **Evidence:** [s] (implicit in [1][2]). There is no standalone definition.

### 2.21 Ladder stall / Glide / Mid-air glide / Ninja defuse / Shield bounce
- **[U] No retrieved source confirms definitions, inputs or title availability for any of these.** Searches surfaced only modern-era (2019+) riot shield and defuse-indicator bugs [unrelated].
- Do not implement from these names alone.

### 2.22 Other title-specific items found during research
| Item | Title | Summary | Evidence |
|---|---|---|---|
| Javelin glitch | MW2 | Player became a "ticking time bomb" exploding on death; **patched 1.06** | [S] [11][15][16] |
| Strafe jumping | MW2 (and others) | Speed gain by flicking aim while airborne; listed in MW2 speedrun fundamentals | [s] [35][36] (the [35] article's year is unspecified) |
| Snake glitch | BO1 | "Press the toggle stance button then double tap the jump button" | [s] [38] |
| Grenade-launcher/camera ricochet shots | BO1 | S&D grenade-launcher round bounced off placed cameras across Summit for a direct-impact kill | [S] [33][34] |
| Highrise spots | MW2 | Crane, helipad and roof "parkour spots that have populated thousands of montages"; wallbang trickshots | [s] [40] |
| Trickshot bind scripts | MW2 (IW4x) | Fully scripted shots via cfg binds, e.g. `+right; com_maxfps 59; cl_yawspeed 919; wait 2; +attack; -attack; wait 2; +reload; -reload; wait 34` (spin, fire, reload timing) | [s] [5] |
| Trickshot game-mode mods | BO1, BO2, MW3, Ghosts (PC clients) | VanityTS trickshot mod supports T5/T6/IW5/IW6 | [s] [41] |

---

## 3. Compatibility matrix

Legend: **Works** = works natively (documented) · **Not in title** = required feature absent · **Patched** · **Unverified** = no retrievable documentation. Superscript notes below.

| Technique | MW2 (2009) | BO1 (2010) | MW3 (2011) | BO2 (2012) | Ghosts (2013) |
|---|---|---|---|---|---|
| NAC / nacswap | Works [1][3] | Unverified ᵃ | Unverified ᵃ | Works [2] | Unverified ᵃ |
| Reload NAC / Hybrid NAC | Unverified | Unverified | Unverified | Works ᵇ [2] | Unverified |
| NAC to same weapon / infinite NAC | Unverified | Unverified | Unverified | Unverified | Unverified |
| Instaswap (equipment-covered, per [2]) | Unverified ᶜ | Unverified | Unverified | Works [2] | Unverified |
| YY double-tap switch | Works [1][7] | Works ᵈ [6] | Unverified | Works [4] | Unverified |
| Reload cancel (switch) | Works [6][7] | Works ᵈ [6] | Unverified ᵉ | Works [4] | Unverified ᵉ |
| Reload cancel (sprint) | Unverified ᶠ | Unverified | Unverified | Unverified | Unverified |
| Mid-air swaps | Works [1][3] | Unverified | Unverified | Unverified ᵍ | Unverified |
| Canswap | Unverified | Unverified | Unverified | Works [1] | Unverified |
| Canswap with equipment | Unverified | Unverified | Unverified | Unverified | Unverified |
| Mala / Mala-NAC | Unverified | Unverified | Unverified | Unverified | Unverified |
| OMA class-swap tricks | Works [18][19] | Not in title | Not in title ʰ [18] | Not in title | Not in title |
| Double Equipment Switch (OMA) | Works [20] | Not in title | Not in title | Not in title | Not in title |
| Select Fire swap / NAC | Not in title ⁱ | Not in title ⁱ | Not in title ⁱ | Unverified ʲ [29][30] | Not in title ⁱ |
| Knife lunge (extended / Commando) | Works [31] | Patched ᵏ [25] | Not in title [31] | Patched ˡ [30] | Unverified |
| Elevators | Patched ᵐ [13][14] | Unverified | Unverified | Unverified | Unverified |
| Bounces | Works [21][22] | Unverified | Unverified | Unverified | Unverified |
| Care-package speed glitch | Patched [13][14] | Unverified | Unverified ⁿ | Unverified | Unverified |
| Care-package stall | Unverified | Unverified | Unverified | Unverified | Unverified |
| Teleport / Shax swap | Unverified | Unverified | Unverified | Unverified | Unverified |
| Drop-shot | Works ᵒ | Works ᵒ | Works ᵒ | Works ᵒ | Works ᵒ |
| Quick-scope | Works [1][23] | Patched [25][26] | Works [26] | Works [23][26] | Patched ᵖ [27][28] |
| No-scope / 360 / 720 | Works [1][33] | Works ᵠ | Works ᵒ | Works [42] | Works ᵒ |
| Hardscope | Works [1] | Works [26] | Works ᵒ | Works [26] | Works ᵒ |
| Ladder stall | Unverified | Unverified | Unverified | Unverified | Unverified |
| Sniper swap (generic) | Works [1] | Unverified | Unverified | Works [1][2] | Unverified |
| Glide / mid-air glide | Unverified | Unverified | Unverified | Unverified | Unverified |
| Bolt cancel | Unverified ʳ | Unverified | Unverified | Unverified | Unverified |
| Ninja defuse | Unverified | Unverified | Unverified | Unverified | Unverified |
| Shield bounce | Unverified | Unverified | Unverified | Unverified | Unverified |
| Javelin glitch (non-trickshot) | Patched [15][16] | Not in title | Unverified | Not in title | Not in title |
| Strafe jump | Works [36] | Unverified | Unverified | Unverified | Unverified |

**Notes**
- ᵃ [2] claims NAC is "available in all Call of Duty titles", a single low-reliability source.
- ᵇ Named only, no description [2].
- ᶜ [1] treats "instaswap" as a synonym of the MW2 nacswap, so under [1]'s definition it works in MW2.
- ᵈ "A bit buggy for BO", which sometimes fails to switch back [6].
- ᵉ General claim that all pre-2022 CoD titles had reload cancels [9]. Not title-specific.
- ᶠ Documented only for CoD4, as an untested user answer [8].
- ᵍ Plausible by engine design but not documented for BO2.
- ʰ OMA appears in MW3 only as a challenge name and map props [18].
- ⁱ The attachment was not found in retrieved sources for these titles. Absence is not separately confirmed [U].
- ʲ The attachment exists and is stateful [29][30]. Its use as a cancel or swap is undocumented.
- ᵏ Lunge range reduced by update [25]. The lunge still exists.
- ˡ Lunge exists. The lunge-launch exploit on Express was patched [30].
- ᵐ "Removing cases of … 'elevators'" (1.08). Residual spots are unverified.
- ⁿ A different MW3 care-package glitch was under investigation in 2012 [37].
- ᵒ Follows from core mechanics present in every title (prone, fire, turning, hip-fire, ADS). Not separately documented per title.
- ᵖ Reduced by design via sway-timing changes, not by patch. Infinity Ward said it "won't be viable in the way it previously was" [27][28].
- ᵠ Possible, but the BO1 sniper accuracy penalty before full scope [26] reduces hip-fire and partial-scope reliability.
- ʳ The only evidence is from a non-CoD game [39].

---

## 4. Implications for the BO2 recreation (summary)

1. **Weapon state machine fidelity matters most.** NAC, YY, reload cancel, canswap and instaswap all come from what the engine allows to interrupt what, and when [1][2][3][6][7][8]. Build interruptible states with discrete timer fields, a reload ammo commit point, a separate first-raise/pickup flag, and offhand/equipment overlap.
2. **Select Fire must be a first-class, stateful attachment.** It must persist across rounds, as fixed by the 2013 patch [30]. Whether its toggle cancels other states needs original-game verification.
3. **Do not re-introduce patched exploits by accident.** BO2 Express lunge-launch was patched [30]. Elevators and care-package speed are MW2-era bugs fixed in MW2 itself [13][14]. Model collision resolution and per-weapon move-speed scaling carefully.
4. **Sniper ADS feel:** BO2 sat between BO1's suppression and MW3's permissiveness [25][26]. Treyarch's stated BO2 targets were about 0.4 s ADS raise for "MW1 style" snipers and no "random bullet" [26].
5. **Content imported from MW2/MW3/BO1 maps:** map-geometry tricks such as bounces, elevators and Highrise spots [21][22][40] depend on BO2's movement code, not the source title. Decide per map whether to preserve them.

---

## Sources

1. How to Trickshot (blog), "Learning to trickshot: Part 1" (2014): https://trickshottutorials.blogspot.com/
2. COD Trickshot Clans (Weebly), "Trickshots": https://cod-trickshot-clans.weebly.com/trickshots.html
3. Urban Dictionary, "NAC Swap" (definitions 2015–2023): https://www.urbandictionary.com/define.php?term=NAC+Swap
4. Payhip, "Black Ops 2 CFG Script Pack" (product listing): https://payhip.com/b/3lMOP
5. Payhip, IW4x trickshot script pack (product listing): https://payhip.com/b/UlGAc
6. GIMX forum, "[macro] Call of Duty - Reloadus Interruptus": https://gimx.fr/forum/viewtopic.php?p=218
7. GIMX forum (same thread, original bind text): https://forum.gimx.fr/viewtopic.php?p=156
8. SuperCheats, CoD4 Xbox 360 "Shoot before done reloading": https://www.supercheats.com/xbox360/questions/callofduty4modernwarfare/116569/shoot-before-done-reloading.htm
9. Sportskeeda, "How to reload cancel in Modern Warfare 2 and Warzone 2": https://www.sportskeeda.com/esports/how-reload-cancel-modern-warfare-2-warzone-2
10. CharlieIntel, "CoD Vanguard brings back Modern Warfare 2's infamous elevator glitch": https://www.charlieintel.com/call-of-duty-vanguard/cod-vanguard-brings-back-modern-warfare-2s-infamous-elevator-glitch-148105/
11. CheatCC, "MW2 glitches: discover unseen aspects of Modern Warfare 2": https://cheatcc.com/articles/mw2-glitches-discover-unseen-aspects-of-modern-warfare-2/
12. Gfinity, "Elevator Glitch Returns in Vanguard Multiplayer": https://www.gfinityesports.com/call-of-duty-vanguard/elevator-glitch/
13. Shacknews, "Modern Warfare 2 PC Patch Targets Caring Types" (Jan 2010 patch notes): https://shacknews.com/article/61993/modern-warfare-2-pc-patch
14. God is a Geek, "Modern Warfare Patch v1.08 Causing Problems": https://godisageek.com/2010/01/modern-warfare-patch-v1-08-causing-problems/
15. Engadget, "Modern Warfare 2 update 1.06 fixes Javelin glitch, infinite care packages, geography exploits": https://www.engadget.com/2009-12-07-modern-warfare-2-update-1-06-fixes-javelin-glitch-infinite-care.html
16. PlayStation LifeStyle, MW2 glitch fixes (1.06): https://www.playstationlifestyle.net/?p=56248
17. Straight Dope Message Board, "CoD MW2" thread: https://boards.straightdope.com/t/cod-mw2/523556
18. Call of Duty Wiki (Fandom), "One Man Army Pro": https://callofduty.fandom.com/wiki/One_Man_Army_Pro
19. Band of Brilliance, "One Man Army": https://bob-bandofbrilliance.yolasite.com/one-man-army.php
20. Video-title aggregator, "How to double equipment switch" (quotes MW2 TSA tutorial): https://old.fbpe.org/how-to-double-equipment-switch.html
21. Video-title aggregator, "MW2 estate bounce tutorial": https://old.fbpe.org/mw2-estate-bounce-tutorial.html
22. Video-title aggregator, "CoD4 Bouncetage 2" (includes MW2 bounce titles): https://irc.pcper.com/cod4-bouncetage-2.html
23. Know Your Meme, "Quickscoping": https://knowyourmeme.com/memes/quickscoping
24. Giant Bomb, "Quick Scoping" (concept): https://wiki-origin.giantbomb.com/wiki/Concepts/Quick_Scoping
25. Accelerated Ideas, "Black Ops update will improve sniper rifle but no return for quickscoping": https://www.accelerated-ideas.com/news/black-ops-update-will-improve-sniper-rifle-but-no-return-for-quickscoping.aspx
26. MP1st, "Call of Duty Developer Seeks Balance Between 'Quickscopers' and 'Hardscopers'" (Vonderhaar, Treyarch): https://mp1st.com/news/call-of-duty-developer-seeks-balance-between-quickscopers-and-hardscopers
27. MP1st, "Quick-Scoping 'Won't Be Viable' in Call of Duty: Ghosts": https://mp1st.com/news/quick-scoping-wont-viable-call-duty-ghosts-according-iw-executive-producer
28. Gameranx, "Call of Duty: Ghosts changes quick-scoping to make it less viable": https://gameranx.com/updates/id/17279/article/call-of-duty-ghosts-changes-quick-scoping-to-make-it-less-viable/
29. GameRevolution, "Call of Duty: Black Ops 2 Attachments Guide and Tips": https://www.gamerevolution.com/guides/58034-call-of-duty-black-ops-2-attachments-guide-and-tips
30. MP1st, "Black Ops 2 Treyarch Community Update 2, Xbox 360 and PS3 Patch Notes" (Mar 2013): https://mp1st.com/2013/03/12/black-ops-2-treyarch-community-update-2-xbox-360-and-ps3-patch-incoming-patch-notes/
31. The Escapist, "Modern Warfare 3 Ditches Commando Perk": https://www.escapistmagazine.com/modern-warfare-3-ditches-commando-perk/
32. Sportskeeda, "How to execute a 'Dropshot' perfectly in COD Mobile": https://sportskeeda.com/esports/how-execute-dropshot-perfectly-cod-mobile
33. Dexerto FR, most impressive CoD kill (BO1 Summit grenade-launcher shot; MW2 Highrise 360s): https://www.dexerto.fr/call-of-duty/kill-call-of-duty-plus-impressionnant-tous-les-temps-1485790/
34. Upcomer, "Incredible Call of Duty Black Ops trickshot sets social media ablaze": https://upcomer.com/incredible-call-of-duty-black-ops-trickshot-sets-social-media-ablaze
35. GGRecon, "Move faster in MW2 by strafe jumping" (year unspecified): https://www.ggrecon.com/guides/mw2-strafe-jump-console-pc/
36. Speedrun.com, Call of Duty: Modern Warfare 2 guides ("Strafejumping, Reloadskipping"): https://www.speedrun.com/mw2/guides
37. MP1st, "New Modern Warfare 3 Terminal Easter Egg, Care Package Glitch Being Looked Into": https://mp1st.com/news/new-modern-warfare-3-terminal-easter-egg-care-package-glitch-being-looked-into-and-more
38. Steam Community, Call of Duty: Black Ops discussion (snake glitch): https://steamcommunity.com/app/42700/discussions/0/341537671985092760
39. EdgeGamers forum (Day of Defeat: Source quick-switch; analogous engine only): https://www.edgegamers.com/threads/288513/
40. GGRecon, "MW3 confirms classic trickshots spots for Highrise remastered": https://ggrecon.com/articles/mw3-confirms-classic-trickshots-spots-for-highrise-remastered
41. GitHub, gasps/VanityTS (trickshot game-mode mod, T4–T7/IW5/IW6/S1/H1): https://github.com/gasps/VanityTS
42. Steam Community, Black Ops II MP discussion (Plutonium "spinner only" servers): https://steamcommunity.com/app/202990/discussions/0/3667553846823700873
43. TheTVDB (YouTube channel index), "Black Ops 2 Noob Tube Montage 5 - Trickshots": https://www.thetvdb.com/series/moo/episodes/6957801
44. Game Informer, "A Changing Battlefield" (MW2 glitch commentary, 2010): https://gameinformer.com/b/news/archive/2010/03/11/a-changing-battlefield.aspx
