# blescan (Bluetooth Low Energy Scanner)

I was playing about with a different project that uses BLE. I noticed that I was seeing some devices being discovered that I wasn't explictly searching for. So, I was curious what you could find out the devices around you.

Hence, this small project.

## How to use

I've only used it on a Mac, so it may not work as follows on other platforms. However, the underlying libraries it uses should work on other platforms.

It's fairly simple:

    cargo run

(The first time you run this on a Mac in a Terminal it will ask for permissions to use Bluetooth)

This will then bring up a simple text UI which shows the named devices that have been discovered. It also shows 'anonymous' devices, where the name is derived from a hash of the "manufacturer data" that is in the BLE advertisement.

Each device is shown with:

- how long ago it was last seen
- the RSSI (Received Signal Strength Indicator)
- a change indicator:

  - '↑' = stronger
  - '=' = same strength
  - '⌄' = weaker
  - '\*' = newly-discovered (so nothing to compare to)

Scans update every second.
