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

