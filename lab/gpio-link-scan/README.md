# RP1 GPIO link scan

This lab firmware drives one approved CM5 bank 0 pin at a time while an
ESP32 or other input-only receiver observes the signal. The lab boot bundle
reserves GPIO4–13 and GPIO16–27 for proc1. Confirm the bench wiring and Linux pin ownership
before issuing a drive command.

The mailbox begins at byte offset 0x400 of the RP1 link page. Use
little-endian 32-bit words. The host writes only word 0. Command bits 31:16
are a fresh, nonzero sequence, bits 12:8 select a bank 0 pin for drive
commands, and bits 7:0 select the operation.

| Operation | Value | Action |
| --- | ---: | --- |
| Check bias | 1 | Test the candidate inputs with both weak pulls. |
| Drive with pull-down bias | 2 | Drive a pin admitted by the last bias check. |
| Drive with pull-up bias | 3 | Drive a pin admitted by the last bias check. |

The ready value in word 1 is 0x47505343. Word 2 echoes the command when
complete. Words 3–6 hold the pad park error mask, admitted pin mask,
operation result (1 means success), and baseline level mask. For drive
commands words 7–24 hold six triples of pulse start time, duration in
microseconds, and RP1 GPIO levels sampled halfway through the pulse. The
first, third, and fifth pulses are high. The other pulses are low.

Each drive pulse uses a 7 ms RP1 timer deadline. All other candidate pads
stay inputs, including the opposite end of a loopback. At completion the
lab parks the pins with output and pulls disabled.
