# HID-HDR Support Library

This library is intended to help ease the usage of HDR's GameCube controller stick-gate
changes that can be found at our fork of exlaunch [here](https://github.com/HDR-Development/exlaunch).

## Curious about the stick changes?
Please read this commentary regarding how Super Smash Bros. Ultimate, combined with the Nintendo Switch HID (human interface device) system firmware module impact inputs and gamefeel from one of our developers:

> TL;DR at bottom
> 
> So basically, this is a modification to how the Nintendo Switch OS reads GameCube controller analog sticks.
> 
> Let's talk about how GCC (GameCube Controller) sticks work in general. GCC sticks report signed, 8-bit integers. That is to say that they will report values between -128 and 127 on each axis. For simplicity sake, I will say that you can only get values in the range of -127 and 127 and we will refer to 0 as the center on both axes.
> 
> I would say that I have an average GCC, that is that I have no modifications, it's not a phob, it doesn't have anything special going on. I mean the left stick is a little bit worn down due to me having to play against S4 bayo, but that's aside the point. Due to the gate of the controller, I can only hit analog values at around +- 100 in each axes. This means that I'm limited to about 80% of the stick range, but that's actually kind of normal for GCC analog sticks. I'm going to refer to this range as the "average" range of GCC sticks.
> 
> Now that we have that context, let's talk about what it means to read values from the analog sticks, and the official GameCube adapter as a whole.
> 
> The average range is just that, average. Some sticks will have a wider range of accessible values from their stick gates, and really unlucky individuals will have smaller ranges available for their values. Obviously, Nintendo and the Smash team wanted all players to be on an equal playing field when it came to their sticks, so they accommodated for the lowest common denominator when writing their GCA (GameCube Adapter) driver for the input service.
> 
> Some basic terminology before I continue:
> - Inner Deadzone: The inner deadzone is a range of values that round down to 0 when they are read. For example, if you have an inner deadzone radius of 30, then any values that report in under that value will be set to 0 instead.
> - Outer Deadzone: The outer deadzone is a range of values that round up to the maximum reportable value when they are read. For example, if you have an outer deadzone radius of 120, then any values that report in above that value will be set to 127 instead.
> - Working Range: The working range is the range of values between the inner deadzone and the outer deadzone, which will report distinct values on the analog stick.
> 
> Another clarification I want to point out before I continue, for the majority of this explanation, when I throw out analog numbers I am talking about the number along the axis. The Nintendo Switch OS clamps and restricts these values based on their components, and not the vector magnitude of the stick. This also means that most of the time, we are dealing with squares instead of circles.
> 
> The Nintendo Switch HID service (I'm just going to call it "HID" from now on) sets the inner deadzone radius to 15 and the outer deadzone radius to 70. That limits the working range of the analog stick axes to 15 - 70. That is 55 distinct analog values that can be reported in both the X and Y directions. This working range will create a hollow square that only allows access to around 60% of the analog stick gate (not the full stick range, only the octagonal gate). While it's hard to explicitly state "this makes my angled moves feel bad", it definitely plays a part in that.
> 
> Inside of this working range, it remaps the values between 0 and 127, which is our max. So functionally, when you have an analog tilt of 15/127, the driver will assign that 0. When you have an analog tilt of 16/127, the driver will assign that 2 (truncated down from 2.3). When you have an analog tilt of 17/127, the driver will assign that 4 (truncated down from 4.6). So on and so forth.
> In the long run, this is a design choice made by the OS devs to accommodate the lowest common denominator of controller, and generally it's probably a good choice. Back in the earlier days of the Wii/GameCube, the games had direct access to the raw analog stick values and likely were able to set their own thresholds for how their games should best be played and enjoyed.
> 
> Now that we have an understanding of how the HID service is handling stick inputs, let's talk about how Smash is handling stick inputs.
> 
> Smash starts by taking what the HID service reports. It then places another inner deadzone, this time with a value of 0.2. As a reminder, any value beneath 0.2 as reported by the HID service will be consider 0.0. It also has another outer deadzone of 0.944, meaning any value above 0.944 will be considered equal to 1.
> 
> The way the inner deadzone works, however, is slightly different. Smash's deadzones don't take the values between 0.2 and 0.944 and remap them between 0 and 1, instead it takes them at face value. Once your stick hits a value of 0.2, then it will read that as a value of 0.2. This means that any values between -0.2 and 0.2 are invisible to the game, always being read as 0.0.
> 
> Smash's deadzones are pretty simple to think through, but let's combine both the deadzones of the console and the deadzones of smash and talk about what that means for the player.
> 
> Recall that we have a working range of 55 analog values on each axes from the HID service. We know that Smash discards the lower 20% and the upper 5.6% of these values, so we actually end up with a working range of 41 analog values. This ends up being about 48% of the analog stick gate's area that is read/handled by Smash. Just barely under half of the reportable values are actually used within Smash, and then because these values are not determined based on stick vector's magnitude, you're going to end up with very strange thresholds for inputs all across the board. All in all, this can contribute to the roughness that can be felt in Ultimate (and by extension, HDR).
> 
> Sooo, what does this new nightly do?
> Well, it only has to change one thing in order to drastically change the feel of the GCC sticks. It changes the HID service's inner and outer deadzone values to 10 and 100 accordingly. When combined with Smash's inner deadzone, this actually makes a deadzone that is 2 units larger than vanilla's, but that most likely won't be very noticeable. Instead, let's talk about what that means for the average GC analog stick gate.
> 
> Now, instead of a working range of 55 on the system level (41 on the game leve), you have a working range of 90 on the system level, and about 57 on the game level. Those working ranges don't really mean a whole bunch though, instead it's more important to compare percentages of how many values within the average stick gate you now have access to.
> 
> |               | X-Y Distinct Values | % of Stick Gate Used |
> |-|-|-|
> | Vanilla HID   |                  55 |                  60% |
> | Vanilla Smash |                  41 |                  48% |
> | HDR HID       |                  90 |                99.6% |
> | HDR Smash     |                  57 |                87.3% |
> 
> 
> This may seem absurd, but it's because with HDR's sticks, we have a larger inner deadzone (admittedly by a pretty small margin), but a much wide outer deadzone. If we assume the stick is circular (which is what I have done here), then the majority of the area of the circle (and hence the majority of the reportable stick values) are going to be in the outside of the stick. This will also make the majority of stick presses which butt up really close to the gate's edge closer to a stick magnitude of 1, instead of what it could was previously which would be larger than one (something something circle inside of a larger square rather than a square inside of a larger circle).
> 
> TL;DR: We have access to much more of the analog stick now

## Basic Usage
This library is intended to be very easy to plug n' play into your own plugin and/or mod if you want to, although you'll likely run into an error if multiple mods are trying to do this at the same time.

First, you want to start by querying the status of the HID system module:
```rs
// This method returns a Result<Status, u32>, where the u32 can be a non-zero result
// type from calling svcSendSyncRequest.
let status = hid_hdr::get_hid_hdr_status().unwrap();
```

After acquiring `status`:
```rs
use hid_hdr::Status;

match status {
    Status::Ok => {
        // This function takes a boolean for whether to enable or disable
        // stick gate changes, they are disabled by default so you shouldn't have to do this
        if !hid_hdr::connect_to_hid_hdr() {
            // Handle unable to connect
        } else {
            hid_hdr::configure_stick_gate_changes(true).unwrap();
        }
    },
    other => {
        // handle other status
    }
}
```

This library provides some helper methods available by specifying the `warnings` feature
in your `Cargo.toml` like this:
```toml
[dependencies]
hid-hdr = { git = "https://github.com/HDR-Development/hid-hdr", features = ["warnings"] }
```

These helper methods provide simple [`skyline-web`](https://github.com/skyline-rs/skyline-web) dialogs that inform players where to go to troubleshoot potential issues:
```rs
pub fn warn_unable_to_connect(discord_channel: &str, mod_name: &str, invite: &str);
pub fn warn_status(status: Status, discord_channel: &str, mod_name: &str, invite: &str);
```

Here is another implementation of the above that uses these:
```rs
use hid_hdr::Status;

match status {
    Status::Ok => {
        // This function takes a boolean for whether to enable or disable
        // stick gate changes, they are disabled by default so you shouldn't have to do this
        if !hid_hdr::connect_to_hid_hdr() {
            hid_hdr::warn_unable_to_connect("troubleshooting", "HDR", "discord.gg/hdr");
        } else {
            hid_hdr::configure_stick_gate_changes(true).unwrap();
        }
    },
    other => {
        hid_hdr::warn_status(other, "troubleshooting", "HDR", "discord.gg/hdr");
    }
}
```