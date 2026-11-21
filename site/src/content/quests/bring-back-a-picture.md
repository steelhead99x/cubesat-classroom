---
title: Bring back a picture
patch: First picture
order: 2
summary: Start the test camera and count how many frames reach the ground.
minutes: 5
skill: Data return
beginner:
  steps:
    - Press **Start video**.
    - Count the pictures as they arrive. Watch the white square move across the colours.
    - When it stops, write down how many pictures arrived out of 8.
  check: You wrote down a number out of 8 and where the white square stopped.
  discuss: Why is it useful that the camera sends lots of small pictures instead of one big one?
intermediate:
  steps:
    - Press **Start video** and time the burst from the first frame to the last.
    - Record the frame count, the burst duration and the time between frames.
    - Sketch the position of the marker in the first and last frames.
  check: You recorded 8 frames, the total time and the interval between frames.
  discuss: If each frame were 20 KB, how many kilobytes per second does this burst need?
professional:
  steps:
    - Start a video request. The reply is one MoQ group on `exchange.v1` with a JSON header frame, then 8 JPEG frames at 500 ms.
    - Note the size of each frame reported in the log and the total bytes for the group.
    - Estimate the average bit rate of the burst in kbit/s.
  check: You have per-frame sizes, the group total and a bit-rate estimate with your working.
  discuss: Each frame is independently decodable. Compare that with H.264 inter-frame video over a lossy link.
---
