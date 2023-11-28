[![Crates.io](https://img.shields.io/crates/v/brack.svg)](https://crates.io/crates/brack)

# brack

tune the backlight!

```
Change backlight brightness

brack [device] [change]

device (optional) the device to change backlight on (see /sys/class/backlight)
change (optional) absolute value in percent or change in percent prefixed with +/-.

no change will display current value.
no device will try to automatically determine device.

Examples:

brack +10 # increase brightness with 10%
brack -10 # decrease brightness with 10%
brack 50 # set brightness to 50%
brack intel_backlight +10 # increase brightness with 10% on the intel_backlight device
```

## Installing

```sh
$ cargo install brack
```

## User access to the backlight

Replace `<vendor>` with the name in /sys/class/blacklight, e.g. `acpi_video0` and add a [udev rule](https://superuser.com/questions/484678/cant-write-to-file-sys-class-backlight-acpi-video0-brightness-ubuntu), e.g.: [`/etc/udev/rules.d/backlight.rules`](./backlight.rules):
```
ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="<vendor>", RUN+="/bin/chgrp video /sys/class/backlight/%k/brightness"
ACTION=="add", SUBSYSTEM=="backlight", KERNEL=="<vendor>", RUN+="/bin/chmod g+w /sys/class/backlight/%k/brightness"
```

and make sure you add the user to the `video` group:

```
$ sudo gpasswd -a $USER video
```

## I3 configuration

```
bindsym XF86MonBrightnessUp exec brack +10 # increase screen brightness
bindsym XF86MonBrightnessDown exec brack -10 # decrease screen brightness
```

