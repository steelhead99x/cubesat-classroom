# CubeSat Classroom Quick Start

## Before you build

- **Computer.** Use macOS, Linux, or Windows. On Windows, run it inside WSL, the Windows Subsystem for Linux. Open a terminal on that computer.
- **Tools.** Check `curl --version` and `git --version`. Ask a mentor to help set up your system’s C compiler and linker (Xcode Command Line Tools on macOS, or build tools such as `build-essential` on Ubuntu).
- **Internet.** The first run downloads the project and build dependencies. The starter can install Rust and Cargo. If you already have Rust, use version 1.91 or newer. A computer is enough for the first mission.

## 1. Start the station

### With an AI assistant

Copy this prompt into Claude, Codex, Grok, or another assistant. A chat-only assistant can walk you through the terminal steps.

```text
Help me run the CubeSat Classroom ground station on my computer. Use the quick-start guide and AGENTS.md in https://github.com/steelhead99x/cubesat-classroom. Ask what operating system and experience level I have. Use an existing checkout or guide me to clone it. Walk me through running cargo run --locked -- classroom from the checkout, one step at a time. Ask before running commands that download, install, or change files. If you can't use my terminal, show me what to type. Help me open the local address and request a simulated reading so I can record its value and unit myself.
```

In a local checkout, use `$cubesat-quest-coach` in Codex or `/cubesat-quest-coach` in Claude Code for guided learning. Use `$cubesat-station` or `/cubesat-station` to troubleshoot the station. The project also includes `cubesat-hardware-evidence` for test planning and `cubesat-web-mobile` for site and app changes.

### With your terminal

Paste the command. The starter asks which hardware you have, recommends a quest, and explains each download. Press Enter to continue or type `n` at a confirmation to stop.

```sh
curl -fsSL https://cubesatclass.org/start.sh | bash
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
