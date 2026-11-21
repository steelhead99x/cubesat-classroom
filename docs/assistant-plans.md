# Proposal: free and paid ways to run a mission

**Status: draft for discussion.** Nothing here is built or for sale yet. Every number is a starting point. Comment on any line in a [plans feedback issue](https://github.com/steelhead99x/cubesat-classroom/issues/new?template=feedback.yml&labels=feedback,plans&title=Plans%20feedback%3A%20), or ask a maintainer to add you to the review.

## The goal

Everything builds toward **three in-person workshops a year** (planned), where teachers, students and licensed radio mentors work with real parts. The website, the quests and any paid plan exist to prepare people for those days and to help fund them.

## The idea in one paragraph

The website is the introduction: anyone can try the practice ground station and five example quests with no account. The repository is the full experience: the real station, every quest, the hardware labs, and quest-guide instructions any AI assistant can follow. A paid plan offers that full guided experience hosted for a classroom, with strict guardrails, so teachers and students never need an AI account of their own.

## Three paths

| | Try it | Run it yourself | Crew Plus (proposed) |
| --- | --- | --- | --- |
| Price | Free | Free, open source | $19.99 per month per teacher or mentor |
| Account | None | None | One adult account holder (teacher, guardian or mentor); students join with a class code |
| What you get | Browser station, 5 example quests at 3 levels, mission patches | Real Rust station over two MoQ relays, full quest board, balloon lab, CAD, hardware quests | Everything in "Run it yourself", plus a hosted quest guide with classroom guardrails |
| AI | None | Your own assistant, using [AGENTS.md](../AGENTS.md) | Hosted guide on Claude Sonnet 5.5 through the Anthropic API |
| Data kept | Patches in your browser only | Whatever you keep locally | Minimal; see "Privacy" |

### Possible Crew Plus benefits (pick the ones that matter)

These are candidates. We want teachers and mentors to choose.

1. **Early access to beta releases** of the station, new demo features and the mobile app.
2. **New quest packs first**, including hardware quests for Arduino, Seeed XIAO and Raspberry Pi.
3. **Class progress view** showing patches by student nickname, with no names or emails.
4. **Crew log export** to PDF or CSV for grading and portfolios.
5. **Teacher controls** for the guide: level, daily limits, topics turned on or off.
6. **Mentor office hours** once a month, if volunteers can staff them.
7. **Workshop seats first.** Early registration for the three in-person workshops each year.
8. **A vote on the roadmap**, such as which quest or lab comes next.
9. **Funding the workshops.** Revenue first covers API and hosting costs. The proposal is that the rest funds the three in-person workshops each year.

### A free guided preview (needs careful review)

A no-signup taste of the guide could run on the website with these limits:

- **Menu questions only.** Visitors pick from suggested questions about the current quest. There is no free-text box, so there is nothing to misuse and no personal information to collect.
- **About 10 questions per device per day**, enforced with a hashed, rotating identifier (see "Limits").
- **Clear disclosure** that answers come from an AI, with a link to the guardrails below.

## Limits

Free and paid limits are separate, so heavy paid use never eats into the free preview, and the other way round.

| Limit | Free preview | Crew Plus |
| --- | --- | --- |
| Who it counts | One device | Each student seat, plus a class total |
| Daily messages | ~10 menu questions | ~40 per student, teacher can lower it |
| Message length | Menu only | ~1,000 characters in, short answers out |
| Class or monthly cap | Global daily budget for the whole site | Per-class monthly cap with a warning at 80% |
| When a limit is hit | "Come back tomorrow" plus the next quest step | Teacher notified; students keep the quest text |

**Hashed identifiers.** Limits are counted against a salted hash, never a raw identifier.

- **Free tier:** the hash is built from a per-day secret and a random value stored in the browser. The secret rotates every day, so yesterday's counts cannot be linked to today's.
- **Paid seats:** the hash is built from the class code and the seat number.
- **Storage:** only counters are kept. There are no IP logs beyond what abuse prevention requires.

Before setting final caps, price them against current Anthropic API pricing for the chosen model and effort setting.

## Guardrails

The same rules apply to the hosted guide and to [AGENTS.md](../AGENTS.md) for people running their own assistant.

**Stays on topic.**
- The guide covers CubeSats, orbits, amateur radio and AMSAT, sensors and units, data and measurement, the ground station, the quests, and how to write a good crew log.
- Anything else gets a short, friendly redirect back to the quest.
- No homework for other subjects, no games, no general chat.

**Teaches, does not do the work.**
- The guide asks questions and gives hints at the student's level.
- It does not fill in crew logs or answer the "talk it over" questions for the student.

**Protects students.**
- It says it is an AI.
- It never asks for names, ages, schools, locations, photos or contact details. If a student shares them, it tells them not to.
- It does not discuss violence, self-harm, dating or other unsafe topics. It points the student to a trusted adult, and in a hosted class it flags the conversation to the teacher.

**Stays honest about radio.**
- Transmitting needs a licensed operator. The guide says so and does not coach anyone around licensing or equipment rules.

**Technical controls (hosted).**
- Anthropic's child-safety system prompt where available.
- Input and output content filtering.
- Per-seat rate limits.
- Conversation logs visible to the class teacher.
- A report button on every answer.
- Logs retained for a short, published period and not used for training.

## Privacy and compliance

- **API, not a personal subscription.** The hosted guide must use an Anthropic API key from the Claude Console under Anthropic's commercial terms. Personal Claude subscriptions cannot be used to serve other people.
- **Minors.** Anthropic's guidelines for organizations whose products let minors use its models require several safeguards: age verification, content filtering, monitoring and reporting, educational resources on safe use, an AI disclosure, compliance with laws such as COPPA, and a public statement of that compliance. [Anthropic: guidelines for organizations serving minors](https://support.claude.com/en/articles/9307344-responsible-use-of-anthropic-s-models-guidelines-for-organizations-serving-minors)
- **Who holds the account.** Paid accounts belong to adults, meaning teachers, guardians or mentors. Sign-in should accept hosted Git services through OAuth (GitHub, GitLab.com and Codeberg), not one provider only. Self-hosted Git servers are not supported. Adults without a Git account can sign in with a phone number and a one-time code. Students never give a phone number. Students use a class code and a nickname. No student email or name is collected.
- **COPPA and school privacy rules** need review by someone qualified before launch, as does a published privacy policy.

## Sign-in and sessions

Two ways to sign in, with different session lengths. Both are planned; none of this exists yet.

| | Git account (GitHub, GitLab.com, Codeberg) | Phone number (adults only) |
| --- | --- | --- |
| How | OAuth with the Git host | A 6-digit one-time code sent by text message |
| Session | Stays signed in while active: expires after 60 days with no activity | Ends 8 hours after sign-in, active or not |
| Renewal | Automatic and silent; no prompt on each visit | Sign in again with a new code |

**Phone numbers are stored only as a keyed hash.** A phone number has too few possible values for a plain hash to protect it: every number in a country can be hashed and matched in minutes. Store `HMAC-SHA-256(server secret, normalised number)` instead, with the secret kept as a Worker secret and never in the repository. The number itself is held only long enough to send the code, then discarded. The hash is the account's identifier.

**The 6-digit code** is random, used once, and expires after 10 minutes. Store only its hash. A wrong code counts against that number and against the device: after 5 wrong tries the code is cancelled, and a number can request at most 5 codes an hour. Without these limits a 6-digit code can be guessed. Text messages cost money and can be abused to send spam, so add a site-wide daily cap on codes sent.

**Sessions** live in a cookie marked `HttpOnly`, `Secure` and `SameSite=Lax`. The cookie holds a random token. The server stores only the token's hash, with its expiry and sign-in method.

- **Git account sessions** extend their 60-day window on activity. To keep writes low, extend at most once a day, and issue a new token when extending so a copied old cookie stops working. Signing out, or removing the app's access at the Git host, ends the session.
- **Phone sessions** have a fixed 8-hour end time that activity does not extend.
- **Signing out ends the session on the server**, not only in the browser. An adult can end all their sessions at once.

## Open questions for reviewers and the community

1. Is $19.99 per month per teacher right, or should it be per class, per school, or free for Title I schools?
2. Which three benefits matter most to teachers?
3. Should the free preview exist at all, or should the free path be "Try it" plus "Run it yourself" only?
4. Who operates the paid service: Teachers in Space, a new nonprofit, or a volunteer team? Is directing surplus to the three yearly workshops the right policy, and who approves the workshop budget?
5. Which daily limits fit a 45-minute class period?
6. Who reviews the guardrails and privacy plan before launch: a teacher, a licensed radio mentor, and a privacy reviewer?
7. Should the guide support languages other than English from the start?

## How to give feedback

- [Open a plans feedback issue](https://github.com/steelhead99x/cubesat-classroom/issues/new?template=feedback.yml&labels=feedback,plans&title=Plans%20feedback%3A%20) and quote the line you are commenting on.
- Comment on the pull request that changes this file.
- Teachers who do not use GitHub can send notes through a mentor, who posts them with credit.
