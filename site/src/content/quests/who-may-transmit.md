---
title: Who may transmit?
patch: Band rules
order: 10
summary: Use the rule finder on cubesat.app to see how the same 915 MHz radio can be allowed, need a licence, or not be allowed.
minutes: 10
skill: Radio rules
tool:
  name: Bands on cubesat.app
  href: https://cubesat.app/learn/spectrum
  note: Reference information, not legal advice. Nothing here transmits.
beginner:
  steps:
    - Open Bands and find **Which rule am I under?**.
    - Pick the first radio in the list. Look at the cards below it. Which say **Fits**?
    - Turn on **Licensed operator in charge**. Does the amateur radio card fit now? Read its reasons to find out what else it needs, then try that too.
  check: You wrote down one rule that fits and one that does not for the same radio, and the two switches the amateur card needed.
  discuss: Radio waves are shared by everyone nearby. Why do we need rules about who may send?
intermediate:
  steps:
    - Pick **Licensed ham, 125 kHz, open messages**. Turn on **Messages encrypted** and see what happens to the amateur radio rule.
    - Pick **Wi-Fi HaLow, 1 MHz channel**. Which rule does it fit, and what does the card say this project uses it for?
    - Find a card marked **Unverified**. Write down the measurement it asks for.
  check: You explained why encryption changes the amateur result and named the measurement one card needs.
  discuss: Students may listen to any band, but a licensed operator must be in charge to transmit on amateur bands. Why is listening treated so differently from sending?
professional:
  steps:
    - Read `findRules` in the cubesat.app repository (`mobile/src/lib/ruleFinder.ts`) and open the rule sections each card cites.
    - Choose one preset and trace, step by step, why each card gives its result.
    - Find one assumption the rule finder makes that a real radio could break, such as antenna gain or duty cycle.
  check: Your trace cites the rule section for each step, and you named one assumption with the measurement that would test it.
  discuss: The app says unverified instead of guessing when a measurement is missing. When should a tool refuse to give an answer?
---
