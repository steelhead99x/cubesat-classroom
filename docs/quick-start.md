# CubeSat Classroom Quick Start

## Before you build

- **Computer.** Use macOS, Linux, or Windows. On Windows, run it inside WSL, the Windows Subsystem for Linux. Open a terminal on that computer.
- **Tools.** Check `curl --version` and `git --version`. Ask a mentor to help set up your system’s C compiler and linker (Xcode Command Line Tools on macOS, or build tools such as `build-essential` on Ubuntu).
- **Internet.** The first run downloads the project and build dependencies. The starter can install Rust and Cargo. If you already have Rust, use version 1.91 or newer. A computer is enough for the first mission.

## 1. Start the build

Paste the command. The starter asks which hardware you have, recommends a quest, and explains each download. Press Enter to continue or type `n` at a confirmation to stop.

```sh
curl -fsSL https://cubesatclassroom.org/start.sh | bash
```

**Check your work.** You should see a recommended quest path and a local web address.

## 2. Get a reading

Open the address. Request a simulated sensor reading, write down its number and unit, then play the practice video. Press Ctrl+C in the terminal to stop.

**Check your work.** Write one sentence another crew could use to repeat your reading.

## 3. Pick the next quest

- **M0 · Try it now.** Choose computer only, Arduino, Seeed XIAO, or Raspberry Pi. Teachers help test instructions and shape the next lesson.
- **M1 · Plan and review.** Close design questions and write test plans with mentors and qualified reviewers.
- **M2 · Verify.** Run approved tests and record evidence for the exact hardware configuration.

Open the [quest log](https://github.com/steelhead99x/cubesat-classroom/tree/main/quest). Read **Goal**, **Plan**, and **Required** if present. Do the task and ask a person to review your evidence. Teachers piloting M0 can share what worked, where students got stuck, and one change. Use a team nickname in public notes.

The Markdown format comes from [moq-dev/quest](https://github.com/moq-dev/quest), which grew out of MoQ. The CLI is optional. If installed, `quest unblocked` lists ready tasks and `quest check` validates the log.

**With Codex, Claude Code, or another LLM.** Open the checkout or share the public quest. Ask it to explain the first step and how to check it, then have it wait before editing files.
